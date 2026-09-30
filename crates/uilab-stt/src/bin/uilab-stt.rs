//! Transcribe one WAV file with the uilab speech-to-text engine.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;
use hound::{SampleFormat, WavReader};
use uilab_stt::{Transcriber, TranscriberConfig, resample_to_16k};

/// Transcribe a 16-bit PCM or 32-bit float WAV file with whisper.
#[derive(Debug, Parser)]
#[command(name = "uilab-stt", version)]
struct Cli {
    /// Path to a ggml whisper model, e.g. ~/.cache/uilab/models/ggml-large-v3-turbo.bin.
    #[arg(long)]
    model: PathBuf,
    /// Spoken language code (en, de, ...); omit to detect it.
    #[arg(long)]
    language: Option<String>,
    /// Run whisper on the CPU instead of the GPU.
    #[arg(long)]
    cpu: bool,
    /// Vocabulary the speaker is likely to use, passed to whisper as the initial prompt.
    #[arg(long, default_value = "")]
    prompt: String,
    /// Print whisper.cpp's own log lines to stderr.
    #[arg(long, short)]
    verbose: bool,
    /// The WAV file to transcribe.
    file: PathBuf,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    if cli.verbose {
        let _ = log::set_logger(&STDERR_LOGGER);
        log::set_max_level(log::LevelFilter::Info);
    }
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("uilab-stt: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let samples = read_wav_16k_mono(&cli.file)?;

    let loading = Instant::now();
    let mut stt = Transcriber::load(TranscriberConfig {
        model: cli.model.clone(),
        language: cli.language.clone(),
        gpu: !cli.cpu,
    })?;
    eprintln!("load_ms={}", loading.elapsed().as_millis());

    let t = stt.transcribe(&samples, &cli.prompt)?;
    println!("{}", t.text);
    println!("audio_ms={} took_ms={}", t.audio_ms, t.took_ms);
    Ok(())
}

/// Read a WAV file, average its channels to mono and resample it to 16 kHz.
fn read_wav_16k_mono(path: &Path) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();
    let interleaved: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Float, 32) => reader.samples::<f32>().collect::<Result<_, _>>()?,
        (SampleFormat::Int, bits @ 8..=32) => {
            let scale = (1u64 << (bits - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / scale))
                .collect::<Result<_, _>>()?
        }
        (format, bits) => {
            return Err(format!("unsupported WAV sample format: {format:?} {bits}-bit").into());
        }
    };
    let channels = usize::from(spec.channels.max(1));
    let mono: Vec<f32> = interleaved
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect();
    Ok(resample_to_16k(&mono, spec.sample_rate))
}

struct StderrLogger;

static STDERR_LOGGER: StderrLogger = StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        eprintln!(
            "[{}] {}",
            record.level(),
            record.args().to_string().trim_end()
        );
    }

    fn flush(&self) {}
}
