//! 声道编码子流的通道元素解析（TS 103 190-1 clauses 4.2.5、4.2.6 与 5.3.4）。

use crate::acpl::{self, AcplConfig, AcplData, Module, FULL, PARTIAL};
use crate::asf::{read_layout, read_lfe_layout, read_master, read_spectrum, Layout, Noise};
use crate::aspx::{self, AspxChannel, AspxConfig, AspxFrame};
use crate::bits::BitReader;
use crate::smp::{self, Matrix};
use crate::{fail, Result};

pub const SIMPLE: u32 = 0;
pub const ASPX: u32 = 1;
pub const ASPX_ACPL_1: u32 = 2;
pub const ASPX_ACPL_2: u32 = 3;
pub const ASPX_ACPL_3: u32 = 4;

const ASF: u32 = 0;
pub const SQRT2: f64 = std::f64::consts::SQRT_2;

#[derive(Clone, Debug)]
pub struct Track {
    pub spectrum: Vec<f64>,
    pub layout: Layout,
}

impl Track {
    pub fn windows(&self) -> &[usize] {
        &self.layout.window_lengths
    }
}

#[derive(Clone, Debug)]
pub struct Companding {
    pub channels: Vec<String>,
    pub sync: bool,
    pub on: Vec<bool>,
    pub avg: bool,
}

#[derive(Clone, Debug)]
pub struct Coupling {
    pub module: String,
    pub decorrelator: usize,
    pub data: AcplData,
    pub config: AcplConfig,
    pub source: String,
    pub residual: Option<String>,
    pub outputs: [String; 2],
    pub scales: [f64; 2],
}

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Element {
    pub channels: Vec<(String, Track)>,
    pub codec_mode: u32,
    pub companding: Option<Companding>,
    pub aspx: Vec<(String, AspxFrame)>,
    pub acpl: Vec<Coupling>,
    pub gains: Vec<(String, f64)>,
}

impl Element {
    pub fn new(codec_mode: u32) -> Self {
        Self {
            channels: Vec::new(),
            codec_mode,
            companding: None,
            aspx: Vec::new(),
            acpl: Vec::new(),
            gains: Vec::new(),
        }
    }

    #[inline]
    pub fn qmf(&self) -> bool {
        self.codec_mode != SIMPLE
    }

    pub fn set_channel(&mut self, name: impl Into<String>, track: Track) {
        let name = name.into();
        if let Some(entry) = self.channels.iter_mut().find(|(n, _)| n == &name) {
            entry.1 = track;
        } else {
            self.channels.push((name, track));
        }
    }

    pub fn channel(&self, name: &str) -> Option<&Track> {
        self.channels.iter().find(|(n, _)| n == name).map(|(_, t)| t)
    }

    pub fn set_aspx(&mut self, name: impl Into<String>, frame: AspxFrame) {
        let name = name.into();
        if let Some(entry) = self.aspx.iter_mut().find(|(n, _)| n == &name) {
            entry.1 = frame;
        } else {
            self.aspx.push((name, frame));
        }
    }

    pub fn aspx_frame(&self, name: &str) -> Option<&AspxFrame> {
        self.aspx.iter().find(|(n, _)| n == name).map(|(_, f)| f)
    }
}

pub struct Tools {
    pub frame_len: usize,
    pub base_48k: bool,
    pub aspx_config: Option<AspxConfig>,
    pub aspx: HashMap<String, AspxChannel>,
    pub acpl_config: Option<AcplConfig>,
    pub modules: HashMap<String, Module>,
}

impl Tools {
    pub fn new(frame_len: usize, base_48k: bool) -> Self {
        Self {
            frame_len,
            base_48k,
            aspx_config: None,
            aspx: HashMap::new(),
            acpl_config: None,
            modules: HashMap::new(),
        }
    }

    pub fn module_mut(&mut self, name: &str, decorrelator: usize) -> &mut Module {
        self.modules
            .entry(name.to_string())
            .or_insert_with(|| Module::new(decorrelator))
    }

