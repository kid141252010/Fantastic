//! 高级耦合 A-CPL（TS 103 190-1 clauses 4.2.13 与 5.7.7）。

use std::collections::HashMap;

use crate::bits::BitReader;
use crate::codes::{huffman, Book};
use crate::qmf::{QmfMatrix, BANDS, C64};
use crate::Result;

pub const FULL: u8 = 0;
pub const PARTIAL: u8 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Alpha,
    Beta,
    Beta3,
    Gamma,
}

const SUBBANDS: usize = BANDS;
const NUM_BANDS: [usize; 4] = [15, 12, 9, 7];
const GROUP_START: [usize; 15] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 14, 18, 23, 35];
const GROUP_BAND: [[usize; 15]; 4] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
    [0, 1, 2, 3, 4, 4, 5, 5, 6, 6, 7, 8, 9, 10, 11],
    [0, 1, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8],
    [0, 1, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 6, 6],
];

const ALPHA_FINE: [f64; 33] = [
    -2.0, -1.809375, -1.6375, -1.484375, -1.35, -1.234375, -1.1375, -1.059375, -1.0,
    -0.940625, -0.8625, -0.765625, -0.65, -0.515625, -0.3625, -0.190625, 0.0,
    0.190625, 0.3625, 0.515625, 0.65, 0.765625, 0.8625, 0.940625, 1.0,
    1.059375, 1.1375, 1.234375, 1.35, 1.484375, 1.6375, 1.809375, 2.0,
];
const IBETA_FINE: [usize; 33] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 7, 6, 5, 4, 3, 2, 1, 0,
    1, 2, 3, 4, 5, 6, 7, 8, 7, 6, 5, 4, 3, 2, 1, 0,
];
const BETA_FINE: [[f64; 9]; 9] = [
    [0.0; 9],
    [0.2375, 0.2035449, 0.1729297, 0.1456543, 0.1217188, 0.101123, 0.0838672, 0.0699512, 0.059375],
    [0.55, 0.4713672, 0.4004688, 0.3373047, 0.281875, 0.2341797, 0.1942188, 0.1619922, 0.1375],
    [0.9375, 0.8034668, 0.6826172, 0.5749512, 0.4804688, 0.3991699, 0.3310547, 0.276123, 0.234375],
    [1.4, 1.199844, 1.019375, 0.8585938, 0.7175, 0.5960938, 0.494375, 0.4123438, 0.35],
    [1.9375, 1.660498, 1.410742, 1.1882319, 0.9929688, 0.8249512, 0.6841797, 0.5706543, 0.484375],
    [2.55, 2.18543, 1.856719, 1.563867, 1.306875, 1.085742, 0.9004688, 0.7510547, 0.6375],
    [3.2375, 2.7746389, 2.357305, 1.985498, 1.659219, 1.378467, 1.143242, 0.9535449, 0.809375],
    [4.0, 3.4281249, 2.9124999, 2.453125, 2.05, 1.703125, 1.4125, 1.178125, 1.0],
];
const ALPHA_COARSE: [f64; 17] = [
    -2.0, -1.6375, -1.35, -1.1375, -1.0, -0.8625, -0.65, -0.3625, 0.0,
    0.3625, 0.65, 0.8625, 1.0, 1.1375, 1.35, 1.6375, 2.0,
];
const IBETA_COARSE: [usize; 17] = [0, 1, 2, 3, 4, 3, 2, 1, 0, 1, 2, 3, 4, 3, 2, 1, 0];
const BETA_COARSE: [[f64; 5]; 5] = [
    [0.0; 5],
    [0.55, 0.4004688, 0.281875, 0.1942188, 0.1375],
    [1.4, 1.019375, 0.7175, 0.494375, 0.35],
    [2.55, 1.856719, 1.306875, 0.9004688, 0.6375],
    [4.0, 2.9124999, 2.05, 1.4125, 1.0],
];

const REGIONS: [(usize, usize, usize); 3] = [(0, 7, 7), (7, 23, 10), (23, 64, 12)];
const DECORRELATORS: [[&[f64]; 3]; 3] = [
    [
        &[1.0, 0.5306, -0.4533, -0.6248, 0.0424, 0.4237, 0.4311, 0.1688],
        &[1.0, 0.5561, -0.3039, -0.5024, -0.1850],
        &[1.0, 0.5773, 0.3321],
    ],
    [
        &[1.0, -0.4178, 0.1082, -0.2368, -0.1014, -0.1052, -0.3528, 0.4665],
        &[1.0, 0.0425, 0.3235, -0.1556, 0.4958],
        &[1.0, 0.2327, -0.3901],
    ],
    [
        &[1.0, 0.4007, 0.4747, 0.2611, -0.1211, -0.4248, -0.2989, -0.1932],
        &[1.0, -0.4361, 0.0345, 0.5215, -0.4178],
        &[1.0, -0.6057, 0.3804],
    ],
];

