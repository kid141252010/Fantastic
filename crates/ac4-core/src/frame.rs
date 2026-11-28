//! 将第一套呈示（presentation）的裸 AC-4 帧解码为各通道/对象 PCM（TS 103 190-1/2）。

use std::collections::{HashMap, VecDeque};

use crate::acpl;
use crate::ajoc::{self, AjocPayload, AjocState, Reconstructor};
use crate::asf::Noise;
use crate::aspx::{self, AspxFrame, Cursor, HFGEN_ATS};
use crate::bits::BitReader;
use crate::elements::{audio_data, Element, Reader, Tools, ASPX_ACPL_1, ASPX_ACPL_2, ASPX_ACPL_3, SIMPLE};
use crate::imdct::Imdct;
use crate::oamd::{self, Frame, Objects, Timing};
use crate::qmf::{Analysis, QmfMatrix, Synthesis, BANDS, DELAY};
use crate::render::{layout_for_channels, Renderer};
use crate::toc::{read_toc, SubstreamGroup, SubstreamKind, Toc, DYN};
use crate::{fail, Result};

pub const IMS_VERSION: u32 = 2;
pub const STEREO: u32 = 1;
pub const EXTRA: &str = "extra:";

/// Table 188（TS 103 190-1 clause 5.6）：每种帧长对应的 `(d_pcm, d_ctrl)`
pub fn alignment(frame_len: usize) -> Option<(usize, usize)> {
    match frame_len {
        1920 => Some((288, 1)),
        2048 => Some((352, 1)),
        1536 => Some((96, 1)),
        960 => Some((960, 2)),
        1024 => Some((1056, 2)),
        768 => Some((672, 2)),
        512 => Some((1312, 4)),
        384 => Some((864, 4)),
        _ => None,
    }
}

pub fn coded_channel_mode(presentation_version: u32, channel_mode: u32) -> u32 {
    if presentation_version == IMS_VERSION {
        STEREO
    } else {
        channel_mode
    }
}

fn check_size(bits: &BitReader<'_>, audio_start: usize, size: usize) -> Result<()> {
    if bits.pos.saturating_sub(audio_start) > size * 8 {
        return fail(format!(
            "audio data overran its {size} bytes by {} bits",
            bits.pos - audio_start - size * 8
        ));
    }
    Ok(())
}

fn joined(meta: &Frame, extra: &Frame) -> Frame {
    let n = meta.object_count;
    let mut program = meta.program.clone();
    program.dynamic_objects += extra.object_count;
    let mut trim = meta.trim.clone();
    for &idx in &extra.trim.exempt {
        let shifted = n + idx;
        if !trim.exempt.contains(&shifted) {
            trim.exempt.push(shifted);
        }
    }
    let mut updates = meta.updates.clone();
    updates.extend(extra.updates.iter().cloned());
    Frame {
        object_count: n + extra.object_count,
        program,
        updates,
        trim,
    }
}

pub struct FrameDecoder {
    transforms: HashMap<String, Imdct>,
    pub channel_mode: Option<u32>,
    tools: Option<Tools>,
    analysis: HashMap<String, Analysis>,
    synthesis: HashMap<String, Synthesis>,
    pub delay: usize,
    ajoc_state: AjocState,
    reconstructor: Option<Reconstructor>,
    pub payload: Option<AjocPayload>,
    held: HashMap<String, Vec<f64>>,
    controls: VecDeque<(Element, Option<AjocPayload>, Option<Frame>)>,
    pub resume: Option<HashMap<String, Cursor>>,
    extra: Option<Box<FrameDecoder>>,
    extra_element: Option<Element>,
    extra_meta: Option<Frame>,
    renderer: Option<Renderer>,
    is_extra: bool,
    objects: Option<Objects>,
    timing: Timing,
    meta: Option<Frame>,
    essences: Vec<&'static str>,
}

impl Default for FrameDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameDecoder {
    pub fn new() -> Self {
        Self {
            transforms: HashMap::new(),
            channel_mode: None,
            tools: None,
            analysis: HashMap::new(),
            synthesis: HashMap::new(),
            delay: 0,
            ajoc_state: AjocState::default(),
            reconstructor: None,
            payload: None,
            held: HashMap::new(),
            controls: VecDeque::new(),
            resume: None,
            extra: None,
            extra_element: None,
            extra_meta: None,
            renderer: None,
            is_extra: false,
            objects: None,
            timing: Timing { blocks: Vec::new() },
            meta: None,
            essences: Vec::new(),
        }
    }

