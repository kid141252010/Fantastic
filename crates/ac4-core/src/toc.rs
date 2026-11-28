//! AC-4 同步帧与 TOC 解析（TS 103 190-1 clause 4.2.3、TS 103 190-2 clause 6.2.1）。

use crate::bits::BitReader;
use crate::oamd::{self, Trim};
use crate::{fail, Result};

pub const SYNC: u16 = 0xAC40;
pub const SYNC_CRC: u16 = 0xAC41;

/// Table 83: 48 kHz 下各 frame_rate_index 对应的基础帧长
pub const FRAME_LENGTHS: [usize; 14] = [
    1920, 1920, 2048, 1536, 1536, 960, 960, 1024, 768, 768, 512, 384, 384, 2048,
];

/// Table 83: 48 kHz 下各 frame_rate_index 的帧率（分子, 分母）
pub const FRAME_RATES: [(u64, u64); 14] = [
    (24000, 1001),
    (24, 1),
    (25, 1),
    (30000, 1001),
    (30, 1),
    (48000, 1001),
    (48, 1),
    (50, 1),
    (60000, 1001),
    (60, 1),
    (100, 1),
    (120000, 1001),
    (120, 1),
    (48000, 2048),
];

pub const BED: u8 = 0;
pub const DYN: u8 = 1;
pub const ISF: u8 = 2;

const ISF_COUNTS: [usize; 8] = [4, 8, 10, 14, 15, 30, 0, 0];

const AJOC_BEDS: [&[&str]; 8] = [
    &["L", "R"],
    &["L", "R", "C"],
    &["L", "R", "C", "Ls", "Rs"],
    &["L", "R", "C", "Ls", "Rs", "Tsl", "Tsr"],
    &["L", "R", "C", "Ls", "Rs", "Tfl", "Tfr", "Tbl", "Tbr"],
    &["L", "R", "C", "Ls", "Rs", "Lb", "Rb"],
    &["L", "R", "C", "Ls", "Rs", "Lb", "Rb", "Tsl", "Tsr"],
    &["L", "R", "C", "Ls", "Rs", "Lb", "Rb", "Tfl", "Tfr", "Tbl", "Tbr"],
];

const NONSTD_LABELS: [&str; 17] = [
    "L", "R", "C", "LFE", "Ls", "Rs", "Lb", "Rb", "Tfl", "Tfr", "Tsl", "Tsr", "Tbl", "Tbr",
    "Lw", "Rw", "LFE2",
];