pub fn band_of_subband(bands_id: usize) -> [usize; SUBBANDS] {
    let mut out = [0usize; SUBBANDS];
    for sb in 0..SUBBANDS {
        let g = GROUP_START.partition_point(|&s| s <= sb).saturating_sub(1);
        out[sb] = GROUP_BAND[bands_id][g];
    }
    out
}

#[derive(Clone, Copy, Debug)]
pub struct AcplConfig {
    pub bands_id: usize,
    pub quant: [usize; 2],
    pub qmf_band: usize,
    pub param_band: usize,
}

impl AcplConfig {
    pub fn bands(&self) -> usize {
        NUM_BANDS[self.bands_id]
    }
}

pub fn read_config_1ch(b: &mut BitReader<'_>, mode: u8) -> Result<AcplConfig> {
    let bands_id = b.read(2)? as usize;
    let quant = b.read(1)? as usize;
    if mode == PARTIAL {
        let qmf_band = b.read(3)? as usize + 1;
        let param_band = band_of_subband(bands_id)[qmf_band];
        Ok(AcplConfig { bands_id, quant: [quant, quant], qmf_band, param_band })
    } else {
        Ok(AcplConfig { bands_id, quant: [quant, quant], qmf_band: 0, param_band: 0 })
    }
}

pub fn read_config_2ch(b: &mut BitReader<'_>) -> Result<AcplConfig> {
    let bands_id = b.read(2)? as usize;
    Ok(AcplConfig {
        bands_id,
        quant: [b.read(1)? as usize, b.read(1)? as usize],
        qmf_band: 0,
        param_band: 0,
    })
}

#[derive(Clone, Debug)]
pub struct AcplData {
    pub interpolation: u32,
    pub sets: usize,
    pub timeslots: Vec<usize>,
    pub params: HashMap<&'static str, (Vec<u32>, Vec<Vec<i32>>)>,
}

fn acpl_book(kind: ParamKind, quant: usize, diff_kind: u8) -> (Book, i32) {
    // diff_kind: 0 = F0, 1 = DF, 2 = DT
    match (kind, quant != 0, diff_kind) {
        (ParamKind::Alpha, false, 0) => (Book::AcplAlphaFineF0, 0),
        (ParamKind::Alpha, false, 1) => (Book::AcplAlphaFineDf, 32),
        (ParamKind::Alpha, false, 2) => (Book::AcplAlphaFineDt, 32),
        (ParamKind::Alpha, true, 0) => (Book::AcplAlphaCoarseF0, 0),
        (ParamKind::Alpha, true, 1) => (Book::AcplAlphaCoarseDf, 16),
        (ParamKind::Alpha, true, 2) => (Book::AcplAlphaCoarseDt, 16),
        (ParamKind::Beta, false, 0) => (Book::AcplBetaFineF0, 0),
        (ParamKind::Beta, false, 1) => (Book::AcplBetaFineDf, 8),
        (ParamKind::Beta, false, 2) => (Book::AcplBetaFineDt, 8),
        (ParamKind::Beta, true, 0) => (Book::AcplBetaCoarseF0, 0),
        (ParamKind::Beta, true, 1) => (Book::AcplBetaCoarseDf, 4),
        (ParamKind::Beta, true, 2) => (Book::AcplBetaCoarseDt, 4),
        (ParamKind::Beta3, false, 0) => (Book::AcplBeta3FineF0, 0),
        (ParamKind::Beta3, false, 1) => (Book::AcplBeta3FineDf, 16),
        (ParamKind::Beta3, false, 2) => (Book::AcplBeta3FineDt, 16),
        (ParamKind::Beta3, true, 0) => (Book::AcplBeta3CoarseF0, 0),
        (ParamKind::Beta3, true, 1) => (Book::AcplBeta3CoarseDf, 8),
        (ParamKind::Beta3, true, 2) => (Book::AcplBeta3CoarseDt, 8),
        (ParamKind::Gamma, false, 0) => (Book::AcplGammaFineF0, 20),
        (ParamKind::Gamma, false, 1) => (Book::AcplGammaFineDf, 40),
        (ParamKind::Gamma, false, 2) => (Book::AcplGammaFineDt, 40),
        (ParamKind::Gamma, true, 0) => (Book::AcplGammaCoarseF0, 10),
        (ParamKind::Gamma, true, 1) => (Book::AcplGammaCoarseDf, 20),
        (ParamKind::Gamma, true, 2) => (Book::AcplGammaCoarseDt, 20),
        _ => unreachable!(),
    }
}

