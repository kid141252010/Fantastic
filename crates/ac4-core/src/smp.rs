//! MDCT 域立体声与多声道矩阵工具（TS 103 190-1 clauses 4.2.10 与 5.3）。

use crate::asf::Layout;
use crate::bits::BitReader;
use crate::codes::{huffman, Book};
use crate::tables::sfb_offsets;
use crate::{fail, Result};

pub const SAP_IDENTITY: u32 = 0;
pub const SAP_MS_SOME: u32 = 1;
pub const SAP_MS_ALL: u32 = 2;
pub const SAP_FULL: u32 = 3;

#[derive(Clone, Debug)]
pub struct ChParam {
    pub mode: u32,
    pub ms_used: Option<Vec<Vec<bool>>>,
    pub coeff_used: Option<Vec<Vec<bool>>>,
    pub delta_code_time: bool,
    pub dpcm_alpha: Option<Vec<Vec<i32>>>,
}

pub fn read_chparam(b: &mut BitReader<'_>, layout: &Layout) -> Result<ChParam> {
    let mode = b.read(2)?;
    let mut param = ChParam {
        mode,
        ms_used: None,
        coeff_used: None,
        delta_code_time: false,
        dpcm_alpha: None,
    };
    let bands: Vec<usize> = (0..layout.groups()).map(|g| layout.band_count(g, false)).collect();
    if mode == SAP_MS_SOME {
        let mut ms = Vec::with_capacity(bands.len());
        for &n in &bands {
            let mut row = Vec::with_capacity(n);
            for _ in 0..n {
                row.push(b.flag()?);
            }
            ms.push(row);
        }
        param.ms_used = Some(ms);
    } else if mode == SAP_FULL {
        let used = if b.flag()? {
            bands.iter().map(|&n| vec![true; n]).collect::<Vec<_>>()
        } else {
            let mut out = Vec::with_capacity(bands.len());
            for &n in &bands {
                let mut row = Vec::with_capacity(n);
                for _ in (0..n).step_by(2) {
                    let flag = b.flag()?;
                    row.push(flag);
                    if row.len() < n {
                        row.push(flag);
                    }
                }
                out.push(row);
            }
            out
        };
        if layout.groups() != 1 {
            param.delta_code_time = b.flag()?;
        }
        let sf = huffman(Book::AsfScalefac);
        let mut dpcm = Vec::with_capacity(bands.len());
        for (g, &n) in bands.iter().enumerate() {
            let mut row = Vec::with_capacity(n);
            for s in 0..n {
                if s % 2 == 0 && used[g][s] {
                    row.push(sf.decode(b)?);
                } else {
                    row.push(0);
                }
            }
            dpcm.push(row);
        }
        param.coeff_used = Some(used);
        param.dpcm_alpha = Some(dpcm);
    }
    Ok(param)
}

#[derive(Clone, Debug)]
pub struct Matrix {
    pub a: Vec<f64>,
    pub b: Vec<f64>,
    pub c: Vec<f64>,
    pub d: Vec<f64>,
}

impl Matrix {
    pub fn identity(frame_len: usize) -> Self {
        Self {
            a: vec![1.0; frame_len],
            b: vec![0.0; frame_len],
            c: vec![0.0; frame_len],
            d: vec![1.0; frame_len],
        }
    }

    pub fn apply(&self, x: &[f64], y: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let n = self.a.len();
        let mut out0 = Vec::with_capacity(n);
        let mut out1 = Vec::with_capacity(n);
        for i in 0..n {
            out0.push(self.a[i] * x[i] + self.b[i] * y[i]);
            out1.push(self.c[i] * x[i] + self.d[i] * y[i]);
        }
        (out0, out1)
    }
}

