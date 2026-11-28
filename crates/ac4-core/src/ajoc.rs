//! 高级联合对象编码 A-JOC（TS 103 190-2 clauses 5.7、6.2.3.4..6.2.5、6.3.6）。

use std::collections::HashMap;

use crate::acpl::Module;
use crate::bits::BitReader;
use crate::codes::{huffman, Book};
use crate::elements::{
    five_x_element, mono_data, three_channel_data, two_channel_data, Element, Reader, Track, ASPX,
    SIMPLE,
};
use crate::oamd::{self, Frame, Objects, Timing, Trim};
use crate::qmf::{QmfMatrix, BANDS, C64};
use crate::toc::SubstreamInfo;
use crate::{fail, Result};

const SUBBANDS: usize = BANDS;
const NUM_BANDS: [usize; 8] = [23, 15, 12, 9, 7, 5, 3, 1];
const ROWS: [usize; 23] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 18, 20, 23, 26, 30, 35, 41, 48,
];

const DRY_MAX: f64 = 5125.0 / 1024.0;
const WET_MAX: f64 = 2050.0 / 1024.0;
const DECORRELATOR: [usize; 7] = [0, 2, 1, 0, 2, 1, 0];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dry,
    Wet,
}

fn sb_to_pb(bands: usize) -> [usize; SUBBANDS] {
    let table: &[usize] = match bands {
        23 => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22],
        15 => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 13, 14, 14, 14],
        12 => &[0, 1, 2, 3, 4, 4, 5, 5, 6, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 10, 11, 11, 11],
        9 => &[0, 1, 2, 3, 3, 4, 4, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8],
        7 => &[0, 1, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6],
        5 => &[0, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4],
        3 => &[0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        _ => &[0; 23],
    };
    let mut out = [0usize; SUBBANDS];
    for sb in 0..SUBBANDS {
        let row = ROWS.partition_point(|&r| r <= sb).saturating_sub(1);
        out[sb] = table[row];
    }
    out
}

#[derive(Clone, Debug)]
pub struct AjocObject {
    pub bands: usize,
    pub coarse: bool,
    pub sparse: bool,
    pub dry_present: Vec<bool>,
    pub wet_present: Vec<bool>,
}

pub type HuffMatrix = Vec<Vec<Vec<Option<(u32, Vec<i32>)>>>>;

#[derive(Clone, Debug)]
pub struct AjocFrame {
    pub decorr: usize,
    pub points: Vec<(usize, usize)>,
    pub objects: Vec<Option<AjocObject>>,
    pub dry: HuffMatrix,
    pub wet: HuffMatrix,
}

fn ajoc_book(kind: Kind, coarse: bool, diff_kind: u8) -> Book {
    match (kind, coarse, diff_kind) {
        (Kind::Dry, false, 0) => Book::AjocDryFineF0,
        (Kind::Dry, false, 1) => Book::AjocDryFineDf,
        (Kind::Dry, false, 2) => Book::AjocDryFineDt,
        (Kind::Dry, true, 0) => Book::AjocDryCoarseF0,
        (Kind::Dry, true, 1) => Book::AjocDryCoarseDf,
        (Kind::Dry, true, 2) => Book::AjocDryCoarseDt,
        (Kind::Wet, false, 0) => Book::AjocWetFineF0,
        (Kind::Wet, false, 1) => Book::AjocWetFineDf,
        (Kind::Wet, false, 2) => Book::AjocWetFineDt,
        (Kind::Wet, true, 0) => Book::AjocWetCoarseF0,
        (Kind::Wet, true, 1) => Book::AjocWetCoarseDf,
        (Kind::Wet, true, 2) => Book::AjocWetCoarseDt,
        _ => unreachable!(),
    }
}

fn nquant(kind: Kind, coarse: bool) -> i32 {
    match kind {
        Kind::Dry => if coarse { 51 } else { 101 },
        Kind::Wet => if coarse { 21 } else { 41 },
    }
}

fn huff_data(
    b: &mut BitReader<'_>,
    kind: Kind,
    bands: usize,
    coarse: bool,
    dfonly: bool,
) -> Result<(u32, Vec<i32>)> {
    let diff = if dfonly { 0 } else { b.read(1)? };
    if diff == 0 {
        let mut values = Vec::with_capacity(bands);
        values.push(huffman(ajoc_book(kind, coarse, 0)).decode(b)?);
        let hdf = huffman(ajoc_book(kind, coarse, 1));
        for _ in 1..bands {
            values.push(hdf.decode(b)?);
        }
        Ok((0, values))
    } else {
        let centre = nquant(kind, coarse) - 1;
        let hdt = huffman(ajoc_book(kind, coarse, 2));
        let mut values = Vec::with_capacity(bands);
        for _ in 0..bands {
            values.push(hdt.decode(b)? - centre);
        }
        Ok((1, values))
    }
}