    pub fn channel_mut(&mut self, name: &str) -> &mut AspxChannel {
        let frame_len = self.frame_len;
        self.aspx
            .entry(name.to_string())
            .or_insert_with(|| AspxChannel::new(frame_len))
    }
}

pub struct Reader<'a, 'b> {
    pub b: &'a mut BitReader<'b>,
    pub frame_len: usize,
    pub iframe: bool,
    pub noise: Noise,
    pub tools: &'a mut Tools,
    pub last_layout: Option<Layout>,
}

impl<'a, 'b> Reader<'a, 'b> {
    pub fn new(
        b: &'a mut BitReader<'b>,
        frame_len: usize,
        iframe: bool,
        noise: Noise,
        tools: &'a mut Tools,
    ) -> Self {
        Self {
            b,
            frame_len,
            iframe,
            noise,
            tools,
            last_layout: None,
        }
    }

    pub fn aspx_config(&mut self) -> Result<()> {
        if self.iframe {
            self.tools.aspx_config = Some(aspx::read_config(self.b)?);
        } else if self.tools.aspx_config.is_none() {
            return fail("A-SPX coded frame before the I-frame that configures it");
        }
        Ok(())
    }

    pub fn configs(&mut self, mode: u32) -> Result<()> {
        if mode == SIMPLE {
            return Ok(());
        }
        self.aspx_config()?;
        if matches!(mode, ASPX_ACPL_1 | ASPX_ACPL_2 | ASPX_ACPL_3) {
            if self.iframe {
                let cfg = if mode == ASPX_ACPL_3 {
                    acpl::read_config_2ch(self.b)?
                } else {
                    let partial = if mode == ASPX_ACPL_1 { PARTIAL } else { FULL };
                    acpl::read_config_1ch(self.b, partial)?
                };
                self.tools.acpl_config = Some(cfg);
            } else if self.tools.acpl_config.is_none() {
                return fail("A-CPL coded frame before the I-frame that configures it");
            }
        }
        Ok(())
    }

    pub fn coupling(
        &mut self,
        element: &mut Element,
        module: &str,
        decorrelator: usize,
        source: &str,
        residual: Option<&str>,
        outputs: [&str; 2],
        scales: [f64; 2],
    ) -> Result<()> {
        let cfg = self
            .tools
            .acpl_config
            .ok_or_else(|| crate::Error::new("missing acpl_config"))?;
        let data = acpl::read_data_1ch(self.b, cfg)?;
        element.acpl.push(Coupling {
            module: module.to_string(),
            decorrelator,
            data,
            config: cfg,
            source: source.to_string(),
            residual: residual.map(str::to_string),
            outputs: [outputs[0].to_string(), outputs[1].to_string()],
            scales,
        });
        Ok(())
    }

    pub fn companding(&mut self, names: &[&str]) -> Result<Companding> {
        let sync = if names.len() > 1 { self.b.flag()? } else { false };
        let count = if sync { 1 } else { names.len() };
        let mut on = Vec::with_capacity(count);
        for _ in 0..count {
            on.push(self.b.flag()?);
        }
        let avg = if !on.iter().all(|&x| x) {
            self.b.flag()?
        } else {
            false
        };
        Ok(Companding {
            channels: names.iter().map(|s| s.to_string()).collect(),
            sync,
            on,
            avg,
        })
    }

    pub fn aspx_1ch(&mut self, element: &mut Element, name: &str) -> Result<()> {
        let cfg = self.tools.aspx_config;
        let iframe = self.iframe;
        let base_48k = self.tools.base_48k;
        let ch = self.tools.channel_mut(name);
        let f = aspx::read_data_1ch(self.b, ch, cfg, iframe, base_48k)?;
        element.set_aspx(name, f);
        Ok(())
    }