pub fn matrix(param: &ChParam, layout: &Layout) -> Matrix {
    let bands: Vec<usize> = (0..layout.groups()).map(|g| layout.band_count(g, false)).collect();
    let total: usize = bands.iter().sum();
    let mut ba = vec![1.0f64; total + 1];
    let mut bb = vec![0.0f64; total + 1];
    let mut bc = vec![0.0f64; total + 1];
    let mut bd = vec![1.0f64; total + 1];

    let mut i = 0usize;
    let mut alpha: Vec<Vec<i32>> = Vec::with_capacity(bands.len());
    let mut previous = bands.first().copied().unwrap_or(0);
    for (g, &n) in bands.iter().enumerate() {
        let mut row = vec![0i32; n];
        for s in 0..n {
            let ms_on = param.mode == SAP_MS_ALL
                || (param.mode == SAP_MS_SOME
                    && param.ms_used.as_ref().is_some_and(|m| m[g][s]));
            if ms_on {
                ba[i] = 1.0;
                bb[i] = 1.0;
                bc[i] = 1.0;
                bd[i] = -1.0;
            } else if param.mode == SAP_FULL
                && param.coeff_used.as_ref().is_some_and(|u| u[g][s])
            {
                if s % 2 == 1 {
                    row[s] = row[s - 1];
                } else {
                    let dpcm = param.dpcm_alpha.as_ref().unwrap();
                    let delta = dpcm[g][s] - 60;
                    let in_time = g > 0 && n == previous && param.delta_code_time;
                    if in_time {
                        row[s] = alpha[g - 1][s] + delta;
                    } else if s == 0 {
                        row[s] = delta;
                    } else {
                        row[s] = row[s - 2] + delta;
                    }
                }
                let gain = f64::from(row[s]) * 0.1;
                ba[i] = 1.0 + gain;
                bb[i] = 1.0;
                bc[i] = 1.0 - gain;
                bd[i] = -1.0;
            }
            i += 1;
        }
        alpha.push(row);
        previous = n;
    }

    let mut line_band = vec![total; layout.frame_len];
    let mut start = 0usize;
    let mut first = 0usize;
    for g in 0..layout.groups() {
        let count = layout.group_windows[g];
        let n = layout.group_lengths[g];
        let max_sfb = bands[g];
        let sfb = sfb_offsets(n).unwrap();
        for _ in 0..count {
            for s in 0..max_sfb {
                let s0 = usize::from(sfb[s]);
                let s1 = usize::from(sfb[s + 1]);
                for line in s0..s1 {
                    line_band[start + line] = first + s;
                }
            }
            start += n;
        }
        first += max_sfb;
    }

    let mut a = Vec::with_capacity(layout.frame_len);
    let mut b = Vec::with_capacity(layout.frame_len);
    let mut c = Vec::with_capacity(layout.frame_len);
    let mut d = Vec::with_capacity(layout.frame_len);
    for &idx in &line_band {
        a.push(ba[idx]);
        b.push(bb[idx]);
        c.push(bc[idx]);
        d.push(bd[idx]);
    }
    Matrix { a, b, c, d }
}