pub fn read_ajoc(b: &mut BitReader<'_>, n_dmx: usize, n_umx: usize) -> Result<AjocFrame> {
    let decorr = b.read(3)? as usize;
    let mut enabled = Vec::with_capacity(decorr);
    for _ in 0..decorr {
        enabled.push(b.flag()?);
    }
    let mut present = Vec::with_capacity(n_umx);
    for _ in 0..n_umx {
        present.push(b.flag()?);
    }
    let n_points = b.read(2)? as usize;
    let mut points = Vec::with_capacity(n_points);
    for _ in 0..n_points {
        points.push((b.read(5)? as usize, b.read(6)? as usize + 1));
    }
    let mut objects: Vec<Option<AjocObject>> = vec![None; n_umx];
    if !points.is_empty() {
        for o in 0..n_umx {
            if !present[o] {
                continue;
            }
            let bands = NUM_BANDS[b.read(3)? as usize];
            let coarse = b.flag()?;
            let sparse = b.flag()?;
            let (dry, wet) = if sparse {
                let mut d = Vec::with_capacity(n_dmx);
                for _ in 0..n_dmx {
                    d.push(b.flag()?);
                }
                let mut w = Vec::with_capacity(decorr);
                for de in 0..decorr {
                    w.push(if enabled[de] { b.flag()? } else { false });
                }
                (d, w)
            } else {
                (vec![true; n_dmx], vec![true; decorr])
            };
            objects[o] = Some(AjocObject {
                bands,
                coarse,
                sparse,
                dry_present: dry,
                wet_present: wet,
            });
        }
    }
    let nodt = b.flag()?;
    let mut dry_data = Vec::with_capacity(n_umx);
    let mut wet_data = Vec::with_capacity(n_umx);
    for o in 0..n_umx {
        let mut dry_o = Vec::new();
        let mut wet_o = Vec::new();
        if let Some(obj) = &objects[o] {
            for dp in 0..points.len() {
                let dfonly = dp == 0 && nodt;
                let mut d_dp = Vec::with_capacity(n_dmx);
                for ch in 0..n_dmx {
                    if obj.dry_present[ch] {
                        d_dp.push(Some(huff_data(b, Kind::Dry, obj.bands, obj.coarse, dfonly)?));
                    } else {
                        d_dp.push(None);
                    }
                }
                let mut w_dp = Vec::with_capacity(decorr);
                for de in 0..decorr {
                    if obj.wet_present[de] {
                        w_dp.push(Some(huff_data(b, Kind::Wet, obj.bands, obj.coarse, dfonly)?));
                    } else {
                        w_dp.push(None);
                    }
                }
                dry_o.push(d_dp);
                wet_o.push(w_dp);
            }
        }
        dry_data.push(dry_o);
        wet_data.push(wet_o);
    }
    Ok(AjocFrame {
        decorr,
        points,
        objects,
        dry: dry_data,
        wet: wet_data,
    })
}

