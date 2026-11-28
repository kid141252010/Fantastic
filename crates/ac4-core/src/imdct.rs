//! AC-4 IMDCT、KBD 窗函数与块切换重叠相加（TS 103 190-1 clause 5.5）。

use std::f64::consts::PI;

use crate::qmf::C64;

fn kbd_alpha(n: usize) -> f64 {
    match n {
        2048 | 1920 | 1536 => 3.0,
        1024 | 960 | 768 => 4.0,
        512 | 480 | 384 => 4.5,
        256 | 240 | 192 => 5.0,
        128 | 120 | 96 => 6.0,
        _ => panic!("unsupported AC-4 transform length {n}"),
    }
}

/// 零阶修正第一类贝塞尔函数 I_0(x)。
fn bessel_i0(x: f64) -> f64 {
    let half = 0.5 * x;
    let mut sum = 1.0;
    let mut term = 1.0;
    for k in 1..60 {
        let ratio = half / k as f64;
        term *= ratio * ratio;
        sum += term;
        if term < sum * 1e-16 {
            break;
        }
    }
    sum
}

pub fn kbd_left(n: usize) -> Vec<f64> {
    let alpha = kbd_alpha(n);
    let pa = PI * alpha;
    let denom = bessel_i0(pa);
    let mut kernel = Vec::with_capacity(n + 1);
    let mut total = 0.0;
    for p in 0..=n {
        let u = 2.0 * p as f64 / n as f64 - 1.0;
        let arg = pa * (1.0 - u * u).max(0.0).sqrt();
        let v = bessel_i0(arg) / denom;
        total += v;
        kernel.push(v);
    }
    let mut cum = 0.0;
    let mut out = Vec::with_capacity(n);
    for p in 0..n {
        cum += kernel[p];
        out.push((cum / total).sqrt());
    }
    out
}

fn left_window(n: usize, previous: usize) -> Vec<f64> {
    let nw = n.min(previous);
    let skip = (n - nw) / 2;
    let kbd = kbd_left(nw);
    let mut out = Vec::with_capacity(n);
    out.resize(skip, 0.0);
    out.extend_from_slice(&kbd);
    out.resize(n, 1.0);
    out
}

fn right_window(n: usize, previous: usize) -> Vec<f64> {
    let nw = n.min(previous);
    let skip = (previous - nw) / 2;
    let kbd = kbd_left(nw);
    let mut out = Vec::with_capacity(previous);
    out.resize(skip, 1.0);
    out.extend(kbd.iter().rev().copied());
    out.resize(previous, 0.0);
    out
}

/// 未归一化的复数逆 DFT：`out[j] = sum_{k=0..N-1} x[k] * exp(+i * 2*pi * k * j / N)`，
/// 对 AC-4 涉及的 2^a * 3^b * 5^c 长度使用混合基 Cooley-Tukey 分治计算。
fn idft_unnormalized(x: &[C64]) -> Vec<C64> {
    let n = x.len();
    if n == 1 {
        return vec![x[0]];
    }
    let radix = if n.is_multiple_of(4) {
        4
    } else if n.is_multiple_of(2) {
        2
    } else if n.is_multiple_of(3) {
        3
    } else if n.is_multiple_of(5) {
        5
    } else {
        // 兜底直接 DFT（AC-4 长度不会走到这里）
        let mut out = vec![C64::default(); n];
        let step = 2.0 * PI / n as f64;
        for j in 0..n {
            let mut acc = C64::default();
            for k in 0..n {
                let a = step * (j * k) as f64;
                acc += x[k] * C64::new(a.cos(), a.sin());
            }
            out[j] = acc;
        }
        return out;
    };

    let m = n / radix;
    let mut sub = Vec::with_capacity(radix);
    for r in 0..radix {
        let slice: Vec<C64> = (0..m).map(|k| x[k * radix + r]).collect();
        sub.push(idft_unnormalized(&slice));
    }

    let mut out = vec![C64::default(); n];
    let base_angle = 2.0 * PI / n as f64;
    for j0 in 0..m {
        // 对每个 r 计算旋转因子后的第 r 支输入 y[r] = twiddle(r, j0) * sub[r][j0]
        let mut y = [C64::default(); 5];
        y[0] = sub[0][j0];
        for r in 1..radix {
            let a = base_angle * (r * j0) as f64;
            y[r] = sub[r][j0] * C64::new(a.cos(), a.sin());
        }
        match radix {
            2 => {
                out[j0] = y[0] + y[1];
                out[j0 + m] = y[0] - y[1];
            }
            4 => {
                let a0 = y[0] + y[2];
                let a1 = y[0] - y[2];
                let b0 = y[1] + y[3];
                let b1 = y[1] - y[3];
                // +i * b1 = (-b1.im, b1.re)，-i * b1 = (b1.im, -b1.re)
                let ib1 = C64::new(-b1.im, b1.re);
                out[j0] = a0 + b0;
                out[j0 + m] = a1 + ib1;
                out[j0 + 2 * m] = a0 - b0;
                out[j0 + 3 * m] = a1 - ib1;
            }
            _ => {
                let r_angle = 2.0 * PI / radix as f64;
                for q in 0..radix {
                    let mut acc = y[0];
                    for r in 1..radix {
                        let a = r_angle * (q * r) as f64;
                        acc += y[r] * C64::new(a.cos(), a.sin());
                    }
                    out[j0 + q * m] = acc;
                }
            }
        }
    }
    out
}

