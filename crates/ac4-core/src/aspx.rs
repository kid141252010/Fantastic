//! 高级频谱扩展 A-SPX（TS 103 190-1 clauses 4.2.12、4.3.10、5.7.6）与压扩（5.7.5）。

use crate::bits::BitReader;
use crate::codes::{huffman, Book};
use crate::qmf::{QmfMatrix, BANDS, C64};
use crate::tables::ASPX_NOISE;
use crate::{fail, Result};

pub const FIXFIX: u8 = 0;
pub const FIXVAR: u8 = 1;
pub const VARFIX: u8 = 2;
pub const VARVAR: u8 = 3;

const SIGNAL: u8 = 0;
const NOISE: u8 = 1;
const LEVEL: u8 = 0;
const BALANCE: u8 = 1;

pub const SUBBANDS: usize = BANDS;
pub const HFGEN_ATS: usize = 3;
pub const HFADJ: usize = 4;

const TEMPLATE_LOW: [usize; 21] = [
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 22, 24, 26, 28, 30, 32, 35, 38, 42, 46,
];
const TEMPLATE_HIGH: [usize; 23] = [
    18, 19, 20, 21, 22, 23, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 47, 50, 53, 56, 59, 62,
];

const NEW_CHIRP: [[f64; 4]; 4] = [
    [0.0, 0.6, 0.9, 0.98],
    [0.6, 0.75, 0.9, 0.98],
    [0.0, 0.75, 0.9, 0.98],
    [0.0, 0.75, 0.9, 0.98],
];
const SINE_RE: [f64; 4] = [1.0, 0.0, -1.0, 0.0];
const SINE_IM: [f64; 4] = [0.0, 1.0, 0.0, -1.0];

