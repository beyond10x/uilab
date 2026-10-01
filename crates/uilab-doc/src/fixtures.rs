//! Fixture rows per view, read the way ESS reads them: the `fixtures` index (`dir`, the `index`
//! file and `views`), and each placeholder read's own `fixture` file.

use std::path::Path;

use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value;

use crate::model::{CompositeKind, Document};

/// Sample rows of one view.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ViewRows {
    /// Total the view reports, where the fixture states one.
    #[serde(default)]
    pub total: Option<u64>,
    /// The rows.
    #[serde(default)]
    pub rows: Vec<Value>,
    /// Whether the rows are made up ([`sample_rows`]) because no fixture answers the view.
    #[serde(skip)]
    pub sample: bool,
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
    /// Reads every fixture the document names, relative to `doc_dir`, as ESS does: the index file
    /// relative to the document, the files it and `views` name relative to `dir`, and each
    /// placeholder's `fixture` relative to the document. A placeholder whose file is missing has
    /// no fixture (its rows are made up); any other file that cannot be read is an error.
    pub fn load(doc: &Document, doc_dir: &Path) -> Result<Self, FixtureError> {
        let mut files: IndexMap<String, std::path::PathBuf> = IndexMap::new();
        if let Some(index) = &doc.fixtures {
            let dir = doc_dir.join(index.dir.as_deref().unwrap_or("."));
            for (view, file) in &index.views {
                files.insert(view.clone(), dir.join(file));
            }
            if let Some(index_file) = &index.index {
                let parsed: IndexFile = read_yaml(&doc_dir.join(index_file))?;
                for (view, file) in parsed.views {
                    files.entry(view).or_insert_with(|| dir.join(file));
                }
            }
        }
        for (_, composite) in crate::check::composites(doc) {
            if let Some(reads) = &composite.reads
                && let (Some(name), Some(file)) = (&reads.placeholder, &reads.fixture)
            {
                let path = doc_dir.join(file);
                if path.is_file() {
                    files.entry(name.clone()).or_insert(path);
                }
            }
        }
        let mut views = IndexMap::new();
        let mut cache: IndexMap<std::path::PathBuf, IndexMap<String, ViewRows>> = IndexMap::new();
        for (view, path) in files {
            if !cache.contains_key(&path) {
                let parsed: ViewFile = read_yaml(&path)?;
                let map = match parsed {
                    ViewFile::Many { views } => views,
                    ViewFile::One { view, rows } => IndexMap::from([(view, rows)]),
                };
                cache.insert(path.clone(), map);
            }
            let rows = cache[&path].get(&view).cloned().unwrap_or_default();
            views.insert(view, rows);
        }
        Ok(Fixtures { views })
    }

    /// The fixture rows of a view or placeholder: none for one without a fixture.
    pub fn rows(&self, view: &str) -> ViewRows {
        self.views.get(view).cloned().unwrap_or_default()
    }

    /// The rows a renderer shows for a view or placeholder of `doc`: its fixture's, or, where no
    /// fixture answers it, rows made up from what reads it ([`sample_rows`]), marked as samples.
    pub fn rows_for(&self, doc: &Document, view: &str) -> ViewRows {
        match self.views.get(view) {
            Some(rows) => rows.clone(),
            None => ViewRows {
                total: None,
                rows: sample_rows(doc, view),
                sample: true,
            },
        }
    }

    /// Whether the view has a fixture.
    pub fn has(&self, view: &str) -> bool {
        self.views.contains_key(view)
    }

    /// Every view with a fixture and the fields its rows carry, in first-seen order.
    pub fn fields(&self) -> IndexMap<String, Vec<String>> {
        self.views
            .iter()
            .map(|(view, rows)| {
                let mut fields: Vec<String> = Vec::new();
                for row in &rows.rows {
                    for key in row.as_object().into_iter().flat_map(|o| o.keys()) {
                        if !fields.contains(key) {
                            fields.push(key.clone());
                        }
                    }
                }
                (view.clone(), fields)
            })
            .collect()
    }
}

/// Columns, a metric's `from` and record fields that name a field the view's fixture rows do not
/// have. A warning: the fixture may be incomplete, but more often the field was guessed.
pub fn field_findings(doc: &Document, fixtures: &Fixtures) -> Vec<crate::Finding> {
    let known = fixtures.fields();
    let mut out = Vec::new();
    for (path, composite) in crate::check::composites(doc) {
        let Some(reads) = &composite.reads else {
            continue;
        };
        let Some(fields) = known.get(reads.name()).filter(|f| !f.is_empty()) else {
            continue;
        };
        let mut named: Vec<String> = Vec::new();
        for key in ["columns", "fields"] {
            for entry in composite
                .props
                .get(key)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                match entry {
                    Value::String(field) => named.push(field.clone()),
                    Value::Object(o) => {
                        named.extend(o.get("field").and_then(Value::as_str).map(str::to_owned))
                    }
                    _ => {}
                }
            }
        }
        named.extend(
            composite
                .props
                .get("from")
                .and_then(Value::as_str)
                .map(str::to_owned),
        );
        for field in named.into_iter().filter(|f| !fields.contains(f)) {
            out.push(crate::Finding {
                check: "column_fields",
                severity: crate::Severity::Warning,
                path: path.to_string(),
                message: format!(
                    "`{field}` is not a field of `{}` rows ({})",
                    reads.name(),
                    fields.join(", ")
                ),
            });
        }
    }
    out
}

fn read_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, FixtureError> {
    let err = |message: String| FixtureError {
        path: path.display().to_string(),
        message,
    };
    let text = std::fs::read_to_string(path).map_err(|e| err(e.to_string()))?;
    serde_yaml::from_str(&text).map_err(|e| err(e.to_string()))
}