    pub fn decode(&mut self, frame: &[u8]) -> Result<HashMap<String, Vec<f64>>> {
        let toc = read_toc(frame)?;
        let element = self.element(frame, &toc)?;
        let mut out = self.pcm(&toc, element)?;
        if let Some(extra) = self.extra.as_mut() {
            let extra_el = extra.extra_element.take().unwrap_or_else(|| {
                self.extra_element.take().expect("missing extra_element")
            });
            let objects = extra.pcm(&toc, extra_el)?;
            if extra.delay != self.delay {
                return fail("the extra objects came out with a delay of their own");
            }
            let essences: Vec<Vec<f64>> = extra
                .essences
                .iter()
                .map(|&name| objects[name].clone())
                .collect();
            if let Some(payload) = &self.payload {
                let first = payload.upmix;
                for (i, x) in essences.into_iter().enumerate() {
                    out.insert(format!("o{}", first + i), x);
                }
            } else {
                self.mix_extra(&mut out, &essences, toc.frame_length())?;
            }
        }
        Ok(out)
    }

    fn element(&mut self, frame: &[u8], toc: &Toc) -> Result<Element> {
        if toc.presentations.is_empty() {
            return fail("the frame has no presentation");
        }
        let presentation = &toc.presentations[0];
        let group = &toc.groups[presentation.groups[0]];
        let info = &group.substreams[0];
        if !group.channel_coded && info.kind != SubstreamKind::Ajoc {
            return fail("object coded substreams are only supported A-JOC coded");
        }

        let iframe_flag = info.iframe || toc.iframe_global;
        let frame_len = toc.frame_length();
        let base_48k = toc.fs_index == 1;
        let sub_idx = info.index;
        let (start, end) = Self::substream_bounds(frame, toc, sub_idx);

        if self.tools.as_ref().is_none_or(|t| t.frame_len != frame_len) {
            self.tools = Some(Tools::new(frame_len, base_48k));
        }
        let tools = self.tools.as_mut().unwrap();
        let mut bits = BitReader::with_range(frame, start * 8, end * 8)?;
        let mut size = bits.read(15)? as usize;
        if bits.flag()? {
            size += (bits.variable_bits(7)? as usize) << 15;
        }
        let audio_start = bits.pos;
        let mut reader = Reader::new(
            &mut bits,
            frame_len,
            iframe_flag,
            Noise::new(toc.sequence_counter),
            tools,
        );

        let element = if info.kind == SubstreamKind::Ajoc {
            self.ajoc_state.group_timing = None;
            if let Some(oamd_idx) = group.oamd_index {
                if !toc.substream_sizes.is_empty() {
                    let (at, stop) = Self::substream_bounds(frame, toc, oamd_idx);
                    let mut ob = BitReader::with_range(frame, at * 8, stop * 8)?;
                    ajoc::oamd_substream(&mut ob, &mut self.ajoc_state)?;
                }
            }
            let (el, pl) =
                ajoc::audio_data_ajoc(&mut reader, info, presentation.alternative, &mut self.ajoc_state)?;
            self.payload = Some(pl);
            el
        } else {
            let mode = coded_channel_mode(presentation.version, info.channel_mode);
            let el = audio_data(&mut reader, mode, info.add_ch_base)?;
            self.payload = None;
            el
        };
        check_size(&bits, audio_start, size)?;
        self.channel_mode = Some(info.channel_mode);

        let extra = &presentation.groups[1..];
        if !extra.is_empty() {
            if extra.len() > 1 {
                return fail("a presentation of more than two substream groups");
            }
            if self.extra.is_none() {
                let mut ex = FrameDecoder::new();
                ex.is_extra = true;
                self.extra = Some(Box::new(ex));
            }
            let extra_dec = self.extra.as_mut().unwrap();
            let ex_el = extra_dec.objects_element(frame, toc, &toc.groups[extra[0]])?;
            self.extra_element = Some(ex_el);
            if let (Some(pl), Some(ex_meta)) = (self.payload.as_mut(), extra_dec.meta.as_ref()) {
                pl.metadata = joined(&pl.metadata, ex_meta);
            }
        } else if self.extra.is_some() {
            return fail("the presentation's extra objects went away");
        }
        Ok(element)
    }

    fn substream_bounds(frame: &[u8], toc: &Toc, sub_idx: usize) -> (usize, usize) {
        let sum_before: usize = toc.substream_sizes.iter().take(sub_idx).sum();
        let start = toc.data_start + toc.payload_base + sum_before;
        let end = if !toc.substream_sizes.is_empty() {
            (start + toc.substream_sizes[sub_idx]).min(frame.len())
        } else {
            frame.len()
        };
        (start.min(frame.len()), end)
    }