fn tab_border(slots: usize, n: usize) -> Option<&'static [usize]> {
    match (slots, n) {
        (6, 1) => Some(&[0, 6]),
        (6, 2) => Some(&[0, 3, 6]),
        (6, 4) => Some(&[0, 2, 3, 4, 6]),
        (8, 1) => Some(&[0, 8]),
        (8, 2) => Some(&[0, 4, 8]),
        (8, 4) => Some(&[0, 2, 4, 6, 8]),
        (12, 1) => Some(&[0, 12]),
        (12, 2) => Some(&[0, 6, 12]),
        (12, 4) => Some(&[0, 3, 6, 9, 12]),
        (15, 1) => Some(&[0, 15]),
        (15, 2) => Some(&[0, 8, 15]),
        (15, 4) => Some(&[0, 4, 8, 12, 15]),
        (16, 1) => Some(&[0, 16]),
        (16, 2) => Some(&[0, 8, 16]),
        (16, 4) => Some(&[0, 4, 8, 12, 16]),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AspxConfig {
    pub quant_mode_env: u32,
    pub start_freq: usize,
    pub stop_freq: usize,
    pub master_freq_scale: usize,
    pub interpolation: u32,
    pub preflat: u32,
    pub limiter: u32,
    pub noise_sbg: usize,
    pub num_env_bits_fixfix: u32,
    pub freq_res_mode: u32,
}

impl AspxConfig {
    pub fn master(&self) -> (usize, usize, usize) {
        (self.start_freq, self.stop_freq, self.master_freq_scale)
    }
}

pub fn read_config(b: &mut BitReader<'_>) -> Result<AspxConfig> {
    Ok(AspxConfig {
        quant_mode_env: b.read(1)?,
        start_freq: b.read(3)? as usize,
        stop_freq: b.read(2)? as usize,
        master_freq_scale: b.read(1)? as usize,
        interpolation: b.read(1)?,
        preflat: b.read(1)?,
        limiter: b.read(1)?,
        noise_sbg: b.read(2)? as usize,
        num_env_bits_fixfix: b.read(1)?,
        freq_res_mode: b.read(2)?,
    })
}

#[derive(Clone, Debug)]
pub struct Bands {
    pub master: Vec<usize>,
    pub high: Vec<usize>,
    pub low: Vec<usize>,
    pub noise: Vec<usize>,
    pub patch_count: Vec<usize>,
    pub patch_start: Vec<usize>,
    pub patches: Vec<usize>,
    pub limiter: Vec<usize>,
    pub patch_src: Vec<usize>,
    pub patch_dst: Vec<usize>,
    pub patch_grp: Vec<usize>,
}

impl Bands {
    pub fn sba(&self) -> usize {
        self.master[0]
    }
    pub fn sbx(&self) -> usize {
        self.high[0]
    }
    pub fn width(&self) -> usize {
        self.high.last().unwrap() - self.high[0]
    }
}

pub fn build_bands(
    config: (usize, usize, usize),
    noise_sbg: usize,
    xover: usize,
    base_48k: bool,
) -> Result<Bands> {
    let (start, stop, scale) = config;
    let (template, total): (&[usize], usize) = if scale != 0 {
        (&TEMPLATE_HIGH, 22)
    } else {
        (&TEMPLATE_LOW, 20)
    };
    let count = total as isize - 2 * start as isize - 2 * stop as isize;
    if count <= 0 {
        return fail(format!("A-SPX start {start} and stop {stop} leave no subband groups"));
    }
    let count = count as usize;
    let master = template[2 * start..2 * start + count + 1].to_vec();
    let sba = master[0];
    let sbz = *master.last().unwrap();
    if xover >= count {
        return fail(format!("A-SPX crossover offset {xover} past {count} groups"));
    }
    let high = master[xover..].to_vec();
    let sbx = high[0];
    let n_high = high.len() - 1;
    let n_low = n_high - n_high / 2;
    let mut low = Vec::with_capacity(n_low + 1);
    low.push(high[0]);
    for s in 1..=n_low {
        low.push(if n_high % 2 == 0 { high[2 * s] } else { high[2 * s - 1] });
    }
    let n_noise = (((noise_sbg as f64) * (sbz as f64 / sbx as f64).log2() + 0.5).floor() as isize)
        .max(1) as usize;
    if n_noise > 5 {
        return fail(format!("{n_noise} A-SPX noise groups"));
    }
    let mut idx = vec![0usize];
    for s in 1..=n_noise {
        let last = *idx.last().unwrap();
        idx.push(last + (n_low - last) / (n_noise + 1 - s));
    }
    let noise: Vec<usize> = idx.iter().map(|&i| low[i]).collect();

    // Pseudocode 71
    let mut msb = sba;
    let mut usb = sbx;
    let goal = if base_48k { 43usize } else { 46usize };
    let source_low = if scale != 0 { 4usize } else { 2usize };
    let width = *high.last().unwrap() - sbx;
    let mut sbg = if goal < sbx + width {
        let mut found = 0usize;
        for (i, &value) in master.iter().enumerate() {
            if value >= goal {
                break;
            }
            found = i + 1;
        }
        found
    } else {
        count
    };

    let mut counts = Vec::new();
    let mut starts = Vec::new();
    loop {
        let mut j = sbg as isize;
        let mut sb = master[j as usize];
        let mut odd = (sb - 2 + sba) % 2;
        while sb as isize > sba as isize - source_low as isize + msb as isize - odd as isize {
            j -= 1;
            if j < 0 {
                return fail("A-SPX patches cannot be built");
            }
            sb = master[j as usize];
            odd = (sb - 2 + sba) % 2;
        }
        let cnt = sb.saturating_sub(usb);
        counts.push(cnt);
        starts.push(sba - odd - cnt);
        if cnt > 0 {
            usb = sb;
            msb = sb;
        } else {
            counts.pop();
            starts.pop();
            msb = sbx;
        }
        if master[sbg] - sb < 3 {
            sbg = count;
        }
        if sb == sbx + width {
            break;
        }
        if counts.len() > 8 {
            return fail("A-SPX patches do not reach the top");
        }
    }
    if counts.len() > 1 && *counts.last().unwrap() < 3 {
        counts.pop();
        starts.pop();
    }
    let mut patches = vec![sbx];
    for &n in &counts {
        patches.push(patches.last().unwrap() + n);
    }

    // Pseudocode 72
    let mut lim = low.clone();
    if patches.len() > 2 {
        lim.extend_from_slice(&patches[1..patches.len() - 1]);
    }
    lim.sort_unstable();
    let mut s = 1usize;
    while s < lim.len() {
        if (lim[s] as f64 / lim[s - 1] as f64).log2() < 0.245 {
            if lim[s] == lim[s - 1] {
                lim.remove(s);
            } else if patches.contains(&lim[s]) {
                if patches.contains(&lim[s - 1]) {
                    s += 1;
                } else {
                    lim.remove(s - 1);
                }
            } else {
                lim.remove(s);
            }
        } else {
            s += 1;
        }
    }

    let mut patch_src = Vec::new();
    let mut patch_dst = Vec::new();
    let mut sb_high = sbx;
    for (&cnt, &st) in counts.iter().zip(starts.iter()) {
        for i in 0..cnt {
            if sb_high + i >= SUBBANDS {
                break;
            }
            patch_src.push(st + i);
            patch_dst.push(sb_high + i);
        }
        sb_high += cnt;
    }
    let mut patch_grp = Vec::with_capacity(patch_dst.len());
    for &tgt in &patch_dst {
        let pos = noise.partition_point(|&v| v <= tgt);
        let g = pos.saturating_sub(1).min(noise.len() - 2);
        patch_grp.push(g);
    }

    Ok(Bands {
        master,
        high,
        low,
        noise,
        patch_count: counts,
        patch_start: starts,
        patches,
        limiter: lim,
        patch_src,
        patch_dst,
        patch_grp,
    })
}

#[derive(Clone, Debug)]
pub struct AspxFrame {
    pub int_class: u8,
    pub num_env: usize,
    pub num_noise: usize,
    pub tsg_ptr: isize,
    pub qmode: u32,
    pub sig_borders: Vec<usize>,
    pub noise_borders: Vec<usize>,
    pub freq_res: Vec<usize>,
    pub sig_delta: Vec<u32>,
    pub noise_delta: Vec<u32>,
    pub tna: Vec<usize>,
    pub harmonics: Vec<bool>,
    pub fic: Vec<bool>,
    pub tic: Vec<bool>,
    pub sig: Vec<Vec<i32>>,
    pub noise: Vec<Vec<i32>>,
    pub balance: bool,
    pub second: bool,
}

impl Default for AspxFrame {
    fn default() -> Self {
        Self {
            int_class: FIXFIX,
            num_env: 1,
            num_noise: 1,
            tsg_ptr: -1,
            qmode: 0,
            sig_borders: Vec::new(),
            noise_borders: Vec::new(),
            freq_res: Vec::new(),
            sig_delta: Vec::new(),
            noise_delta: Vec::new(),
            tna: Vec::new(),
            harmonics: Vec::new(),
            fic: Vec::new(),
            tic: Vec::new(),
            sig: Vec::new(),
            noise: Vec::new(),
            balance: false,
            second: false,
        }
    }
}

pub struct AspxChannel {
    pub slots: usize,
    pub ts_in_ats: usize,
    pub aspx_slots: usize,
    pub hfgen: usize,
    pub config: Option<AspxConfig>,
    pub xover: usize,
    pub bands: Option<Bands>,
    pub master_reset: bool,
    pub previous_stop: usize,
    pub prev_num_env: usize,
    pub prev_freq_res: Vec<usize>,
    pub prev_sig: [f64; 32],
    pub prev_noise: [f64; 8],
    pub prev_num_noise: usize,
    pub prev_tna: Vec<usize>,
    pub prev_chirp: [f64; 8],
    pub prev_tsg_ptr: isize,
    pub prev_sine: [bool; SUBBANDS],
    pub noise_cursor: usize,
    pub sine_cursor: usize,
    pub first: bool,
    pub q_prev: QmfMatrix,
    pub y_prev: QmfMatrix,
    pub frame: Option<AspxFrame>,
}

impl AspxChannel {
    pub fn new(frame_len: usize) -> Self {
        let slots = frame_len / SUBBANDS;
        let ts_in_ats = if frame_len >= 1536 { 2 } else { 1 };
        let aspx_slots = slots / ts_in_ats;
        let hfgen = HFGEN_ATS * ts_in_ats;
        Self {
            slots,
            ts_in_ats,
            aspx_slots,
            hfgen,
            config: None,
            xover: 0,
            bands: None,
            master_reset: true,
            previous_stop: aspx_slots,
            prev_num_env: 1,
            prev_freq_res: vec![0],
            prev_sig: [0.0; 32],
            prev_noise: [0.0; 8],
            prev_num_noise: 1,
            prev_tna: vec![0; 8],
            prev_chirp: [0.0; 8],
            prev_tsg_ptr: -1,
            prev_sine: [false; SUBBANDS],
            noise_cursor: 0,
            sine_cursor: 0,
            first: true,
            q_prev: QmfMatrix::zeros(slots),
            y_prev: QmfMatrix::zeros(slots + hfgen),
            frame: None,
        }
    }

    pub fn configure(
        &mut self,
        config: AspxConfig,
        base_48k: bool,
        xover: Option<usize>,
    ) -> Result<()> {
        if self.config.is_none_or(|c| c.master() != config.master()) {
            self.master_reset = true;
        }
        self.config = Some(config);
        if let Some(x) = xover {
            self.xover = x;
        }
        self.bands = Some(build_bands(config.master(), config.noise_sbg, self.xover, base_48k)?);
        Ok(())
    }

    fn framing(&mut self, b: &mut BitReader<'_>, iframe: bool) -> Result<AspxFrame> {
        let config = self.config.unwrap();
        let mut f = AspxFrame::default();
        if !b.flag()? {
            f.int_class = FIXFIX;
        } else if !b.flag()? {
            f.int_class = FIXVAR;
        } else {
            f.int_class = if b.flag()? { VARVAR } else { VARFIX };
        }
        let rel_bits = if self.aspx_slots > 8 { 2u32 } else { 1u32 };
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut var_left = 0usize;
        let mut var_right = 0usize;
        let mut transmitted = Vec::new();
        if f.int_class == FIXFIX {
            f.num_env = 1usize << b.read(config.num_env_bits_fixfix + 1)?;
            if f.num_env > 4 {
                return fail("8 A-SPX envelopes");
            }
            if config.freq_res_mode == 0 {
                transmitted.push(b.read(1)? as usize);
            }
        } else {
            if f.int_class == VARFIX || f.int_class == VARVAR {
                if iframe {
                    var_left = b.read(2)? as usize;
                }
                let n = b.read(rel_bits)? as usize;
                for _ in 0..n {
                    left.push(2 * b.read(rel_bits)? as usize + 2);
                }
            }
            if f.int_class == FIXVAR || f.int_class == VARVAR {
                var_right = b.read(2)? as usize;
                let n = b.read(rel_bits)? as usize;
                for _ in 0..n {
                    right.push(2 * b.read(rel_bits)? as usize + 2);
                }
            }
            f.num_env = left.len() + right.len() + 1;
            if f.num_env > 5 {
                return fail(format!("{} A-SPX envelopes", f.num_env));
            }
            let bits_needed = (f.num_env + 2).next_power_of_two().trailing_zeros();
            f.tsg_ptr = b.read(bits_needed)? as isize - 1;
            if config.freq_res_mode == 0 {
                for _ in 0..f.num_env {
                    transmitted.push(b.read(1)? as usize);
                }
            }
        }
        f.num_noise = if f.num_env > 1 { 2 } else { 1 };
        self.borders(&mut f, iframe, &left, &right, var_left, var_right, &transmitted)?;
        Ok(f)
    }

    fn borders(
        &mut self,
        f: &mut AspxFrame,
        iframe: bool,
        left: &[usize],
        right: &[usize],
        var_left: usize,
        var_right: usize,
        transmitted: &[usize],
    ) -> Result<()> {
        let n = f.num_env;
        let slots = self.aspx_slots;
        let (sig, noise) = if f.int_class == FIXFIX {
            let s = tab_border(slots, n)
                .ok_or_else(|| crate::Error::new(format!("unsupported A-SPX border ({slots}, {n})")))?
                .to_vec();
            let ns = tab_border(slots, f.num_noise)
                .ok_or_else(|| crate::Error::new("unsupported A-SPX noise border"))?
                .to_vec();
            (s, ns)
        } else {
            let mut sig = vec![0isize; n + 1];
            sig[0] = if f.int_class == FIXVAR {
                0
            } else if iframe {
                var_left as isize
            } else {
                self.previous_stop as isize - slots as isize
            };
            sig[n] = if f.int_class == VARFIX {
                slots as isize
            } else {
                (var_right + slots) as isize
            };
            for (k, &step) in left.iter().enumerate() {
                sig[k + 1] = sig[k] + step as isize;
            }
            for (k, &step) in right.iter().enumerate() {
                sig[n - k - 1] = sig[n - k] - step as isize;
            }
            for w in sig.windows(2) {
                if w[1] <= w[0] {
                    return fail(format!("A-SPX borders {:?} do not rise", sig));
                }
            }
            if sig[0] < 0 {
                return fail(format!("A-SPX borders {:?} negative start", sig));
            }
            let sig_u: Vec<usize> = sig.iter().map(|&v| v as usize).collect();
            let mut noise = vec![sig_u[0]; f.num_noise + 1];
            noise[f.num_noise] = sig_u[n];
            if f.num_noise > 1 {
                let mid = if f.tsg_ptr == -1 {
                    if f.int_class == VARFIX { 1 } else { n - 1 }
                } else if f.int_class == VARFIX {
                    n - 1
                } else {
                    (f.tsg_ptr as usize).clamp(1, n - 1)
                };
                noise[1] = sig_u[mid];
            }
            (sig_u, noise)
        };
        if sig.windows(2).any(|w| w[1] <= w[0]) {
            return fail(format!("A-SPX borders {:?} do not rise", sig));
        }

        let mode = self.config.unwrap().freq_res_mode;
        let mut res = Vec::with_capacity(n);
        for k in 0..n {
            match mode {
                0 => res.push(if f.int_class == FIXFIX { transmitted[0] } else { transmitted[k] }),
                1 => res.push(0),
                2 => {
                    let long = (sig[k + 1] - sig[k]) as f64 > slots as f64 / 6.0 + 3.25;
                    res.push(usize::from(((k as isize) < f.tsg_ptr && slots > 8) || long));
                }
                _ => res.push(1),
            }
        }
        if f.int_class == FIXFIX {
            let r0 = res[0];
            res.fill(r0);
        }
        self.previous_stop = sig[n];
        f.sig_borders = sig;
        f.noise_borders = noise;
        f.freq_res = res;
        Ok(())
    }

    fn ec_data(
        &self,
        b: &mut BitReader<'_>,
        f: &AspxFrame,
        data_type: u8,
        stereo: u8,
    ) -> Result<Vec<Vec<i32>>> {
        let bands = self.bands.as_ref().unwrap();
        let count = if data_type == SIGNAL { f.num_env } else { f.num_noise };
        let mut out = Vec::with_capacity(count);
        for env in 0..count {
            let (num, direction) = if data_type == SIGNAL {
                let n = if f.freq_res[env] != 0 {
                    bands.high.len() - 1
                } else {
                    bands.low.len() - 1
                };
                (n, f.sig_delta[env])
            } else {
                (bands.noise.len() - 1, f.noise_delta[env])
            };
            let mut values = Vec::with_capacity(num);
            if direction == 0 {
                let (book_f0, _) = aspx_book(data_type, f.qmode, stereo, 0);
                values.push(huffman(book_f0).decode(b)?);
                let (book_df, off_df) = aspx_book(data_type, f.qmode, stereo, 1);
                let hdf = huffman(book_df);
                for _ in 1..num {
                    values.push(hdf.decode(b)? - off_df);
                }
            } else {
                let (book_dt, off_dt) = aspx_book(data_type, f.qmode, stereo, 2);
                let hdt = huffman(book_dt);
                for _ in 0..num {
                    values.push(hdt.decode(b)? - off_dt);
                }
            }
            out.push(values);
        }
        Ok(out)
    }

    fn delta_dir(&self, b: &mut BitReader<'_>, f: &mut AspxFrame) -> Result<()> {
        f.sig_delta = Vec::with_capacity(f.num_env);
        for _ in 0..f.num_env {
            f.sig_delta.push(b.read(1)?);
        }
        f.noise_delta = Vec::with_capacity(f.num_noise);
        for _ in 0..f.num_noise {
            f.noise_delta.push(b.read(1)?);
        }
        Ok(())
    }
}

fn aspx_book(data_type: u8, qmode: u32, stereo: u8, kind: u8) -> (Book, i32) {
    // kind: 0 = F0, 1 = DF, 2 = DT
    match (data_type, stereo, qmode != 0, kind) {
        (SIGNAL, LEVEL, false, 0) => (Book::AspxEnvLevel15F0, 0),
        (SIGNAL, LEVEL, false, 1) => (Book::AspxEnvLevel15Df, 70),
        (SIGNAL, LEVEL, false, 2) => (Book::AspxEnvLevel15Dt, 70),
        (SIGNAL, LEVEL, true, 0) => (Book::AspxEnvLevel30F0, 0),
        (SIGNAL, LEVEL, true, 1) => (Book::AspxEnvLevel30Df, 35),
        (SIGNAL, LEVEL, true, 2) => (Book::AspxEnvLevel30Dt, 35),
        (SIGNAL, BALANCE, false, 0) => (Book::AspxEnvBalance15F0, 0),
        (SIGNAL, BALANCE, false, 1) => (Book::AspxEnvBalance15Df, 24),
        (SIGNAL, BALANCE, false, 2) => (Book::AspxEnvBalance15Dt, 24),
        (SIGNAL, BALANCE, true, 0) => (Book::AspxEnvBalance30F0, 0),
        (SIGNAL, BALANCE, true, 1) => (Book::AspxEnvBalance30Df, 12),
        (SIGNAL, BALANCE, true, 2) => (Book::AspxEnvBalance30Dt, 12),
        (NOISE, LEVEL, _, 0) => (Book::AspxNoiseLevelF0, 0),
        (NOISE, LEVEL, _, 1) => (Book::AspxNoiseLevelDf, 29),
        (NOISE, LEVEL, _, 2) => (Book::AspxNoiseLevelDt, 29),
        (NOISE, BALANCE, _, 0) => (Book::AspxNoiseBalanceF0, 0),
        (NOISE, BALANCE, _, 1) => (Book::AspxNoiseBalanceDf, 12),
        (NOISE, BALANCE, _, 2) => (Book::AspxNoiseBalanceDt, 12),
        _ => unreachable!(),
    }
}

fn qmode(config: AspxConfig, f: &AspxFrame) -> u32 {
    if f.int_class == FIXFIX && f.num_env == 1 {
        0
    } else {
        config.quant_mode_env
    }
}

fn check_ready(ch: &AspxChannel, config: Option<AspxConfig>, iframe: bool) -> Result<()> {
    if config.is_none() || (!iframe && ch.bands.is_none()) {
        return fail("A-SPX data before the I-frame that configures it");
    }
    Ok(())
}

pub fn read_data_1ch(
    b: &mut BitReader<'_>,
    ch: &mut AspxChannel,
    config: Option<AspxConfig>,
    iframe: bool,
    base_48k: bool,
) -> Result<AspxFrame> {
    check_ready(ch, config, iframe)?;
    let cfg = config.unwrap();
    if iframe {
        let xover = b.read(3)? as usize;
        ch.configure(cfg, base_48k, Some(xover))?;
    }
    let nb = ch.bands.as_ref().unwrap();
    let n_noise = nb.noise.len() - 1;
    let n_high = nb.high.len() - 1;
    let mut f = ch.framing(b, iframe)?;
    f.qmode = qmode(ch.config.unwrap(), &f);
    ch.delta_dir(b, &mut f)?;
    f.tna = Vec::with_capacity(n_noise);
    for _ in 0..n_noise {
        f.tna.push(b.read(2)? as usize);
    }
    f.harmonics = if b.flag()? {
        let mut v = Vec::with_capacity(n_high);
        for _ in 0..n_high {
            v.push(b.flag()?);
        }
        v
    } else {
        vec![false; n_high]
    };
    f.fic = if b.flag()? {
        let mut v = Vec::with_capacity(n_high);
        for _ in 0..n_high {
            v.push(b.flag()?);
        }
        v
    } else {
        vec![false; n_high]
    };
    f.tic = if b.flag()? {
        let mut v = Vec::with_capacity(ch.aspx_slots);
        for _ in 0..ch.aspx_slots {
            v.push(b.flag()?);
        }
        v
    } else {
        vec![false; ch.aspx_slots]
    };
    f.sig = ch.ec_data(b, &f, SIGNAL, LEVEL)?;
    f.noise = ch.ec_data(b, &f, NOISE, LEVEL)?;
    ch.frame = Some(f.clone());
    Ok(f)
}

pub fn read_data_2ch(
    b: &mut BitReader<'_>,
    a: &mut AspxChannel,
    c: &mut AspxChannel,
    config: Option<AspxConfig>,
    iframe: bool,
    base_48k: bool,
) -> Result<(AspxFrame, AspxFrame)> {
    check_ready(a, config, iframe)?;
    check_ready(c, config, iframe)?;
    let cfg = config.unwrap();
    if iframe {
        let xover = b.read(3)? as usize;
        a.configure(cfg, base_48k, Some(xover))?;
        c.configure(cfg, base_48k, Some(xover))?;
    }
    let nb = a.bands.as_ref().unwrap();
    let n_noise = nb.noise.len() - 1;
    let n_high = nb.high.len() - 1;
    let mut f0 = a.framing(b, iframe)?;
    f0.qmode = qmode(a.config.unwrap(), &f0);
    let balance = b.flag()?;
    let mut f1 = if !balance {
        let mut f = c.framing(b, iframe)?;
        f.qmode = qmode(c.config.unwrap(), &f);
        f
    } else {
        c.previous_stop = a.previous_stop;
        AspxFrame {
            int_class: f0.int_class,
            num_env: f0.num_env,
            num_noise: f0.num_noise,
            tsg_ptr: f0.tsg_ptr,
            qmode: f0.qmode,
            sig_borders: f0.sig_borders.clone(),
            noise_borders: f0.noise_borders.clone(),
            freq_res: f0.freq_res.clone(),
            ..AspxFrame::default()
        }
    };
    f0.balance = balance;
    f1.balance = balance;
    f1.second = true;
    a.delta_dir(b, &mut f0)?;
    c.delta_dir(b, &mut f1)?;

    f0.tna = Vec::with_capacity(n_noise);
    for _ in 0..n_noise {
        f0.tna.push(b.read(2)? as usize);
    }
    f1.tna = if balance {
        f0.tna.clone()
    } else {
        let mut v = Vec::with_capacity(n_noise);
        for _ in 0..n_noise {
            v.push(b.read(2)? as usize);
        }
        v
    };
    f0.harmonics = if b.flag()? {
        let mut v = Vec::with_capacity(n_high);
        for _ in 0..n_high {
            v.push(b.flag()?);
        }
        v
    } else {
        vec![false; n_high]
    };
    f1.harmonics = if b.flag()? {
        let mut v = Vec::with_capacity(n_high);
        for _ in 0..n_high {
            v.push(b.flag()?);
        }
        v
    } else {
        vec![false; n_high]
    };
    f0.fic = vec![false; n_high];
    f1.fic = vec![false; n_high];
    if b.flag()? {
        if b.flag()? {
            for slot in &mut f0.fic {
                *slot = b.flag()?;
            }
        }
        if b.flag()? {
            for slot in &mut f1.fic {
                *slot = b.flag()?;
            }
        }
    }
    f0.tic = vec![false; a.aspx_slots];
    f1.tic = vec![false; a.aspx_slots];
    if b.flag()? {
        let copy = b.flag()?;
        let (left, right) = if !copy {
            (b.flag()?, b.flag()?)
        } else {
            (false, false)
        };
        if copy || left {
            for slot in &mut f0.tic {
                *slot = b.flag()?;
            }
        }
        if right {
            for slot in &mut f1.tic {
                *slot = b.flag()?;
            }
        }
        if copy {
            f1.tic = f0.tic.clone();
        }
    }
    f0.sig = a.ec_data(b, &f0, SIGNAL, LEVEL)?;
    f1.sig = c.ec_data(b, &f1, SIGNAL, if balance { BALANCE } else { LEVEL })?;
    f0.noise = a.ec_data(b, &f0, NOISE, LEVEL)?;
    f1.noise = c.ec_data(b, &f1, NOISE, if balance { BALANCE } else { LEVEL })?;
    a.frame = Some(f0.clone());
    c.frame = Some(f1.clone());
    Ok((f0, f1))
}

fn quantized(ch: &AspxChannel, f: &AspxFrame) -> (Vec<[f64; 32]>, Vec<[f64; 8]>) {
    let nb = ch.bands.as_ref().unwrap();
    let high = &nb.high;
    let low = &nb.low;
    let mut high2low = Vec::with_capacity(high.len() - 1);
    let mut low2high = vec![0usize];
    let mut lo = 0usize;
    for s in 0..high.len() - 1 {
        if lo + 1 < low.len() && low[lo + 1] == high[s] {
            lo += 1;
            low2high.push(s);
        }
        high2low.push(lo);
    }
    let delta = if f.second && f.balance { 2.0 } else { 1.0 };
    let mut sig = Vec::with_capacity(f.num_env);
    for env in 0..f.num_env {
        let res = f.freq_res[env];
        let prev_res = if env == 0 {
            *ch.prev_freq_res.last().unwrap()
        } else {
            f.freq_res[env - 1]
        };
        let prev = if env == 0 { &ch.prev_sig } else { &sig[env - 1] };
        let num = if res != 0 { high.len() - 1 } else { low.len() - 1 };
        let mut out = [0.0f64; 32];
        if f.sig_delta[env] == 0 {
            let mut acc = 0.0;
            for (s, &val) in f.sig[env].iter().take(num).enumerate() {
                acc += f64::from(val);
                out[s] = delta * acc;
            }
        } else {
            for s in 0..num {
                let idx = if res == prev_res {
                    s
                } else if res == 0 {
                    low2high[s]
                } else {
                    high2low[s]
                };
                out[s] = prev[idx] + delta * f64::from(f.sig[env][s]);
            }
        }
        sig.push(out);
    }
    let mut noise = Vec::with_capacity(f.num_noise);
    for env in 0..f.num_noise {
        let n = f.noise[env].len();
        let mut out = [0.0f64; 8];
        if f.noise_delta[env] == 0 {
            let mut acc = 0.0;
            for s in 0..n {
                acc += f64::from(f.noise[env][s]);
                out[s] = delta * acc;
            }
        } else {
            let prev = if env == 0 { &ch.prev_noise } else { &noise[env - 1] };
            for s in 0..n {
                out[s] = prev[s] + delta * f64::from(f.noise[env][s]);
            }
        }
        noise.push(out);
    }
    (sig, noise)
}

type QuantItem<'a> = (&'a AspxFrame, Vec<[f64; 32]>, Vec<[f64; 8]>);
type DequantItem = (Vec<[f64; 32]>, Vec<[f64; 8]>);

