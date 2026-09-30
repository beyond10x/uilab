//! Fixture rows per view, read from the files a document's `fixtures` index names.

use std::path::Path;

use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value;

use crate::model::{DRAFT_VIEW_PREFIX, Document};

/// Sample rows of one view.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ViewRows {
    /// Total the view reports, where the fixture states one.
    #[serde(default)]
    pub total: Option<u64>,
    /// The rows.
    #[serde(default)]
    pub rows: Vec<Value>,
}

/// A fixture file: one view, or several.
#[derive(Deserialize)]
#[serde(untagged)]
enum ViewFile {
    Many {
        views: IndexMap<String, ViewRows>,
    },
    One {
        view: String,
        #[serde(flatten)]
        rows: ViewRows,
    },
}

#[derive(Deserialize)]
struct IndexFile {
    #[serde(default)]
    views: IndexMap<String, String>,
}

/// Rows per view for one document.
#[derive(Debug, Clone, Default)]
pub struct Fixtures {
    views: IndexMap<String, ViewRows>,
}

/// A fixture file that cannot be read.
#[derive(Debug, thiserror::Error)]
#[error("fixture `{path}`: {message}")]
pub struct FixtureError {
    /// The file.
    pub path: String,
    /// What went wrong.
    pub message: String,
}

impl Fixtures {
    /// Reads every fixture the document's index names, relative to `doc_dir`.
    pub fn load(doc: &Document, doc_dir: &Path) -> Result<Self, FixtureError> {
        let Some(index) = &doc.fixtures else {
            return Ok(Self::default());
        };
        let dir = doc_dir.join(index.dir.as_deref().unwrap_or("."));
        let mut files = index.views.clone();
        if let Some(index_file) = &index.index {
            let path = doc_dir.join(index_file);
            let parsed: IndexFile = read_yaml(&path)?;
            let base = path.parent().unwrap_or(&dir).to_path_buf();
            for (view, file) in parsed.views {
                files
                    .entry(view)
                    .or_insert_with(|| base.join(file).to_string_lossy().into_owned());
            }
        }
        let mut views = IndexMap::new();
        let mut cache: IndexMap<String, IndexMap<String, ViewRows>> = IndexMap::new();
        for (view, file) in files {
            let path = dir.join(&file);
            let key = path.to_string_lossy().into_owned();
            if !cache.contains_key(&key) {
                let parsed: ViewFile = read_yaml(&path)?;
                let map = match parsed {
                    ViewFile::Many { views } => views,
                    ViewFile::One { view, rows } => IndexMap::from([(view, rows)]),
                };
                cache.insert(key.clone(), map);
            }
            let rows = cache[&key].get(&view).cloned().unwrap_or_default();
            views.insert(view, rows);
        }
        Ok(Fixtures { views })
    }

    /// The rows of a view: none for a `draft.` placeholder or a view without a fixture.
    pub fn rows(&self, view: &str) -> ViewRows {
        if view.starts_with(DRAFT_VIEW_PREFIX) {
            return ViewRows::default();
        }
        self.views.get(view).cloned().unwrap_or_default()
    }

    /// Whether the view has a fixture.
    pub fn has(&self, view: &str) -> bool {
        self.views.contains_key(view)
    }
}

fn read_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, FixtureError> {
    let err = |message: String| FixtureError {
        path: path.display().to_string(),
        message,
    };
    let text = std::fs::read_to_string(path).map_err(|e| err(e.to_string()))?;
    serde_yaml::from_str(&text).map_err(|e| err(e.to_string()))
}