    pub fn aspx_2ch(&mut self, element: &mut Element, first: &str, second: &str) -> Result<()> {
        let cfg = self.tools.aspx_config;
        let iframe = self.iframe;
        let base_48k = self.tools.base_48k;
        let frame_len = self.tools.frame_len;
        // 为同时可变借用两个通道，先取出并插回
        let mut a = self
            .tools
            .aspx
            .remove(first)
            .unwrap_or_else(|| AspxChannel::new(frame_len));
        let mut c = self
            .tools
            .aspx
            .remove(second)
            .unwrap_or_else(|| AspxChannel::new(frame_len));
        let res = aspx::read_data_2ch(self.b, &mut a, &mut c, cfg, iframe, base_48k);
        self.tools.aspx.insert(first.to_string(), a);
        self.tools.aspx.insert(second.to_string(), c);
        let (f0, f1) = res?;
        element.set_aspx(first, f0);
        element.set_aspx(second, f1);
        Ok(())
    }

    pub fn layout(&mut self, dual_maxsfb: bool, side_limited: bool) -> Result<Layout> {
        let l = read_layout(self.b, self.frame_len, dual_maxsfb, side_limited)?;
        self.last_layout = Some(l.clone());
        Ok(l)
    }

    pub fn spectrum(&mut self, layout: &Layout, side: bool) -> Result<Track> {
        let spec = read_spectrum(self.b, layout, &mut self.noise, side)?;
        Ok(Track { spectrum: spec, layout: layout.clone() })
    }

    pub fn frontend(&mut self) -> Result<()> {
        if self.b.read(1)? != ASF {
            return fail("the speech spectral frontend is not supported");
        }
        Ok(())
    }
}

pub fn mono_data(r: &mut Reader<'_, '_>, lfe: bool) -> Result<Track> {
    let layout = if lfe {
        read_lfe_layout(r.b, r.frame_len)?
    } else {
        r.frontend()?;
        r.layout(false, false)?
    };
    r.spectrum(&layout, false)
}

fn pair(r: &mut Reader<'_, '_>, stereo: bool) -> Result<[Track; 2]> {
    if r.b.flag()? {
        let layout = r.layout(false, false)?;
        let param = smp::read_chparam(r.b, &layout)?;
        let t0 = r.spectrum(&layout, false)?;
        let t1 = r.spectrum(&layout, false)?;
        let (x0, x1) = smp::matrix(&param, &layout).apply(&t0.spectrum, &t1.spectrum);
        return Ok([
            Track { spectrum: x0, layout: layout.clone() },
            Track { spectrum: x1, layout },
        ]);
    }
    if stereo {
        r.frontend()?;
    }
    let l0 = r.layout(false, false)?;
    if stereo {
        r.frontend()?;
    }
    let l1 = r.layout(false, false)?;
    Ok([r.spectrum(&l0, false)?, r.spectrum(&l1, false)?])
}

pub fn two_channel_data(r: &mut Reader<'_, '_>) -> Result<[Track; 2]> {
    pair(r, false)
}

pub fn stereo_data(r: &mut Reader<'_, '_>) -> Result<[Track; 2]> {
    pair(r, true)
}

pub fn three_channel_data(r: &mut Reader<'_, '_>) -> Result<[Track; 3]> {
    let layout = r.layout(false, false)?;
    let selector = r.b.read(4)?;
    let p0 = smp::read_chparam(r.b, &layout)?;
    let p1 = smp::read_chparam(r.b, &layout)?;
    let t0 = r.spectrum(&layout, false)?;
    let t1 = r.spectrum(&layout, false)?;
    let t2 = r.spectrum(&layout, false)?;
    let m0 = smp::matrix(&p0, &layout);
    let m1 = smp::matrix(&p1, &layout);
    let [x0, x1, x2] = smp::three(selector, &m0, &m1, [&t0.spectrum, &t1.spectrum, &t2.spectrum])?;
    Ok([
        Track { spectrum: x0, layout: layout.clone() },
        Track { spectrum: x1, layout: layout.clone() },
        Track { spectrum: x2, layout },
    ])
}

