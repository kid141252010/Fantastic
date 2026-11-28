//! AC-4 对象音频元数据（TS 103 190-2 clauses 6.2.8、6.3.9 与 TS 103 420 clause 5）。

use std::f64::consts::PI;

use crate::bits::BitReader;
use crate::toc::{BED, DYN, ISF};
use crate::{fail, Result};

pub const ISF_RINGS: [[usize; 4]; 6] = [
    [3, 1, 0, 0],
    [5, 3, 0, 0],
    [7, 3, 0, 0],
    [9, 5, 0, 0],
    [7, 5, 3, 0],
    [15, 9, 5, 1],
];
const RING_HEIGHTS: [f64; 4] = [0.0, 1.0, -1.0, 1.0];
pub const RAMP_DURATIONS: [usize; 16] = [
    32, 64, 128, 256, 320, 480, 1000, 1001, 1024, 1600, 1601, 1602, 1920, 2000, 2002, 2048,
];
pub const DISTANCE_FACTORS: [f64; 16] = [
    1.1, 1.3, 1.6, 2.0, 2.5, 3.2, 4.0, 5.0, 6.3, 7.9, 10.0, 12.6, 15.8, 20.0, 25.1, 50.1,
];

const FINE_STEPS: [i32; 4] = [1, 2, -1, -2];
const CENTRE_TRIMS: [f64; 16] = [
    6.0, 3.0, 1.5, 0.75, -0.75, -1.5, -3.0, -4.5, -6.0, -7.5, -9.0, -10.5, -12.0, -13.5, -15.0,
    -36.0,
];
const TRIMS: [Option<f64>; 16] = [
    None,
    None,
    None,
    None,
    Some(-0.75),
    Some(-1.5),
    Some(-3.0),
    Some(-4.5),
    Some(-6.0),
    Some(-7.5),
    Some(-9.0),
    Some(-10.5),
    Some(-12.0),
    Some(-13.5),
    Some(-15.0),
    Some(-36.0),
];
const DEFAULT_TRIM_MODE: u32 = 0;
const DISABLE_TRIM_MODE: u32 = 1;
const CUSTOM_TRIM_MODE: u32 = 2;
pub const NUM_TRIM_CONFIGS: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Levels {
    pub centre: f64,
    pub surround: f64,
    pub height: f64,
}

