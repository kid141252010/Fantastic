//! 纯 Rust 实现的 Dolby AC-4（ETSI TS 103 190-1 / TS 103 190-2）与 A-JOC 解码器，
//! 按 SoundWeaver 的 `core/ac4` 实现 1:1 移植。

#![allow(clippy::needless_range_loop, clippy::too_many_arguments)]

pub mod acpl;
pub mod ajoc;
pub mod asf;
pub mod aspx;
pub mod bits;
pub mod codes;
pub mod elements;
pub mod frame;
pub mod imdct;
pub mod oamd;
pub mod qmf;
pub mod render;
pub mod smp;
pub mod stream;
pub mod tables;
pub mod toc;

pub use stream::{
    frame_samples, Ac4Controls, Ac4Mode, Ac4Scene, Ac4Stream, Ac4StreamConfig, Ac4StreamInfo,
    BinauralProfile,
};

#[derive(Clone, Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

pub(crate) fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(Error::new(message))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::frame::FrameDecoder;
    use crate::imdct::kbd_left;
    use crate::qmf::{Analysis, Synthesis, BANDS, DELAY};
    use crate::render::{joc_objects, Renderer, LAYOUT_2_0};
    use crate::stream::{Ac4Mode, Ac4Stream, Ac4StreamConfig};

    #[test]
    fn qmf_near_perfect_reconstruction() {
        let slots = 32;
        let len = slots * BANDS;
        let input: Vec<f64> = (0..len)
            .map(|i| (0.07 * i as f64).sin() + 0.5 * (0.23 * i as f64).cos())
            .collect();
        let mut analysis = Analysis::new();
        let mut synthesis = Synthesis::new();
        let q = analysis.process(&input);
        let out = synthesis.process(&q);
        // 越过 DELAY 后的重建误差约为 -62 dB（原型滤波器自身特性）
        let mut max_err = 0.0f64;
        for i in (DELAY + 64)..len {
            let err = (out[i] - input[i - DELAY]).abs();
            max_err = max_err.max(err);
        }
        assert!(max_err < 3e-3, "QMF reconstruction error too large: {max_err}");
    }

    #[test]
    fn kbd_window_power_complementary() {
        for &n in &[96usize, 120, 128, 192, 240, 256, 384, 480, 512, 768, 960, 1024, 1536, 1920, 2048] {
            let w = kbd_left(n);
            assert_eq!(w.len(), n);
            for i in 0..n {
                let sum = w[i] * w[i] + w[n - 1 - i] * w[n - 1 - i];
                assert!((sum - 1.0).abs() < 1e-12, "KBD({n}) at {i}: sum={sum}");
            }
        }
    }

    fn read_frames_bin(bytes: &[u8]) -> Vec<&[u8]> {
        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos + 4 <= bytes.len() {
            let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            out.push(&bytes[pos..pos + len]);
            pos += len;
        }
        out
    }

    fn read_f64_vec(bytes: &[u8]) -> Vec<f64> {
        bytes
            .as_chunks::<8>()
            .0
            .iter()
            .map(|&c| f64::from_le_bytes(c))
            .collect()
    }

    #[test]
    fn matches_soundweaver_reference_on_real_samples() {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(run_soundweaver_reference_check)
            .unwrap()
            .join()
            .unwrap();
    }

    fn run_soundweaver_reference_check() {
        let Ok(ref_dir) = std::env::var("AC4_REF_DIR") else {
            return;
        };
        let ref_dir = Path::new(&ref_dir);
        if !ref_dir.is_dir() {
            return;
        }

        for tag in ["da_ims", "ajoc_448", "ajoc_1500", "ajoc_raw"] {
            let frames_path = ref_dir.join(format!("{tag}_frames.bin"));
            if !frames_path.is_file() {
                continue;
            }
            let frames_bytes = std::fs::read(&frames_path).unwrap();
            let frames = read_frames_bin(&frames_bytes);
            assert!(!frames.is_empty(), "{tag} has no frames");

            let mut dec = FrameDecoder::new();
            let mut renderer: Option<Renderer> = None;
            let mut rust_ch: std::collections::HashMap<String, Vec<f64>> =
                std::collections::HashMap::new();
            let mut rust_gains: Vec<Vec<f64>> = Vec::new();

            for (k, f) in frames.iter().enumerate() {
                let pcm = dec.decode(f).unwrap_or_else(|e| panic!("{tag} frame {k} failed: {e}"));
                let frame_len = pcm.values().next().unwrap().len();
                for (name, arr) in pcm {
                    rust_ch.entry(name).or_default().extend_from_slice(&arr);
                }
                if let Some(pl) = &dec.payload {
                    let meta = &pl.metadata;
                    let n_obj = joc_objects(meta).len();
                    if renderer.is_none() {
                        renderer = Some(Renderer::new(n_obj, &LAYOUT_2_0));
                        rust_gains = vec![Vec::new(); n_obj];
                    }
                    let g = renderer.as_mut().unwrap().gains(meta, frame_len);
                    for (o, g_row) in g.into_iter().enumerate() {
                        rust_gains[o].extend_from_slice(&g_row);
                    }
                }
            }

            let mut max_pcm_diff_norm = 0.0f64;
            let mut worst_ch = String::new();
            for (name, rust_samples) in &rust_ch {
                let ref_bytes =
                    std::fs::read(ref_dir.join(format!("{tag}_pcm_{name}.f64"))).unwrap();
                let ref_samples = read_f64_vec(&ref_bytes);
                assert_eq!(rust_samples.len(), ref_samples.len(), "{tag} {name} length mismatch");
                for (&r, &p) in rust_samples.iter().zip(ref_samples.iter()) {
                    let diff = (r - p).abs() / 32768.0;
                    if diff > max_pcm_diff_norm {
                        max_pcm_diff_norm = diff;
                        worst_ch = name.clone();
                    }
                }
            }
            println!(
                "[{tag}] frames={} channels={} max_normalized_pcm_diff={:.3e} (channel {})",
                frames.len(),
                rust_ch.len(),
                max_pcm_diff_norm,
                worst_ch
            );
            assert!(
                max_pcm_diff_norm < 1e-4,
                "{tag} PCM diverged from SoundWeaver by {max_pcm_diff_norm} on {worst_ch}"
            );

            let gains_path = ref_dir.join(format!("{tag}_gains.f64"));
            if gains_path.is_file() {
                let ref_g = read_f64_vec(&std::fs::read(&gains_path).unwrap());
                let flat_rust: Vec<f64> = rust_gains.into_iter().flatten().collect();
                assert_eq!(flat_rust.len(), ref_g.len(), "{tag} gains length mismatch");
                let mut max_g_diff = 0.0f64;
                for (&r, &p) in flat_rust.iter().zip(ref_g.iter()) {
                    max_g_diff = max_g_diff.max((r - p).abs());
                }
                println!("[{tag}] max_gains_diff={:.3e}", max_g_diff);
                assert!(max_g_diff < 1e-9, "{tag} OAMD renderer gains diverged: {max_g_diff}");
            }

            // 同时验证 Ac4Stream（Speaker 2.0 与 Binaural）在完整流式推送/拉取下正常输出有限样本
            for mode in [Ac4Mode::Speaker20, Ac4Mode::Binaural] {
                let hrtf = if mode == Ac4Mode::Binaural {
                    Some(include_bytes!("../../../web/joc/hrtf/D2.jochrtf").to_vec())
                } else {
                    None
                };
                let mut stream = Ac4Stream::new(Ac4StreamConfig {
                    mode,
                    hrtf,
                    tail_seconds: 0.1,
                    ..Ac4StreamConfig::default()
                })
                .unwrap();
                for f in &frames[..frames.len().min(8)] {
                    stream.push_frame(f).unwrap();
                }
                stream.flush().unwrap();
                let mut out = vec![0.0f32; 65536 * 2];
                let pulled = stream.pull(&mut out, 65536).unwrap();
                assert!(pulled > 0, "{tag} {mode:?} produced 0 samples");
                assert_eq!(stream.info().non_finite, 0, "{tag} {mode:?} produced non-finite samples");
                assert!(stream.info().peak > 1e-5, "{tag} {mode:?} output is silent");
            }
        }
    }
}