pub fn four_channel_data(r: &mut Reader<'_, '_>) -> Result<[Track; 4]> {
    let layout = r.layout(false, false)?;
    let p0 = smp::read_chparam(r.b, &layout)?;
    let p1 = smp::read_chparam(r.b, &layout)?;
    let p2 = smp::read_chparam(r.b, &layout)?;
    let p3 = smp::read_chparam(r.b, &layout)?;
    let t0 = r.spectrum(&layout, false)?;
    let t1 = r.spectrum(&layout, false)?;
    let t2 = r.spectrum(&layout, false)?;
    let t3 = r.spectrum(&layout, false)?;
    let mats = [
        smp::matrix(&p0, &layout),
        smp::matrix(&p1, &layout),
        smp::matrix(&p2, &layout),
        smp::matrix(&p3, &layout),
    ];
    let [x0, x1, x2, x3] = smp::four(&mats, [&t0.spectrum, &t1.spectrum, &t2.spectrum, &t3.spectrum]);
    Ok([
        Track { spectrum: x0, layout: layout.clone() },
        Track { spectrum: x1, layout: layout.clone() },
        Track { spectrum: x2, layout: layout.clone() },
        Track { spectrum: x3, layout },
    ])
}

pub fn five_channel_data(r: &mut Reader<'_, '_>) -> Result<[Track; 5]> {
    let layout = r.layout(false, false)?;
    let selector = r.b.read(4)?;
    let p0 = smp::read_chparam(r.b, &layout)?;
    let p1 = smp::read_chparam(r.b, &layout)?;
    let p2 = smp::read_chparam(r.b, &layout)?;
    let p3 = smp::read_chparam(r.b, &layout)?;
    let p4 = smp::read_chparam(r.b, &layout)?;
    let t0 = r.spectrum(&layout, false)?;
    let t1 = r.spectrum(&layout, false)?;
    let t2 = r.spectrum(&layout, false)?;
    let t3 = r.spectrum(&layout, false)?;
    let t4 = r.spectrum(&layout, false)?;
    let mats = [
        smp::matrix(&p0, &layout),
        smp::matrix(&p1, &layout),
        smp::matrix(&p2, &layout),
        smp::matrix(&p3, &layout),
        smp::matrix(&p4, &layout),
    ];
    let [x0, x1, x2, x3, x4] = smp::five(
        selector,
        &mats,
        [&t0.spectrum, &t1.spectrum, &t2.spectrum, &t3.spectrum, &t4.spectrum],
    )?;
    Ok([
        Track { spectrum: x0, layout: layout.clone() },
        Track { spectrum: x1, layout: layout.clone() },
        Track { spectrum: x2, layout: layout.clone() },
        Track { spectrum: x3, layout: layout.clone() },
        Track { spectrum: x4, layout },
    ])
}

pub fn single_channel_element(r: &mut Reader<'_, '_>) -> Result<Element> {
    let mode = r.b.read(1)?;
    r.configs(mode)?;
    let mut element = Element::new(mode);
    if mode == ASPX {
        element.companding = Some(r.companding(&["C"])?);
    }
    let c = mono_data(r, false)?;
    element.set_channel("C", c);
    if mode == ASPX {
        r.aspx_1ch(&mut element, "C")?;
    }
    Ok(element)
}

pub fn channel_pair_element(r: &mut Reader<'_, '_>) -> Result<Element> {
    let mode = r.b.read(2)?;
    r.configs(mode)?;
    let mut element = Element::new(mode);
    if mode == SIMPLE || mode == ASPX {
        if mode == ASPX {
            element.companding = Some(r.companding(&["L", "R"])?);
        }
        let [tl, tr] = stereo_data(r)?;
        element.set_channel("L", tl);
        element.set_channel("R", tr);
        if mode == ASPX {
            r.aspx_2ch(&mut element, "L", "R")?;
        }
        return Ok(element);
    }
    element.companding = Some(r.companding(&["L"])?);
    if mode == ASPX_ACPL_1 {
        if r.b.flag()? {
            let layout = r.layout(true, false)?;
            let param = smp::read_chparam(r.b, &layout)?;
            let mid = r.spectrum(&layout, false)?;
            let side = r.spectrum(&layout, true)?;
            let (x0, x1) = smp::matrix(&param, &layout).apply(&mid.spectrum, &side.spectrum);
            element.set_channel("L", Track { spectrum: x0, layout: layout.clone() });
            element.set_channel("R", Track { spectrum: x1, layout });
        } else {
            r.frontend()?;
            let l0 = r.layout(false, false)?;
            r.frontend()?;
            let l1 = r.layout(false, true)?;
            let tl = r.spectrum(&l0, false)?;
            let tr = r.spectrum(&l1, false)?;
            element.set_channel("L", tl);
            element.set_channel("R", tr);
        }
        r.aspx_1ch(&mut element, "L")?;
        r.coupling(&mut element, "pair", 0, "L", Some("R"), ["L", "R"], [1.0, 1.0])?;
    } else {
        r.frontend()?;
        let layout = r.layout(false, false)?;
        let tl = r.spectrum(&layout, false)?;
        let frame_len = r.frame_len;
        element.set_channel("L", tl);
        element.set_channel("R", Track { spectrum: vec![0.0; frame_len], layout });
        r.aspx_1ch(&mut element, "L")?;
        r.coupling(&mut element, "pair", 0, "L", None, ["L", "R"], [1.0, 1.0])?;
    }
    Ok(element)
}