fn read_dmx_de_data(
    b: &mut BitReader<'_>,
    n_dmx: usize,
    n_umx: usize,
    state: &mut AjocState,
    iframe: bool,
) -> Result<()> {
    let cfg = b.flag()?;
    let keep = b.flag()?;
    if cfg {
        b.skip(2)?;
        let mut dlg = 0usize;
        for _ in 0..n_umx {
            if b.flag()? {
                dlg += 1;
            }
        }
        state.dialogue = dlg;
    }
    if !keep || cfg || iframe {
        for _ in 0..(state.dialogue * n_dmx) {
            if b.flag()? && b.read(3)? != 0b111 {
                b.skip(1)?;
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default)]
pub struct AjocState {
    pub dmx_objects: Option<Objects>,
    pub umx_objects: Option<Objects>,
    pub dmx_timing: Option<Timing>,
    pub umx_timing: Option<Timing>,
    pub dialogue: usize,
    pub group_timing: Option<Timing>,
    pub trim: Option<Trim>,
}

#[derive(Clone, Debug)]
pub struct AjocPayload {
    pub inputs: Vec<String>,
    pub lfe: Option<String>,
    pub frame: AjocFrame,
    pub metadata: Frame,
    pub upmix: usize,
    pub core: Option<Frame>,
}

pub fn audio_data_ajoc(
    r: &mut Reader<'_, '_>,
    info: &SubstreamInfo,
    alternative: bool,
    state: &mut AjocState,
) -> Result<(Element, AjocPayload)> {
    let iframe = r.iframe;
    let n_dmx = info.dmx_signals;
    let n_umx = info.upmix_signals;
    if state.umx_objects.is_none() || iframe {
        let (types, program) = oamd::program_for(
            &info.upmix_objects,
            &info.upmix_beds,
            info.isf_config,
            info.b_lfe,
        );
        let trim = state
            .umx_objects
            .as_ref()
            .map(|o| o.trim.clone())
            .unwrap_or_default();
        state.umx_objects = Some(Objects::new(types, program, trim));
        let (dmx_types, dmx_program) =
            oamd::program_for(&info.dmx_objects, &info.dmx_beds, None, info.b_lfe);
        state.dmx_objects = Some(Objects::new(dmx_types, dmx_program, Trim::default()));
    }
    if let Some(common) = &info.oamd_common {
        state.trim = Some(common.clone());
    }
    if let Some(trim) = &state.trim {
        if let Some(umx) = state.umx_objects.as_mut() {
            umx.trim = trim.clone();
        }
    }

    let mut core = None;
    let (element, inputs) = if info.static_dmx {
        let el = five_x_element(r, info.b_lfe)?;
        let inp = vec![
            "L".to_string(),
            "R".to_string(),
            "C".to_string(),
            "Ls".to_string(),
            "Rs".to_string(),
        ];
        (el, inp)
    } else {
        if r.b.flag()? {
            r.b.skip(n_dmx)?;
        }
        let (el, inp) = var_channel_element(r, n_dmx, info.b_lfe)?;
        if r.b.flag()? {
            state.dmx_timing = Some(oamd::read_timing(r.b)?);
        } else if let Some(gt) = &state.group_timing {
            state.dmx_timing = Some(gt.clone());
        }
        let dmx_timing = state.dmx_timing.clone().unwrap_or(Timing { blocks: Vec::new() });
        core = Some(oamd::read_dyndata(
            r.b,
            state.dmx_objects.as_mut().unwrap(),
            &dmx_timing,
            iframe,
            alternative,
        )?);
        if r.b.flag()? {
            let size = r.b.variable_bits(3)? as usize + 1;
            let end = r.b.pos + 8 * size;
            if r.b.flag()? {
                r.b.skip(3)?;
            }
            if r.b.pos > end {
                return fail("ajoc_bed_info overran the OAMD extension");
            }
            if end > r.b.end {
                return fail("ajoc_bed_info overran buffer");
            }
            r.b.pos = end;
        }
        (el, inp)
    };

    let frame = read_ajoc(r.b, n_dmx, n_umx)?;
    read_dmx_de_data(r.b, n_dmx, n_umx, state, iframe)?;
    if r.b.flag()? {
        state.umx_timing = Some(oamd::read_timing(r.b)?);
    } else if r.b.flag()? {
        state.umx_timing = state.dmx_timing.clone();
    } else if let Some(gt) = &state.group_timing {
        state.umx_timing = Some(gt.clone());
    }
    let umx_timing = state.umx_timing.clone().unwrap_or(Timing { blocks: Vec::new() });
    let metadata = oamd::read_dyndata(
        r.b,
        state.umx_objects.as_mut().unwrap(),
        &umx_timing,
        iframe,
        alternative,
    )?;
    let lfe = if info.b_lfe { Some("LFE".to_string()) } else { None };
    Ok((
        element,
        AjocPayload { inputs, lfe, frame, metadata, upmix: n_umx, core },
    ))
}

pub fn oamd_substream(b: &mut BitReader<'_>, state: &mut AjocState) -> Result<()> {
    state.group_timing = None;
    if b.flag()? {
        if let Some(trim) = oamd::read_common(b)? {
            state.trim = Some(trim);
        }
    }
    if b.flag()? {
        state.group_timing = Some(oamd::read_timing(b)?);
    }
    Ok(())
}

fn var_channel_element(
    r: &mut Reader<'_, '_>,
    n: usize,
    lfe: bool,
) -> Result<(Element, Vec<String>)> {
    let mode = if r.b.flag()? { ASPX } else { SIMPLE };
    let mut element = Element::new(mode);
    let names: Vec<String> = (0..n).map(|i| format!("d{i}")).collect();
    if mode == ASPX {
        r.aspx_config()?;
        if n <= 5 {
            let refs: Vec<&str> = names.iter().map(String::as_str).collect();
            element.companding = Some(r.companding(&refs)?);
        }
    }
    if lfe {
        let t = mono_data(r, true)?;
        element.set_channel("LFE", t);
    }
    let mut tracks: Vec<Track> = Vec::with_capacity(n);
    let pairs = n / 2;
    if n % 2 == 1 {
        if n == 1 {
            tracks.push(mono_data(r, false)?);
        } else {
            for _ in 0..(pairs - 1) {
                tracks.extend(two_channel_data(r)?);
            }
            if !r.b.flag()? {
                tracks.extend(two_channel_data(r)?);
                tracks.push(mono_data(r, false)?);
            } else {
                tracks.extend(three_channel_data(r)?);
            }
        }
    } else {
        for _ in 0..pairs {
            tracks.extend(two_channel_data(r)?);
        }
    }
    for (name, track) in names.iter().zip(tracks) {
        element.set_channel(name.clone(), track);
    }
    if mode == ASPX {
        for p in 0..pairs {
            r.aspx_2ch(&mut element, &names[2 * p], &names[2 * p + 1])?;
        }
        if n % 2 == 1 {
            r.aspx_1ch(&mut element, &names[n - 1])?;
        }
    }
    let order = if n > 3 {
        let offset = 2 + (n % 2);
        let mut v = names[n - offset..].to_vec();
        v.extend_from_slice(&names[..n - offset]);
        v
    } else {
        names
    };
    Ok((element, order))
}

/// `[out][in][subband]` 扁平矩阵（共 `out_ch * in_ch * 64` 项）
#[derive(Clone)]
struct Matrix3 {
    cols: usize,
    data: Vec<f64>,
}

impl Matrix3 {
    fn zeros(rows: usize, cols: usize) -> Self {
        Self { cols, data: vec![0.0; rows * cols * SUBBANDS] }
    }

    #[inline]
    fn get(&self, r: usize, c: usize, sb: usize) -> f64 {
        self.data[(r * self.cols + c) * SUBBANDS + sb]
    }

    #[inline]
    fn set(&mut self, r: usize, c: usize, sb: usize, v: f64) {
        self.data[(r * self.cols + c) * SUBBANDS + sb] = v;
    }

    fn sub(&self, rhs: &Self) -> Self {
        let mut out = self.clone();
        for (a, &b) in out.data.iter_mut().zip(rhs.data.iter()) {
            *a -= b;
        }
        out
    }

    fn scale(&self, s: f64) -> Self {
        let mut out = self.clone();
        for a in &mut out.data {
            *a *= s;
        }
        out
    }

    fn add_scaled(&self, step: &Self, k: f64) -> Self {
        let mut out = self.clone();
        for (a, &s) in out.data.iter_mut().zip(step.data.iter()) {
            *a += s * k;
        }
        out
    }
}

#[derive(Clone)]
struct Targets {
    dry: Matrix3,
    wet: Matrix3,
    pre: Matrix3,
}

struct Run {
    start: usize,
    stop: usize,
    base: Targets,
    steps: Option<Targets>,
}

pub struct Reconstructor {
    pub n_dmx: usize,
    pub n_umx: usize,
    decorr: usize,
    q_dry: Vec<HashMap<usize, Vec<i32>>>,
    q_wet: Vec<HashMap<usize, Vec<i32>>>,
    dry: Matrix3,
    wet: Matrix3,
    pre: Matrix3,
    steps: Option<Targets>,
    final_targets: Option<Targets>,
    ramp_remaining: usize,
    modules: Vec<Module>,
}

impl Reconstructor {
    pub fn new(n_dmx: usize, n_umx: usize) -> Self {
        Self {
            n_dmx,
            n_umx,
            decorr: 0,
            q_dry: vec![HashMap::new(); n_umx],
            q_wet: vec![HashMap::new(); n_umx],
            dry: Matrix3::zeros(n_umx, n_dmx),
            wet: Matrix3::zeros(n_umx, 0),
            pre: Matrix3::zeros(0, n_dmx),
            steps: None,
            final_targets: None,
            ramp_remaining: 0,
            modules: Vec::new(),
        }
    }

    fn resize(&mut self, decorr: usize) {
        if decorr == self.decorr {
            return;
        }
        let mut wet = Matrix3::zeros(self.n_umx, decorr);
        let mut pre = Matrix3::zeros(decorr, self.n_dmx);
        let keep = decorr.min(self.decorr);
        for o in 0..self.n_umx {
            for d in 0..keep {
                for sb in 0..SUBBANDS {
                    wet.set(o, d, sb, self.wet.get(o, d, sb));
                }
            }
        }
        for d in 0..keep {
            for c in 0..self.n_dmx {
                for sb in 0..SUBBANDS {
                    pre.set(d, c, sb, self.pre.get(d, c, sb));
                }
            }
        }
        self.wet = wet;
        self.pre = pre;
        self.decorr = decorr;
        self.modules.truncate(decorr);
        for d in self.modules.len()..decorr {
            self.modules.push(Module::new(DECORRELATOR[d]));
        }
        self.steps = None;
        self.final_targets = None;
        self.ramp_remaining = 0;
    }

    fn targets(&mut self, frame: &AjocFrame) -> Vec<Targets> {
        let mut out = Vec::with_capacity(frame.points.len());
        for dp in 0..frame.points.len() {
            let mut dry = Matrix3::zeros(self.n_umx, self.n_dmx);
            let mut wet = Matrix3::zeros(self.n_umx, frame.decorr);
            for (o, obj_opt) in frame.objects.iter().enumerate() {
                let Some(obj) = obj_opt else {
                    continue;
                };
                let pb = sb_to_pb(obj.bands);
                // DRY
                {
                    let nq = nquant(Kind::Dry, obj.coarse);
                    let centre = (nq - 1) / 2;
                    let scale = DRY_MAX / f64::from(centre);
                    for i in 0..self.n_dmx {
                        let q = match &frame.dry[o][dp][i] {
                            None => vec![centre; obj.bands],
                            Some((0, vals)) => {
                                let mut acc = 0i32;
                                vals.iter()
                                    .map(|&v| {
                                        acc = (acc + v).rem_euclid(nq);
                                        acc
                                    })
                                    .collect()
                            }
                            Some((_, vals)) => {
                                let before = self.q_dry[o]
                                    .get(&i)
                                    .filter(|b| b.len() == obj.bands)
                                    .cloned()
                                    .unwrap_or_else(|| vec![centre; obj.bands]);
                                before.iter().zip(vals.iter()).map(|(&b, &v)| b + v).collect()
                            }
                        };
                        for sb in 0..SUBBANDS {
                            dry.set(o, i, sb, f64::from(q[pb[sb]] - centre) * scale);
                        }
                        self.q_dry[o].insert(i, q);
                    }
                }
                // WET
                {
                    let nq = nquant(Kind::Wet, obj.coarse);
                    let centre = (nq - 1) / 2;
                    let scale = WET_MAX / f64::from(centre);
                    for i in 0..frame.decorr {
                        let q = match &frame.wet[o][dp][i] {
                            None => vec![centre; obj.bands],
                            Some((0, vals)) => {
                                let mut acc = 0i32;
                                vals.iter()
                                    .map(|&v| {
                                        acc = (acc + v).rem_euclid(nq);
                                        acc
                                    })
                                    .collect()
                            }
                            Some((_, vals)) => {
                                let before = self.q_wet[o]
                                    .get(&i)
                                    .filter(|b| b.len() == obj.bands)
                                    .cloned()
                                    .unwrap_or_else(|| vec![centre; obj.bands]);
                                before.iter().zip(vals.iter()).map(|(&b, &v)| b + v).collect()
                            }
                        };
                        for sb in 0..SUBBANDS {
                            wet.set(o, i, sb, f64::from(q[pb[sb]] - centre) * scale);
                        }
                        self.q_wet[o].insert(i, q);
                    }
                }
            }
            let mut pre = Matrix3::zeros(frame.decorr, self.n_dmx);
            for d in 0..frame.decorr {
                for c in 0..self.n_dmx {
                    for sb in 0..SUBBANDS {
                        let mut sum = 0.0;
                        for o in 0..self.n_umx {
                            sum += wet.get(o, d, sb).abs() * dry.get(o, c, sb);
                        }
                        pre.set(d, c, sb, sum);
                    }
                }
            }
            out.push(Targets { dry, wet, pre });
        }
        out
    }

    pub fn process(&mut self, frame: &AjocFrame, x: &[&QmfMatrix]) -> Vec<QmfMatrix> {
        self.resize(frame.decorr);
        let targets = self.targets(frame);
        let slots = x[0].slots;
        let mut starts: HashMap<usize, usize> = HashMap::new();
        for (dp, &(start, _)) in frame.points.iter().enumerate() {
            starts.insert(start, dp);
        }
        let mut runs: Vec<Run> = Vec::new();
        let mut ts = 0usize;
        while ts < slots {
            if let Some(&dp) = starts.get(&ts) {
                let length = frame.points[dp].1;
                let inv_len = 1.0 / length as f64;
                let t = &targets[dp];
                self.steps = Some(Targets {
                    dry: t.dry.sub(&self.dry).scale(inv_len),
                    wet: t.wet.sub(&self.wet).scale(inv_len),
                    pre: t.pre.sub(&self.pre).scale(inv_len),
                });
                self.ramp_remaining = length;
                self.final_targets = Some(t.clone());
            }
            let stop = starts
                .keys()
                .copied()
                .filter(|&s| s > ts && s < slots)
                .min()
                .unwrap_or(slots);
            let ramp = self.ramp_remaining.min(stop - ts);
            let mut current = Targets {
                dry: self.dry.clone(),
                wet: self.wet.clone(),
                pre: self.pre.clone(),
            };
            if ramp > 0 {
                let steps = self.steps.clone();
                runs.push(Run {
                    start: ts,
                    stop: ts + ramp,
                    base: current.clone(),
                    steps: steps.clone(),
                });
                self.ramp_remaining -= ramp;
                if self.ramp_remaining == 0 {
                    current = self.final_targets.as_ref().unwrap().clone();
                } else {
                    let st = steps.as_ref().unwrap();
                    let r = ramp as f64;
                    current = Targets {
                        dry: current.dry.add_scaled(&st.dry, r),
                        wet: current.wet.add_scaled(&st.wet, r),
                        pre: current.pre.add_scaled(&st.pre, r),
                    };
                }
                self.dry = current.dry.clone();
                self.wet = current.wet.clone();
                self.pre = current.pre.clone();
            }
            if ts + ramp < stop {
                runs.push(Run {
                    start: ts + ramp,
                    stop,
                    base: current,
                    steps: None,
                });
            }
            ts = stop;
        }

        let mut z = mix(&runs, 0, x, self.n_umx);
        if self.decorr > 0 {
            let u = mix(&runs, 2, x, self.decorr);
            let mut y = Vec::with_capacity(self.decorr);
            for (d, m) in self.modules.iter_mut().enumerate() {
                let ud = m.decorrelate(&u[d]);
                y.push(m.duck(&ud));
            }
            let y_refs: Vec<&QmfMatrix> = y.iter().collect();
            let wet_z = mix(&runs, 1, &y_refs, self.n_umx);
            for (zo, wo) in z.iter_mut().zip(wet_z.iter()) {
                for (a, &b) in zo.data.iter_mut().zip(wo.data.iter()) {
                    *a += b;
                }
            }
        }
        z
    }
}

fn select_mat(t: &Targets, which: u8) -> &Matrix3 {
    match which {
        0 => &t.dry,
        1 => &t.wet,
        _ => &t.pre,
    }
}

fn mix(runs: &[Run], which: u8, x: &[&QmfMatrix], out_count: usize) -> Vec<QmfMatrix> {
    let slots = x[0].slots;
    let in_count = x.len();
    let mut out: Vec<QmfMatrix> = (0..out_count).map(|_| QmfMatrix::zeros(slots)).collect();
    for run in runs {
        let bm = select_mat(&run.base, which);
        let sm = run.steps.as_ref().map(|s| select_mat(s, which));
        for sb in 0..SUBBANDS {
            for t in run.start..run.stop {
                let ramp = (t - run.start + 1) as f64;
                for o in 0..out_count {
                    let mut sum = C64::default();
                    for c in 0..in_count {
                        let w = match sm {
                            Some(s) => bm.get(o, c, sb) + ramp * s.get(o, c, sb),
                            None => bm.get(o, c, sb),
                        };
                        sum += x[c].get(sb, t).scale(w);
                    }
                    out[o].set(sb, t, sum);
                }
            }
        }
    }
    out
}