fn dequantize_all(items: &[QuantItem<'_>]) -> Vec<DequantItem> {
    if items.len() == 2 && items[0].0.balance {
        let fa = items[0].0;
        let a = if fa.qmode == 0 { 2.0 } else { 1.0 };
        let sa = &items[0].1;
        let sb = &items[1].1;
        let na = &items[0].2;
        let nb = &items[1].2;
        let mut sig_a = Vec::with_capacity(sa.len());
        let mut sig_b = Vec::with_capacity(sb.len());
        for (qa, qb) in sa.iter().zip(sb.iter()) {
            let mut oa = [0.0f64; 32];
            let mut ob = [0.0f64; 32];
            for i in 0..32 {
                let nom = 2.0f64.powf(qa[i] / a + 1.0) * SUBBANDS as f64;
                oa[i] = nom / (1.0 + 2.0f64.powf(12.0 - qb[i] / a));
                ob[i] = nom / (1.0 + 2.0f64.powf(qb[i] / a - 12.0));
            }
            sig_a.push(oa);
            sig_b.push(ob);
        }
        let mut noise_a = Vec::with_capacity(na.len());
        let mut noise_b = Vec::with_capacity(nb.len());
        for (qa, qb) in na.iter().zip(nb.iter()) {
            let mut oa = [0.0f64; 8];
            let mut ob = [0.0f64; 8];
            for i in 0..8 {
                let nom = 2.0f64.powf(6.0 - qa[i] + 1.0);
                oa[i] = nom / (1.0 + 2.0f64.powf(12.0 - qb[i]));
                ob[i] = nom / (1.0 + 2.0f64.powf(qb[i] - 12.0));
            }
            noise_a.push(oa);
            noise_b.push(ob);
        }
        return vec![(sig_a, noise_a), (sig_b, noise_b)];
    }

    items
        .iter()
        .map(|(f, sig, noise)| {
            let a = if f.qmode == 0 { 2.0 } else { 1.0 };
            let mut s_out = Vec::with_capacity(sig.len());
            for q in sig {
                let mut o = [0.0f64; 32];
                for i in 0..32 {
                    o[i] = SUBBANDS as f64 * 2.0f64.powf(q[i] / a);
                }
                s_out.push(o);
            }
            let mut n_out = Vec::with_capacity(noise.len());
            for q in noise {
                let mut o = [0.0f64; 8];
                for i in 0..8 {
                    o[i] = 2.0f64.powf(6.0 - q[i]);
                }
                n_out.push(o);
            }
            (s_out, n_out)
        })
        .collect()
}

const COMPAND_ALPHA: f64 = 0.65;

/// 在归一化到 1.0 的 QMF 矩阵上原地执行压扩（clause 5.7.5）。
pub fn compand(
    qs: &mut [&mut QmfMatrix],
    spans: &[(usize, usize, usize, usize)],
    on: &[bool],
    avg: bool,
    sync: bool,
) {
    let mut levels: Vec<Vec<f64>> = Vec::with_capacity(qs.len());
    for (q, &(sb0, sb1, ts0, ts1)) in qs.iter().zip(spans.iter()) {
        let len = ts1.saturating_sub(ts0);
        let mut lv = vec![0.0f64; len];
        if sb1 > sb0 && len > 0 {
            let inv_bands = 1.0 / (sb1 - sb0) as f64;
            for (ti, ts) in (ts0..ts1).enumerate() {
                let mut sum = 0.0;
                for sb in sb0..sb1 {
                    let c = q.get(sb, ts);
                    let re = c.re.abs();
                    let im = c.im.abs();
                    sum += re.max(im) + 0.5 * re.min(im);
                }
                lv[ti] = 0.9105 * sum * inv_bands;
            }
        }
        levels.push(lv);
    }
    let boost = 2.0f64.powf(1.0 / COMPAND_ALPHA);
    let exponent = (1.0 - COMPAND_ALPHA) / COMPAND_ALPHA;

    let per: Vec<Option<Vec<f64>>> = if sync {
        let len = levels.first().map_or(0, |v| v.len());
        let mut gains = vec![0.0f64; len];
        let inv_n = 1.0 / levels.len().max(1) as f64;
        for lv in &levels {
            for (i, &val) in lv.iter().enumerate() {
                gains[i] += val.max(1e-12).powf(exponent) * inv_n;
            }
        }
        if on[0] {
            vec![Some(gains); qs.len()]
        } else if avg {
            let mean = if gains.is_empty() {
                0.0
            } else {
                gains.iter().sum::<f64>() / gains.len() as f64
            };
            vec![Some(vec![mean; len]); qs.len()]
        } else {
            return;
        }
    } else {
        levels
            .iter()
            .enumerate()
            .map(|(k, lv)| {
                let flag = on[k.min(on.len() - 1)];
                if flag {
                    Some(lv.iter().map(|&v| v.max(1e-12).powf(exponent)).collect())
                } else if avg && !lv.is_empty() {
                    let mean = lv.iter().sum::<f64>() / lv.len() as f64;
                    let g = mean.max(1e-12).powf(exponent);
                    Some(vec![g; lv.len()])
                } else {
                    None
                }
            })
            .collect()
    };

    for ((q, g_opt), &(sb0, sb1, ts0, ts1)) in qs.iter_mut().zip(per.iter()).zip(spans.iter()) {
        if let Some(g) = g_opt {
            for (ti, ts) in (ts0..ts1).enumerate() {
                let scale = g[ti] * boost;
                for sb in sb0..sb1 {
                    let c = q.get(sb, ts);
                    q.set(sb, ts, c.scale(scale));
                }
            }
        }
    }
}

fn predictors(ext: &QmfMatrix, sba: usize) -> (Vec<C64>, Vec<C64>) {
    let n = ext.slots;
    let mut alpha0 = vec![C64::default(); sba];
    let mut alpha1 = vec![C64::default(); sba];
    let eps = 2.0f64.powi(-20);
    for sb in 0..sba {
        let row = ext.row(sb);
        let mut c01 = C64::default();
        let mut c02 = C64::default();
        let mut c11 = 0.0f64;
        let mut c12 = C64::default();
        let mut c22 = 0.0f64;
        for ts in (HFADJ..n).step_by(2) {
            let x0 = row[ts];
            let x1 = row[ts - 2];
            let x2 = row[ts - 4];
            c01 += x0 * x1.conj();
            c02 += x0 * x2.conj();
            c11 += x1.norm_sq();
            c12 += x1 * x2.conj();
            c22 += x2.norm_sq();
        }
        let denom = c22 * c11 - c12.norm_sq() / (1.0 + eps);
        let a1 = if denom != 0.0 {
            (c01 * c12 - c02.scale(c11)).scale(1.0 / denom)
        } else {
            C64::default()
        };
        let a0 = if c11 != 0.0 {
            (C64::new(-c01.re, -c01.im) - a1 * c12.conj()).scale(1.0 / c11)
        } else {
            C64::default()
        };
        if a0.re.is_finite()
            && a0.im.is_finite()
            && a1.re.is_finite()
            && a1.im.is_finite()
            && a0.norm_sq() < 16.0
            && a1.norm_sq() < 16.0
        {
            alpha0[sb] = a0;
            alpha1[sb] = a1;
        }
    }
    (alpha0, alpha1)
}

fn preflatten(low: &QmfMatrix, sbx: usize, ts0: usize, ts1: usize) -> Vec<f64> {
    let span = ts1.saturating_sub(ts0).max(1) as f64;
    let mut env = Vec::with_capacity(sbx);
    let mut sum_env = 0.0f64;
    for sb in 0..sbx {
        let row = low.row(sb);
        let mut p = 0.0f64;
        for &c in &row[ts0..ts1] {
            p += c.norm_sq();
        }
        let e = 10.0 * (p / span + 1.0).log10();
        sum_env += e;
        env.push(e);
    }
    let mean_env = if sbx > 0 { sum_env / sbx as f64 } else { 0.0 };
    let fit = if sbx > 3 {
        // 三次多项式最小二乘拟合（匹配 np.polyfit(x, env, 3)）
        let mut gram = [[0.0f64; 4]; 4];
        let mut rhs = [0.0f64; 4];
        for (xi, &y) in env.iter().enumerate() {
            let x = xi as f64;
            let mut xp = [1.0f64; 7];
            for k in 1..7 {
                xp[k] = xp[k - 1] * x;
            }
            for r in 0..4 {
                for c in 0..4 {
                    gram[r][c] += xp[r + c];
                }
                rhs[r] += xp[r] * y;
            }
        }
        // 高斯消元（部分主元）
        for col in 0..4 {
            let mut pivot = col;
            let mut best = gram[col][col].abs();
            for row in col + 1..4 {
                if gram[row][col].abs() > best {
                    best = gram[row][col].abs();
                    pivot = row;
                }
            }
            if pivot != col {
                gram.swap(col, pivot);
                rhs.swap(col, pivot);
            }
            let diag = gram[col][col];
            if diag.abs() > 1e-30 {
                for row in col + 1..4 {
                    let factor = gram[row][col] / diag;
                    for c in col..4 {
                        gram[row][c] -= factor * gram[col][c];
                    }
                    rhs[row] -= factor * rhs[col];
                }
            }
        }
        let mut coef = [0.0f64; 4];
        for r in (0..4).rev() {
            let mut s = rhs[r];
            for c in r + 1..4 {
                s -= gram[r][c] * coef[c];
            }
            coef[r] = if gram[r][r].abs() > 1e-30 { s / gram[r][r] } else { 0.0 };
        }
        (0..sbx)
            .map(|xi| {
                let x = xi as f64;
                coef[0] + x * (coef[1] + x * (coef[2] + x * coef[3]))
            })
            .collect::<Vec<_>>()
    } else {
        env
    };
    fit.into_iter().map(|f| 10.0f64.powf((f - mean_env) / 20.0)).collect()
}

pub fn process(
    ch: &mut AspxChannel,
    f: &AspxFrame,
    q: &QmfMatrix,
    sig_scf: &[[f64; 32]],
    noise_scf: &[[f64; 8]],
) -> QmfMatrix {
    let slots = ch.slots;
    let k = ch.ts_in_ats;
    let hfgen = ch.hfgen;
    let config = ch.config.unwrap();
    let nb = ch.bands.as_ref().unwrap();
    let sbx = nb.sbx();
    let sba = nb.sba();
    let width = nb.width();

    // history: [q_prev (slots), q (slots)] -> 长度 2 * slots
    let mut history = QmfMatrix::zeros(2 * slots);
    for sb in 0..SUBBANDS {
        history.row_mut(sb)[..slots].copy_from_slice(ch.q_prev.row(sb));
        history.row_mut(sb)[slots..].copy_from_slice(q.row(sb));
    }

    let mut delayed = QmfMatrix::zeros(slots);
    let mut low = QmfMatrix::zeros(slots + hfgen);
    let mut ext = QmfMatrix::zeros(slots + hfgen + HFADJ);
    for sb in 0..SUBBANDS {
        let hrow = history.row(sb);
        delayed.row_mut(sb).copy_from_slice(&hrow[slots - hfgen..2 * slots - hfgen]);
        if sb < sbx {
            low.row_mut(sb).copy_from_slice(&hrow[slots - hfgen..]);
        }
        ext.row_mut(sb).copy_from_slice(&hrow[slots - hfgen - HFADJ..]);
    }

    let ts0 = f.sig_borders[0] * k;
    let ts1 = *f.sig_borders.last().unwrap() * k;
    let n_env = f.num_env;

    let (alpha0, alpha1) = predictors(&ext, sba);
    let n_noise_groups = nb.noise.len() - 1;
    let mut chirps = vec![0.0f64; n_noise_groups];
    for g in 0..n_noise_groups {
        let prev_tna = if g < ch.prev_tna.len() { ch.prev_tna[g] } else { 0 };
        let new = NEW_CHIRP[prev_tna][f.tna[g]];
        let prev = ch.prev_chirp[g];
        let smoothed = if new < prev {
            0.75 * new + 0.25 * prev
        } else {
            0.90625 * new + 0.09375 * prev
        };
        chirps[g] = if smoothed < 0.015625 { 0.0 } else { smoothed };
    }

    let gain_vec = if config.preflat != 0 {
        Some(preflatten(&low, sbx, ts0, ts1))
    } else {
        None
    };

    let mut high = QmfMatrix::zeros(slots + hfgen);
    for idx in 0..nb.patch_src.len() {
        let src = nb.patch_src[idx];
        let dst = nb.patch_dst[idx];
        let c = chirps[nb.patch_grp[idx]];
        let ca0 = alpha0[src].scale(c);
        let c2a1 = alpha1[src].scale(c * c);
        let inv_gv = gain_vec.as_ref().map_or(1.0, |gv| 1.0 / gv[src]);
        let ext_row = ext.row(src);
        let dst_row = high.row_mut(dst);
        for t in ts0..ts1 {
            let n = t + HFADJ;
            let val = ext_row[n] + ca0 * ext_row[n - 2] + c2a1 * ext_row[n - 4];
            dst_row[t] = val.scale(inv_gv);
        }
    }

    // Pseudocode 90: est[width][n_env]
    let mut est = vec![vec![0.0f64; n_env]; width];
    let tables_ref: Vec<&[usize]> = f
        .freq_res
        .iter()
        .map(|&r| if r != 0 { nb.high.as_slice() } else { nb.low.as_slice() })
        .collect();
    for env in 0..n_env {
        let a = f.sig_borders[env] * k;
        let z = f.sig_borders[env + 1] * k;
        let span = (f.sig_borders[env + 1] - f.sig_borders[env]) as f64;
        if config.interpolation != 0 {
            for row in 0..width {
                let hrow = high.row(sbx + row);
                let mut e = 0.0;
                for &c in &hrow[a..z] {
                    e += c.norm_sq();
                }
                est[row][env] = e / span;
            }
        } else {
            let table = tables_ref[env];
            for grp in 0..table.len() - 1 {
                let sb0 = table[grp];
                let sb1 = table[grp + 1];
                let mut sum = 0.0;
                for sb in sb0..sb1 {
                    let hrow = high.row(sb);
                    for &c in &hrow[a..z] {
                        sum += c.norm_sq();
                    }
                }
                let e = sum / (sb1 - sb0) as f64 / span;
                for sb in sb0..sb1 {
                    est[sb - sbx][env] = e;
                }
            }
        }
    }

    // Pseudocode 91
    let mut scf_sig = vec![vec![0.0f64; n_env]; width];
    let mut scf_noise = vec![vec![0.0f64; n_env]; width];
    let mut noise_env = 0usize;
    for env in 0..n_env {
        let table = tables_ref[env];
        for grp in 0..table.len() - 1 {
            let val = sig_scf[env][grp];
            for sb in table[grp]..table[grp + 1] {
                scf_sig[sb - sbx][env] = val;
            }
        }
        if f.num_noise > 1 && f.sig_borders[env] == f.noise_borders[noise_env + 1] {
            noise_env += 1;
        }
        for grp in 0..nb.noise.len() - 1 {
            let val = noise_scf[noise_env][grp];
            for sb in nb.noise[grp]..nb.noise[grp + 1] {
                scf_noise[sb - sbx][env] = val;
            }
        }
    }

    // Pseudocodes 92 and 93
    let p_sine_at_end: isize = if ch.prev_tsg_ptr == ch.prev_num_env as isize { 0 } else { -1 };
    let mut sine = vec![vec![false; n_env]; width];
    for g in 0..nb.high.len() - 1 {
        if f.harmonics[g] {
            let mid = ((nb.high[g] - sbx) + (nb.high[g + 1] - sbx)) / 2;
            for env in 0..n_env {
                if p_sine_at_end == 0 {
                    sine[mid][env] = true;
                } else {
                    sine[mid][env] = (env as isize >= f.tsg_ptr) || ch.prev_sine[mid];
                }
            }
        }
    }
    let mut area = vec![vec![false; n_env]; width];
    for env in 0..n_env {
        let table = tables_ref[env];
        for grp in 0..table.len() - 1 {
            let any_sine = (table[grp]..table[grp + 1]).any(|sb| sine[sb - sbx][env]);
            if any_sine {
                for sb in table[grp]..table[grp + 1] {
                    area[sb - sbx][env] = true;
                }
            }
        }
    }

    // Pseudocodes 94 and 95
    let mut sine_lev = vec![vec![0.0f64; n_env]; width];
    let mut noise_lev = vec![vec![0.0f64; n_env]; width];
    let mut gain = vec![vec![0.0f64; n_env]; width];
    let mut noise_on = vec![vec![false; n_env]; width];
    let onset: Vec<bool> = (0..n_env)
        .map(|env| env as isize == f.tsg_ptr || env as isize == p_sine_at_end)
        .collect();

    for row in 0..width {
        for env in 0..n_env {
            let factor = scf_sig[row][env] / (1.0 + scf_noise[row][env]);
            let sl = if sine[row][env] { factor.sqrt() } else { 0.0 };
            let nl = (factor * scf_noise[row][env]).sqrt();
            let denom = 1.0 + est[row][env];
            let g = if area[row][env] {
                (scf_sig[row][env] * scf_noise[row][env] / (denom * (1.0 + scf_noise[row][env])))
                    .sqrt()
            } else {
                let denom_plain = if onset[env] {
                    denom
                } else {
                    denom * (1.0 + scf_noise[row][env])
                };
                (scf_sig[row][env] / denom_plain).sqrt()
            };
            sine_lev[row][env] = sl;
            noise_lev[row][env] = nl;
            gain[row][env] = g;
            noise_on[row][env] = !onset[env] && sl == 0.0;
        }
    }

    if config.limiter != 0 {
        let lim = &nb.limiter;
        let lo = lim[0] - sbx;
        let hi = *lim.last().unwrap() - sbx;
        let mut max_gain = vec![vec![0.0f64; n_env]; width];
        for grp in 0..lim.len() - 1 {
            let g0 = lim[grp] - sbx;
            let g1 = lim[grp + 1] - sbx;
            for env in 0..n_env {
                let mut sum_sig = 0.0;
                let mut sum_est = 0.0;
                for row in g0..g1 {
                    sum_sig += scf_sig[row][env];
                    sum_est += est[row][env];
                }
                let ratio = ((sum_sig / (1e-12 + sum_est)).sqrt() * 1.41254).min(1e5);
                for row in g0..g1 {
                    max_gain[row][env] = ratio;
                }
            }
        }
        let mut limited_noise = vec![vec![0.0f64; n_env]; width];
        let mut gain_lim = vec![vec![0.0f64; n_env]; width];
        for row in 0..width {
            for env in 0..n_env {
                let nl = noise_lev[row][env];
                let g = gain[row][env];
                let mg = max_gain[row][env];
                let scaled = if g == 0.0 {
                    if nl == 0.0 || mg == 0.0 { 0.0 } else { f64::MAX }
                } else {
                    let v = nl * mg / g;
                    if v.is_nan() { 0.0 } else { v }
                };
                limited_noise[row][env] = nl.min(scaled);
                gain_lim[row][env] = g.min(mg);
            }
        }
        let mut boost = vec![vec![0.0f64; n_env]; width];
        for grp in 0..lim.len() - 1 {
            let g0 = lim[grp] - sbx;
            let g1 = lim[grp + 1] - sbx;
            for env in 0..n_env {
                let mut sum_sig = 0.0;
                let mut sum_den = 0.0;
                for row in g0..g1 {
                    sum_sig += scf_sig[row][env];
                    let gl = gain_lim[row][env];
                    let sl = sine_lev[row][env];
                    let ln = if noise_on[row][env] { limited_noise[row][env] } else { 0.0 };
                    sum_den += est[row][env] * gl * gl + sl * sl + ln * ln;
                }
                let nom = 1e-12 + sum_sig;
                let den = 1e-12 + sum_den;
                let b_val = (nom / den).sqrt().min(1.584893192);
                for row in g0..g1 {
                    boost[row][env] = b_val;
                }
            }
        }
        for row in lo..hi {
            for env in 0..n_env {
                let b_val = boost[row][env];
                gain[row][env] = gain_lim[row][env] * b_val;
                noise_lev[row][env] = limited_noise[row][env] * b_val;
                sine_lev[row][env] *= b_val;
            }
        }
    }

    // Pseudocodes 102 to 108
    let mut y = QmfMatrix::zeros(slots + hfgen);
    for sb in 0..SUBBANDS {
        if ts0 > 0 {
            y.row_mut(sb)[..ts0].copy_from_slice(&ch.y_prev.row(sb)[slots..slots + ts0]);
        }
    }
    let base = if ch.master_reset { 0 } else { ch.noise_cursor };
    let sine_base = if ch.first { 1 } else { (ch.sine_cursor + 1) % 4 };
    if ts1 > ts0 {
        for t in ts0..ts1 {
            let env = f
                .sig_borders
                .partition_point(|&b| b * k <= t)
                .saturating_sub(1)
                .min(n_env - 1);
            let sine_idx = (sine_base + (t - ts0)) % 4;
            let s_re = SINE_RE[sine_idx];
            let s_im = SINE_IM[sine_idx];
            for row in 0..width {
                let sb = sbx + row;
                let noise_idx = (base + width * (t - ts0) + row + 1) % 512;
                let (nr, ni) = ASPX_NOISE[noise_idx];
                let parity = if !(row + sbx).is_multiple_of(2) { -1.0 } else { 1.0 };
                let g = gain[row][env];
                let nl = if noise_on[row][env] { noise_lev[row][env] } else { 0.0 };
                let sl = sine_lev[row][env];
                let h = high.get(sb, t);
                let val = C64::new(
                    g * h.re + nl * nr + sl * s_re,
                    g * h.im + nl * ni + sl * parity * s_im,
                );
                y.set(sb, t, val);
                if t + 1 == ts1 && row + 1 == width {
                    ch.noise_cursor = noise_idx;
                    ch.sine_cursor = sine_idx;
                }
            }
        }
    }

    let mut out = QmfMatrix::zeros(slots);
    for sb in 0..sbx {
        out.row_mut(sb).copy_from_slice(delayed.row(sb));
    }
    for sb in sbx..SUBBANDS {
        out.row_mut(sb).copy_from_slice(&y.row(sb)[..slots]);
    }
    for (g, &used) in f.fic.iter().enumerate() {
        if used {
            for sb in nb.high[g]..nb.high[g + 1] {
                let drow = delayed.row(sb);
                let orow = out.row_mut(sb);
                for ts in 0..slots {
                    orow[ts] += drow[ts];
                }
            }
        }
    }
    for (n, &used) in f.tic.iter().enumerate() {
        if used {
            for sb in sbx..SUBBANDS {
                let drow = delayed.row(sb);
                let orow = out.row_mut(sb);
                orow[n * k..(n + 1) * k].copy_from_slice(&drow[n * k..(n + 1) * k]);
            }
        }
    }

    ch.q_prev = q.clone();
    ch.y_prev = y;
    ch.prev_num_env = n_env;
    ch.prev_freq_res = f.freq_res.clone();
    ch.prev_tsg_ptr = f.tsg_ptr;
    ch.prev_tna = f.tna.clone();
    ch.prev_chirp.fill(0.0);
    for (i, &c) in chirps.iter().enumerate() {
        ch.prev_chirp[i] = c;
    }
    ch.prev_sine.fill(false);
    for row in 0..width {
        ch.prev_sine[row] = sine[row][n_env - 1];
    }
    ch.master_reset = false;
    ch.first = false;
    out
}

#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    pub noise: usize,
    pub sine: usize,
    pub first: bool,
    pub reset: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self { noise: 0, sine: 0, first: true, reset: true }
    }
}

