//! 房间对象渲染器与声道布局（移植自 SoundWeaver `atmos/render.py` 与 `speakers.py`）。

use std::f64::consts::FRAC_PI_2;

use crate::oamd::{Frame, Levels, Render};

const SPREAD: f64 = std::f64::consts::FRAC_1_SQRT_2;
const WIDE_Y: f64 = 10.0 / 62.0;

#[derive(Clone, Debug)]
pub struct Row {
    pub y: f64,
    pub names: &'static [&'static str],
}

#[derive(Clone, Debug)]
pub struct SpeakerLayout {
    pub name: &'static str,
    pub channels: &'static [&'static str],
    pub floor: &'static [Row],
    pub top: &'static [Row],
}

impl SpeakerLayout {
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.channels.iter().position(|&c| c == name)
    }

    pub fn has(&self, name: &str) -> bool {
        self.index_of(name).is_some()
    }

    pub fn surrounds(&self) -> Vec<usize> {
        let mut out = Vec::new();
        for row in self.floor {
            if row.y > 0.0 {
                for &name in row.names {
                    if let Some(idx) = self.index_of(name) {
                        out.push(idx);
                    }
                }
            }
        }
        out
    }

    pub fn positions(&self) -> Vec<(&'static str, usize, [f64; 3])> {
        let mut out = Vec::new();
        for &(z, rows) in &[(0.0, self.floor), (1.0, self.top)] {
            for row in rows {
                let span = row.names.len().saturating_sub(1);
                for (i, &name) in row.names.iter().enumerate() {
                    if let Some(idx) = self.index_of(name) {
                        let x = if span > 0 { i as f64 / span as f64 } else { 0.5 };
                        out.push((name, idx, [x, row.y, z]));
                    }
                }
            }
        }
        out
    }
}

pub const LAYOUT_2_0: SpeakerLayout = SpeakerLayout {
    name: "2.0",
    channels: &["L", "R"],
    floor: &[Row { y: 0.0, names: &["L", "R"] }],
    top: &[],
};

pub const LAYOUT_5_1: SpeakerLayout = SpeakerLayout {
    name: "5.1",
    channels: &["L", "R", "C", "LFE", "Ls", "Rs"],
    floor: &[
        Row { y: 0.0, names: &["L", "C", "R"] },
        Row { y: 1.0, names: &["Ls", "Rs"] },
    ],
    top: &[],
};

pub const LAYOUT_7_1: SpeakerLayout = SpeakerLayout {
    name: "7.1",
    channels: &["L", "R", "C", "LFE", "Lb", "Rb", "Ls", "Rs"],
    floor: &[
        Row { y: 0.0, names: &["L", "C", "R"] },
        Row { y: 0.5, names: &["Ls", "Rs"] },
        Row { y: 1.0, names: &["Lb", "Rb"] },
    ],
    top: &[],
};

pub const LAYOUT_5_1_2: SpeakerLayout = SpeakerLayout {
    name: "5.1.2",
    channels: &["L", "R", "C", "LFE", "Ls", "Rs", "Tsl", "Tsr"],
    floor: &[
        Row { y: 0.0, names: &["L", "C", "R"] },
        Row { y: 1.0, names: &["Ls", "Rs"] },
    ],
    top: &[Row { y: 0.5, names: &["Tsl", "Tsr"] }],
};

pub const LAYOUT_7_1_2: SpeakerLayout = SpeakerLayout {
    name: "7.1.2",
    channels: &["L", "R", "C", "LFE", "Lb", "Rb", "Ls", "Rs", "Tsl", "Tsr"],
    floor: &[
        Row { y: 0.0, names: &["L", "C", "R"] },
        Row { y: 0.5, names: &["Ls", "Rs"] },
        Row { y: 1.0, names: &["Lb", "Rb"] },
    ],
    top: &[Row { y: 0.5, names: &["Tsl", "Tsr"] }],
};

pub const LAYOUT_9_1_2: SpeakerLayout = SpeakerLayout {
    name: "9.1.2",
    channels: &["L", "R", "C", "LFE", "Lb", "Rb", "Lw", "Rw", "Ls", "Rs", "Tsl", "Tsr"],
    floor: &[
        Row { y: 0.0, names: &["L", "C", "R"] },
        Row { y: WIDE_Y, names: &["Lw", "Rw"] },
        Row { y: 0.5, names: &["Ls", "Rs"] },
        Row { y: 1.0, names: &["Lb", "Rb"] },
    ],
    top: &[Row { y: 0.5, names: &["Tsl", "Tsr"] }],
};