/// Made-up rows for a view that has no fixture, most often a placeholder, so the canvas
/// shows the composites that read it with data of the right shape. The fields are the ones those
/// composites name (columns, fields, a metric's `from`, a chart's `x` and `series`). A field read
/// as a quantity (a metric's `from`, a chart's `series`) is a number whatever its name; any other
/// value is shaped by the field name. Never data anybody should read as real.
pub fn sample_rows(doc: &Document, view: &str) -> Vec<Value> {
    let mut fields: Vec<String> = Vec::new();
    let mut quantities: Vec<String> = Vec::new();
    let mut categories: Vec<String> = Vec::new();
    let add = |list: &mut Vec<String>, f: &str| {
        if !f.is_empty() && !list.iter().any(|x| x == f) {
            list.push(f.to_owned());
        }
    };
    for (_, composite) in crate::check::composites(doc) {
        if composite.reads.as_ref().is_none_or(|r| r.name() != view) {
            continue;
        }
        let kind = composite.component.kind();
        for key in ["columns", "fields", "series"] {
            let quantity = key == "series" && kind == Some(CompositeKind::Chart);
            for entry in composite
                .props
                .get(key)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let field = match entry {
                    Value::String(f) => f.as_str(),
                    Value::Object(o) => o.get("field").and_then(Value::as_str).unwrap_or(""),
                    _ => "",
                };
                add(&mut fields, field);
                if quantity {
                    add(&mut quantities, field);
                }
            }
        }
        for key in ["from", "x"] {
            let field = composite
                .props
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or("");
            add(&mut fields, field);
            if key == "from" && kind == Some(CompositeKind::Metric) {
                add(&mut quantities, field);
            }
            if key == "x" && kind == Some(CompositeKind::Chart) {
                add(&mut categories, field);
            }
        }
    }
    if fields.is_empty() {
        fields = fields_from_view_name(view);
    }
    let mut seen: Vec<Vec<Value>> = Vec::new();
    (1..=5)
        .map(|n| {
            Value::Object(
                fields
                    .iter()
                    .map(|f| {
                        let value = if quantities.contains(f) {
                            sample_number(n)
                        } else {
                            sample_value(f, n)
                        };
                        (f.clone(), value)
                    })
                    .collect(),
            )
        })
        // A chart draws one bar per category: a row whose `x` values repeat an earlier row's is
        // left out (a state field cycles through three values over five rows).
        .filter(|row| {
            let key: Vec<Value> = categories.iter().map(|c| row[c.as_str()].clone()).collect();
            if categories.is_empty() || !seen.contains(&key) {
                seen.push(key);
                true
            } else {
                false
            }
        })
        .collect()
}

fn sample_number(n: i64) -> Value {
    Value::from(n * 7 % 23 + 3)
}

/// A value shaped by the field's name. The last word names what the field is (`created_on`,
/// `loan_status`, `day_count`); a flag also shows in its first word (`is_member`, `show_badge`).
/// A word that marks a kind at one end of a name does not mark it at the other: `on_loan`,
/// `at_risk` and `due_amount` are not dates, `active_loans` is not a flag, `month_total` is not a
/// month.
fn sample_value(field: &str, n: i64) -> Value {
    let f = field.to_lowercase();
    let parts: Vec<&str> = f.split(['_', '-', ' ']).filter(|p| !p.is_empty()).collect();
    let first = parts.first().copied().unwrap_or("");
    let last = parts.last().copied().unwrap_or("");
    let has = |words: &[&str]| words.iter().any(|w| parts.contains(w));
    let counted = has(&[
        "count", "total", "number", "amount", "value", "sum", "loans", "overdue", "qty", "score",
    ]);
    if ["is", "has", "can", "allow", "show", "notify"].contains(&first)
        || [
            "enabled", "active", "allowed", "visible", "verified", "notify",
        ]
        .contains(&last)
    {
        Value::Bool(n % 2 == 1)
    } else if last == "month" {
        Value::String(format!("2026-{:02}", n + 4))
    } else if last == "week" || (first == "week" && !counted) {
        Value::String(format!("2026-W{}", 30 + n))
    } else if ["date", "due", "joined", "at", "day", "time", "on"].contains(&last)
        || first == "date"
    {
        Value::String(format!("2026-10-{:02}", n * 3))
    } else if counted {
        sample_number(n)
    } else if ["state", "status", "standing", "stage"].contains(&last) {
        Value::String(["open", "overdue", "closed"][(n % 3) as usize].to_owned())
    } else if has(&["email"]) {
        Value::String(format!("person{n}@example.com"))
    } else {
        Value::String(format!("{} {n}", field.replace('_', " ")))
    }
}

/// Fields guessed from a view's own name when nothing reading it names any: `LoansPerMonth` gives
/// a `month` label and a `loans` count; anything else a `name` and a `value`.
fn fields_from_view_name(view: &str) -> Vec<String> {
    let name = view.rsplit('.').next().unwrap_or(view);
    let mut words: Vec<String> = Vec::new();
    for c in name.chars() {
        if c.is_uppercase() || words.is_empty() {
            words.push(String::new());
        }
        words
            .last_mut()
            .expect("a word was pushed")
            .push(c.to_ascii_lowercase());
    }
    match words.iter().position(|w| w == "per") {
        Some(i) if i > 0 && i + 1 < words.len() => {
            vec![words[i + 1..].join("_"), words[..i].join("_")]
        }
        _ => vec!["name".into(), "value".into()],
    }
}
