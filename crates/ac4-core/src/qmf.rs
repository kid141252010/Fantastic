//! TS 103 420 clause 7 的 64 频带复数 QMF 滤波器组（分析与综合）。

use std::f64::consts::PI;
use std::ops::{Add, AddAssign, Mul, Sub};
use std::sync::OnceLock;

use crate::tables::QWIN;

pub const BANDS: usize = 64;
pub const LENGTH: usize = 640;
/// 分析后直接综合造成的信号延迟（样本数）
pub const DELAY: usize = LENGTH - BANDS + 1;
const TAPS: usize = LENGTH / BANDS;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct C64 {
    pub re: f64,
    pub im: f64,
}

impl C64 {
    #[inline]
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[inline]
    pub fn conj(self) -> Self {
        Self { re: self.re, im: -self.im }
    }

    #[inline]
    pub fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    #[inline]
    pub fn scale(self, s: f64) -> Self {
        Self { re: self.re * s, im: self.im * s }
    }
}

impl Add for C64 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self { re: self.re + rhs.re, im: self.im + rhs.im }
    }
}

impl AddAssign for C64 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl Sub for C64 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self { re: self.re - rhs.re, im: self.im - rhs.im }
    }
}

impl Mul for C64 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

/// 单个通道的 64×slots 复数 QMF 矩阵（按 `[subband][slot]` 存储）。
#[derive(Clone)]
pub struct QmfMatrix {
    pub slots: usize,
    pub data: Vec<C64>,
}

impl QmfMatrix {
    pub fn zeros(slots: usize) -> Self {
        Self { slots, data: vec![C64::default(); BANDS * slots] }
    }

    #[inline]
    pub fn row(&self, sb: usize) -> &[C64] {
        let start = sb * self.slots;
        &self.data[start..start + self.slots]
    }

    #[inline]
    pub fn row_mut(&mut self, sb: usize) -> &mut [C64] {
        let start = sb * self.slots;
        &mut self.data[start..start + self.slots]
    }

    #[inline]
    pub fn get(&self, sb: usize, slot: usize) -> C64 {
        self.data[sb * self.slots + slot]
    }

    #[inline]
    pub fn set(&mut self, sb: usize, slot: usize, val: C64) {
        self.data[sb * self.slots + slot] = val;
    }
}

struct Tables {
    /// analysis[k][j] = exp(i * pi * (k + 0.5) * (j - 0.5) / 64)
    analysis: Vec<[C64; 2 * BANDS]>,
    /// synthesis[j][k] = exp(i * pi / (4 * 64) * (2*k + 1) * (2*j - 4*64 + 1)) / 64
    synthesis: Vec<[C64; BANDS]>,
    window_reversed: [f64; LENGTH],
    upper_phases: [[f64; BANDS]; 5],
    lower_phases: [[f64; BANDS]; 5],
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut analysis = vec![[C64::default(); 2 * BANDS]; BANDS];
        for k in 0..BANDS {
            for j in 0..(2 * BANDS) {
                let angle = PI * (k as f64 + 0.5) * (j as f64 - 0.5) / BANDS as f64;
                analysis[k][j] = C64::new(angle.cos(), angle.sin());
            }
        }
        let mut synthesis = vec![[C64::default(); BANDS]; 2 * BANDS];
        let inv_bands = 1.0 / BANDS as f64;
        for j in 0..(2 * BANDS) {
            for k in 0..BANDS {
                let angle = PI / (4.0 * BANDS as f64)
                    * (2.0 * k as f64 + 1.0)
                    * (2.0 * j as f64 - 4.0 * BANDS as f64 + 1.0);
                synthesis[j][k] = C64::new(angle.cos() * inv_bands, angle.sin() * inv_bands);
            }
        }
        let mut window_reversed = [0.0f64; LENGTH];
        for i in 0..LENGTH {
            window_reversed[i] = QWIN[LENGTH - 1 - i];
        }
        let mut upper_phases = [[0.0f64; BANDS]; 5];
        let mut lower_phases = [[0.0f64; BANDS]; 5];
        for i in 0..5 {
            let up_tap = TAPS - 1 - 2 * i;
            let lo_tap = TAPS - 2 - 2 * i;
            upper_phases[i].copy_from_slice(&QWIN[up_tap * BANDS..(up_tap + 1) * BANDS]);
            lower_phases[i].copy_from_slice(&QWIN[lo_tap * BANDS..(lo_tap + 1) * BANDS]);
        }
        Tables { analysis, synthesis, window_reversed, upper_phases, lower_phases }
    })
}