pub fn layout_for_channels(names: &[&str]) -> Option<&'static SpeakerLayout> {
    let mut non_lfe: Vec<&str> = names.iter().copied().filter(|&n| n != "LFE").collect();
    non_lfe.sort_unstable();
    let layouts = [
        &LAYOUT_2_0,
        &LAYOUT_5_1,
        &LAYOUT_5_1_2,
        &LAYOUT_7_1,
        &LAYOUT_7_1_2,
        &LAYOUT_9_1_2,
    ];
    for layout in layouts {
        let mut l_names: Vec<&str> = layout
            .channels
            .iter()
            .copied()
            .filter(|&n| n != "LFE")
            .collect();
        l_names.sort_unstable();
        if l_names == non_lfe {
            return Some(layout);
        }
    }
    None
}

fn bed_position(label: &str, backs: bool) -> [f64; 3] {
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
        "LFE" | "LFE2" => [0.5, 0.0, -1.0],
        "Ls" | "Lb" => [0.0, 1.0, 0.0],
        "Rs" | "Rb" => [1.0, 1.0, 0.0],
        "Tfl" => [0.0, 0.0, 1.0],
        "Tfr" => [1.0, 0.0, 1.0],
        "Tsl" => [0.0, 0.5, 1.0],
        "Tsr" => [1.0, 0.5, 1.0],
        "Tbl" => [0.0, 1.0, 1.0],
        "Tbr" => [1.0, 1.0, 1.0],
        "Lw" => [0.0, WIDE_Y, 0.0],
        "Rw" => [1.0, WIDE_Y, 0.0],
        "Lfc" => [0.25, 0.0, 0.0],
        "Rfc" => [0.75, 0.0, 0.0],
        "Cb" => [0.5, 1.0, 0.0],
        "Tfc" => [0.5, 0.0, 1.0],
        "Tsc" => [0.5, 0.5, 1.0],
        "Tbc" => [0.5, 1.0, 1.0],
        _ => [0.5, 0.0, 0.0],
    }
}

#[inline]
fn pair_pan(t: f64) -> (f64, f64) {
    let a = t.clamp(0.0, 1.0) * FRAC_PI_2;
    (a.cos(), a.sin())
}

fn pan_row(x: f64, names: &[&'static str]) -> [Option<(&'static str, f64)>; 2] {
    if names.len() == 1 {
        return [Some((names[0], 1.0)), None];
    }
    let x = x.clamp(0.0, 1.0);
    let span = names.len() - 1;
    let i = ((x * span as f64) as usize).min(span - 1);
    let (a, b) = pair_pan(x * span as f64 - i as f64);
    [Some((names[i], a)), Some((names[i + 1], b))]
}

fn pan_rows(y: f64, rows: &[Row]) -> [Option<(f64, &'static [&'static str])>; 2] {
    if rows.len() == 1 {
        return [Some((1.0, rows[0].names)), None];
    }
    let y = y.clamp(rows[0].y, rows.last().unwrap().y);
    let i = (0..rows.len() - 1)
        .find(|&idx| y <= rows[idx + 1].y)
        .unwrap_or(rows.len() - 2);
    let near = &rows[i];
    let far = &rows[i + 1];
    let (a, b) = pair_pan((y - near.y) / (far.y - near.y));
    [Some((a, near.names)), Some((b, far.names))]
}

pub fn place(layout: &SpeakerLayout, x: f64, y: f64, z: f64) -> Vec<f64> {
    let mut g = vec![0.0f64; layout.channels.len()];
    let (floor_w, top_w) = if !layout.top.is_empty() { pair_pan(z) } else { (1.0, 0.0) };
    for &(layer, rows) in &[(floor_w, layout.floor), (top_w, layout.top)] {
        if layer == 0.0 || rows.is_empty() {
            continue;
        }
        for (weight, names) in pan_rows(y, rows).into_iter().flatten() {
            for (name, w) in pan_row(x, names).into_iter().flatten() {
                if let Some(idx) = layout.index_of(name) {
                    g[idx] += layer * weight * w;
                }
            }
        }
    }
    g
}

fn apply_trim(layout: &SpeakerLayout, g: &mut [f64], z: f64, trim: Option<Levels>) {
    let Some(t) = trim else {
        return;
    };
    let h_factor = 10.0f64.powf(t.height * z.clamp(0.0, 1.0) / 20.0);
    let s_factor = 10.0f64.powf(t.surround / 20.0);
    let surrounds = layout.surrounds();
    for (idx, val) in g.iter_mut().enumerate() {
        let f = if surrounds.contains(&idx) { s_factor } else { h_factor };
        *val *= f;
    }
    if let Some(c_idx) = layout.index_of("C") {
        g[c_idx] *= 10.0f64.powf(t.centre / 20.0);
    }
}