pub fn three_zero_element(r: &mut Reader<'_, '_>) -> Result<Element> {
    let mode = r.b.read(1)?;
    if mode == ASPX {
        r.aspx_config()?;
    }
    let mut element = Element::new(mode);
    if mode == ASPX {
        element.companding = Some(r.companding(&["L", "R", "C"])?);
    }
    if r.b.read(1)? != 0 {
        let [tl, tr, tc] = three_channel_data(r)?;
        element.set_channel("L", tl);
        element.set_channel("R", tr);
        element.set_channel("C", tc);
    } else {
        let [tl, tr] = stereo_data(r)?;
        element.set_channel("L", tl);
        element.set_channel("R", tr);
        let tc = mono_data(r, false)?;
        element.set_channel("C", tc);
    }
    if mode == ASPX {
        r.aspx_2ch(&mut element, "L", "R")?;
        r.aspx_1ch(&mut element, "C")?;
    }
    Ok(element)
}

fn residuals(
    r: &mut Reader<'_, '_>,
    element: &mut Element,
    pairs: [(&str, &str); 2],
    last: &Layout,
) -> Result<()> {
    let layout = read_master(r.b, last)?;
    let p0 = smp::read_chparam(r.b, &layout)?;
    let p1 = smp::read_chparam(r.b, &layout)?;
    let t0 = r.spectrum(&layout, false)?;
    let t1 = r.spectrum(&layout, false)?;
    for (((front, back), param), track) in pairs.into_iter().zip([p0, p1]).zip([t0, t1]) {
        let front_track = element.channel(front).unwrap().clone();
        let (x0, x1) = smp::matrix(&param, &layout).apply(&front_track.spectrum, &track.spectrum);
        element.set_channel(front, Track { spectrum: x0, layout: front_track.layout });
        element.set_channel(back, Track { spectrum: x1, layout: layout.clone() });
    }
    Ok(())
}

fn core_five(r: &mut Reader<'_, '_>, element: &mut Element, names: [&str; 5]) -> Result<u32> {
    let config = r.b.read(2)?;
    let [a, b, c, d, e] = names;
    match config {
        0 => {
            let swap = r.b.flag()?;
            let first = two_channel_data(r)?;
            let second = two_channel_data(r)?;
            let pairs = if swap { [(a, d), (b, e)] } else { [(a, b), (d, e)] };
            for ((n0, n1), [t0, t1]) in pairs.into_iter().zip([first, second]) {
                element.set_channel(n0, t0);
                element.set_channel(n1, t1);
            }
        }
        1 => {
            let [t0, t1, t2] = three_channel_data(r)?;
            element.set_channel(a, t0);
            element.set_channel(b, t1);
            element.set_channel(c, t2);
            let [t3, t4] = two_channel_data(r)?;
            element.set_channel(d, t3);
            element.set_channel(e, t4);
        }
        2 => {
            let [t0, t1, t2, t3] = four_channel_data(r)?;
            element.set_channel(a, t0);
            element.set_channel(b, t1);
            element.set_channel(d, t2);
            element.set_channel(e, t3);
        }
        _ => {
            let [t0, t1, t2, t3, t4] = five_channel_data(r)?;
            element.set_channel(a, t0);
            element.set_channel(b, t1);
            element.set_channel(c, t2);
            element.set_channel(d, t3);
            element.set_channel(e, t4);
        }
    }
    Ok(config)
}

