//! Fixture rows per view, read from the files a document's `fixtures` index names.

use std::path::Path;

use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value;

use crate::model::{CompositeKind, DRAFT_VIEW_PREFIX, Document};

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
        let Some(fields) = known.get(&reads.view).filter(|f| !f.is_empty()) else {
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
                    reads.view,
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

/// Made-up rows for a view that has no fixture, most often a `draft.` placeholder, so the canvas
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
        if composite.reads.as_ref().is_none_or(|r| r.view != view) {
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
