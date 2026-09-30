//! Local speech-to-text for uilab: whisper.cpp through `whisper-rs`, on the GPU with the default
//! `cuda` feature and on the CPU with `--no-default-features` or `gpu: false`.
//!
//! A [`Transcriber`] loads the model once and is reused for every utterance. Input is 16 kHz mono
//! `f32` audio; [`resample_to_16k`] converts other rates. whisper.cpp's own log output is routed
//! into the `log` facade, so it prints nothing unless the host installs a logger.

use std::path::PathBuf;
use std::time::Instant;

use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperError,
    WhisperState,
};

/// The sample rate whisper expects.
pub const SAMPLE_RATE: u32 = 16_000;

/// The shortest utterance [`Transcriber::transcribe`] accepts, in milliseconds. Shorter audio is
/// refused because whisper tends to invent text for it.
pub const MIN_AUDIO_MS: u64 = 300;

/// whisper.cpp returns no segments for input under one second, so shorter accepted audio is
/// padded with silence to this many samples (1.1 s).
const PAD_TO_SAMPLES: usize = 17_600;

/// How a [`Transcriber`] is loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriberConfig {
    /// Path to a ggml whisper model, for example `ggml-large-v3-turbo.bin`.
    pub model: PathBuf,
    /// Spoken language as a whisper code (`en`, `de`, ...); `None` or `"auto"` detects it.
    pub language: Option<String>,
    /// Run on the GPU. Without the `cuda` feature this has no effect and whisper runs on the CPU.
    pub gpu: bool,
}

/// The text of one utterance and how long it took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transcript {
    /// Trimmed text with whisper's non-speech markers removed; may be empty.
    pub text: String,
    /// Length of the input audio.
    pub audio_ms: u64,
    /// Wall-clock time of the transcription.
    pub took_ms: u64,
}

/// Everything that can go wrong loading a model or transcribing audio.
#[derive(Debug, thiserror::Error)]
pub enum SttError {
    #[error("model file not found: {0}")]
    ModelNotFound(PathBuf),
    #[error("unknown whisper language code: {0:?}")]
    UnknownLanguage(String),
    #[error("failed to load model {model}: {source}")]
    Load {
        model: PathBuf,
        #[source]
        source: WhisperError,
    },
    #[error("audio too short: {audio_ms} ms, at least {min_ms} ms is required")]
    TooShort { audio_ms: u64, min_ms: u64 },
    #[error("audio sample {index} is not a finite number")]
    NonFiniteSample { index: usize },
    #[error("whisper failed: {0}")]
    Whisper(#[from] WhisperError),
}

/// A loaded whisper model plus its decoding state, reused across utterances.
pub struct Transcriber {
    // The state keeps its own reference to the model; the context is held so the model's lifetime
    // is explicit here.
    _ctx: WhisperContext,
    state: WhisperState,
    language: Option<String>,
    threads: i32,
}

impl std::fmt::Debug for Transcriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transcriber")
            .field("language", &self.language)
            .field("threads", &self.threads)
            .finish_non_exhaustive()
    }
}

impl Transcriber {
    /// Load the model named in `config`. This is the slow step (seconds); do it once.
    pub fn load(config: TranscriberConfig) -> Result<Self, SttError> {
        whisper_rs::install_logging_hooks();

        let language = normalize_language(config.language.as_deref())?;
        if !config.model.is_file() {
            return Err(SttError::ModelNotFound(config.model));
        }

        let mut params = WhisperContextParameters::default();
        params.use_gpu(config.gpu);
        let ctx = WhisperContext::new_with_params(&config.model, params).map_err(|source| {
            SttError::Load {
                model: config.model.clone(),
                source,
            }
        })?;
        let state = ctx.create_state().map_err(|source| SttError::Load {
            model: config.model.clone(),
            source,
        })?;
        let threads = std::thread::available_parallelism()
            .map(|n| n.get().min(16) as i32)
            .unwrap_or(4);

        Ok(Self {
            _ctx: ctx,
            state,
            language,
            threads,
        })
    }

