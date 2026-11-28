//! 音频频谱前端 ASF（TS 103 190-1 clauses 4.2.7、4.2.8 与 5.1）。

use crate::bits::BitReader;
use crate::codes::{asf_book, huffman, Book};
use crate::tables::{codebook, sfb_offsets, sfb_side, RANDOM_NOISE_TABLE};
use crate::{fail, Result};

const LONG: usize = 4;
const ESCAPE_CODEBOOK: usize = 11;

const GRP_BITS_LONG: [[usize; 4]; 4] = [
    [15, 10, 8, 7],
    [10, 7, 4, 3],
    [8, 4, 3, 1],
    [7, 3, 1, 1],
];
const GRP_BITS_MID: [usize; 4] = [7, 3, 1, 0];
const GRP_BITS_SHORT: [usize; 3] = [3, 1, 0];

/// 频谱码本 1..=11 参数：(dimension, unsigned, cb_mod, cb_off)
const CODEBOOK_META: [(usize, bool, i32, i32); 12] = [
    (0, false, 0, 0),
    (4, false, 3, 1),
    (4, false, 3, 1),
    (4, true, 3, 0),
    (4, true, 3, 0),
    (2, false, 9, 4),
    (2, false, 9, 4),
    (2, true, 8, 0),
    (2, true, 8, 0),
    (2, true, 13, 0),
    (2, true, 13, 0),
    (2, true, 17, 0),
];

pub fn transform_lengths(frame_len: usize) -> Result<&'static [usize]> {
    match frame_len {
        2048 => Ok(&[128, 256, 512, 1024]),
        1920 => Ok(&[120, 240, 480, 960]),
        1536 => Ok(&[96, 192, 384, 768]),
        1024 => Ok(&[128, 256, 512, 1024]),
        960 => Ok(&[120, 240, 480, 960]),
        768 => Ok(&[96, 192, 384, 768]),
        512 => Ok(&[128, 256, 512]),
        384 => Ok(&[96, 192, 384]),
        _ => fail(format!("unsupported frame_len {frame_len}")),
    }
}

pub fn num_sfb_48(n: usize) -> usize {
    match n {
        2048 => 63,
        1920 => 61,
        1536 => 55,
        1024 | 960 => 49,
        768 => 43,
        512 | 480 => 36,
        384 => 33,
        256 | 240 => 20,
        192 => 18,
        128 | 120 => 14,
        96 => 12,
        _ => panic!("unsupported transform length {n}"),
    }
}

pub fn msfb_bits(length: usize) -> u32 {
    if length >= 384 {
        6
    } else if length >= 192 {
        5
    } else {
        4
    }
}

pub fn side_bits(length: usize) -> u32 {
    if length >= 480 {
        5
    } else if length >= 240 {
        4
    } else {
        3
    }
}

pub fn msfbl_bits(frame_len: usize) -> u32 {
    if frame_len >= 1536 {
        3
    } else {
        2
    }
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub frame_len: usize,
    pub long_frame: bool,
    pub transform: [usize; 2],
    pub max_sfb: [usize; 2],
    pub max_sfb_side: [usize; 2],
    pub side_limited: bool,
    pub dual_maxsfb: bool,
    pub different_framing: bool,
    pub window_to_group: Vec<usize>,
    pub group_windows: Vec<usize>,
    pub first_half_group: usize,
    pub group_transform: Vec<usize>,
    pub group_lengths: Vec<usize>,
    pub window_lengths: Vec<usize>,
}