fn zone_mask(layout: &SpeakerLayout, zone: u8) -> Vec<bool> {
    let backs = layout.has("Lb");
    let has_back = |name: &str| {
        if backs {
            matches!(name, "Lb" | "Rb")
        } else {
            matches!(name, "Ls" | "Rs")
        }
    };
    let has_side = |name: &str| {
        (backs && matches!(name, "Ls" | "Rs")) || matches!(name, "Lw" | "Rw")
    };
    let is_height = |name: &str| {
        layout.top.iter().any(|r| r.names.contains(&name))
    };
    layout
        .channels
        .iter()
        .map(|&name| match zone {
            1 => !has_back(name),
            2 => !has_side(name),
            3 => name == "C" || has_back(name) || is_height(name),
            4 => matches!(name, "L" | "R" | "C") || is_height(name),
            5 => matches!(name, "Ls" | "Rs") || is_height(name),
            _ => true,
        })
        .collect()
}

fn nearest(layout: &SpeakerLayout, mask: &[bool], x: f64, y: f64, z: f64) -> Vec<f64> {
    let mut g = vec![0.0f64; layout.channels.len()];
    let mut best_idx = 0usize;
    let mut best_dist = f64::INFINITY;
    for (_, idx, pos) in layout.positions() {
        if !mask[idx] {
            continue;
        }
        let dx = pos[0] - x;
        let dy = pos[1] - y;
        let dz = pos[2] - z;
        let d = dx * dx + dy * dy + dz * dz;
        if d < best_dist {
            best_dist = d;
            best_idx = idx;
        }
    }
    if best_dist.is_finite() {
        g[best_idx] = 1.0;
    }
    g
}

pub fn object_gains(layout: &SpeakerLayout, render: &Render, trim: Option<Levels>) -> Vec<f64> {
    let [x, y, mut z] = render.position();
    if !render.elevation {
        z = 0.0;
    }
    let mut mask = zone_mask(layout, render.zone);
    if !mask.iter().any(|&m| m) {
        mask = vec![true; layout.channels.len()];
    }
    let mut g = if render.snap {
        nearest(layout, &mask, x, y, z)
    } else {
        let mut p = place(layout, x, y, z);
        let mut power = 0.0;
        for (val, &allowed) in p.iter_mut().zip(mask.iter()) {
            if !allowed {
                *val = 0.0;
            }
            power += *val * *val;
        }
        if power > 0.0 {
            let inv = 1.0 / power.sqrt();
            for val in &mut p {
                *val *= inv;
            }
            p
        } else {
            nearest(layout, &mask, x, y, z)
        }
    };
    apply_trim(layout, &mut g, z, trim);
    g
}

pub fn bed_gains(
    layout: &SpeakerLayout,
    label: &str,
    backs: bool,
    trim: Option<Levels>,
) -> Vec<f64> {
    let mut g = vec![0.0f64; layout.channels.len()];
    if label == "LFE" || label == "LFE2" {
        if let Some(idx) = layout.index_of("LFE") {
            g[idx] = 1.0;
        }
        return g;
    }
    let mut z = 0.0;
    if (label == "Ls" || label == "Rs") && !backs && layout.has("Lb") {
        let back_name = if label == "Ls" { "Lb" } else { "Rb" };
        if let Some(i1) = layout.index_of(label) {
            g[i1] = SPREAD;
        }
        if let Some(i2) = layout.index_of(back_name) {
            g[i2] = SPREAD;
        }
    } else if let Some(idx) = layout.index_of(label) {
        g[idx] = 1.0;
    } else {
        let [px, py, pz] = bed_position(label, backs);
        z = pz;
        g = place(layout, px, py, pz);
    }
    apply_trim(layout, &mut g, z, trim);
    g
}

pub fn joc_objects(frame: &Frame) -> Vec<usize> {
    let labels = &frame.program.bed_labels;
    (0..frame.object_count)
        .filter(|&i| i >= labels.len() || !labels[i].starts_with("LFE"))
        .collect()
}

/// 将对象更新转为逐样本增益曲线：`out[o][sample * width + col]`（最后一列是对象自身线性增益）。
pub struct Renderer {
    pub layout: &'static SpeakerLayout,
    pub objects: usize,
    pub width: usize,
    current: Vec<Vec<f64>>,
    target: Vec<Vec<f64>>,
    remaining: Vec<usize>,
}