fn huff_data(
    b: &mut BitReader<'_>,
    kind: ParamKind,
    quant: usize,
    bands: usize,
    start: usize,
) -> Result<(u32, Vec<i32>)> {
    let mut values = vec![0i32; bands];
    let diff = b.read(1)?;
    if diff == 0 {
        let (bf0, off0) = acpl_book(kind, quant, 0);
        values[start] = huffman(bf0).decode(b)? - off0;
        let (bdf, off_df) = acpl_book(kind, quant, 1);
        let hcb = huffman(bdf);
        for v in values.iter_mut().skip(start + 1) {
            *v = hcb.decode(b)? - off_df;
        }
    } else {
        let (bdt, off_dt) = acpl_book(kind, quant, 2);
        let hcb = huffman(bdt);
        for v in values.iter_mut().skip(start) {
            *v = hcb.decode(b)? - off_dt;
        }
    }
    Ok((diff, values))
}

pub fn read_data_1ch(b: &mut BitReader<'_>, config: AcplConfig) -> Result<AcplData> {
    let interpolation = b.read(1)?;
    let sets = b.read(1)? as usize + 1;
    let mut timeslots = Vec::new();
    if interpolation != 0 {
        for _ in 0..sets {
            timeslots.push(b.read(5)? as usize);
        }
    }
    let mut params = HashMap::new();
    for (name, kind, which) in [("alpha1", ParamKind::Alpha, 0usize), ("beta1", ParamKind::Beta, 0)] {
        let mut diffs = Vec::with_capacity(sets);
        let mut values = Vec::with_capacity(sets);
        for _ in 0..sets {
            let (d, v) = huff_data(b, kind, config.quant[which], config.bands(), config.param_band)?;
            diffs.push(d);
            values.push(v);
        }
        params.insert(name, (diffs, values));
    }
    Ok(AcplData { interpolation, sets, timeslots, params })
}

pub struct Module {
    pub decorrelator: usize,
    zi: [Vec<Vec<C64>>; 3],
    peak: [f64; 15],
    smooth: [f64; 15],
    diff: [f64; 15],
    q_prev: HashMap<&'static str, Vec<f64>>,
    dq_prev_a: [f64; SUBBANDS],
    dq_prev_b: [f64; SUBBANDS],
}

impl Module {
    pub fn new(decorrelator: usize) -> Self {
        let mut zi = [Vec::new(), Vec::new(), Vec::new()];
        for (r, &(first, end, delay)) in REGIONS.iter().enumerate() {
            let a = DECORRELATORS[decorrelator][r];
            let state_len = delay + a.len() - 1;
            zi[r] = vec![vec![C64::default(); state_len]; end - first];
        }
        Self {
            decorrelator,
            zi,
            peak: [0.0; 15],
            smooth: [0.0; 15],
            diff: [0.0; 15],
            q_prev: HashMap::new(),
            dq_prev_a: [0.0; SUBBANDS],
            dq_prev_b: [0.0; SUBBANDS],
        }
    }

    pub fn quantized(&mut self, data: &AcplData, name: &'static str, bands: usize) -> Vec<Vec<f64>> {
        let (diffs, values) = &data.params[name];
        let mut prev = self.q_prev.get(name).cloned().unwrap_or_else(|| vec![0.0; bands]);
        let mut out = Vec::with_capacity(diffs.len());
        for (&diff, v) in diffs.iter().zip(values.iter()) {
            let q: Vec<f64> = if diff == 0 {
                let mut acc = 0.0;
                v.iter()
                    .map(|&x| {
                        acc += f64::from(x);
                        acc
                    })
                    .collect()
            } else {
                prev.iter().zip(v.iter()).map(|(&p, &x)| p + f64::from(x)).collect()
            };
            prev = q.clone();
            out.push(q);
        }
        self.q_prev.insert(name, prev);
        out
    }