impl Layout {
    pub fn new(
        frame_len: usize,
        long_frame: bool,
        transform: [usize; 2],
        max_sfb: [usize; 2],
        max_sfb_side: [usize; 2],
        grouping: &[bool],
        side_limited: bool,
        dual_maxsfb: bool,
    ) -> Result<Self> {
        let lengths = transform_lengths(frame_len)?;
        let different_framing = frame_len >= 1536 && !long_frame && transform[0] != transform[1];
        let mut to_group = vec![0usize];
        if !long_frame {
            let mut grp_bits = grouping.to_vec();
            let mut windows = grp_bits.len() + 1;
            if different_framing {
                let first = 1usize << (3 - transform[0]);
                grp_bits.insert(first - 1, false);
                windows += 1;
            }
            let mut groups = 1usize;
            for &bit in grp_bits.iter().take(windows - 1) {
                if !bit {
                    groups += 1;
                }
                to_group.push(groups - 1);
            }
        }
        let count = to_group.last().copied().unwrap_or(0) + 1;
        let mut group_windows = vec![0usize; count];
        for &g in &to_group {
            group_windows[g] += 1;
        }
        let first_half_group = if frame_len >= 1536 && !long_frame {
            to_group[1usize << (3 - transform[0])]
        } else {
            count
        };
        let mut group_transform = Vec::with_capacity(count);
        let mut group_lengths = Vec::with_capacity(count);
        for g in 0..count {
            let t = if frame_len >= 1536 {
                if long_frame {
                    LONG
                } else if g < first_half_group {
                    transform[0]
                } else {
                    transform[1]
                }
            } else {
                transform[0]
            };
            group_transform.push(t);
            group_lengths.push(if t == LONG { frame_len } else { lengths[t] });
        }
        let window_lengths: Vec<usize> = to_group.iter().map(|&g| group_lengths[g]).collect();
        if window_lengths.iter().sum::<usize>() != frame_len {
            return fail(format!(
                "windows of {:?} do not fill a frame of {frame_len}",
                window_lengths
            ));
        }
        Ok(Self {
            frame_len,
            long_frame,
            transform,
            max_sfb,
            max_sfb_side,
            side_limited,
            dual_maxsfb,
            different_framing,
            window_to_group: to_group,
            group_windows,
            first_half_group,
            group_transform,
            group_lengths,
            window_lengths,
        })
    }

    #[inline]
    pub fn groups(&self) -> usize {
        self.group_windows.len()
    }

    #[inline]
    pub fn band_count(&self, g: usize, side: bool) -> usize {
        let idx = usize::from(self.different_framing && g >= self.first_half_group);
        if self.side_limited || (self.dual_maxsfb && side) {
            self.max_sfb_side[idx]
        } else {
            self.max_sfb[idx]
        }
    }
}

pub fn read_layout(
    b: &mut BitReader<'_>,
    frame_len: usize,
    dual_maxsfb: bool,
    side_limited: bool,
) -> Result<Layout> {
    let lengths = transform_lengths(frame_len)?;
    let (long_frame, transform) = if frame_len >= 1536 {
        let lf = b.flag()?;
        if lf {
            (true, [LONG, LONG])
        } else {
            (false, [b.read(2)? as usize, b.read(2)? as usize])
        }
    } else {
        let t = b.read(2)? as usize;
        if t >= lengths.len() {
            return fail(format!("transf_length {t} for a frame of {frame_len}"));
        }
        (false, [t, t])
    };
    let first_len = if long_frame { frame_len } else { lengths[transform[0]] };
    let different = frame_len >= 1536 && !long_frame && transform[0] != transform[1];
    let mut max_sfb = [0usize; 2];
    let mut max_side = [0usize; 2];

    let mut read_counts = |i: usize, length: usize| -> Result<()> {
        if side_limited {
            max_side[i] = b.read(side_bits(length))? as usize;
        } else {
            max_sfb[i] = b.read(msfb_bits(length))? as usize;
            if dual_maxsfb {
                max_side[i] = b.read(msfb_bits(length))? as usize;
            }
        }
        Ok(())
    };
    read_counts(0, first_len)?;
    if different {
        read_counts(1, lengths[transform[1]])?;
    }

    let grp = if frame_len >= 1536 {
        if long_frame {
            0
        } else {
            GRP_BITS_LONG[transform[0]][transform[1]]
        }
    } else if frame_len > 512 {
        GRP_BITS_MID[transform[0]]
    } else {
        GRP_BITS_SHORT[transform[0]]
    };
    let mut grouping = Vec::with_capacity(grp);
    for _ in 0..grp {
        grouping.push(b.flag()?);
    }
    let layout = Layout::new(
        frame_len,
        long_frame,
        transform,
        max_sfb,
        max_side,
        &grouping,
        side_limited,
        dual_maxsfb,
    )?;
    check_bands(&layout)?;
    Ok(layout)
}