    fn objects_element(
        &mut self,
        frame: &[u8],
        toc: &Toc,
        group: &SubstreamGroup,
    ) -> Result<Element> {
        let info = &group.substreams[0];
        if group.substreams.len() > 1 || info.kind != SubstreamKind::Obj {
            return fail("a second substream group is only supported as one object substream");
        }
        if !matches!(info.objects.len(), 1 | 2)
            || info.objects.iter().any(|&(k, lfe)| k != DYN || lfe)
        {
            return fail(format!(
                "{} object essences are not supported yet",
                info.objects.len()
            ));
        }
        let Some(oamd_idx) = group.oamd_index else {
            return fail("object essences without an oamd_substream");
        };
        if toc.substream_sizes.is_empty() && self.is_extra {
            return fail("an object substream without a substream_index_table");
        }
        let frame_len = toc.frame_length();
        let base_48k = toc.fs_index == 1;
        let iframe = info.iframe || toc.iframe_global;
        let (start, end) = Self::substream_bounds(frame, toc, info.index);
        if self.tools.as_ref().is_none_or(|t| t.frame_len != frame_len) {
            self.tools = Some(Tools::new(frame_len, base_48k));
        }
        let tools = self.tools.as_mut().unwrap();
        let mut bits = BitReader::with_range(frame, start * 8, end * 8)?;
        let mut size = bits.read(15)? as usize;
        if bits.flag()? {
            size += (bits.variable_bits(7)? as usize) << 15;
        }
        let audio_start = bits.pos;
        let mut reader = Reader::new(
            &mut bits,
            frame_len,
            iframe,
            Noise::new(toc.sequence_counter),
            tools,
        );
        let element = audio_data(&mut reader, (info.objects.len() - 1) as u32, 0)?;
        check_size(&bits, audio_start, size)?;

        if self.objects.is_none() || iframe {
            let (types, program) = oamd::program_for(&info.objects, &[], None, false);
            let trim = self
                .objects
                .as_ref()
                .map(|o| o.trim.clone())
                .unwrap_or_default();
            self.objects = Some(Objects::new(types, program, trim));
        }
        let (at, stop) = Self::substream_bounds(frame, toc, oamd_idx);
        let mut b = BitReader::with_range(frame, at * 8, stop * 8)?;
        if b.flag()? {
            if let Some(trim) = oamd::read_common(&mut b)? {
                self.objects.as_mut().unwrap().trim = trim;
            }
        }
        if b.flag()? {
            self.timing = oamd::read_timing(&mut b)?;
        }
        if toc.presentations[0].alternative {
            return fail("alternative object properties are not supported yet");
        }
        self.meta = Some(oamd::read_dyndata(
            &mut b,
            self.objects.as_mut().unwrap(),
            &self.timing,
            iframe,
            false,
        )?);
        self.essences = if info.objects.len() == 1 {
            vec!["C"]
        } else {
            vec!["L", "R"]
        };
        Ok(element)
    }

    fn mix_extra(
        &mut self,
        out: &mut HashMap<String, Vec<f64>>,
        essences: &[Vec<f64>],
        frame_len: usize,
    ) -> Result<()> {
        let names: Vec<&str> = out.keys().map(String::as_str).collect();
        let layout = layout_for_channels(&names)
            .ok_or_else(|| crate::Error::new("no speaker layout to render extra objects"))?;
        if self.renderer.is_none() {
            self.renderer = Some(Renderer::new(essences.len(), layout));
        }
        let renderer = self.renderer.as_mut().unwrap();
        let meta = self.extra_meta.as_ref().unwrap();
        let gains = renderer.gains(meta, frame_len);
        let width = renderer.width;
        let spk_count = layout.channels.len();
        for (s_idx, &ch_name) in layout.channels.iter().enumerate() {
            if let Some(buf) = out.get_mut(ch_name) {
                for t in 0..frame_len {
                    let mut sum = 0.0;
                    for (o, ess) in essences.iter().enumerate() {
                        sum += gains[o][t * width + s_idx] * ess[t];
                    }
                    buf[t] += sum;
                }
            }
        }
        let _ = spk_count;
        Ok(())
    }