pub fn five_x_element(r: &mut Reader<'_, '_>, lfe: bool) -> Result<Element> {
    let mode = r.b.read(3)?;
    if mode >= ASPX_ACPL_3 {
        return fail(format!("5.X codec mode {mode} is not supported yet"));
    }
    r.configs(mode)?;
    let mut element = Element::new(mode);
    if lfe {
        let t = mono_data(r, true)?;
        element.set_channel("LFE", t);
    }
    if mode == SIMPLE || mode == ASPX {
        if mode == ASPX {
            element.companding = Some(r.companding(&["L", "R", "C", "Ls", "Rs"])?);
        }
        let config = core_five(r, &mut element, ["L", "R", "C", "Ls", "Rs"])?;
        if config == 0 || config == 2 {
            let t = mono_data(r, false)?;
            element.set_channel("C", t);
        }
        if mode == ASPX {
            r.aspx_2ch(&mut element, "L", "R")?;
            r.aspx_2ch(&mut element, "Ls", "Rs")?;
            r.aspx_1ch(&mut element, "C")?;
        }
        return Ok(element);
    }

    element.companding = Some(r.companding(&["L", "R", "C"])?);
    let has_centre = if r.b.read(1)? != 0 {
        let [tl, tr, tc] = three_channel_data(r)?;
        element.set_channel("L", tl);
        element.set_channel("R", tr);
        element.set_channel("C", tc);
        true
    } else {
        let [tl, tr] = two_channel_data(r)?;
        element.set_channel("L", tl);
        element.set_channel("R", tr);
        false
    };
    if mode == ASPX_ACPL_1 {
        let last = r.last_layout.clone().unwrap();
        residuals(r, &mut element, [("L", "Ls"), ("R", "Rs")], &last)?;
    }
    if !has_centre {
        let t = mono_data(r, false)?;
        element.set_channel("C", t);
    }
    r.aspx_2ch(&mut element, "L", "R")?;
    r.aspx_1ch(&mut element, "C")?;
    let residual = mode == ASPX_ACPL_1;
    r.coupling(
        &mut element,
        "left",
        0,
        "L",
        if residual { Some("Ls") } else { None },
        ["L", "Ls"],
        [1.0, SQRT2],
    )?;
    r.coupling(
        &mut element,
        "right",
        1,
        "R",
        if residual { Some("Rs") } else { None },
        ["R", "Rs"],
        [1.0, SQRT2],
    )?;
    Ok(element)
}

fn seven_x_info(channel_mode: u32) -> Option<(&'static str, [&'static str; 2])> {
    match channel_mode {
        5 | 6 => Some(("back", ["Lb", "Rb"])),
        7 | 8 => Some(("wide", ["Lw", "Rw"])),
        9 | 10 => Some(("top", ["Tfl", "Tfr"])),
        _ => None,
    }
}

fn seven_x_acpl(kind: &str, add_ch_base: u32) -> [&'static str; 6] {
    match (kind, add_ch_base) {
        ("back", _) => ["Ls", "Rs", "Lb", "Rb", "L", "R"],
        ("wide", 0) => ["L", "R", "Lw", "Rw", "Ls", "Rs"],
        ("wide", _) => ["Ls", "Rs", "Lw", "Rw", "L", "R"],
        ("top", 0) => ["L", "R", "Tfl", "Tfr", "Ls", "Rs"],
        ("top", _) => ["Ls", "Rs", "Tfl", "Tfr", "L", "R"],
        _ => unreachable!(),
    }
}