pub fn read_lfe_layout(b: &mut BitReader<'_>, frame_len: usize) -> Result<Layout> {
    let max_sfb = [b.read(msfbl_bits(frame_len))? as usize, 0];
    let layout = Layout::new(
        frame_len,
        true,
        [LONG, LONG],
        max_sfb,
        [0, 0],
        &[],
        false,
        false,
    )?;
    check_bands(&layout)?;
    Ok(layout)
}

fn check_bands(layout: &Layout) -> Result<()> {
    for g in 0..layout.groups() {
        let n = layout.group_lengths[g];
        let sfb = sfb_offsets(n).ok_or_else(|| crate::Error::new(format!("missing sfb for {n}")))?;
        for side in [false, true] {
            if layout.band_count(g, side) >= sfb.len() {
                return fail(format!(
                    "max_sfb {} for a {n}-line transform",
                    layout.band_count(g, side)
                ));
            }
        }
    }
    Ok(())
}

pub fn read_master(b: &mut BitReader<'_>, layout: &Layout) -> Result<Layout> {
    let largest = *layout.group_lengths.iter().max().unwrap_or(&layout.frame_len);
    let master = b.read(side_bits(largest))? as usize;
    let map_count = |n: usize| -> Result<usize> {
        if n == largest {
            return Ok(master);
        }
        let col = sfb_side(largest, n).ok_or_else(|| {
            crate::Error::new(format!("max_sfb_master {master} has no mapping from {largest} to {n}"))
        })?;
        if master >= col.len() {
            return fail(format!("max_sfb_master {master} has no mapping from {largest} to {n}"));
        }
        Ok(usize::from(col[master]))
    };
    let first = layout.group_lengths[0];
    let last = *layout.group_lengths.last().unwrap();
    let c0 = map_count(first)?;
    let c1 = map_count(last)?;
    let mut copy = layout.clone();
    copy.max_sfb = [c0, c1];
    copy.max_sfb_side = [c0, c1];
    copy.side_limited = false;
    copy.dual_maxsfb = false;
    check_bands(&copy)?;
    Ok(copy)
}

pub struct Noise {
    a: u32,
    b: u32,
    current: u32,
    state: u32,
}

impl Noise {
    pub fn new(sequence_counter: u32) -> Self {
        let start = 255 * (sequence_counter % 256);
        let a = start % 255;
        let b = (start / 255) % 256;
        let mut state = (a + 1) * ((a + 2) + 2 * b) / 2 + 255 * b * (255 + b) / 2;
        if start % 130560 >= 65280 {
            state += 128;
        }
        Self {
            a,
            b,
            state: state % 256,
            current: (a * (a + 1) / 2 + b * 32386) % 256,
        }
    }

    pub fn take(&mut self, count: usize, out: &mut [f64]) {
        for slot in out.iter_mut().take(count) {
            *slot = RANDOM_NOISE_TABLE[self.current as usize] + RANDOM_NOISE_TABLE[self.state as usize];
            self.a = (self.a + 1) & 0xFF;
            self.state = (self.state + 1) & 0xFF;
            if self.a == 255 {
                self.current = (self.current + 1) & 0xFF;
                self.b = (self.b + 1) & 0xFF;
                self.a = 0;
            }
            self.current = (self.current + self.a) & 0xFF;
            self.state = (self.state + self.b + self.a) & 0xFF;
        }
    }
}

fn symbol_tuple(cb: usize, idx: i32) -> [i32; 4] {
    let (dim, _, m, off) = CODEBOOK_META[cb];
    if dim == 4 {
        let q1 = idx / (m * m * m) - off;
        let r = idx - (q1 + off) * m * m * m;
        let q2 = r / (m * m) - off;
        let r = r - (q2 + off) * m * m;
        let q3 = r / m - off;
        let r = r - (q3 + off) * m;
        [q1, q2, q3, r - off]
    } else {
        let q1 = idx / m - off;
        [q1, idx - (q1 + off) * m - off, 0, 0]
    }
}

fn ext_decode(b: &mut BitReader<'_>) -> Result<i32> {
    let mut n = 0u32;
    while b.read(1)? != 0 {
        n += 1;
    }
    Ok((1i32 << (n + 4)) + b.read(n + 4)? as i32)
}