    /// Transcribe one utterance of 16 kHz mono `f32` samples in `[-1, 1]`.
    ///
    /// `prompt` is vocabulary the speaker is likely to use; it is passed to whisper as the initial
    /// prompt. An empty prompt passes none.
    pub fn transcribe(&mut self, samples: &[f32], prompt: &str) -> Result<Transcript, SttError> {
        let audio_ms = check_audio(samples)?;
        let started = Instant::now();

        let mut padded;
        let input = if samples.len() < PAD_TO_SAMPLES {
            padded = samples.to_vec();
            padded.resize(PAD_TO_SAMPLES, 0.0);
            &padded[..]
        } else {
            samples
        };

        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: -1.0,
        });
        params.set_n_threads(self.threads);
        params.set_language(self.language.as_deref());
        params.set_translate(false);
        params.set_no_context(true);
        params.set_suppress_nst(true);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        let prompt = sanitize_prompt(prompt);
        if !prompt.is_empty() {
            params.set_initial_prompt(&prompt);
        }

        self.state.full(params, input)?;

        let mut raw = String::new();
        for segment in self.state.as_iter() {
            raw.push_str(&segment.to_str_lossy()?);
            raw.push(' ');
        }

        Ok(Transcript {
            text: clean_text(&raw),
            audio_ms,
            took_ms: started.elapsed().as_millis() as u64,
        })
    }
}

/// Map `None`, `""` and `"auto"` to auto-detection and reject codes whisper does not know.
fn normalize_language(language: Option<&str>) -> Result<Option<String>, SttError> {
    let Some(code) = language.map(|l| l.trim().to_ascii_lowercase()) else {
        return Ok(None);
    };
    if code.is_empty() || code == "auto" {
        return Ok(None);
    }
    if code.contains('\0') || whisper_rs::get_lang_id(&code).is_none() {
        return Err(SttError::UnknownLanguage(code));
    }
    Ok(Some(code))
}

/// Validate an utterance and return its length in milliseconds.
fn check_audio(samples: &[f32]) -> Result<u64, SttError> {
    let audio_ms = samples.len() as u64 * 1000 / u64::from(SAMPLE_RATE);
    if audio_ms < MIN_AUDIO_MS {
        return Err(SttError::TooShort {
            audio_ms,
            min_ms: MIN_AUDIO_MS,
        });
    }
    if let Some(index) = samples.iter().position(|s| !s.is_finite()) {
        return Err(SttError::NonFiniteSample { index });
    }
    Ok(audio_ms)
}

/// whisper's C API takes the prompt as a C string; drop NUL bytes and surrounding whitespace.
fn sanitize_prompt(prompt: &str) -> String {
    prompt.replace('\0', "").trim().to_owned()
}

/// Remove whisper's non-speech markers (`[BLANK_AUDIO]`, `(music)`, `♪`), collapse whitespace
/// and trim.
fn clean_text(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        let close = match c {
            '[' => ']',
            '(' => ')',
            '♪' | '♫' => continue,
            _ => {
                out.push(c);
                continue;
            }
        };
        // Skip to the matching close; an unclosed marker drops the rest of the text.
        for inner in chars.by_ref() {
            if inner == close {
                break;
            }
        }
        out.push(' ');
    }
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    tidy_punctuation(&collapsed)
}

/// Removing a marker can leave a space before punctuation (`"done [BLANK_AUDIO] ."`).
fn tidy_punctuation(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '.' | ',' | '!' | '?' | ';' | ':') && out.ends_with(' ') {
            out.pop();
        }
        out.push(c);
    }
    out.trim().to_owned()
}