pub fn seven_x_element(
    r: &mut Reader<'_, '_>,
    channel_mode: u32,
    add_ch_base: u32,
) -> Result<Element> {
    let mode = r.b.read(2)?;
    r.configs(mode)?;
    let mut element = Element::new(mode);
    let (kind, extra) = seven_x_info(channel_mode)
        .ok_or_else(|| crate::Error::new(format!("unsupported 7.X channel_mode {channel_mode}")))?;
    if matches!(channel_mode, 6 | 8 | 10) {
        let t = mono_data(r, true)?;
        element.set_channel("LFE", t);
    }
    let acpl_mode = mode == ASPX_ACPL_1 || mode == ASPX_ACPL_2;
    let [x0, x1, x3, x4, x6, x7] = seven_x_acpl(kind, add_ch_base);
    if acpl_mode {
        element.companding = Some(r.companding(&[x0, x1, "C", x6, x7])?);
    }
    let config = core_five(r, &mut element, ["L", "R", "C", "Ls", "Rs"])?;
    if mode == SIMPLE || mode == ASPX {
        let (reference, params) = if r.b.flag()? {
            let rf = r.last_layout.clone().unwrap();
            let p0 = smp::read_chparam(r.b, &rf)?;
            let p1 = smp::read_chparam(r.b, &rf)?;
            (Some(rf), Some([p0, p1]))
        } else {
            (None, None)
        };
        let [f, g] = two_channel_data(r)?;
        let frame_len = f.spectrum.len();
        let (m0, m1) = match (&reference, &params) {
            (Some(rf), Some([p0, p1])) => (smp::matrix(p0, rf), smp::matrix(p1, rf)),
            _ => (Matrix::identity(frame_len), Matrix::identity(frame_len)),
        };
        let (left, right) = if kind == "back" { ("Ls", "Rs") } else { ("L", "R") };
        let lt = element.channel(left).unwrap().clone();
        let rt = element.channel(right).unwrap().clone();
        let (a0, a1) = m0.apply(&lt.spectrum, &f.spectrum);
        let (b0, b1) = m1.apply(&rt.spectrum, &g.spectrum);
        element.set_channel(left, Track { spectrum: a0, layout: lt.layout });
        element.set_channel(extra[0], Track { spectrum: a1, layout: f.layout });
        element.set_channel(right, Track { spectrum: b0, layout: rt.layout });
        element.set_channel(extra[1], Track { spectrum: b1, layout: g.layout });
    } else if mode == ASPX_ACPL_1 {
        let last = r.last_layout.clone().unwrap();
        residuals(r, &mut element, [(x0, x3), (x1, x4)], &last)?;
    }
    if config == 0 || config == 2 {
        let t = mono_data(r, false)?;
        element.set_channel("C", t);
    }
    if mode != SIMPLE {
        r.aspx_2ch(&mut element, "L", "R")?;
        r.aspx_2ch(&mut element, "Ls", "Rs")?;
        r.aspx_1ch(&mut element, "C")?;
    }
    if mode == ASPX {
        r.aspx_2ch(&mut element, extra[0], extra[1])?;
    }
    if acpl_mode {
        let residual = mode == ASPX_ACPL_1;
        let front = if add_ch_base == 1 || kind == "back" { SQRT2 } else { 1.0 };
        r.coupling(
            &mut element,
            "left",
            0,
            x0,
            if residual { Some(x3) } else { None },
            [x0, x3],
            [front, SQRT2],
        )?;
        r.coupling(
            &mut element,
            "right",
            1,
            x1,
            if residual { Some(x4) } else { None },
            [x1, x4],
            [front, SQRT2],
        )?;
        if add_ch_base == 0 {
            element.gains.push((x6.to_string(), SQRT2));
            element.gains.push((x7.to_string(), SQRT2));
        }
    }
    Ok(element)
}

pub fn audio_data(r: &mut Reader<'_, '_>, channel_mode: u32, add_ch_base: u32) -> Result<Element> {
    match channel_mode {
        0 => single_channel_element(r),
        1 => channel_pair_element(r),
        2 => three_zero_element(r),
        3 | 4 => five_x_element(r, channel_mode == 4),
        5..=10 => seven_x_element(r, channel_mode, add_ch_base),
        _ => fail(format!("channel_mode {channel_mode} is not supported yet")),
    }
}