    fn pcm(&mut self, toc: &Toc, element: Element) -> Result<HashMap<String, Vec<f64>>> {
        let frame_len = toc.frame_length();
        let mut out: HashMap<String, Vec<f64>> = HashMap::new();
        for (name, track) in &element.channels {
            let transform = self
                .transforms
                .entry(name.clone())
                .or_insert_with(|| Imdct::new(frame_len));
            out.insert(name.clone(), transform.process(&track.spectrum, track.windows()));
        }
        let banks = self.is_extra || self.extra.is_some();
        if element.qmf() || self.delay > 0 || self.payload.is_some() || banks {
            let (pcm_delay, ctrl_delay) = alignment(frame_len)
                .ok_or_else(|| crate::Error::new(format!("unsupported frame_len {frame_len}")))?;
            for (name, x) in out.iter_mut() {
                let held = self
                    .held
                    .entry(name.clone())
                    .or_insert_with(|| vec![0.0; pcm_delay]);
                let mut y = Vec::with_capacity(held.len() + x.len());
                y.extend_from_slice(held);
                y.extend_from_slice(x);
                let len = x.len();
                x.copy_from_slice(&y[..len]);
                *held = y[len..].to_vec();
            }
            let extra_meta = self.extra.as_ref().and_then(|ex| ex.meta.clone());
            self.controls
                .push_back((element, self.payload.take(), extra_meta));
            let (el, pl, ex_meta) = if self.controls.len() > ctrl_delay {
                let popped = self.controls.pop_front().unwrap();
                if let Some(resume) = self.resume.take() {
                    let tools = self.tools.as_mut().unwrap();
                    let mut extra_resume = HashMap::new();
                    for (name, cursor) in resume {
                        if let Some(stripped) = name.strip_prefix(EXTRA) {
                            extra_resume.insert(stripped.to_string(), cursor);
                        } else {
                            cursor.restore(tools.channel_mut(&name));
                        }
                    }
                    if let Some(extra) = self.extra.as_mut() {
                        extra.resume = Some(extra_resume);
                    }
                }
                popped
            } else {
                let front = self.controls.front().unwrap();
                (front.0.clone(), front.1.clone(), front.2.clone())
            };
            self.payload = pl;
            self.extra_meta = ex_meta;
            out = self.qmf(&el, out, frame_len);
            self.delay += pcm_delay;
        }
        Ok(out)
    }

