//! 流式 AC-4 / A-JOC 解码与立体声渲染器（Speaker 2.0 / 双耳 HRTF）。

use std::collections::VecDeque;
use std::f64::consts::{FRAC_1_SQRT_2, PI};

use joc_core::jochrtf::{builtin_kernels, load_jochrtf, Field, Kernels};
use joc_core::sofa_binaural::{head_rotation, source_offset, BinauralRenderer, RoomConstants};

use crate::frame::FrameDecoder;
use crate::oamd::Frame;
use crate::render::{joc_objects, Renderer, LAYOUT_2_0};
use crate::toc::{self, read_toc, SubstreamKind};
use crate::{fail, Result};

const RATE: u32 = 48000;
const BINAURAL_CHANNELS: usize = 16;
const BINAURAL_BLOCK: usize = 512;
const BINAURAL_HOP: usize = 64;
const BINAURAL_LATENCY: usize = 961;
const LFE_POSITION: [f64; 3] = [0.0, 1.0, 0.0];
const MAX_SCENE_OBJECTS: usize = 15;
const SCENE_SOURCES: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ac4Mode {
    Speaker20,
    Binaural,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinauralProfile {
    Near = 0,
    Mid = 1,
    Far = 2,
}

pub struct Ac4StreamConfig {
    pub mode: Ac4Mode,
    pub gain_db: f64,
    pub tail_seconds: f64,
    pub binaural_profile: BinauralProfile,
    pub hrtf: Option<Vec<u8>>,
}

impl Default for Ac4StreamConfig {
    fn default() -> Self {
        Self {
            mode: Ac4Mode::Speaker20,
            gain_db: 0.0,
            tail_seconds: 5.0,
            binaural_profile: BinauralProfile::Mid,
            hrtf: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Ac4Controls {
    pub head: [[f64; 3]; 3],
    pub walk: [f64; 3],
    pub gains: [f64; SCENE_SOURCES],
    pub focus: f64,
    pub height: f64,
}

impl Default for Ac4Controls {
    fn default() -> Self {
        Self {
            head: head_rotation(0.0, 0.0, 0.0),
            walk: [0.0; 3],
            gains: [1.0; SCENE_SOURCES],
            focus: 0.0,
            height: 1.0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Ac4Scene {
    pub positions: [[f64; 3]; MAX_SCENE_OBJECTS],
    pub levels: [f64; SCENE_SOURCES],
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Ac4StreamInfo {
    pub frames_in: u64,
    pub frames_out: u64,
    pub samples_in: u64,
    pub samples_out: u64,
    pub output_channels: u32,
    pub oamd_payloads: u64,
    pub peak: f64,
    pub over_unity: u64,
    pub non_finite: u64,
    pub ended: bool,
}

/// 返回单个 AC-4 帧在输出采样率下的样本数（供 Seek 预滚跳帧估算）。
pub fn frame_samples(data: &[u8]) -> Result<usize> {
    let raw = toc::unwrap_sync_frame(data);
    let toc = read_toc(raw)?;
    let (coded_num, coded_den, delivered) = toc.rates()?;
    let samples = (u128::from(toc.frame_length() as u64)
        * u128::from(delivered)
        * u128::from(coded_den))
        / u128::from(coded_num);
    Ok(samples as usize)
}

/// Kaiser 窗加权 sinc 流式立体声重采样器（ratio == 1 时零拷贝透传）。
struct StereoResampler {
    up: u64,
    down: u64,
    buf_l: Vec<f64>,
    buf_r: Vec<f64>,
    dropped: u64,
    total_in: u64,
    out_idx: u64,
}

impl StereoResampler {
    const HALF_TAPS: usize = 16;

    fn new(up: u64, down: u64) -> Self {
        Self {
            up,
            down,
            buf_l: Vec::new(),
            buf_r: Vec::new(),
            dropped: 0,
            total_in: 0,
            out_idx: 0,
        }
    }

    fn reset(&mut self) {
        self.buf_l.clear();
        self.buf_r.clear();
        self.dropped = 0;
        self.total_in = 0;
        self.out_idx = 0;
    }

    fn set_ratio(&mut self, up: u64, down: u64) {
        if self.up != up || self.down != down {
            self.up = up;
            self.down = down;
        }
    }

    fn push(&mut self, stereo: &[f64], out: &mut Vec<f32>) {
        if self.up == self.down {
            out.reserve(stereo.len());
            for &v in stereo {
                out.push(v as f32);
            }
            return;
        }
        let frames = stereo.len() / 2;
        self.buf_l.reserve(frames);
        self.buf_r.reserve(frames);
        for i in 0..frames {
            self.buf_l.push(stereo[i * 2]);
            self.buf_r.push(stereo[i * 2 + 1]);
        }
        self.total_in += frames as u64;
        self.render(false, out);
    }

    fn finish(&mut self, out: &mut Vec<f32>) {
        if self.up == self.down {
            return;
        }
        self.render(true, out);
    }

    fn render(&mut self, flush: bool, out: &mut Vec<f32>) {
        let step = self.down as f64 / self.up as f64;
        let cutoff = 0.5 * (self.up as f64 / self.down as f64).min(1.0);
        let half = Self::HALF_TAPS as isize;
        let max_out = if flush {
            (self.total_in * self.up).div_ceil(self.down)
        } else {
            u64::MAX
        };

        while self.out_idx < max_out {
            let t = self.out_idx as f64 * step;
            let center = t.floor() as isize;
            if !flush && (center + half + 1) as u64 >= self.total_in {
                break;
            }
            let mut sum_l = 0.0f64;
            let mut sum_r = 0.0f64;
            let mut wsum = 0.0f64;
            let i0_denom = bessel_i0_fast(5.0);
            for k in (center - half)..=(center + half) {
                let dx = t - k as f64;
                let norm = dx / half as f64;
                let w = if norm.abs() < 1.0 {
                    bessel_i0_fast(5.0 * (1.0 - norm * norm).sqrt()) / i0_denom
                } else {
                    0.0
                };
                let sinc = if dx.abs() < 1e-12 {
                    2.0 * cutoff
                } else {
                    (2.0 * PI * cutoff * dx).sin() / (PI * dx)
                };
                let coeff = w * sinc;
                wsum += coeff;
                if k >= 0 && (k as u64) < self.total_in {
                    let idx = (k as u64 - self.dropped) as usize;
                    sum_l += self.buf_l[idx] * coeff;
                    sum_r += self.buf_r[idx] * coeff;
                }
            }
            let inv = if wsum.abs() > 1e-12 { 1.0 / wsum } else { 1.0 };
            out.push((sum_l * inv) as f32);
            out.push((sum_r * inv) as f32);
            self.out_idx += 1;
        }

        let safe_keep = ((self.out_idx as f64 * step).floor() as i64 - Self::HALF_TAPS as i64 - 2).max(0) as u64;
        if safe_keep > self.dropped + 4096 {
            let drop_n = (safe_keep - self.dropped) as usize;
            if drop_n < self.buf_l.len() {
                self.buf_l.drain(..drop_n);
                self.buf_r.drain(..drop_n);
                self.dropped += drop_n as u64;
            }
        }
    }
}

fn bessel_i0_fast(x: f64) -> f64 {
    let half = 0.5 * x;
    let mut sum = 1.0;
    let mut term = 1.0;
    for k in 1..25 {
        let r = half / k as f64;
        term *= r * r;
        sum += term;
        if term < sum * 1e-14 {
            break;
        }
    }
    sum
}

#[derive(Clone, Copy)]
struct PosTransition {
    start_sample: i64,
    duration_samples: i64,
    origin: [f64; 3],
    target: [f64; 3],
}

#[derive(Default)]
struct ObjPosTrack {
    initial: [f64; 3],
    last_target: [f64; 3],
    transitions: VecDeque<PosTransition>,
    initialized: bool,
}

impl ObjPosTrack {
    fn new() -> Self {
        Self {
            initial: [0.0, 1.0, 0.0],
            last_target: [0.0, 1.0, 0.0],
            transitions: VecDeque::new(),
            initialized: false,
        }
    }

    fn append(&mut self, start_sample: i64, duration_samples: i64, target: [f64; 3]) {
        if !self.initialized {
            self.initial = target;
            self.last_target = target;
            self.initialized = true;
        }
        self.transitions.push_back(PosTransition {
            start_sample,
            duration_samples,
            origin: self.last_target,
            target,
        });
        self.last_target = target;
    }

    fn position_at(&mut self, sample: i64) -> [f64; 3] {
        while let Some(front) = self.transitions.front() {
            if sample < front.start_sample + front.duration_samples {
                break;
            }
            self.initial = front.target;
            self.transitions.pop_front();
        }
        let Some(tr) = self.transitions.front() else {
            return self.initial;
        };
        if sample < tr.start_sample {
            return self.initial;
        }
        if tr.duration_samples <= 0 {
            return tr.target;
        }
        let amount = (sample - tr.start_sample) as f64 / tr.duration_samples as f64;
        [
            tr.origin[0] + (tr.target[0] - tr.origin[0]) * amount,
            tr.origin[1] + (tr.target[1] - tr.origin[1]) * amount,
            tr.origin[2] + (tr.target[2] - tr.origin[2]) * amount,
        ]
    }
}

struct BinauralEngine {
    field: Field,
    kernels: Kernels,
    profile: u32,
    banks: Vec<Box<BinauralRenderer>>,
    controls: Ac4Controls,
    tracks: Vec<ObjPosTrack>,
    staging: Vec<f64>,
    staged: usize,
    processed_samples: i64,
    rendered_frames: i64,
    hrtf_history_slots: u32,
    scene_positions: [[f64; 3]; MAX_SCENE_OBJECTS],
    scene_energy: [f64; SCENE_SOURCES],
    scene_samples: u64,
    n_sources: usize,
}

impl BinauralEngine {
    fn new(hrtf_bytes: &[u8], profile: u32) -> Result<Self> {
        let field = load_jochrtf(hrtf_bytes).map_err(|e| crate::Error::new(e.to_string()))?;
        let kernels = builtin_kernels();
        let maximum = field.delay_bounds.iter().copied().fold(0.0f64, f64::max);
        let maximum_delay = (maximum - 1e-9).ceil() as i64;
        let hrtf_history_slots = ((maximum_delay + 63) / 64).max(1) as u32;
        Ok(Self {
            field,
            kernels,
            profile,
            banks: Vec::new(),
            controls: Ac4Controls::default(),
            tracks: Vec::new(),
            staging: Vec::new(),
            staged: 0,
            processed_samples: 0,
            rendered_frames: 0,
            hrtf_history_slots,
            scene_positions: [[0.0, 1.0, 0.0]; MAX_SCENE_OBJECTS],
            scene_energy: [0.0; SCENE_SOURCES],
            scene_samples: 0,
            n_sources: 0,
        })
    }

    fn ensure_banks(&mut self, n_objects: usize) -> Result<()> {
        let n_sources = 1 + n_objects;
        let needed = n_sources.div_ceil(BINAURAL_CHANNELS).max(1);
        if self.n_sources != n_sources {
            self.n_sources = n_sources;
            self.tracks = (0..n_objects).map(|_| ObjPosTrack::new()).collect();
            self.staging = vec![0.0; BINAURAL_BLOCK * needed * BINAURAL_CHANNELS];
        }
        while self.banks.len() < needed {
            let r = Self::create_bank(&self.kernels, &self.field, &self.controls)?;
            self.banks.push(r);
        }
        Ok(())
    }

    #[inline(never)]
    fn create_bank(
        kernels: &Kernels,
        field: &Field,
        controls: &Ac4Controls,
    ) -> Result<Box<BinauralRenderer>> {
        let mut r = Box::new(BinauralRenderer::new());
        r.configure_kernels(kernels).map_err(|e| crate::Error::new(e.to_string()))?;
        r.configure_field(field).map_err(|e| crate::Error::new(e.to_string()))?;
        r.configure_room(&RoomConstants::default()).map_err(|e| crate::Error::new(e.to_string()))?;
        r.set_listener_pose(&controls.head, &controls.walk);
        Ok(r)
    }

    fn reset(&mut self) {
        for bank in &mut self.banks {
            bank.reset();
            bank.set_listener_pose(&self.controls.head, &self.controls.walk);
        }
        for tr in &mut self.tracks {
            *tr = ObjPosTrack::new();
        }
        self.staged = 0;
        self.processed_samples = 0;
        self.rendered_frames = 0;
        self.scene_energy = [0.0; SCENE_SOURCES];
        self.scene_samples = 0;
    }

    fn set_controls(&mut self, controls: &Ac4Controls) {
        if controls.head != self.controls.head || controls.walk != self.controls.walk {
            for bank in &mut self.banks {
                bank.set_listener_pose(&controls.head, &controls.walk);
            }
        }
        self.controls = *controls;
        if let Some(bank) = self.banks.first() {
            self.controls.walk = bank.walk();
        }
        self.controls.focus = if controls.focus.is_finite() {
            controls.focus.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.controls.height = if controls.height.is_finite() {
            controls.height.clamp(0.0, 4.0)
        } else {
            1.0
        };
        for g in &mut self.controls.gains {
            *g = if g.is_finite() { g.clamp(0.0, 4.0) } else { 1.0 };
        }
    }

    fn take_scene(&mut self) -> Ac4Scene {
        let mut levels = [0.0f64; SCENE_SOURCES];
        if self.scene_samples > 0 {
            for ch in 0..SCENE_SOURCES {
                levels[ch] = (self.scene_energy[ch] / self.scene_samples as f64).sqrt();
            }
        }
        self.scene_energy = [0.0; SCENE_SOURCES];
        self.scene_samples = 0;
        Ac4Scene { positions: self.scene_positions, levels }
    }

    fn focus_gain(&self, adm_pos: &[f64; 3]) -> f64 {
        let focus = self.controls.focus;
        if focus == 0.0 {
            return 1.0;
        }
        let offset = source_offset(adm_pos, self.profile);
        let walk = self.controls.walk;
        let v = [offset[0] - walk[0], offset[1] - walk[1], offset[2] - walk[2]];
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if len < 1.0e-9 {
            return 10.0f64.powf(6.0 * focus / 20.0);
        }
        let face = self.controls.head[1];
        let cos = (face[0] * v[0] + face[1] * v[1] + face[2] * v[2]) / len;
        let weight = ((cos + 1.0) / 2.0).powi(4);
        10.0f64.powf(focus * (-15.0 + 21.0 * weight) / 20.0)
    }

    fn submit_frame(
        &mut self,
        meta: &Frame,
        gains_env: &[Vec<f64>],
        gain_col: usize,
        width: usize,
        obj_pcm: &[Vec<f64>],
        lfe_pcm: &[f64],
        frame_len: usize,
        out_stereo: &mut Vec<f64>,
    ) -> Result<()> {
        let n_obj = obj_pcm.len();
        self.ensure_banks(n_obj)?;

        // 记录每个对象本帧的位置跳变/斜坡到时间线
        let joc_objs = joc_objects(meta);
        let labels = &meta.program.bed_labels;
        let backs = labels.iter().any(|l| l == "Lb");
        let base_sample = self.rendered_frames * frame_len as i64;
        for (o, &meta_idx) in joc_objs.iter().enumerate() {
            let anchored_label = labels.get(meta_idx).map(String::as_str);
            let dynamic = meta_idx >= meta.program.anchored();
            for u in &meta.updates[meta_idx] {
                let [rx, ry, mut rz] = if let Some(lbl) = anchored_label {
                    bed_room_pos(lbl, backs)
                } else if dynamic {
                    u.render.warped(&meta.trim).position()
                } else {
                    u.render.position()
                };
                if !u.render.elevation {
                    rz = 0.0;
                }
                let adm = [rx * 2.0 - 1.0, 1.0 - ry * 2.0, rz];
                self.tracks[o].append(base_sample + u.start as i64, u.ramp as i64, adm);
            }
        }
        self.rendered_frames += 1;

        let stride = self.banks.len() * BINAURAL_CHANNELS;
        let mut offset = 0usize;
        while offset < frame_len {
            let room = BINAURAL_BLOCK - self.staged;
            let count = room.min(frame_len - offset);
            for s in 0..count {
                let row = (self.staged + s) * stride;
                let t = offset + s;
                self.staging[row] = lfe_pcm[t];
                for o in 0..n_obj {
                    let g = gains_env[o][t * width + gain_col];
                    self.staging[row + 1 + o] = obj_pcm[o][t] * g;
                }
            }
            self.staged += count;
            offset += count;
            if self.staged == BINAURAL_BLOCK {
                self.process_block(BINAURAL_BLOCK, out_stereo)?;
            }
        }
        Ok(())
    }

    fn process_block(&mut self, samples: usize, out_stereo: &mut Vec<f64>) -> Result<()> {
        if samples == 0 {
            return Ok(());
        }
        let n_obj = self.tracks.len();
        let mut positions = Vec::with_capacity(n_obj);
        for tr in &mut self.tracks {
            positions.push(tr.position_at(self.processed_samples));
        }
        for i in 0..MAX_SCENE_OBJECTS {
            self.scene_positions[i] = positions.get(i).copied().unwrap_or([0.0, 1.0, 0.0]);
        }

        let stride = self.banks.len() * BINAURAL_CHANNELS;
        for s in 0..samples {
            let row = s * stride;
            for ch in 0..SCENE_SOURCES.min(stride) {
                let val = self.staging[row + ch];
                self.scene_energy[ch] += val * val;
            }
        }
        self.scene_samples += samples as u64;

        let height = self.controls.height;
        if height != 1.0 {
            for pos in &mut positions {
                pos[2] = (pos[2] * height).clamp(-1.0, 1.0);
            }
        }

        let mut produced_len = 0usize;
        let mut mixed = vec![0.0f64; samples * 2];
        let mut bank_in = vec![0.0f64; samples * BINAURAL_CHANNELS];
        let mut bank_out = vec![0.0f64; samples * 2];

        for bank_idx in 0..self.banks.len() {
            let base_src = bank_idx * BINAURAL_CHANNELS;
            for local in 0..BINAURAL_CHANNELS {
                let src = base_src + local;
                if src == 0 {
                    self.banks[bank_idx]
                        .set_source(
                            0,
                            &LFE_POSITION,
                            self.profile,
                            self.controls.gains[0],
                            true,
                            true,
                            true,
                        )
                        .map_err(|e| crate::Error::new(e.to_string()))?;
                } else if src <= n_obj {
                    let o = src - 1;
                    let ctl_gain = if src < SCENE_SOURCES { self.controls.gains[src] } else { 1.0 };
                    let g = ctl_gain * self.focus_gain(&positions[o]);
                    self.banks[bank_idx]
                        .set_source(local, &positions[o], self.profile, g, true, false, true)
                        .map_err(|e| crate::Error::new(e.to_string()))?;
                } else {
                    self.banks[bank_idx]
                        .set_source(local, &LFE_POSITION, self.profile, 0.0, false, false, false)
                        .map_err(|e| crate::Error::new(e.to_string()))?;
                }
            }
            for s in 0..samples {
                let src_row = s * stride + base_src;
                let dst_row = s * BINAURAL_CHANNELS;
                bank_in[dst_row..dst_row + BINAURAL_CHANNELS]
                    .copy_from_slice(&self.staging[src_row..src_row + BINAURAL_CHANNELS]);
            }
            let written = self.banks[bank_idx]
                .process(&bank_in, samples, 1.0, &mut bank_out)
                .map_err(|e| crate::Error::new(e.to_string()))?;
            produced_len = written;
            for i in 0..(written * 2) {
                mixed[i] += bank_out[i];
            }
        }
        if produced_len > 0 {
            out_stereo.extend_from_slice(&mixed[..produced_len * 2]);
        }
        self.staged = 0;
        self.processed_samples += samples as i64;
        Ok(())
    }

    fn finish_capacity(&self, tail_seconds: f64) -> u32 {
        let requested = if tail_seconds >= 0.0 {
            (tail_seconds * f64::from(RATE) - 1e-9).ceil() as i64
        } else {
            61200
        };
        let hrtf_bound = i64::from(self.hrtf_history_slots) * 64;
        let early_bound = hrtf_bound + 2048 + 256 * 64;
        let mut drain = requested.max(early_bound) + BINAURAL_LATENCY as i64;
        drain = ((drain + 63) / 64) * 64;
        drain.max(0) as u32
    }

    fn finish(&mut self, tail_seconds: f64, out_stereo: &mut Vec<f64>) -> Result<()> {
        if self.staged > 0 {
            let aligned = self.staged.div_ceil(BINAURAL_HOP) * BINAURAL_HOP;
            let stride = self.banks.len() * BINAURAL_CHANNELS;
            self.staging[self.staged * stride..aligned * stride].fill(0.0);
            let mut partial = Vec::new();
            self.process_block(aligned, &mut partial)?;
            out_stereo.extend_from_slice(&partial);
        }
        let mut rem = self.finish_capacity(tail_seconds) as usize;
        if rem == 0 || self.banks.is_empty() {
            return Ok(());
        }
        let mut chunk = vec![0.0f64; BINAURAL_BLOCK * 2];
        let mut combined = vec![0.0f64; BINAURAL_BLOCK * 2];
        while rem > 0 {
            let req = rem.min(BINAURAL_BLOCK);
            let mut written_any = 0usize;
            combined[..req * 2].fill(0.0);
            for bank in &mut self.banks {
                let n = bank.finish(req, &mut chunk).map_err(|e| crate::Error::new(e.to_string()))?;
                written_any = n;
                for i in 0..(n * 2) {
                    combined[i] += chunk[i];
                }
            }
            if written_any == 0 {
                break;
            }
            out_stereo.extend_from_slice(&combined[..written_any * 2]);
            rem -= written_any;
        }
        Ok(())
    }
}

fn bed_room_pos(label: &str, backs: bool) -> [f64; 3] {
    if backs {
        match label {
            "Ls" => return [0.0, 0.5, 0.0],
            "Rs" => return [1.0, 0.5, 0.0],
            _ => {}
        }
    }
    match label {
        "L" => [0.0, 0.0, 0.0],
        "R" => [1.0, 0.0, 0.0],
        "C" => [0.5, 0.0, 0.0],
        "Ls" | "Lb" => [0.0, 1.0, 0.0],
        "Rs" | "Rb" => [1.0, 1.0, 0.0],
        "Tfl" => [0.0, 0.0, 1.0],
        "Tfr" => [1.0, 0.0, 1.0],
        "Tsl" => [0.0, 0.5, 1.0],
        "Tsr" => [1.0, 0.5, 1.0],
        "Tbl" => [0.0, 1.0, 1.0],
        "Tbr" => [1.0, 1.0, 1.0],
        "Lw" => [0.0, 10.0 / 62.0, 0.0],
        "Rw" => [1.0, 10.0 / 62.0, 0.0],
        _ => [0.5, 0.0, 0.0],
    }
}

pub struct Ac4Stream {
    config: Ac4StreamConfig,
    decoder: FrameDecoder,
    gain: f64,
    waiting_for_iframe: bool,
    trimmed: usize,
    is_ajoc: Option<bool>,
    frame_len: usize,
    resampler: StereoResampler,
    speaker_renderer: Option<Renderer>,
    binaural: Option<BinauralEngine>,
    obj_buf: Vec<Vec<f64>>,
    lfe_buf: Vec<f64>,
    meta_queue: VecDeque<(usize, Frame)>,
    output: Vec<f32>,
    read_offset: usize,
    info: Ac4StreamInfo,
}

impl Ac4Stream {
    pub fn new(config: Ac4StreamConfig) -> Result<Self> {
        let gain = 10.0f64.powf(config.gain_db / 20.0);
        let binaural = if config.mode == Ac4Mode::Binaural {
            let bytes = config
                .hrtf
                .as_deref()
                .ok_or_else(|| crate::Error::new("binaural playback needs a compiled .jochrtf file"))?;
            Some(BinauralEngine::new(bytes, config.binaural_profile as u32)?)
        } else {
            None
        };
        Ok(Self {
            config,
            decoder: FrameDecoder::new(),
            gain,
            waiting_for_iframe: true,
            trimmed: 0,
            is_ajoc: None,
            frame_len: 0,
            resampler: StereoResampler::new(1, 1),
            speaker_renderer: None,
            binaural,
            obj_buf: Vec::new(),
            lfe_buf: Vec::new(),
            meta_queue: VecDeque::new(),
            output: Vec::new(),
            read_offset: 0,
            info: Ac4StreamInfo { output_channels: 2, ..Ac4StreamInfo::default() },
        })
    }

    pub fn info(&self) -> Ac4StreamInfo {
        self.info
    }

    pub fn set_gain_db(&mut self, gain_db: f64) {
        self.gain = 10.0f64.powf(gain_db / 20.0);
    }

    pub fn controls(&self) -> Option<Ac4Controls> {
        self.binaural.as_ref().map(|b| b.controls)
    }

    pub fn set_controls(&mut self, controls: &Ac4Controls) {
        if let Some(b) = self.binaural.as_mut() {
            b.set_controls(controls);
        }
    }

    pub fn take_scene(&mut self) -> Option<Ac4Scene> {
        if self.is_ajoc == Some(true) {
            self.binaural.as_mut().map(|b| b.take_scene())
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.decoder = FrameDecoder::new();
        self.waiting_for_iframe = true;
        self.trimmed = 0;
        self.resampler.reset();
        if let Some(r) = self.speaker_renderer.as_mut() {
            r.reset();
        }
        if let Some(b) = self.binaural.as_mut() {
            b.reset();
        }
        self.obj_buf.clear();
        self.lfe_buf.clear();
        self.meta_queue.clear();
        self.output.clear();
        self.read_offset = 0;
        self.info.ended = false;
    }

    pub fn push_frame(&mut self, data: &[u8]) -> Result<()> {
        let raw = toc::unwrap_sync_frame(data);
        let toc = read_toc(raw)?;
        if toc.presentations.is_empty() {
            return fail("AC-4 frame has no presentation");
        }
        let pres = &toc.presentations[0];
        let group = toc
            .groups
            .get(pres.groups[0])
            .ok_or_else(|| crate::Error::new("AC-4 presentation references missing group"))?;
        let info = group
            .substreams
            .first()
            .ok_or_else(|| crate::Error::new("AC-4 group has no substreams"))?;

        let (coded_num, coded_den, delivered) = toc.rates()?;
        let up = u64::from(delivered) * coded_den;
        let down = coded_num;
        let g = gcd(up, down);
        self.resampler.set_ratio(up / g, down / g);

        let frame_len = toc.frame_length();
        self.frame_len = frame_len;
        let ajoc = !group.channel_coded && info.kind == SubstreamKind::Ajoc;
        self.is_ajoc = Some(ajoc);

        let is_iframe = toc.iframe_global || info.iframe;
        if self.waiting_for_iframe {
            if !is_iframe {
                // Seek 命中非 I 帧时输出静音保持时间轴对齐，直到首个 I 帧初始化工具状态
                let silence = vec![0.0f64; frame_len * 2];
                self.push_stereo_coded(&silence);
                self.info.frames_in += 1;
                self.info.samples_in += frame_len as u64;
                return Ok(());
            }
            self.waiting_for_iframe = false;
        }

        let pcm_map = self.decoder.decode(raw)?;
        self.info.frames_in += 1;
        self.info.samples_in += frame_len as u64;

        if let Some(payload) = self.decoder.payload.take() {
            self.info.oamd_payloads += 1;
            let mut obj_names: Vec<(usize, String)> = pcm_map
                .keys()
                .filter_map(|k| k.strip_prefix('o').and_then(|s| s.parse::<usize>().ok()).map(|i| (i, k.clone())))
                .collect();
            obj_names.sort_by_key(|(i, _)| *i);
            let n_obj = obj_names.len();
            if self.obj_buf.len() != n_obj {
                self.obj_buf = vec![Vec::new(); n_obj];
            }
            let delay = self.decoder.delay;
            let cut = if self.trimmed < delay {
                let c = (delay - self.trimmed).min(frame_len);
                self.trimmed += c;
                c
            } else {
                0
            };
            let inv = 1.0 / 32768.0;
            for (o, (_, name)) in obj_names.iter().enumerate() {
                let src = &pcm_map[name];
                self.obj_buf[o].extend(src[cut..].iter().map(|&v| v * inv));
            }
            if let Some(lfe_src) = pcm_map.get("LFE") {
                self.lfe_buf.extend(lfe_src[cut..].iter().map(|&v| v * inv));
            } else {
                self.lfe_buf.resize(self.lfe_buf.len() + (frame_len - cut), 0.0);
            }
            self.meta_queue.push_back((frame_len, payload.metadata));
            self.render_ready_objects()?;
        } else {
            let delay = self.decoder.delay;
            let cut = if self.trimmed < delay {
                let c = (delay - self.trimmed).min(frame_len);
                self.trimmed += c;
                c
            } else {
                0
            };
            let mut stereo = downmix_stereo(&pcm_map, frame_len, cut);
            for v in &mut stereo {
                *v *= self.gain;
            }
            self.push_stereo_coded(&stereo);
            self.info.frames_out += 1;
        }
        Ok(())
    }

    fn render_ready_objects(&mut self) -> Result<()> {
        while let Some(&(flen, _)) = self.meta_queue.front() {
            if self.obj_buf.is_empty() || self.obj_buf[0].len() < flen {
                break;
            }
            let (_, meta) = self.meta_queue.pop_front().unwrap();
            let n_obj = self.obj_buf.len();
            let mut objs = Vec::with_capacity(n_obj);
            for buf in &mut self.obj_buf {
                objs.push(buf.drain(..flen).collect::<Vec<f64>>());
            }
            let lfe: Vec<f64> = self.lfe_buf.drain(..flen).collect();

            if self
                .speaker_renderer
                .as_ref()
                .is_none_or(|r| r.objects != n_obj)
            {
                self.speaker_renderer = Some(Renderer::new(n_obj, &LAYOUT_2_0));
            }
            let gains = self.speaker_renderer.as_mut().unwrap().gains(&meta, flen);
            let width = self.speaker_renderer.as_ref().unwrap().width;

            let mut stereo = Vec::new();
            if let Some(binaural) = self.binaural.as_mut() {
                binaural.submit_frame(&meta, &gains, width - 1, width, &objs, &lfe, flen, &mut stereo)?;
            } else {
                stereo.resize(flen * 2, 0.0);
                for o in 0..n_obj {
                    let g = &gains[o];
                    let ess = &objs[o];
                    for t in 0..flen {
                        let s = ess[t];
                        stereo[t * 2] += g[t * width] * s;
                        stereo[t * 2 + 1] += g[t * width + 1] * s;
                    }
                }
            }
            for v in &mut stereo {
                *v *= self.gain;
            }
            self.push_stereo_coded(&stereo);
            self.info.frames_out += 1;
        }
        Ok(())
    }

    fn push_stereo_coded(&mut self, stereo: &[f64]) {
        let before = self.output.len();
        self.resampler.push(stereo, &mut self.output);
        let added = (self.output.len() - before) / 2;
        self.info.samples_out += added as u64;
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.is_ajoc == Some(true) {
            let needed: usize = self.meta_queue.iter().map(|(flen, _)| *flen).sum();
            let have = self.obj_buf.first().map_or(0, Vec::len);
            if needed > have {
                let pad = needed - have;
                for buf in &mut self.obj_buf {
                    buf.resize(needed, 0.0);
                }
                self.lfe_buf.resize(self.lfe_buf.len() + pad, 0.0);
            }
            self.render_ready_objects()?;
            if let Some(binaural) = self.binaural.as_mut() {
                let mut tail = Vec::new();
                binaural.finish(self.config.tail_seconds, &mut tail)?;
                for v in &mut tail {
                    *v *= self.gain;
                }
                if !tail.is_empty() {
                    self.push_stereo_coded(&tail);
                }
            }
        } else if self.trimmed > 0 {
            let zeros = vec![0.0f64; self.trimmed * 2];
            self.push_stereo_coded(&zeros);
        }
        let before = self.output.len();
        self.resampler.finish(&mut self.output);
        let added = (self.output.len() - before) / 2;
        self.info.samples_out += added as u64;
        self.info.ended = true;
        Ok(())
    }

    pub fn tail_capacity(&self) -> u32 {
        self.binaural
            .as_ref()
            .map_or(0, |b| b.finish_capacity(self.config.tail_seconds))
    }

    pub fn buffered_frames(&self) -> usize {
        (self.output.len() - self.read_offset) / 2
    }

    pub fn pull(&mut self, out: &mut [f32], frames: usize) -> Result<usize> {
        let available = self.buffered_frames();
        let count = available.min(frames);
        if count != 0 {
            let start = self.read_offset;
            let end = start + count * 2;
            out[..count * 2].copy_from_slice(&self.output[start..end]);
            for &value in &out[..count * 2] {
                let mag = f64::from(value.abs());
                if !value.is_finite() {
                    self.info.non_finite += 1;
                } else {
                    if mag > self.info.peak {
                        self.info.peak = mag;
                    }
                    if mag > 1.0 {
                        self.info.over_unity += 1;
                    }
                }
            }
            self.read_offset = end;
            if self.read_offset == self.output.len() {
                self.output.clear();
                self.read_offset = 0;
            } else if self.read_offset >= 1 << 20 {
                self.output.drain(..self.read_offset);
                self.read_offset = 0;
            }
        }
        Ok(count)
    }
}

fn downmix_stereo(
    pcm: &std::collections::HashMap<String, Vec<f64>>,
    frame_len: usize,
    cut: usize,
) -> Vec<f64> {
    let n = frame_len - cut;
    let inv = 1.0 / 32768.0;
    let mut out = vec![0.0f64; n * 2];

    let add = |out: &mut [f64], key: &str, wl: f64, wr: f64| {
        if let Some(ch) = pcm.get(key) {
            for i in 0..n {
                let s = ch[cut + i] * inv;
                out[i * 2] += s * wl;
                out[i * 2 + 1] += s * wr;
            }
        }
    };

    if pcm.len() == 2 && pcm.contains_key("L") && pcm.contains_key("R") {
        add(&mut out, "L", 1.0, 0.0);
        add(&mut out, "R", 0.0, 1.0);
        return out;
    }

    add(&mut out, "L", 1.0, 0.0);
    add(&mut out, "R", 0.0, 1.0);
    add(&mut out, "C", FRAC_1_SQRT_2, FRAC_1_SQRT_2);
    add(&mut out, "Ls", FRAC_1_SQRT_2, 0.0);
    add(&mut out, "Rs", 0.0, FRAC_1_SQRT_2);
    add(&mut out, "Lb", FRAC_1_SQRT_2, 0.0);
    add(&mut out, "Rb", 0.0, FRAC_1_SQRT_2);
    add(&mut out, "Lw", FRAC_1_SQRT_2, 0.0);
    add(&mut out, "Rw", 0.0, FRAC_1_SQRT_2);
    add(&mut out, "Tfl", FRAC_1_SQRT_2, 0.0);
    add(&mut out, "Tfr", 0.0, FRAC_1_SQRT_2);
    out
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.max(1)
}