    /// Pseudocode 111：直接 II 型转置 IIR 全通去相关器（与 scipy.signal.lfilter 等价）。
    pub fn decorrelate(&mut self, x: &QmfMatrix) -> QmfMatrix {
        let slots = x.slots;
        let mut y = QmfMatrix::zeros(slots);
        for (r, &(first, end, delay)) in REGIONS.iter().enumerate() {
            let a = DECORRELATORS[self.decorrelator][r];
            let m = delay + a.len(); // len(b) == len(a_rev) + delay
            let n = m - 1;
            let mut b = vec![0.0f64; m];
            for (i, &av) in a.iter().rev().enumerate() {
                b[delay + i] = av;
            }
            let mut a_pad = vec![0.0f64; m];
            a_pad[..a.len()].copy_from_slice(a);
            for (idx, sb) in (first..end).enumerate() {
                let z = &mut self.zi[r][idx];
                let x_row = x.row(sb);
                let y_row = y.row_mut(sb);
                for ts in 0..slots {
                    let xn = x_row[ts];
                    let yn = xn.scale(b[0]) + z[0];
                    y_row[ts] = yn;
                    for i in 0..n - 1 {
                        z[i] = xn.scale(b[i + 1]) + z[i + 1] - yn.scale(a_pad[i + 1]);
                    }
                    z[n - 1] = xn.scale(b[n]) - yn.scale(a_pad[n]);
                }
            }
        }
        y
    }

    /// Pseudocodes 112..=114：瞬态抑制（ducker）。
    pub fn duck(&mut self, u: &QmfMatrix) -> QmfMatrix {
        let slots = u.slots;
        if slots == 0 {
            return u.clone();
        }
        let pb0 = band_of_subband(0);
        let mut out = QmfMatrix::zeros(slots);
        let mut energy = [0.0f64; 15];
        let mut gains = vec![[1.0f64; 15]; slots];
        for ts in 0..slots {
            energy.fill(0.0);
            for sb in 0..SUBBANDS {
                energy[pb0[sb]] += u.get(sb, ts).norm_sq();
            }
            for b in 0..15 {
                let e = energy[b];
                self.peak[b] = (0.76592833836465 * self.peak[b]).max(e);
                let sm = 0.25 * e + 0.75 * self.smooth[b];
                let df = 0.25 * (self.peak[b] - e) + 0.75 * self.diff[b];
                self.smooth[b] = sm;
                self.diff[b] = df;
                gains[ts][b] = if 1.5 * df > sm { sm / (1.5 * (df + 1e-9)) } else { 1.0 };
            }
        }
        for sb in 0..SUBBANDS {
            let band = pb0[sb];
            let src = u.row(sb);
            let dst = out.row_mut(sb);
            for ts in 0..slots {
                dst[ts] = src[ts].scale(gains[ts][band]);
            }
        }
        out
    }
}

fn interpolate(
    data: &AcplData,
    sets: &[Vec<f64>],
    prev: &[f64; SUBBANDS],
    bands_id: usize,
    slots: usize,
) -> Vec<[f64; SUBBANDS]> {
    let pb = band_of_subband(bands_id);
    let per_sb: Vec<[f64; SUBBANDS]> = sets
        .iter()
        .map(|s| {
            let mut arr = [0.0f64; SUBBANDS];
            for sb in 0..SUBBANDS {
                arr[sb] = s[pb[sb].min(s.len() - 1)];
            }
            arr
        })
        .collect();
    let mut out = vec![[0.0f64; SUBBANDS]; slots];
    if data.interpolation == 0 {
        if sets.len() == 1 {
            for ts in 0..slots {
                let frac = (ts + 1) as f64 / slots as f64;
                for sb in 0..SUBBANDS {
                    out[ts][sb] = prev[sb] + frac * (per_sb[0][sb] - prev[sb]);
                }
            }
        } else {
            let half = slots / 2;
            for ts in 0..slots {
                if ts < half {
                    let frac = (ts + 1) as f64 / half as f64;
                    for sb in 0..SUBBANDS {
                        out[ts][sb] = prev[sb] + frac * (per_sb[0][sb] - prev[sb]);
                    }
                } else {
                    let frac = (ts - half + 1) as f64 / (slots - half) as f64;
                    for sb in 0..SUBBANDS {
                        out[ts][sb] = per_sb[0][sb] + frac * (per_sb[1][sb] - per_sb[0][sb]);
                    }
                }
            }
        }
    } else {
        for ts in 0..slots {
            for sb in 0..SUBBANDS {
                let mut v = if ts < data.timeslots[0] { prev[sb] } else { per_sb[0][sb] };
                if sets.len() == 2 && ts >= data.timeslots[1] {
                    v = per_sb[1][sb];
                }
                out[ts][sb] = v;
            }
        }
    }
    out
}