fn read_lines(
    b: &mut BitReader<'_>,
    cb: usize,
    out: &mut [i64],
    start: usize,
    stop: usize,
) -> Result<()> {
    let (dim, unsigned, _, _) = CODEBOOK_META[cb];
    let hcb = huffman(asf_book(cb));
    let mut k = start;
    if !unsigned {
        let count = (stop - start) / dim;
        for _ in 0..count {
            let sym = hcb.decode(b)?;
            let lines = symbol_tuple(cb, sym);
            for d in 0..dim {
                out[k + d] = i64::from(lines[d]);
            }
            k += dim;
        }
        return Ok(());
    }
    if cb != ESCAPE_CODEBOOK {
        let count = (stop - start).div_ceil(dim);
        for _ in 0..count {
            let sym = hcb.decode(b)?;
            let lines = symbol_tuple(cb, sym);
            for d in 0..dim {
                let mut q = lines[d];
                if q != 0 && b.flag()? {
                    q = -q;
                }
                if k + d < out.len() {
                    out[k + d] = i64::from(q);
                }
            }
            k += dim;
        }
        return Ok(());
    }
    // 确认 cb == 11 的规模足够校验符号索引
    let _cb_len = codebook("ASF_HCB_11").map_or(0, |c| c.lengths.len());
    while k < stop {
        let sym = hcb.decode(b)?;
        let mut lines = symbol_tuple(cb, sym);
        for d in 0..dim {
            if lines[d] != 0 && b.flag()? {
                lines[d] = -lines[d];
            }
        }
        for d in 0..dim {
            let q = lines[d];
            let val = if q.abs() == 16 {
                let ext = ext_decode(b)?;
                if q > 0 { ext } else { -ext }
            } else {
                q
            };
            if k + d < out.len() {
                out[k + d] = i64::from(val);
            }
        }
        k += dim;
    }
    Ok(())
}

