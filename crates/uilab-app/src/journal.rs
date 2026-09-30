//! The session journal: one directory per `uilab serve`, holding `events.jsonl` (one JSON object
//! per line: what the operator did, what speech heard, what the agent proposed or why it was
//! refused, and how long each took) and every utterance as a WAV file, so a failure can be read
//! and a transcription replayed after the fact.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

pub struct Journal {
    dir: PathBuf,
    events: Mutex<File>,
    started: Instant,
    utterances: Mutex<u32>,
}

impl Journal {
    /// Creates `<root>/<unix seconds>/` and its `events.jsonl`.
    pub fn open(root: &Path) -> std::io::Result<Self> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let dir = root.join(stamp.to_string());
        std::fs::create_dir_all(&dir)?;
        let events = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("events.jsonl"))?;
        Ok(Journal {
            dir,
            events: Mutex::new(events),
            started: Instant::now(),
            utterances: Mutex::new(0),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Appends one event. A journal that cannot be written never stops the session.
    pub fn write(&self, event: &str, mut fields: Value) {
        if let Value::Object(map) = &mut fields {
            map.insert("event".into(), json!(event));
            map.insert(
                "at_ms".into(),
                json!(self.started.elapsed().as_millis() as u64),
            );
        }
        if let Ok(mut file) = self.events.lock() {
            let _ = writeln!(file, "{fields}");
        }
    }

    /// Writes one utterance as a 16 kHz mono float WAV and returns its file name.
    pub fn audio(&self, samples: &[f32]) -> Option<String> {
        let n = {
            let mut count = self.utterances.lock().ok()?;
            *count += 1;
            *count
        };
        let name = format!("utterance-{n:03}.wav");
        write_wav(&self.dir.join(&name), samples).ok()?;
        Some(name)
    }
}

fn write_wav(path: &Path, samples: &[f32]) -> std::io::Result<()> {
    const RATE: u32 = 16_000;
    let data = (samples.len() * 4) as u32;
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&3u16.to_le_bytes()); // IEEE float
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 4).to_le_bytes());
    out.extend_from_slice(&4u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    std::fs::write(path, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_and_audio_land_in_one_directory() {
        let root = std::env::temp_dir().join(format!("uilab-journal-test-{}", std::process::id()));
        let journal = Journal::open(&root).unwrap();
        journal.write("heard", json!({"text": "add a table"}));
        let wav = journal.audio(&[0.0, 0.5, -0.5]).unwrap();
        let line = std::fs::read_to_string(journal.dir().join("events.jsonl")).unwrap();
        let event: Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(event["event"], "heard");
        assert_eq!(event["text"], "add a table");
        let bytes = std::fs::read(journal.dir().join(&wav)).unwrap();
        assert_eq!(bytes.len(), 44 + 12);
        assert_eq!(&bytes[..4], b"RIFF");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