fn alpha_beta(
    module: &mut Module,
    data: &AcplData,
    config: AcplConfig,
    alpha_name: &'static str,
    beta_name: &'static str,
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let qa = module.quantized(data, alpha_name, config.bands());
    let qb = module.quantized(data, beta_name, config.bands());
    let coarse = config.quant[0] != 0;
    let mut alphas = Vec::with_capacity(qa.len());
    let mut betas = Vec::with_capacity(qb.len());
    for (a_set, b_set) in qa.iter().zip(qb.iter()) {
        let mut da = vec![0.0f64; a_set.len()];
        let mut db = vec![0.0f64; b_set.len()];
        for i in config.param_band..a_set.len() {
            if coarse {
                let ai = (a_set[i] as isize).clamp(0, 16) as usize;
                let bi = (b_set[i] as isize).clamp(0, 4) as usize;
                da[i] = ALPHA_COARSE[ai];
                db[i] = BETA_COARSE[bi][IBETA_COARSE[ai]];
            } else {
                let ai = (a_set[i] as isize).clamp(0, 32) as usize;
                let bi = (b_set[i] as isize).clamp(0, 8) as usize;
                da[i] = ALPHA_FINE[ai];
                db[i] = BETA_FINE[bi][IBETA_FINE[ai]];
            }
        }
        alphas.push(da);
        betas.push(db);
    }
    (alphas, betas)
}

pub fn couple(
    module: &mut Module,
    data: &AcplData,
    config: AcplConfig,
    x0: &QmfMatrix,
    x1: Option<&QmfMatrix>,
) -> (QmfMatrix, QmfMatrix) {
    let slots = x0.slots;
    let mut x0in = QmfMatrix::zeros(slots);
    for (dst, &src) in x0in.data.iter_mut().zip(x0.data.iter()) {
        *dst = src.scale(2.0);
    }
    let y_dec = module.decorrelate(&x0in);
    let y = module.duck(&y_dec);
    let (alphas, betas) = alpha_beta(module, data, config, "alpha1", "beta1");
    let pb = band_of_subband(config.bands_id);
    let prev_a = module.dq_prev_a;
    let prev_b = module.dq_prev_b;
    let ia = interpolate(data, &alphas, &prev_a, config.bands_id, slots);
    let ib = interpolate(data, &betas, &prev_b, config.bands_id, slots);

    let last_a = alphas.last().unwrap();
    let last_b = betas.last().unwrap();
    for sb in 0..SUBBANDS {
        module.dq_prev_a[sb] = last_a[pb[sb].min(last_a.len() - 1)];
        module.dq_prev_b[sb] = last_b[pb[sb].min(last_b.len() - 1)];
    }

    let mut z0 = QmfMatrix::zeros(slots);
    let mut z1 = QmfMatrix::zeros(slots);
    for sb in 0..SUBBANDS {
        let x0_row = x0in.row(sb);
        let y_row = y.row(sb);
        let z0_row = z0.row_mut(sb);
        let z1_row = z1.row_mut(sb);
        if sb < config.qmf_band {
            if let Some(res) = x1 {
                let r_row = res.row(sb);
                for ts in 0..slots {
                    let x1in = r_row[ts].scale(2.0);
                    z0_row[ts] = (x0_row[ts] + x1in).scale(0.5);
                    z1_row[ts] = (x0_row[ts] - x1in).scale(0.5);
                }
            } else {
                for ts in 0..slots {
                    z0_row[ts] = x0_row[ts].scale(0.5);
                    z1_row[ts] = x0_row[ts].scale(0.5);
                }
            }
        } else {
            for ts in 0..slots {
                let a = ia[ts][sb];
                let b = ib[ts][sb];
                z0_row[ts] = (x0_row[ts].scale(1.0 + a) + y_row[ts].scale(b)).scale(0.5);
                z1_row[ts] = (x0_row[ts].scale(1.0 - a) - y_row[ts].scale(b)).scale(0.5);
            }
        }
    }
    (z0, z1)
}