impl Renderer {
    pub fn new(objects: usize, layout: &'static SpeakerLayout) -> Self {
        let width = layout.channels.len() + 1;
        Self {
            layout,
            objects,
            width,
            current: vec![vec![0.0; width]; objects],
            target: vec![vec![0.0; width]; objects],
            remaining: vec![0; objects],
        }
    }

    pub fn reset(&mut self) {
        for row in &mut self.current {
            row.fill(0.0);
        }
        for row in &mut self.target {
            row.fill(0.0);
        }
        self.remaining.fill(0);
    }

    /// 计算一帧每个对象的逐样本增益：`gains[o]` 长度为 `samples * self.width`。
    pub fn gains(&mut self, frame: &Frame, samples: usize) -> Vec<Vec<f64>> {
        let labels = &frame.program.bed_labels;
        let backs = labels.iter().any(|l| l == "Lb");
        let joc_objs = joc_objects(frame);
        let mut out = vec![vec![0.0f64; samples * self.width]; self.objects];
        let silent = vec![0.0f64; self.width];

        for (o, &meta_idx) in joc_objs.iter().enumerate() {
            let source = labels.get(meta_idx).map(String::as_str);
            let dynamic = meta_idx >= frame.program.anchored();
            let trim = if !self.layout.top.is_empty() {
                None
            } else if self.layout.name == "2.0" {
                frame.trim.levels_2_0(meta_idx)
            } else if self.layout.name == "5.1" {
                if frame.trim.exempt.contains(&meta_idx) { None } else { frame.trim.configs[1] }
            } else if self.layout.name == "7.1" {
                if frame.trim.exempt.contains(&meta_idx) { None } else { frame.trim.configs[2] }
            } else {
                None
            };

            let mut events: Vec<(usize, usize, Vec<f64>)> = Vec::new();
            for u in &frame.updates[meta_idx] {
                let g = if u.active && u.gain_db != f64::NEG_INFINITY {
                    let lin = 10.0f64.powf(u.gain_db / 20.0);
                    let spk = match source {
                        Some(label) => bed_gains(self.layout, label, backs, trim),
                        None => {
                            let r = if dynamic { u.render.warped(&frame.trim) } else { u.render };
                            object_gains(self.layout, &r, trim)
                        }
                    };
                    let mut row = Vec::with_capacity(self.width);
                    for s in spk {
                        row.push(s * lin);
                    }
                    row.push(lin);
                    row
                } else {
                    silent.clone()
                };
                events.push((u.start, u.ramp, g));
            }
            self.envelope(o, &mut events, samples, &mut out[o]);
        }
        out
    }

    fn envelope(
        &mut self,
        o: usize,
        events: &mut [(usize, usize, Vec<f64>)],
        samples: usize,
        env: &mut [f64],
    ) {
        let w = self.width;
        let current = &mut self.current[o];
        let target = &mut self.target[o];
        let settled = self.remaining[o] == 0 || current == target;
        if settled && events.iter().all(|(_, _, g)| g == current) {
            for s in 0..samples {
                env[s * w..(s + 1) * w].copy_from_slice(current);
            }
            target.copy_from_slice(current);
            self.remaining[o] = 0;
            return;
        }
        events.sort_by_key(|e| e.0);
        let mut t = 0usize;
        for idx in 0..=events.len() {
            let (start, ramp, g_opt) = if idx < events.len() {
                let (s, r, ref g) = events[idx];
                (s.min(samples), r, Some(g.as_slice()))
            } else {
                (samples, 0, None)
            };
            while t < start {
                let rem = self.remaining[o];
                if rem > 0 {
                    let step = rem.min(start - t);
                    let inv_rem = 1.0 / rem as f64;
                    for s in 0..step {
                        let frac = (s + 1) as f64 * inv_rem;
                        let dst = &mut env[(t + s) * w..(t + s + 1) * w];
                        for c in 0..w {
                            dst[c] = current[c] + frac * (target[c] - current[c]);
                        }
                    }
                    let last = &env[(t + step - 1) * w..(t + step) * w];
                    current.copy_from_slice(last);
                    self.remaining[o] = rem - step;
                    t += step;
                } else {
                    for s in t..start {
                        env[s * w..(s + 1) * w].copy_from_slice(current);
                    }
                    t = start;
                }
            }
            if let Some(g) = g_opt {
                if ramp > 0 {
                    target.copy_from_slice(g);
                    self.remaining[o] = ramp;
                } else {
                    current.copy_from_slice(g);
                    target.copy_from_slice(g);
                    self.remaining[o] = 0;
                }
            }
        }
    }
}