impl Cursor {
    pub fn advance(&mut self, width: usize, slots: usize) {
        if slots > 0 {
            let base = if self.reset { 0 } else { self.noise };
            self.noise = (base + width * slots) % 512;
            let sbase = if self.first { 1 } else { (self.sine + 1) % 4 };
            self.sine = (sbase + slots - 1) % 4;
        }
        self.first = false;
        self.reset = false;
    }

    pub fn restore(&self, ch: &mut AspxChannel) {
        ch.noise_cursor = self.noise;
        ch.sine_cursor = self.sine;
        ch.first = self.first;
        ch.master_reset = self.reset;
    }
}

pub fn decode_channels(
    pairs: &mut [(&mut AspxChannel, &AspxFrame)],
    qs: &[&QmfMatrix],
) -> Vec<QmfMatrix> {
    let quant: Vec<QuantItem<'_>> = pairs
        .iter()
        .map(|(ch, f)| {
            let (sig, noise) = quantized(ch, f);
            (*f, sig, noise)
        })
        .collect();
    let scales = dequantize_all(&quant);
    let mut out = Vec::with_capacity(pairs.len());
    for ((((ch, f), (_, sig, noise)), (sig_scf, noise_scf)), &q) in pairs
        .iter_mut()
        .zip(quant.iter())
        .zip(scales.iter())
        .zip(qs.iter())
    {
        out.push(process(ch, f, q, sig_scf, noise_scf));
        ch.prev_sig = *sig.last().unwrap();
        ch.prev_noise = *noise.last().unwrap();
        ch.prev_num_noise = f.num_noise;
    }
    out
}

pub fn skip_channel(ch: &mut AspxChannel, q: &QmfMatrix) -> QmfMatrix {
    let slots = ch.slots;
    let hfgen = ch.hfgen;
    let mut out = QmfMatrix::zeros(slots);
    for sb in 0..SUBBANDS {
        let prev = ch.q_prev.row(sb);
        let cur = q.row(sb);
        let dst = out.row_mut(sb);
        dst[..hfgen].copy_from_slice(&prev[slots - hfgen..]);
        dst[hfgen..].copy_from_slice(&cur[..slots - hfgen]);
    }
    ch.q_prev = q.clone();
    out
}