const STD_GROUPS: [&[&str]; 10] = [
    &["L", "R"],
    &["C"],
    &["LFE"],
    &["Ls", "Rs"],
    &["Lb", "Rb"],
    &["Tfl", "Tfr"],
    &["Tsl", "Tsr"],
    &["Tbl", "Tbr"],
    &["Lw", "Rw"],
    &["LFE2"],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubstreamKind {
    Chan,
    Ajoc,
    Obj,
}

#[derive(Clone, Debug)]
pub struct SubstreamInfo {
    pub kind: SubstreamKind,
    pub index: usize,
    pub channel_mode: u32,
    pub b_lfe: bool,
    pub static_dmx: bool,
    pub dmx_signals: usize,
    pub upmix_signals: usize,
    pub dmx_objects: Vec<(u8, bool)>,
    pub upmix_objects: Vec<(u8, bool)>,
    pub dmx_beds: Vec<String>,
    pub upmix_beds: Vec<String>,
    pub isf_config: Option<usize>,
    pub oamd_common: Option<Trim>,
    pub objects: Vec<(u8, bool)>,
    pub iframe: bool,
    pub add_ch_base: u32,
}

impl SubstreamInfo {
    fn new(kind: SubstreamKind) -> Self {
        Self {
            kind,
            index: 0,
            channel_mode: 0,
            b_lfe: false,
            static_dmx: true,
            dmx_signals: 0,
            upmix_signals: 0,
            dmx_objects: Vec::new(),
            upmix_objects: Vec::new(),
            dmx_beds: Vec::new(),
            upmix_beds: Vec::new(),
            isf_config: None,
            oamd_common: None,
            objects: Vec::new(),
            iframe: false,
            add_ch_base: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubstreamGroup {
    pub substreams_present: bool,
    pub channel_coded: bool,
    pub oamd_index: Option<usize>,
    pub substreams: Vec<SubstreamInfo>,
}

#[derive(Clone, Debug)]
pub struct Presentation {
    pub version: u32,
    pub config: i32,
    pub groups: Vec<usize>,
    pub presentation_substream: Option<usize>,
    pub enabled: bool,
    pub alternative: bool,
}

impl Default for Presentation {
    fn default() -> Self {
        Self {
            version: 0,
            config: -1,
            groups: Vec::new(),
            presentation_substream: None,
            enabled: true,
            alternative: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Toc {
    pub bitstream_version: u32,
    pub sequence_counter: u32,
    pub fs_index: u32,
    pub frame_rate_index: usize,
    pub iframe_global: bool,
    pub presentations: Vec<Presentation>,
    pub groups: Vec<SubstreamGroup>,
    pub substream_sizes: Vec<usize>,
    pub payload_base: usize,
    pub data_start: usize,
}

impl Toc {
    #[inline]
    pub fn frame_length(&self) -> usize {
        FRAME_LENGTHS[self.frame_rate_index]
    }

    /// 返回 `(编码采样率分子, 分母, 输出采样率)`；若为 44.1 kHz 则两者相等。
    pub fn rates(&self) -> Result<(u64, u64, u32)> {
        let delivered = if self.fs_index != 0 { 48000u32 } else { 44100u32 };
        if self.fs_index == 0 {
            return Ok((u64::from(delivered), 1, delivered));
        }
        let (fr_num, fr_den) = FRAME_RATES[self.frame_rate_index];
        let coded_num = self.frame_length() as u64 * fr_num;
        let g = gcd(coded_num, fr_den);
        let coded_num = coded_num / g;
        let coded_den = fr_den / g;
        // ratio = delivered / coded = (delivered * coded_den) / coded_num
        let r_num = u64::from(delivered) * coded_den;
        let r_den = coded_num;
        let rg = gcd(r_num, r_den);
        if (r_num / rg).max(r_den / rg) > 4096 {
            return fail(format!(
                "unsupported coded sample rate {:.2} Hz",
                coded_num as f64 / coded_den as f64
            ));
        }
        Ok((coded_num, coded_den, delivered))
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.max(1)
}

/// 若单个输入帧带有 `0xAC40` / `0xAC41` 同步头且长度精确匹配，则剥离同步头与尾部 CRC；
/// 否则按 MP4 样本里的裸 `raw_ac4_frame` 原样返回。
pub fn unwrap_sync_frame(data: &[u8]) -> &[u8] {
    if data.len() >= 4 {
        let sync = u16::from_be_bytes([data[0], data[1]]);
        if sync == SYNC || sync == SYNC_CRC {
            let mut length = usize::from(u16::from_be_bytes([data[2], data[3]]));
            let mut head = 4usize;
            if length == 0xFFFF && data.len() >= 7 {
                length = (usize::from(data[4]) << 16)
                    | (usize::from(data[5]) << 8)
                    | usize::from(data[6]);
                head = 7;
            }
            let crc_len = if sync == SYNC_CRC { 2 } else { 0 };
            if head + length + crc_len == data.len() {
                return &data[head..head + length];
            }
        }
    }
    data
}

/// 从 Annex G.3 同步字节流中切分出所有 `raw_ac4_frame`。
pub fn sync_frames(stream: &[u8]) -> Vec<&[u8]> {
    let mut frames = Vec::new();
    let mut pos = 0usize;
    let size = stream.len();
    while pos + 4 <= size {
        let sync = u16::from_be_bytes([stream[pos], stream[pos + 1]]);
        if sync != SYNC && sync != SYNC_CRC {
            let mut found = size;
            for i in (pos + 1)..size.saturating_sub(1) {
                let s = u16::from_be_bytes([stream[i], stream[i + 1]]);
                if s == SYNC || s == SYNC_CRC {
                    found = i;
                    break;
                }
            }
            if found >= size {
                break;
            }
            pos = found;
            continue;
        }
        let mut length = usize::from(u16::from_be_bytes([stream[pos + 2], stream[pos + 3]]));
        let mut head = 4usize;
        if length == 0xFFFF {
            if pos + 7 > size {
                break;
            }
            length = (usize::from(stream[pos + 4]) << 16)
                | (usize::from(stream[pos + 5]) << 8)
                | usize::from(stream[pos + 6]);
            head = 7;
        }
        let end = pos + head + length;
        if end > size {
            break;
        }
        frames.push(&stream[pos + head..end]);
        pos = end + if sync == SYNC_CRC { 2 } else { 0 };
    }
    frames
}

pub fn read_toc(frame: &[u8]) -> Result<Toc> {
    let mut reader = TocReader::new(BitReader::new(frame));
    reader.read()
}

struct TocReader<'a> {
    bits: BitReader<'a>,
    version: u32,
    fs_index: u32,
    frame_rate_index: usize,
    groups_wanted: usize,
    factor: usize,
}

type BedDynAssignment = (Vec<(u8, bool)>, Vec<String>, Option<usize>);

impl<'a> TocReader<'a> {
    fn new(bits: BitReader<'a>) -> Self {
        Self {
            bits,
            version: 0,
            fs_index: 1,
            frame_rate_index: 0,
            groups_wanted: 0,
            factor: 1,
        }
    }

    fn read(&mut self) -> Result<Toc> {
        let mut version = self.bits.read(2)?;
        if version == 3 {
            version += self.bits.variable_bits(2)?;
        }
        self.version = version;
        let sequence_counter = self.bits.read(10)?;
        if self.bits.flag()? && self.bits.read(3)? > 0 {
            self.bits.read(2)?;
        }
        self.fs_index = self.bits.read(1)?;
        self.frame_rate_index = self.bits.read(4)? as usize;
        if self.frame_rate_index >= FRAME_LENGTHS.len() {
            return fail(format!("reserved frame_rate_index {}", self.frame_rate_index));
        }
        let iframe_global = self.bits.flag()?;
        let n_presentations = if self.bits.flag()? {
            1
        } else if self.bits.flag()? {
            self.bits.variable_bits(2)? as usize + 2
        } else {
            0
        };
        let mut payload_base = 0usize;
        if self.bits.flag()? {
            payload_base = self.bits.read(5)? as usize + 1;
            if payload_base == 0x20 {
                payload_base += self.bits.variable_bits(3)? as usize;
            }
        }
        if version <= 1 {
            return fail(format!(
                "bitstream_version {version} (TS 103 190-1 only) is not supported"
            ));
        }
        if self.bits.flag()? {
            self.bits.read(16)?;
            if self.bits.flag()? {
                self.bits.skip(128)?;
            }
        }
        let mut presentations = Vec::with_capacity(n_presentations);
        for _ in 0..n_presentations {
            presentations.push(self.presentation_v1()?);
        }
        let mut groups = Vec::with_capacity(self.groups_wanted);
        for _ in 0..self.groups_wanted {
            groups.push(self.substream_group()?);
        }
        let substream_sizes = self.substream_index_table()?;
        self.bits.pos = (self.bits.pos + 7) & !7;
        Ok(Toc {
            bitstream_version: version,
            sequence_counter,
            fs_index: self.fs_index,
            frame_rate_index: self.frame_rate_index,
            iframe_global,
            presentations,
            groups,
            substream_sizes,
            payload_base,
            data_start: self.bits.pos / 8,
        })
    }

    fn frame_rate_factor(&mut self) -> Result<usize> {
        let index = self.frame_rate_index;
        if matches!(index, 2..=4) {
            if self.bits.flag()? {
                return Ok(if self.bits.flag()? { 4 } else { 2 });
            }
            return Ok(1);
        }
        if matches!(index, 0 | 1 | 7 | 8 | 9) {
            return Ok(if self.bits.flag()? { 2 } else { 1 });
        }
        Ok(1)
    }

    fn emdf_info(&mut self) -> Result<()> {
        if self.bits.read(2)? == 3 {
            self.bits.variable_bits(2)?;
        }
        if self.bits.read(3)? == 7 {
            self.bits.variable_bits(3)?;
        }
        if self.bits.flag()? {
            self.substream_index()?;
        }
        self.emdf_reserved()
    }

    fn emdf_reserved(&mut self) -> Result<()> {
        let primary = self.bits.read(2)?;
        let secondary = self.bits.read(2)?;
        let mut skip = 0usize;
        if primary != 0 {
            skip += 1usize << (2 * (primary - 1));
        }
        if secondary != 0 {
            skip += 1usize << (2 * (secondary - 1));
        }
        self.bits.skip(8 * skip)
    }

    fn substream_index(&mut self) -> Result<usize> {
        let mut index = self.bits.read(2)? as usize;
        if index == 3 {
            index += self.bits.variable_bits(2)? as usize;
        }
        Ok(index)
    }

    fn presentation_v1(&mut self) -> Result<Presentation> {
        let mut p = Presentation::default();
        let single = self.bits.flag()?;
        let mut config = -1i32;
        if !single {
            config = self.bits.read(3)? as i32;
            if config == 7 {
                config += self.bits.variable_bits(2)? as i32;
            }
        }
        p.config = config;
        if self.version != 1 {
            while self.bits.flag()? {
                p.version += 1;
            }
        }
        let add_emdf = if !single && config == 6 {
            true
        } else {
            if self.version != 1 {
                self.bits.read(3)?;
            }
            if self.bits.flag()? {
                self.bits.variable_bits(2)?;
            }
            let factor = self.frame_rate_factor()?;
            self.factor = factor;
            self.frame_rate_fractions(factor)?;
            self.emdf_info()?;
            if self.bits.flag()? {
                p.enabled = self.bits.flag()?;
            }
            if single {
                p.groups.push(self.sgi_specifier()?);
            } else {
                self.bits.flag()?;
                let count: Option<usize> = match config {
                    0..=2 => Some(2),
                    3 | 4 => Some(3),
                    5 => {
                        let mut c = self.bits.read(2)? as usize + 2;
                        if c == 5 {
                            c += self.bits.variable_bits(2)? as usize;
                        }
                        Some(c)
                    }
                    _ => None,
                };
                if let Some(c) = count {
                    for _ in 0..c {
                        p.groups.push(self.sgi_specifier()?);
                    }
                } else {
                    self.config_ext_info()?;
                }
            }
            self.bits.flag()?;
            let add_emdf = self.bits.flag()?;
            p.alternative = self.bits.flag()?;
            self.bits.flag()?;
            p.presentation_substream = Some(self.substream_index()?);
            add_emdf
        };
        if add_emdf {
            let mut n = self.bits.read(2)? as usize;
            if n == 0 {
                n = self.bits.variable_bits(2)? as usize + 4;
            }
            for _ in 0..n {
                self.emdf_info()?;
            }
        }
        Ok(p)
    }

    fn frame_rate_fractions(&mut self, factor: usize) -> Result<()> {
        let index = self.frame_rate_index;
        if matches!(index, 5..=9) && factor == 1 {
            self.bits.flag()?;
        }
        if matches!(index, 10..=12) && self.bits.flag()? {
            self.bits.flag()?;
        }
        Ok(())
    }

    fn config_ext_info(&mut self) -> Result<()> {
        let mut skip = self.bits.read(5)? as usize;
        if self.bits.flag()? {
            skip += (self.bits.variable_bits(2)? as usize) << 5;
        }
        self.bits.skip(8 * skip)
    }

    fn sgi_specifier(&mut self) -> Result<usize> {
        let mut index = self.bits.read(3)? as usize;
        if index == 7 {
            index += self.bits.variable_bits(2)? as usize;
        }
        self.groups_wanted = self.groups_wanted.max(index + 1);
        Ok(index)
    }

    fn substream_group(&mut self) -> Result<SubstreamGroup> {
        let mut g = SubstreamGroup {
            substreams_present: self.bits.flag()?,
            ..SubstreamGroup::default()
        };
        let hsf = self.bits.flag()?;
        let n = if self.bits.flag()? {
            1
        } else {
            let mut c = self.bits.read(2)? as usize + 2;
            if c == 5 {
                c += self.bits.variable_bits(2)? as usize;
            }
            c
        };
        g.channel_coded = self.bits.flag()?;
        if g.channel_coded {
            for _ in 0..n {
                if self.version == 1 {
                    self.bits.flag()?;
                }
                g.substreams.push(self.info_chan(g.substreams_present)?);
                if hsf {
                    self.hsf_info(g.substreams_present)?;
                }
            }
        } else {
            if self.bits.flag()? {
                self.bits.flag()?;
                if g.substreams_present {
                    g.oamd_index = Some(self.substream_index()?);
                }
            }
            for _ in 0..n {
                let s = if self.bits.flag()? {
                    self.info_ajoc(g.substreams_present)?
                } else {
                    self.info_obj(g.substreams_present)?
                };
                g.substreams.push(s);
                if hsf {
                    self.hsf_info(g.substreams_present)?;
                }
            }
        }
        if self.bits.flag()? {
            self.content_type()?;
        }
        Ok(g)
    }

    fn hsf_info(&mut self, present: bool) -> Result<()> {
        if present {
            self.substream_index()?;
        }
        Ok(())
    }

    fn content_type(&mut self) -> Result<()> {
        self.bits.read(3)?;
        if self.bits.flag()? {
            if self.bits.flag()? {
                self.bits.read(1)?;
                self.bits.read(16)?;
            } else {
                let n = self.bits.read(6)? as usize;
                self.bits.skip(8 * n)?;
            }
        }
        Ok(())
    }

    fn channel_mode(&mut self) -> Result<u32> {
        if !self.bits.flag()? {
            return Ok(0);
        }
        if !self.bits.flag()? {
            return Ok(1);
        }
        let mut code = (0b11u32 << 2) | self.bits.read(2)?;
        if code < 0b1111 {
            return Ok(code - 0b1100 + 2);
        }
        code = (code << 3) | self.bits.read(3)?;
        if code < 0b1111101 {
            return Ok(code - 0b1111000 + 5);
        }
        if code == 0b1111101 {
            return Ok(10);
        }
        code = (code << 1) | self.bits.read(1)?;
        if code == 0b11111100 {
            return Ok(11);
        }
        if code == 0b11111101 {
            return Ok(12);
        }
        code = (code << 1) | self.bits.read(1)?;
        if code == 0b111111100 {
            return Ok(13);
        }
        if code == 0b111111101 {
            return Ok(14);
        }
        if code == 0b111111110 {
            return Ok(15);
        }
        Ok(16 + self.bits.variable_bits(2)?)
    }

    fn info_tail(&mut self, info: &mut SubstreamInfo, present: bool, factor: usize) -> Result<()> {
        if self.fs_index == 1 && self.bits.flag()? {
            self.bits.flag()?;
        }
        if self.bits.flag()? {
            self.bitrate_indicator()?;
        }
        let mut first = false;
        for i in 0..factor {
            let bit = self.bits.flag()?;
            if i == 0 {
                first = bit;
            }
        }
        info.iframe = first;
        if present {
            info.index = self.substream_index()?;
        }
        Ok(())
    }

    fn info_chan(&mut self, present: bool) -> Result<SubstreamInfo> {
        let mut info = SubstreamInfo::new(SubstreamKind::Chan);
        let mode = self.channel_mode()?;
        info.channel_mode = mode;
        if matches!(mode, 11..=14) {
            self.bits.read(4)?;
        }
        if self.fs_index == 1 && self.bits.flag()? {
            self.bits.flag()?;
        }
        if self.bits.flag()? {
            self.bitrate_indicator()?;
        }
        if matches!(mode, 7..=10) {
            info.add_ch_base = self.bits.read(1)?;
        }
        let mut first = false;
        for i in 0..self.factor {
            let bit = self.bits.flag()?;
            if i == 0 {
                first = bit;
            }
        }
        info.iframe = first;
        if present {
            info.index = self.substream_index()?;
        }
        Ok(info)
    }

    fn info_ajoc(&mut self, present: bool) -> Result<SubstreamInfo> {
        let mut info = SubstreamInfo::new(SubstreamKind::Ajoc);
        info.b_lfe = self.bits.flag()?;
        info.static_dmx = self.bits.flag()?;
        if info.static_dmx {
            info.dmx_signals = 5;
        } else {
            info.dmx_signals = self.bits.read(4)? as usize + 1;
            let (objs, beds, _) = self.bed_dyn_obj_assignment(info.dmx_signals)?;
            info.dmx_objects = objs;
            info.dmx_beds = beds;
        }
        if self.bits.flag()? {
            info.oamd_common = oamd::read_common(&mut self.bits)?;
        }
        let mut n = self.bits.read(4)? as usize + 1;
        if n == 16 {
            n += self.bits.variable_bits(3)? as usize;
        }
        info.upmix_signals = n;
        let (objs, beds, isf) = self.bed_dyn_obj_assignment(n)?;
        info.upmix_objects = objs;
        info.upmix_beds = beds;
        info.isf_config = isf;
        self.info_tail(&mut info, present, self.factor)?;
        Ok(info)
    }

    fn info_obj(&mut self, present: bool) -> Result<SubstreamInfo> {
        let mut info = SubstreamInfo::new(SubstreamKind::Obj);
        let mut objs: Vec<(u8, bool)> = Vec::new();
        let count = [0usize, 1, 2, 3, 5, 7, 0, 0][self.bits.read(3)? as usize];
        if self.bits.flag()? {
            let lfe = self.bits.flag()?;
            for i in 0..count {
                objs.push(if lfe && i == 0 { (BED, true) } else { (DYN, false) });
            }
        } else if self.bits.flag()? {
            if self.bits.flag()? {
                if self.bits.flag()? {
                    let n = [2usize, 3, 6, 8, 10, 8, 10, 12][self.bits.read(3)? as usize];
                    for i in 0..n {
                        objs.push((BED, i == 3));
                    }
                } else if self.bits.flag()? {
                    let flags = self.bits.read(17)?;
                    for i in 0..17 {
                        if ((flags >> (16 - i)) & 1) != 0 {
                            objs.push((BED, i == 3 || i == 16));
                        }
                    }
                } else {
                    let flags = self.bits.read(10)?;
                    let widths = [2usize, 1, 1, 2, 2, 2, 2, 2, 2, 1];
                    for (i, &width) in widths.iter().enumerate() {
                        if ((flags >> (9 - i)) & 1) != 0 {
                            for _ in 0..width {
                                objs.push((BED, i == 2 || i == 9));
                            }
                        }
                    }
                }
            }
        } else if self.bits.flag()? {
            if self.bits.flag()? {
                let n = ISF_COUNTS[self.bits.read(3)? as usize];
                objs = vec![(ISF, false); n];
            }
        } else {
            let skip = self.bits.read(4)? as usize;
            self.bits.skip(8 * skip)?;
        }
        info.objects = objs;
        self.info_tail(&mut info, present, self.factor)?;
        Ok(info)
    }

    fn bed_dyn_obj_assignment(
        &mut self,
        n_signals: usize,
    ) -> Result<BedDynAssignment> {
        let mut beds: Vec<String> = Vec::new();
        let mut isf = None;
        if !self.bits.flag()? {
            if self.bits.flag()? {
                isf = Some(self.bits.read(3)? as usize);
            } else if self.bits.flag()? {
                let code = self.bits.read(3)? as usize;
                for &label in AJOC_BEDS[code] {
                    beds.push(label.to_string());
                }
            } else if self.bits.flag()? {
                if self.bits.flag()? {
                    let flags = self.bits.read(17)?;
                    for (i, &label) in NONSTD_LABELS.iter().enumerate() {
                        if ((flags >> (16 - i)) & 1) != 0 && i != 3 && i != 16 {
                            beds.push(label.to_string());
                        }
                    }
                } else {
                    let flags = self.bits.read(10)?;
                    for (i, group) in STD_GROUPS.iter().enumerate() {
                        if ((flags >> (9 - i)) & 1) != 0 && i != 2 && i != 9 {
                            for &label in *group {
                                beds.push(label.to_string());
                            }
                        }
                    }
                }
            } else {
                let bit_len = if n_signals > 1 {
                    usize::BITS - (n_signals - 1).leading_zeros()
                } else {
                    1
                };
                let n_bed = self.bits.read(bit_len)? as usize + 1;
                for _ in 0..n_bed {
                    let code = self.bits.read(4)? as usize;
                    if code != 3 {
                        if code >= NONSTD_LABELS.len() {
                            return fail(format!("invalid bed code {code}"));
                        }
                        beds.push(NONSTD_LABELS[code].to_string());
                    }
                }
            }
        }
        let mut objs = if let Some(idx) = isf {
            vec![(ISF, false); ISF_COUNTS[idx]]
        } else {
            vec![(BED, false); beds.len()]
        };
        if objs.len() < n_signals {
            objs.resize(n_signals, (DYN, false));
        }
        Ok((objs, beds, isf))
    }

    fn substream_index_table(&mut self) -> Result<Vec<usize>> {
        let mut n = self.bits.read(2)? as usize;
        if n == 0 {
            n = self.bits.variable_bits(2)? as usize + 4;
        }
        let present = if n == 1 { self.bits.flag()? } else { true };
        let mut sizes = Vec::new();
        if present {
            sizes.reserve(n);
            for _ in 0..n {
                let more = self.bits.flag()?;
                let mut size = self.bits.read(10)? as usize;
                if more {
                    size += (self.bits.variable_bits(2)? as usize) << 10;
                }
                sizes.push(size);
            }
        }
        Ok(sizes)
    }

    fn bitrate_indicator(&mut self) -> Result<()> {
        if (self.bits.read(3)? & 1) != 0 {
            self.bits.read(2)?;
        }
        Ok(())
    }
}