pub fn read_spectrum(
    b: &mut BitReader<'_>,
    layout: &Layout,
    noise: &mut Noise,
    side: bool,
) -> Result<Vec<f64>> {
    let groups = layout.groups();
    let bands: Vec<usize> = (0..groups).map(|g| layout.band_count(g, side)).collect();

    let mut order = Vec::with_capacity(layout.frame_len);
    let mut offsets: Vec<Vec<usize>> = Vec::with_capacity(groups);
    let mut group_offset = 0usize;
    let mut win_start = 0usize;
    for g in 0..groups {
        let count = layout.group_windows[g];
        let n = layout.group_lengths[g];
        let max_sfb = bands[g];
        let sfb = sfb_offsets(n).unwrap();
        let mut g_off = Vec::with_capacity(max_sfb + 1);
        for s in 0..=max_sfb {
            g_off.push(group_offset + usize::from(sfb[s]) * count);
        }
        offsets.push(g_off);
        for s in 0..max_sfb {
            let s0 = usize::from(sfb[s]);
            let s1 = usize::from(sfb[s + 1]);
            for w in 0..count {
                let base = win_start + n * w;
                for line in s0..s1 {
                    order.push(base + line);
                }
            }
        }
        group_offset += usize::from(sfb[max_sfb]) * count;
        win_start += n * count;
    }
    let total = offsets.last().and_then(|v| v.last()).copied().unwrap_or(0);

    // asf_section_data
    let mut sections: Vec<Vec<(usize, usize, usize)>> = Vec::with_capacity(groups);
    let mut band_cb: Vec<Vec<usize>> = Vec::with_capacity(groups);
    for g in 0..groups {
        let bits = if layout.group_transform[g] <= 2 { 3u32 } else { 5u32 };
        let escape = (1usize << bits) - 1;
        let mut k = 0usize;
        let mut found = Vec::new();
        let mut cbs = Vec::with_capacity(bands[g]);
        while k < bands[g] {
            let cb = b.read(4)? as usize;
            if cb > 11 {
                return fail(format!("reserved section codebook {cb}"));
            }
            let mut length = 1usize;
            let mut incr = b.read(bits)? as usize;
            while incr == escape {
                length += escape;
                incr = b.read(bits)? as usize;
            }
            length += incr;
            if k + length > bands[g] {
                return fail("a section runs past max_sfb");
            }
            found.push((cb, k, k + length));
            cbs.resize(k + length, cb);
            k += length;
        }
        sections.push(found);
        band_cb.push(cbs);
    }

    // asf_spectral_data
    let mut quant = vec![0i64; total];
    for g in 0..groups {
        let num48 = num_sfb_48(layout.group_lengths[g]);
        for &(cb, s0, s1) in &sections[g] {
            let s1 = s1.min(num48);
            if cb != 0 && s0 < s1 {
                read_lines(b, cb, &mut quant, offsets[g][s0], offsets[g][s1])?;
            }
        }
    }

    let mut starts = Vec::new();
    for g in 0..groups {
        for s in 0..bands[g] {
            starts.push(offsets[g][s]);
        }
    }
    let num_starts = starts.len();
    let mut widths = Vec::with_capacity(num_starts);
    let mut peak = Vec::with_capacity(num_starts);
    for i in 0..num_starts {
        let s = starts[i];
        let e = if i + 1 < num_starts { starts[i + 1] } else { total };
        widths.push(e - s);
        let mut pk = 0i64;
        for &q in &quant[s..e] {
            pk = pk.max(q.abs());
        }
        peak.push(pk);
    }

    // asf_scalefac_data
    let mut gains = vec![0.0f64; num_starts];
    let mut scale = b.read(8)? as i32;
    let mut first = true;
    let mut i = 0usize;
    let sf_hcb = huffman(Book::AsfScalefac);
    for g in 0..groups {
        let num48 = num_sfb_48(layout.group_lengths[g]);
        for s in 0..bands[g] {
            if s < num48 && band_cb[g][s] != 0 && peak[i] != 0 {
                if first {
                    first = false;
                } else {
                    scale += sf_hcb.decode(b)? - 60;
                }
                gains[i] = 2.0f64.powf(0.25 * f64::from(scale - 100));
            }
            i += 1;
        }
    }

    // asf_snf_data
    let mut snf: Vec<Option<i32>> = vec![None; num_starts];
    let mut has_snf = false;
    if b.flag()? {
        has_snf = true;
        let snf_hcb = huffman(Book::AsfSnf);
        let mut idx = 0usize;
        for g in 0..groups {
            let num48 = num_sfb_48(layout.group_lengths[g]);
            for s in 0..bands[g] {
                if s < num48 && (band_cb[g][s] == 0 || peak[idx] == 0) {
                    snf[idx] = Some(snf_hcb.decode(b)?);
                }
                idx += 1;
            }
        }
    }

    let mut scaled = vec![0.0f64; total];
    for idx in 0..num_starts {
        let s = starts[idx];
        let w = widths[idx];
        let g = gains[idx];
        for j in s..s + w {
            let q = quant[j];
            if q != 0 {
                let sign = if q > 0 { 1.0 } else { -1.0 };
                scaled[j] = sign * (q.abs() as f64).powf(4.0 / 3.0) * g;
            }
        }
    }

    if has_snf {
        noise_fill(&mut scaled, &starts, &widths, &snf, noise);
    }

    let mut spectrum = vec![0.0f64; layout.frame_len];
    for (&dst, &val) in order.iter().zip(scaled.iter()) {
        spectrum[dst] = val;
    }
    Ok(spectrum)
}

fn noise_fill(
    scaled: &mut [f64],
    starts: &[usize],
    widths: &[usize],
    snf: &[Option<i32>],
    noise: &mut Noise,
) {
    let mut energy = Vec::with_capacity(starts.len());
    let mut previous = -1000.0f64;
    let mut found_prev = false;
    for (&s, &w) in starts.iter().zip(widths.iter()) {
        let mut e = 0.0f64;
        for &v in &scaled[s..s + w] {
            e += v * v;
        }
        energy.push(e);
        if !found_prev && e > 0.0 {
            previous = std::f64::consts::LOG2_E * (e / w as f64).ln();
            found_prev = true;
        }
    }
    let mut buf = Vec::new();
    for (i, (&s, &w)) in starts.iter().zip(widths.iter()).enumerate() {
        if energy[i] > 0.0 {
            previous = std::f64::consts::LOG2_E * (energy[i] / w as f64).ln();
        } else if let Some(val) = snf[i] {
            if val != 0 {
                let level = previous + f64::from(val - 17);
                previous = level;
                buf.resize(w, 0.0);
                noise.take(w, &mut buf);
                let scale = 2.0f64.powf(0.5 * level);
                for j in 0..w {
                    scaled[s + j] = buf[j] * scale;
                }
            }
        }
    }
}