/// 单声道 QMF 分析滤波器组（实数样本 -> 64×slots 复数子带矩阵）。
pub struct Analysis {
    history: [f64; LENGTH - BANDS],
}

impl Default for Analysis {
    fn default() -> Self {
        Self::new()
    }
}

impl Analysis {
    pub fn new() -> Self {
        Self { history: [0.0; LENGTH - BANDS] }
    }

    pub fn process(&mut self, pcm: &[f64]) -> QmfMatrix {
        debug_assert_eq!(pcm.len() % BANDS, 0);
        let t = tables();
        let slots = pcm.len() / BANDS;
        let mut x = Vec::with_capacity(self.history.len() + pcm.len());
        x.extend_from_slice(&self.history);
        x.extend_from_slice(pcm);
        let keep_start = x.len() - self.history.len();
        self.history.copy_from_slice(&x[keep_start..]);

        let mut out = QmfMatrix::zeros(slots);
        let mut u = [0.0f64; 2 * BANDS];
        for slot in 0..slots {
            u.fill(0.0);
            let base = slot * BANDS;
            for m in 0..(LENGTH / (2 * BANDS)) {
                let win_off = m * (2 * BANDS);
                for p in 0..(2 * BANDS) {
                    u[2 * BANDS - 1 - p] +=
                        x[base + win_off + p] * t.window_reversed[win_off + p];
                }
            }
            for k in 0..BANDS {
                let mut re = 0.0;
                let mut im = 0.0;
                let row = &t.analysis[k];
                for j in 0..(2 * BANDS) {
                    re += u[j] * row[j].re;
                    im += u[j] * row[j].im;
                }
                out.set(k, slot, C64::new(re, im));
            }
        }
        out
    }
}

/// 单声道 QMF 综合滤波器组（64×slots 复数子带矩阵 -> 实数样本）。
pub struct Synthesis {
    history: [[f64; 2 * BANDS]; TAPS - 1],
}

impl Default for Synthesis {
    fn default() -> Self {
        Self::new()
    }
}

impl Synthesis {
    pub fn new() -> Self {
        Self { history: [[0.0; 2 * BANDS]; TAPS - 1] }
    }

    pub fn process(&mut self, q: &QmfMatrix) -> Vec<f64> {
        let t = tables();
        let slots = q.slots;
        let mut full = Vec::with_capacity((TAPS - 1) + slots);
        full.extend_from_slice(&self.history);

        for slot in 0..slots {
            let mut v = [0.0f64; 2 * BANDS];
            for j in 0..(2 * BANDS) {
                let row = &t.synthesis[j];
                let mut sum = 0.0;
                for k in 0..BANDS {
                    let qv = q.get(k, slot);
                    sum += qv.re * row[k].re - qv.im * row[k].im;
                }
                v[j] = sum;
            }
            full.push(v);
        }

        self.history.copy_from_slice(&full[slots..slots + (TAPS - 1)]);

        let mut out = vec![0.0f64; slots * BANDS];
        for slot in 0..slots {
            let dst = &mut out[slot * BANDS..(slot + 1) * BANDS];
            for i in 0..5 {
                let w_up = &full[slot + 2 * i];
                let w_lo = &full[slot + 2 * i + 1];
                let up = &t.upper_phases[i];
                let lo = &t.lower_phases[i];
                for n in 0..BANDS {
                    dst[n] += w_up[BANDS + n] * up[n] + w_lo[n] * lo[n];
                }
            }
        }
        out
    }
}