    fn qmf(
        &mut self,
        element: &Element,
        pcm: HashMap<String, Vec<f64>>,
        frame_len: usize,
    ) -> HashMap<String, Vec<f64>> {
        let tools = self.tools.as_mut().unwrap();
        let mut q: HashMap<String, QmfMatrix> = HashMap::new();
        for (name, x) in pcm {
            let bank = self.analysis.entry(name.clone()).or_default();
            q.insert(name, bank.process(&x));
        }
        let k = if frame_len >= 1536 { 2 } else { 1 };

        if let Some(comp) = &element.companding {
            let mut spans = Vec::with_capacity(comp.channels.len());
            let mut mats: Vec<QmfMatrix> = Vec::with_capacity(comp.channels.len());
            for name in &comp.channels {
                let sbx = tools
                    .aspx
                    .get(name)
                    .and_then(|ch| ch.bands.as_ref())
                    .map_or(BANDS, |b| b.sbx());
                let sb0 = if element.codec_mode == ASPX_ACPL_1 && !element.acpl.is_empty() {
                    element.acpl[0].config.qmf_band
                } else {
                    0
                };
                let slots = q[name].slots;
                let (ts0, ts1) = if let Some(f) = element.aspx_frame(name) {
                    (f.sig_borders[0] * k, (*f.sig_borders.last().unwrap() * k).min(slots))
                } else {
                    (0, slots)
                };
                spans.push((sb0, sbx, ts0, ts1));
                let mut m = q[name].clone();
                for c in &mut m.data {
                    *c = c.scale(1.0 / 32768.0);
                }
                mats.push(m);
            }
            {
                let mut refs: Vec<&mut QmfMatrix> = mats.iter_mut().collect();
                aspx::compand(&mut refs, &spans, &comp.on, comp.avg, comp.sync);
            }
            for (name, mut m) in comp.channels.iter().zip(mats) {
                for c in &mut m.data {
                    *c = c.scale(32768.0);
                }
                q.insert(name.clone(), m);
            }
        }

        // A-SPX：按插入顺序处理平衡对或单通道，其余通道仅做相同延迟
        let mut done: Vec<String> = Vec::new();
        let _ = ASPX_ACPL_2;
        let _ = ASPX_ACPL_3;
        let _ = SIMPLE;

        for i in 0..element.aspx.len() {
            let name = &element.aspx[i].0;
            if done.contains(name) {
                continue;
            }
            let mut group = vec![name.clone()];
            if i + 1 < element.aspx.len() && element.aspx[i + 1].1.second {
                group.push(element.aspx[i + 1].0.clone());
            }
            if group.len() == 2 {
                let n0 = &group[0];
                let n1 = &group[1];
                let mut ch0 = tools.aspx.remove(n0).unwrap_or_else(|| aspx::AspxChannel::new(frame_len));
                let mut ch1 = tools.aspx.remove(n1).unwrap_or_else(|| aspx::AspxChannel::new(frame_len));
                let f0 = element.aspx_frame(n0).unwrap();
                let f1 = element.aspx_frame(n1).unwrap();
                let q0 = &q[n0];
                let q1 = &q[n1];
                let out_mats = aspx::decode_channels(&mut [(&mut ch0, f0), (&mut ch1, f1)], &[q0, q1]);
                tools.aspx.insert(n0.clone(), ch0);
                tools.aspx.insert(n1.clone(), ch1);
                q.insert(n0.clone(), out_mats[0].clone());
                q.insert(n1.clone(), out_mats[1].clone());
            } else {
                let n0 = &group[0];
                let ch0 = tools.channel_mut(n0);
                let f0 = element.aspx_frame(n0).unwrap();
                let q0 = &q[n0];
                let out_mats = aspx::decode_channels(&mut [(ch0, f0)], &[q0]);
                q.insert(n0.clone(), out_mats[0].clone());
            }
            done.extend(group);
        }

        let q_keys: Vec<String> = q.keys().cloned().collect();
        for name in q_keys {
            if !done.contains(&name) {
                let ch = tools.channel_mut(&name);
                let delayed = aspx::skip_channel(ch, &q[&name]);
                q.insert(name, delayed);
            }
        }

        for c in &element.acpl {
            let src = &q[&c.source];
            let res = c.residual.as_ref().and_then(|r| q.get(r));
            let module = tools.module_mut(&c.module, c.decorrelator);
            let (mut z0, mut z1) = acpl::couple(module, &c.data, c.config, src, res);
            if c.scales[0] != 1.0 {
                for v in &mut z0.data {
                    *v = v.scale(c.scales[0]);
                }
            }
            if c.scales[1] != 1.0 {
                for v in &mut z1.data {
                    *v = v.scale(c.scales[1]);
                }
            }
            q.insert(c.outputs[0].clone(), z0);
            q.insert(c.outputs[1].clone(), z1);
        }

        for (name, gain) in &element.gains {
            if let Some(m) = q.get_mut(name) {
                for v in &mut m.data {
                    *v = v.scale(*gain);
                }
            }
        }

        if self.payload.is_some() {
            q = self.rebuild_objects(&q);
        }

        let mut out = HashMap::new();
        for (name, m) in q {
            let bank = self.synthesis.entry(name.clone()).or_default();
            out.insert(name, bank.process(&m));
        }
        self.delay = DELAY + HFGEN_ATS * k * BANDS;
        out
    }

    fn rebuild_objects(&mut self, q: &HashMap<String, QmfMatrix>) -> HashMap<String, QmfMatrix> {
        let p = self.payload.as_ref().unwrap();
        let n_dmx = p.inputs.len();
        let n_umx = p.upmix;
        if self.reconstructor.as_ref().is_none_or(|r| (r.n_dmx, r.n_umx) != (n_dmx, n_umx)) {
            self.reconstructor = Some(Reconstructor::new(n_dmx, n_umx));
        }
        let r = self.reconstructor.as_mut().unwrap();
        let inputs: Vec<&QmfMatrix> = p.inputs.iter().map(|name| &q[name]).collect();
        let z = r.process(&p.frame, &inputs);
        let mut out = HashMap::new();
        for (i, mat) in z.into_iter().enumerate() {
            out.insert(format!("o{i}"), mat);
        }
        if let Some(lfe_name) = &p.lfe {
            if let Some(lfe_mat) = q.get(lfe_name) {
                out.insert("LFE".to_string(), lfe_mat.clone());
            }
        }
        out
    }
}

/// 仅供 `element.aspx` 顺序检查（Rust HashMap 顺序不固定，所以上面按 `element.channels` 顺序保证配对顺序一致）。
#[allow(dead_code)]
fn _check_aspx_pair(_: &AspxFrame) {}