/// Resample mono audio at `rate` Hz to 16 kHz with a Hann-windowed sinc filter. Downsampling
/// low-passes at the new Nyquist frequency. `rate == 0` or empty input yields an empty vector.
pub fn resample_to_16k(samples: &[f32], rate: u32) -> Vec<f32> {
    if samples.is_empty() || rate == 0 {
        return Vec::new();
    }
    if rate == SAMPLE_RATE {
        return samples.to_vec();
    }

    const ZERO_CROSSINGS: f64 = 16.0;
    let ratio = f64::from(SAMPLE_RATE) / f64::from(rate);
    let cutoff = ratio.min(1.0);
    let half_width = ZERO_CROSSINGS / cutoff;
    let out_len = ((samples.len() as f64) * ratio).round() as usize;
    let last = samples.len() as i64 - 1;

    let mut out = Vec::with_capacity(out_len);
    for n in 0..out_len {
        let t = n as f64 / ratio;
        let first = ((t - half_width).ceil() as i64).max(0);
        let end = ((t + half_width).floor() as i64).min(last);
        let mut acc = 0.0f64;
        let mut weight_sum = 0.0f64;
        for k in first..=end {
            let x = t - k as f64;
            let window = 0.5 + 0.5 * (std::f64::consts::PI * x / half_width).cos();
            let w = sinc(cutoff * x) * window;
            acc += w * f64::from(samples[k as usize]);
            weight_sum += w;
        }
        let value = if weight_sum.abs() > 1e-9 {
            acc / weight_sum
        } else {
            0.0
        };
        out.push(value as f32);
    }
    out
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        let px = std::f64::consts::PI * x;
        px.sin() / px
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn tone(freq: f32, rate: u32, secs: f32) -> Vec<f32> {
        let n = (rate as f32 * secs) as usize;
        (0..n)
            .map(|i| (2.0 * PI * freq * i as f32 / rate as f32).sin() * 0.5)
            .collect()
    }

    /// Amplitude of `freq` in `samples` at `rate`, by correlation with sine and cosine.
    fn amplitude(samples: &[f32], freq: f32, rate: u32) -> f32 {
        let (mut s, mut c) = (0.0f64, 0.0f64);
        for (i, &x) in samples.iter().enumerate() {
            let phase = 2.0 * std::f64::consts::PI * f64::from(freq) * i as f64 / f64::from(rate);
            s += f64::from(x) * phase.sin();
            c += f64::from(x) * phase.cos();
        }
        (2.0 * (s * s + c * c).sqrt() / samples.len() as f64) as f32
    }

    #[test]
    fn transcriber_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Transcriber>();
    }

    #[test]
    fn resample_identity_at_16k() {
        let input = tone(440.0, 16_000, 0.1);
        assert_eq!(resample_to_16k(&input, 16_000), input);
    }

    #[test]
    fn resample_empty_and_zero_rate() {
        assert!(resample_to_16k(&[], 48_000).is_empty());
        assert!(resample_to_16k(&[0.1, 0.2], 0).is_empty());
    }

    #[test]
    fn resample_lengths() {
        assert_eq!(resample_to_16k(&vec![0.0; 48_000], 48_000).len(), 16_000);
        assert_eq!(resample_to_16k(&vec![0.0; 44_100], 44_100).len(), 16_000);
        assert_eq!(resample_to_16k(&vec![0.0; 8_000], 8_000).len(), 16_000);
        assert_eq!(resample_to_16k(&vec![0.0; 22_050], 22_050).len(), 16_000);
    }

    #[test]
    fn resample_preserves_dc() {
        let out = resample_to_16k(&vec![0.25; 44_100], 44_100);
        assert!(out.iter().all(|&x| (x - 0.25).abs() < 1e-3), "{out:?}");
    }

    #[test]
    fn resample_keeps_passband_tone() {
        for rate in [8_000, 22_050, 44_100, 48_000] {
            let out = resample_to_16k(&tone(1_000.0, rate, 1.0), rate);
            // Ignore the edges, where the filter sees only half its window.
            let middle = &out[1_000..out.len() - 1_000];
            let a = amplitude(middle, 1_000.0, 16_000);
            assert!((a - 0.5).abs() < 0.02, "rate {rate}: amplitude {a}");
        }
    }

    #[test]
    fn resample_rejects_tone_above_new_nyquist() {
        // 12 kHz is representable at 48 kHz but above 16 kHz's 8 kHz Nyquist; it must not alias
        // down to 4 kHz.
        let out = resample_to_16k(&tone(12_000.0, 48_000, 1.0), 48_000);
        let middle = &out[1_000..out.len() - 1_000];
        let aliased = amplitude(middle, 4_000.0, 16_000);
        let energy = middle.iter().map(|x| x * x).sum::<f32>() / middle.len() as f32;
        assert!(aliased < 0.01, "aliased amplitude {aliased}");
        assert!(energy < 1e-4, "residual energy {energy}");
    }

    #[test]
    fn short_audio_is_refused() {
        let err = check_audio(&vec![0.0; 4_799]).unwrap_err();
        assert!(
            matches!(
                err,
                SttError::TooShort {
                    audio_ms: 299,
                    min_ms: 300
                }
            ),
            "{err:?}"
        );
        assert!(matches!(
            check_audio(&[]),
            Err(SttError::TooShort { audio_ms: 0, .. })
        ));
        assert_eq!(check_audio(&vec![0.0; 4_800]).unwrap(), 300);
        assert_eq!(check_audio(&vec![0.0; 32_000]).unwrap(), 2_000);
    }

    #[test]
    fn non_finite_audio_is_refused() {
        let mut samples = vec![0.0; 16_000];
        samples[123] = f32::NAN;
        assert!(matches!(
            check_audio(&samples),
            Err(SttError::NonFiniteSample { index: 123 })
        ));
    }

    #[test]
    fn markers_are_stripped() {
        assert_eq!(clean_text(" [BLANK_AUDIO]"), "");
        assert_eq!(clean_text("(music)"), "");
        assert_eq!(clean_text(" ♪ ♪ "), "");
        assert_eq!(
            clean_text("  Add a page [BLANK_AUDIO] called loans. "),
            "Add a page called loans."
        );
        assert_eq!(
            clean_text("(upbeat music) Rename it [Music] ."),
            "Rename it."
        );
        assert_eq!(
            clean_text(" Füge eine Spalte hinzu.\n Danke. "),
            "Füge eine Spalte hinzu. Danke."
        );
        assert_eq!(clean_text("keep this [unclosed"), "keep this");
    }

    #[test]
    fn prompt_is_sanitized() {
        assert_eq!(sanitize_prompt("  loans\0, members "), "loans, members");
        assert_eq!(sanitize_prompt("   "), "");
    }

    #[test]
    fn missing_model_is_a_typed_error() {
        let err = Transcriber::load(TranscriberConfig {
            model: PathBuf::from("/nonexistent/uilab-stt/ggml-none.bin"),
            language: None,
            gpu: false,
        })
        .unwrap_err();
        assert!(matches!(err, SttError::ModelNotFound(_)), "{err:?}");
    }

    #[test]
    fn language_codes_are_checked() {
        assert_eq!(normalize_language(None).unwrap(), None);
        assert_eq!(normalize_language(Some("auto")).unwrap(), None);
        assert_eq!(normalize_language(Some(" ")).unwrap(), None);
        assert_eq!(normalize_language(Some("EN")).unwrap(), Some("en".into()));
        assert_eq!(normalize_language(Some("de")).unwrap(), Some("de".into()));
        assert!(matches!(
            normalize_language(Some("klingon")),
            Err(SttError::UnknownLanguage(_))
        ));
    }

    /// Needs the real model: `UILAB_STT_MODEL=~/.cache/uilab/models/ggml-large-v3-turbo.bin
    /// cargo test -p uilab-stt -- --ignored`.
    #[test]
    #[ignore = "needs a whisper model file (set UILAB_STT_MODEL) and takes seconds to load"]
    fn silence_transcribes_to_nothing_useful() {
        let model = std::env::var_os("UILAB_STT_MODEL").expect("UILAB_STT_MODEL is set");
        let mut stt = Transcriber::load(TranscriberConfig {
            model: model.into(),
            language: Some("en".into()),
            gpu: cfg!(feature = "cuda"),
        })
        .unwrap();
        let t = stt.transcribe(&vec![0.0; 32_000], "").unwrap();
        assert_eq!(t.audio_ms, 2_000);
        assert!(!t.text.contains('['), "{:?}", t.text);
        assert!(matches!(
            stt.transcribe(&vec![0.0; 1_000], ""),
            Err(SttError::TooShort { .. })
        ));
    }
}