pub fn unfold(spectrum: &[f64]) -> Vec<f64> {
    let n = spectrum.len();
    let half = n / 2;
    let mut tw = Vec::with_capacity(half);
    let mut z = Vec::with_capacity(half);
    for k in 0..half {
        let angle = 2.0 * PI * (8 * k + 1) as f64 / (16 * n) as f64;
        let t = C64::new(-angle.cos(), -angle.sin());
        tw.push(t);
        let s = C64::new(spectrum[n - 1 - 2 * k], spectrum[2 * k]);
        z.push(s * t);
    }
    let y_raw = idft_unnormalized(&z);
    let inv_n = 1.0 / n as f64;
    let mut yr = Vec::with_capacity(half);
    let mut yi = Vec::with_capacity(half);
    for k in 0..half {
        let y = y_raw[k] * tw[k];
        yr.push(y.re * inv_n);
        yi.push(y.im * inv_n);
    }
    let q = n / 4;
    let mut x = vec![0.0f64; 2 * n];
    for i in 0..q {
        x[2 * i] = yi[q + i];
        x[2 * i + 1] = -yr[q - i - 1];
        x[half + 2 * i] = yr[i];
        x[half + 2 * i + 1] = -yi[half - i - 1];
        x[n + 2 * i] = yr[q + i];
        x[n + 2 * i + 1] = -yi[q - i - 1];
        x[3 * half + 2 * i] = -yi[i];
        x[3 * half + 2 * i + 1] = yr[half - i - 1];
    }
    x
}

/// 单声道变换状态：重叠缓冲与上一块长度。
pub struct Imdct {
    frame_len: usize,
    overlap: Vec<f64>,
    previous: usize,
}

impl Imdct {
    pub fn new(frame_len: usize) -> Self {
        Self { frame_len, overlap: vec![0.0; frame_len], previous: frame_len }
    }

    pub fn process(&mut self, spectrum: &[f64], windows: &[usize]) -> Vec<f64> {
        let full = self.frame_len;
        let mut pcm = vec![0.0f64; full];
        let mut start = 0usize;
        for &n in windows {
            let mut x = unfold(&spectrum[start..start + n]);
            let lw = left_window(n, self.previous);
            for i in 0..n {
                x[i] *= lw[i];
            }
            let skip = (full - n) / 2;
            let skip_prev = (full - self.previous) / 2;
            let rw = right_window(n, self.previous);
            for i in 0..self.previous {
                self.overlap[skip_prev + i] *= rw[i];
            }
            for i in 0..n {
                self.overlap[skip + i] += x[i];
            }
            pcm[start..start + n].copy_from_slice(&self.overlap[..n]);
            for i in 0..skip {
                self.overlap[i] = self.overlap[n + i];
            }
            self.overlap[skip..skip + n].copy_from_slice(&x[n..2 * n]);
            self.previous = n;
            start += n;
        }
        pcm
    }
}