impl Default for Levels {
    fn default() -> Self {
        Self { centre: 0.0, surround: -3.0, height: -3.0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Trim {
    pub warp: bool,
    pub configs: [Option<Levels>; NUM_TRIM_CONFIGS],
    pub exempt: Vec<usize>,
}

impl Default for Trim {
    fn default() -> Self {
        Self {
            warp: false,
            configs: [Some(Levels::default()); NUM_TRIM_CONFIGS],
            exempt: Vec::new(),
        }
    }
}

impl Trim {
    /// 目标布局（当前只使用 "2.0"）的 trim 配置；若对象在豁免列表则返回 None。
    pub fn levels_2_0(&self, obj: usize) -> Option<Levels> {
        if self.exempt.contains(&obj) {
            None
        } else {
            self.configs[0]
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Render {
    pub x_bits: f64,
    pub y_bits: f64,
    pub z_bits: f64,
    pub distance: Option<f64>,
    pub zone: u8,
    pub elevation: bool,
    pub size: [f64; 3],
    pub screen: bool,
    pub snap: bool,
    pub fine: [i32; 3],
    pub warp: bool,
}

impl Default for Render {
    fn default() -> Self {
        Self {
            x_bits: 31.0,
            y_bits: 31.0,
            z_bits: 0.0,
            distance: None,
            zone: 0,
            elevation: true,
            size: [0.0; 3],
            screen: false,
            snap: false,
            fine: [0; 3],
            warp: false,
        }
    }
}

impl Render {
    pub fn position(&self) -> [f64; 3] {
        let [fx, fy, fz] = self.fine;
        let x = (self.x_bits / 62.0 + f64::from(fx) / 310.0).clamp(0.0, 1.0);
        let mut y = (self.y_bits / 62.0 + f64::from(fy) / 310.0).clamp(0.0, 1.0);
        let z = (self.z_bits / 15.0 + f64::from(fz) / 75.0).clamp(-1.0, 1.0);
        if self.warp {
            y = (2.0 * y).min(1.0);
        }
        if self.distance.is_some() {
            return to_walls(x, y, z);
        }
        [x, y, z]
    }

    pub fn warped(&self, trim: &Trim) -> Self {
        if trim.warp {
            let mut copy = *self;
            copy.warp = true;
            copy
        } else {
            *self
        }
    }
}

fn to_walls(x: f64, y: f64, z: f64) -> [f64; 3] {
    let dx = x - 0.5;
    let dy = y - 0.5;
    let reach = (dx.abs() / 0.5).max(dy.abs() / 0.5).max(z.abs());
    if reach == 0.0 {
        [x, y, z]
    } else {
        [0.5 + dx / reach, 0.5 + dy / reach, z / reach]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Update {
    pub start: usize,
    pub ramp: usize,
    pub active: bool,
    pub gain_db: f64,
    pub render: Render,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub bed_labels: Vec<String>,
    pub isf_objects: usize,
    pub dynamic_objects: usize,
    pub isf_rings: Vec<usize>,
}

impl Program {
    pub fn anchored(&self) -> usize {
        self.bed_labels.len() + self.isf_objects
    }

    pub fn isf_position(&self, index: usize) -> Option<[f64; 3]> {
        let mut k = index.checked_sub(self.bed_labels.len())?;
        if k >= self.isf_objects {
            return None;
        }
        for (&count, &z) in self.isf_rings.iter().zip(RING_HEIGHTS.iter()) {
            if k < count {
                return Some(ring_position(k, count, z));
            }
            k -= count;
        }
        None
    }
}

fn ring_position(k: usize, count: usize, z: f64) -> [f64; 3] {
    if count == 1 {
        return [0.5, 0.5, z];
    }
    let azimuth = 2.0 * PI * k as f64 / count as f64;
    let left = azimuth.sin();
    let front = azimuth.cos();
    let reach = left.abs().max(front.abs());
    [0.5 - 0.5 * left / reach, 0.5 - 0.5 * front / reach, z]
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub object_count: usize,
    pub program: Program,
    pub updates: Vec<Vec<Update>>,
    pub trim: Trim,
}

#[derive(Clone, Debug)]
pub struct Timing {
    pub blocks: Vec<(usize, usize)>,
}

#[derive(Clone, Debug)]
pub struct ObjectState {
    pub gain_db: f64,
    pub render: Render,
}

impl Default for ObjectState {
    fn default() -> Self {
        Self { gain_db: f64::NEG_INFINITY, render: Render::default() }
    }
}

#[derive(Clone, Debug)]
pub struct Objects {
    pub types: Vec<(u8, bool)>,
    pub program: Program,
    pub states: Vec<ObjectState>,
    pub coded: Vec<[f64; 3]>,
    pub exempt: Vec<usize>,
    pub trim: Trim,
}

impl Objects {
    pub fn new(types: Vec<(u8, bool)>, program: Program, trim: Trim) -> Self {
        let count = types.len();
        Self {
            types,
            program,
            states: vec![ObjectState::default(); count],
            coded: vec![[31.0, 31.0, 0.0]; count],
            exempt: Vec::new(),
            trim,
        }
    }
}

pub fn program_for(
    types: &[(u8, bool)],
    beds: &[String],
    isf: Option<usize>,
    lfe: bool,
) -> (Vec<(u8, bool)>, Program) {
    let mut labels = Vec::with_capacity(beds.len() + usize::from(lfe));
    let mut full = Vec::with_capacity(types.len() + usize::from(lfe));
    if lfe {
        labels.push("LFE".to_string());
        full.push((BED, true));
    }
    labels.extend(beds.iter().cloned());
    full.extend_from_slice(types);
    let isf_count = types.iter().filter(|&&(t, _)| t == ISF).count();
    let rings = isf.map(|idx| ISF_RINGS[idx].to_vec()).unwrap_or_default();
    let dynamic = types.iter().filter(|&&(t, _)| t == DYN).count();
    (full, Program { bed_labels: labels, isf_objects: isf_count, dynamic_objects: dynamic, isf_rings: rings })
}

pub fn read_timing(b: &mut BitReader<'_>) -> Result<Timing> {
    let offset = if !b.flag()? {
        0
    } else if !b.flag()? {
        if !b.flag()? {
            16
        } else if !b.flag()? {
            8
        } else {
            24
        }
    } else {
        b.read(5)? as usize
    };
    let n = b.read(3)? as usize;
    let mut blocks = Vec::with_capacity(n);
    for _ in 0..n {
        let start = offset + 32 * b.read(6)? as usize;
        let code = b.read(2)?;
        let ramp = if code < 3 {
            [0usize, 512, 1536][code as usize]
        } else if b.flag()? {
            RAMP_DURATIONS[b.read(4)? as usize]
        } else {
            b.read(11)? as usize
        };
        blocks.push((start, ramp));
    }
    Ok(Timing { blocks })
}

pub fn read_common(b: &mut BitReader<'_>) -> Result<Option<Trim>> {
    if !b.flag()? {
        b.skip(5)?;
    }
    b.skip(1)?;
    if !b.flag()? {
        return Ok(None);
    }
    let mut size = b.read(1)? + 1;
    if size == 2 {
        size += b.variable_bits(2)?;
    }
    let end = b.pos + 8 * size as usize;
    let trim = read_trim(b)?;
    if b.pos > end {
        return fail("oamd_common_data overran its additional data");
    }
    if end > b.end {
        return fail("oamd_common_data overran buffer");
    }
    b.pos = end;
    Ok(trim)
}

fn read_trim(b: &mut BitReader<'_>) -> Result<Option<Trim>> {
    if !b.flag()? {
        return Ok(None);
    }
    let warp = b.read(2)? == 0b01;
    b.skip(2)?;
    let mode = b.read(2)?;
    let configs = match mode {
        CUSTOM_TRIM_MODE => {
            let mut arr = [None; NUM_TRIM_CONFIGS];
            for slot in &mut arr {
                *slot = read_trim_config(b)?;
            }
            arr
        }
        DEFAULT_TRIM_MODE => [Some(Levels::default()); NUM_TRIM_CONFIGS],
        DISABLE_TRIM_MODE => [None; NUM_TRIM_CONFIGS],
        _ => return fail("reserved global_trim_mode"),
    };
    Ok(Some(Trim { warp, configs, exempt: Vec::new() }))
}

fn read_trim_config(b: &mut BitReader<'_>) -> Result<Option<Levels>> {
    if b.flag()? {
        return Ok(Some(Levels::default()));
    }
    if b.flag()? {
        return Ok(None);
    }
    let present = b.read(5)?;
    let mut levels = Levels::default();
    if (present & 0b10000) != 0 {
        levels.centre = CENTRE_TRIMS[b.read(4)? as usize];
    }
    if (present & 0b01000) != 0 {
        if let Some(db) = TRIMS[b.read(4)? as usize] {
            levels.surround = db;
        }
    }
    if (present & 0b00100) != 0 {
        if let Some(db) = TRIMS[b.read(4)? as usize] {
            levels.height = db;
        }
    }
    for flag in [0b00010u32, 0b00001u32] {
        if (present & flag) != 0 {
            b.skip(5)?;
        }
    }
    Ok(Some(levels))
}

pub fn read_dyndata(
    b: &mut BitReader<'_>,
    objects: &mut Objects,
    timing: &Timing,
    iframe: bool,
    alternative: bool,
) -> Result<Frame> {
    let count = objects.types.len();
    let mut frame = Frame {
        object_count: count,
        program: objects.program.clone(),
        updates: vec![Vec::with_capacity(timing.blocks.len()); count],
        trim: Trim::default(),
    };
    for obj in 0..count {
        let (kind, lfe) = objects.types[obj];
        let dynamic = kind == DYN && !lfe;
        let fixed = objects.program.isf_position(obj);
        for (blk, &(start, ramp)) in timing.blocks.iter().enumerate() {
            let upd = info_block(
                b,
                objects,
                &frame,
                obj,
                blk,
                iframe && blk == 0,
                dynamic,
                fixed,
                start,
                ramp,
            )?;
            frame.updates[obj].push(upd);
        }
    }
    if alternative {
        skip_alternative(b, objects)?;
    }
    let mut trim = objects.trim.clone();
    trim.exempt = objects.exempt.clone();
    frame.trim = trim;
    Ok(frame)
}

fn info_block(
    b: &mut BitReader<'_>,
    objects: &mut Objects,
    frame: &Frame,
    obj: usize,
    blk: usize,
    no_delta: bool,
    dynamic: bool,
    fixed: Option<[f64; 3]>,
    start: usize,
    ramp: usize,
) -> Result<Update> {
    let inactive = b.flag()?;
    if inactive {
        objects.states[obj].gain_db = f64::NEG_INFINITY;
    } else if no_delta || !b.flag()? {
        objects.states[obj].gain_db = basic_info(b, frame, obj, blk)?;
    }
    if inactive || !dynamic {
        objects.states[obj].render = Render::default();
        objects.coded[obj] = [31.0, 31.0, 0.0];
    } else if no_delta {
        let prev = objects.states[obj].render;
        objects.states[obj].render = render_info(b, objects, obj, prev, true, true)?;
    } else if !b.flag()? {
        let partial = b.flag()?;
        let prev = objects.states[obj].render;
        objects.states[obj].render = render_info(b, objects, obj, prev, !partial, false)?;
    }
    if let Some([x, y, z]) = fixed {
        objects.states[obj].render = Render {
            x_bits: 62.0 * x,
            y_bits: 62.0 * y,
            z_bits: 15.0 * z,
            ..Render::default()
        };
    }
    let gain_db = objects.states[obj].gain_db;
    let mut render = objects.states[obj].render;
    if b.flag()? {
        let size = b.read(4)? as usize + 1;
        let end = b.pos + 8 * size;
        if b.flag()? {
            if !objects.exempt.contains(&obj) {
                objects.exempt.push(obj);
            }
        } else {
            objects.exempt.retain(|&x| x != obj);
        }
        if !inactive && dynamic && b.flag()? {
            let present = b.read(3)?;
            let mut fine = [0i32; 3];
            for (idx, flag) in [0b100u32, 0b010, 0b001].into_iter().enumerate() {
                if (present & flag) != 0 {
                    fine[idx] = FINE_STEPS[b.read(2)? as usize];
                }
            }
            render.fine = fine;
        }
        if b.flag()? {
            b.skip(3)?;
        }
        if b.pos > end {
            return fail("add_table_data overran its size");
        }
        if end > b.end {
            return fail("add_table_data overran buffer");
        }
        b.pos = end;
    }
    Ok(Update { start, ramp, active: !inactive, gain_db, render })
}

fn basic_info(b: &mut BitReader<'_>, frame: &Frame, obj: usize, blk: usize) -> Result<f64> {
    if b.flag()? {
        return Ok(0.0);
    }
    let (gain, priority) = if !b.flag()? {
        (true, false)
    } else {
        (!b.flag()?, true)
    };
    let mut db = 0.0;
    if gain {
        if !b.flag()? {
            let g = b.read(6)? as i32;
            db = f64::from(if g < 15 { 15 - g } else { 14 - g });
        } else if !b.flag()? {
            db = f64::NEG_INFINITY;
        } else {
            db = if obj > 0 { frame.updates[obj - 1][blk].gain_db } else { 0.0 };
        }
    }
    if priority {
        b.skip(5)?;
    }
    Ok(db)
}

fn render_info(
    b: &mut BitReader<'_>,
    objects: &mut Objects,
    obj: usize,
    prev: Render,
    all_new: bool,
    no_delta: bool,
) -> Result<Render> {
    let (other, zone, position) = if all_new {
        (true, true, true)
    } else {
        (b.flag()?, b.flag()?, b.flag()?)
    };
    let [mut x, mut y, mut z] = objects.coded[obj];
    let mut distance = prev.distance;
    let mut zone_mask = prev.zone;
    let mut elevation = prev.elevation;
    let mut snap = prev.snap;
    let mut size = prev.size;
    let mut screen = prev.screen;

    if position {
        if !no_delta && b.flag()? {
            x = (x + f64::from(b.signed(3)?)).clamp(0.0, 62.0);
            y = (y + f64::from(b.signed(3)?)).clamp(0.0, 62.0);
            z = (z + f64::from(b.signed(3)?)).clamp(-15.0, 15.0);
        } else {
            x = f64::from(b.read(6)?);
            y = f64::from(b.read(6)?);
            let sign = if b.read(1)? != 0 { 1.0 } else { -1.0 };
            z = sign * f64::from(b.read(4)?);
        }
        objects.coded[obj] = [x, y, z];
    }
    if zone {
        zone_mask = 0;
        elevation = true;
        snap = false;
        if !b.flag()? {
            let flags = b.read(3)?;
            if (flags & 0b100) != 0 {
                zone_mask = b.read(3)? as u8;
            }
            elevation = (flags & 0b010) == 0;
            snap = (flags & 0b001) != 0;
        }
    }
    if other {
        size = [0.0; 3];
        screen = false;
        distance = None;
        if !b.flag()? {
            let mask = b.read(4)?;
            if (mask & 0b0001) != 0 {
                if b.flag()? {
                    size = [
                        f64::from(b.read(5)?) / 31.0,
                        f64::from(b.read(5)?) / 31.0,
                        f64::from(b.read(5)?) / 31.0,
                    ];
                } else {
                    let s = f64::from(b.read(5)?) / 31.0;
                    size = [s, s, s];
                }
            }
            if (mask & 0b0010) != 0 {
                screen = b.read(3)? > 0;
                b.skip(2)?;
            }
            if (mask & 0b0100) != 0 {
                distance = if b.flag()? {
                    Some(f64::INFINITY)
                } else {
                    Some(DISTANCE_FACTORS[b.read(4)? as usize])
                };
            }
            if (mask & 0b1000) != 0 {
                let mode = b.read(2)?;
                if mode == 0 {
                    b.skip(2)?;
                } else if (mode & 0b10) != 0 {
                    b.skip(6)?;
                }
            }
        }
    }
    Ok(Render {
        x_bits: x,
        y_bits: y,
        z_bits: z,
        distance,
        zone: zone_mask,
        elevation,
        size,
        screen,
        snap,
        fine: [0; 3],
        warp: false,
    })
}

fn skip_alternative(b: &mut BitReader<'_>, objects: &Objects) -> Result<()> {
    let types = &objects.types;
    b.skip(1)?;
    if b.read(2)? == 3 {
        b.variable_bits(2)?;
    }
    let mut sets = b.read(2)?;
    if sets == 3 {
        sets += b.variable_bits(2)?;
    }
    for _ in 0..sets {
        let keep = b.flag()?;
        if !keep {
            let mut points = types.len();
            if (!types.is_empty() && types[0].0 == ISF) || b.flag()? {
                points = 1;
            }
            for &(kind, lfe) in &types[..points] {
                if b.flag()? {
                    b.skip(6)?;
                }
                if kind == DYN && !lfe && b.flag()? {
                    b.skip(17)?;
                }
            }
        }
        if b.flag()? {
            let size = b.variable_bits(2)? as usize + 1;
            let end = b.pos + 8 * size;
            if !keep {
                for &(kind, lfe) in types {
                    if kind == DYN && !lfe && b.flag()? {
                        let present = b.read(3)?;
                        b.skip(2 * present.count_ones() as usize)?;
                    }
                }
            }
            if end > b.end {
                return fail("alternative oamd overran buffer");
            }
            b.pos = end;
        }
    }
    Ok(())
}
