//! Where documents are read from and saved to.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use uilab_doc::Document;

/// Reads and saves `ui-spec/1` documents by path.
pub trait DocumentSource {
    /// The document at `path`, or why it cannot be read as one.
    fn load(&self, path: &str) -> Result<Document, String>;

    /// Writes `doc` to `path`.
    fn save(&self, path: &str, doc: &Document) -> Result<(), String>;
}

/// Documents as YAML files on disk; a path is a file path.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileSource;

impl DocumentSource for FileSource {
    fn load(&self, path: &str) -> Result<Document, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        Document::from_yaml(&text).map_err(|e| format!("{path}: {e}"))
    }

    fn save(&self, path: &str, doc: &Document) -> Result<(), String> {
        let text = doc.to_yaml().map_err(|e| format!("{path}: {e}"))?;
        std::fs::write(path, text).map_err(|e| format!("{path}: {e}"))
    }
}

#[derive(Debug, Default)]
struct Files {
    files: HashMap<String, Document>,
    fallback: Option<Document>,
    saves: usize,
}

/// Documents held in memory, keyed by path.
///
/// A clone shares the same files, so a test can keep one and read what the behaviour saved.
#[derive(Debug, Clone, Default)]
pub struct MemorySource(Arc<Mutex<Files>>);

impl MemorySource {
    /// No files, and no fallback: every load fails.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file.
    pub fn with_file(self, path: &str, doc: Document) -> Self {
        self.lock().files.insert(path.to_owned(), doc);
        self
    }

    /// Loads `doc` for every path that has no file.
    pub fn with_fallback(self, doc: Document) -> Self {
        self.lock().fallback = Some(doc);
        self
    }

    /// The document last saved or added at `path`.
    pub fn file(&self, path: &str) -> Option<Document> {
        self.lock().files.get(path).cloned()
    }

    /// How many saves succeeded.
    pub fn saves(&self) -> usize {
        self.lock().saves
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Files> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl DocumentSource for MemorySource {
    fn load(&self, path: &str) -> Result<Document, String> {
        let files = self.lock();
        files
            .files
            .get(path)
            .or(files.fallback.as_ref())
            .cloned()
            .ok_or_else(|| format!("{path}: no such file"))
    }

    fn save(&self, path: &str, doc: &Document) -> Result<(), String> {
        let mut files = self.lock();
        files.files.insert(path.to_owned(), doc.clone());
        files.saves += 1;
        Ok(())
    }
}