pub fn three(selector: u32, m0: &Matrix, m1: &Matrix, x: [&[f64]; 3]) -> Result<[Vec<f64>; 3]> {
    let n = x[0].len();
    if selector >= 12 {
        return fail(format!("reserved chel_matsel {selector}"));
    }
    let mut o0 = vec![0.0f64; n];
    let mut o1 = vec![0.0f64; n];
    let mut o2 = vec![0.0f64; n];
    for i in 0..n {
        let a0 = m0.a[i];
        let b0 = m0.b[i];
        let c0 = m0.c[i];
        let d0 = m0.d[i];
        let a1 = m1.a[i];
        let b1 = m1.b[i];
        let c1 = m1.c[i];
        let d1 = m1.d[i];
        let i0 = x[0][i];
        let i1 = x[1][i];
        let i2 = x[2][i];
        let (r0, r1, r2) = match selector {
            0 => (
                a0 * a1 * i0 + b0 * a1 * i1 + b1 * i2,
                c0 * i0 + d0 * i1,
                a0 * c1 * i0 + b0 * c1 * i1 + d1 * i2,
            ),
            1 => (
                d0 * i0 + c0 * i1,
                b0 * a1 * i0 + a0 * a1 * i1 + b1 * i2,
                b0 * c1 * i0 + a0 * c1 * i1 + d1 * i2,
            ),
            2 => (
                a0 * a1 * i0 + b1 * i1 + b0 * a1 * i2,
                a0 * c1 * i0 + d1 * i1 + b0 * c1 * i2,
                c0 * i0 + d0 * i2,
            ),
            3 => (
                a1 * i0 + c0 * b1 * i1 + d0 * b1 * i2,
                a0 * i1 + b0 * i2,
                c1 * i0 + c0 * d1 * i1 + d0 * d1 * i2,
            ),
            4 => (
                a0 * i0 + b0 * i2,
                c0 * b1 * i0 + a1 * i1 + d0 * b1 * i2,
                c0 * d1 * i0 + c1 * i1 + d0 * d1 * i2,
            ),
            5 => (
                a1 * i0 + d0 * b1 * i1 + c0 * b1 * i2,
                c1 * i0 + d0 * d1 * i1 + c0 * d1 * i2,
                b0 * i1 + a0 * i2,
            ),
            6 => (
                d0 * d1 * i0 + c0 * d1 * i1 + c1 * i2,
                b0 * i0 + a0 * i1,
                d0 * b1 * i0 + c0 * b1 * i1 + a1 * i2,
            ),
            7 => (
                a0 * i0 + b0 * i1,
                c0 * d1 * i0 + d0 * d1 * i1 + c1 * i2,
                c0 * b1 * i0 + d0 * b1 * i1 + a1 * i2,
            ),
            8 => (
                d0 * d1 * i0 + c1 * i1 + c0 * d1 * i2,
                d0 * b1 * i0 + a1 * i1 + c0 * b1 * i2,
                b0 * i0 + a0 * i2,
            ),
            9 => (
                d1 * i0 + b0 * c1 * i1 + a0 * c1 * i2,
                d0 * i1 + c0 * i2,
                b1 * i0 + b0 * a1 * i1 + a0 * a1 * i2,
            ),
            10 => (
                d0 * i0 + c0 * i2,
                b0 * c1 * i0 + d1 * i1 + a0 * c1 * i2,
                b0 * a1 * i0 + b1 * i1 + a0 * a1 * i2,
            ),
            11 => (
                d1 * i0 + a0 * c1 * i1 + b0 * c1 * i2,
                b1 * i0 + a0 * a1 * i1 + b0 * a1 * i2,
                c0 * i1 + d0 * i2,
            ),
            _ => unreachable!(),
        };
        o0[i] = r0;
        o1[i] = r1;
        o2[i] = r2;
    }
    Ok([o0, o1, o2])
}

pub fn four(m: &[Matrix; 4], x: [&[f64]; 4]) -> [Vec<f64>; 4] {
    let (p0, p1) = m[0].apply(x[0], x[1]);
    let (q0, q1) = m[1].apply(x[2], x[3]);
    let (o0, o2) = m[2].apply(&p0, &q0);
    let (o1, o3) = m[3].apply(&p1, &q1);
    [o0, o1, o2, o3]
}

pub fn five(selector: u32, m: &[Matrix; 5], x: [&[f64]; 5]) -> Result<[Vec<f64>; 5]> {
    let [t0, t1, t2] = three(selector, &m[0], &m[1], [x[0], x[1], x[2]])?;
    let (y0, y1) = m[2].apply(x[3], x[4]);
    let (o0, o3) = m[3].apply(&t0, &y0);
    let (o1, o4) = m[4].apply(&t1, &y1);
    Ok([o0, o1, t2, o3, o4])
}
