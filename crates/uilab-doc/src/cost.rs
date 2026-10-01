//! What ESS's loader would build when it expands a document, weighed without building it.
//!
//! ESS 0.48.0 expands a document in passes (`ess-ui` `expand.rs`): it merges each page's kind into
//! the page, resolves `same_as` overlays as copies, then expands every widget use under `shells`
//! and `pages` in full, substituting each `args.<param>` by the bound value, once per use
//! (beyond10x/ess#300). This module follows the same passes on the authored document and counts
//! the maps the expansion would hold: a widget is weighed once per distinct sizes of its
//! arguments, so the cost is computed in time linear in the document, not in the expansion.
//!
//! It is a fast first filter for [`crate::check::GUARDS`]' `expansion_bound`; the deadline around
//! every call into ESS ([`crate::ess`]) catches what it misses.

use std::collections::{BTreeMap, HashMap};

use serde_json::{Map, Value};

use crate::model::CompositeKind;

/// Keys whose values ESS treats as data and never expands (`expand.rs` `OPAQUE`).
const OPAQUE: &[&str] = &[
    "types",
    "type",
    "default",
    "props",
    "fixtures",
    "channels",
    "args",
    "params",
    "bind",
    "set",
    "sets",
    "place",
    "grid",
    "tone_by",
    "variant_by",
];

/// How deep `extends` and `same_as` chains are followed; ESS refuses cycles itself.
const DEPTH: usize = 32;

/// The weight of a document's widget expansion.
pub(crate) struct Cost {
    /// Maps the expansion of every widget use would hold, saturating.
    pub(crate) total: u64,
    /// The use that weighs most: its ESS path and weight.
    pub(crate) largest: Option<(String, u64)>,
}

/// Weighs the expansion of `raw`, an authored document as JSON.
pub(crate) fn expansion_cost(raw: &Value) -> Cost {
    let Some(doc) = raw.as_object() else {
        return Cost {
            total: 0,
            largest: None,
        };
    };
    let empty = Map::new();
    let kinds = doc
        .get("page_kinds")
        .and_then(Value::as_object)
        .unwrap_or(&empty);

    // 1. Page kinds merged into pages.
    let mut pages = Map::new();
    if let Some(Value::Object(authored)) = doc.get("pages") {
        for (name, page) in authored {
            let Value::Object(body) = page else { continue };
            let kind = body.get("kind").and_then(Value::as_str).unwrap_or("");
            let base = resolve_kind(kinds, kind, 0);
            pages.insert(name.clone(), Value::Object(merge_map(&base, body)));
        }
    }
    let mut shells = doc.get("shells").cloned().unwrap_or(Value::Null);

    // 2. `same_as` overlays become copies of what they name.
    let lookup = pages.clone();
    for page in pages.values_mut() {
        resolve_overlays(page, &lookup);
    }
    if let Value::Object(shells) = &mut shells {
        for shell in shells.values_mut() {
            resolve_overlays(shell, &lookup);
        }
    }

    // 3. Every widget use under `shells` and `pages`.
    let widgets = doc
        .get("widgets")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let mut walk = Walk {
        widgets,
        memo: HashMap::new(),
        stack: Vec::new(),
        total: 0,
        largest: None,
    };
    walk.uses(&shells, "shells");
    walk.uses(&Value::Object(pages), "pages");
    Cost {
        total: walk.total,
        largest: walk.largest,
    }
}

/// A page kind with everything it extends merged under it; a built-in kind contributes no widget
/// use, so it is empty here.
fn resolve_kind(kinds: &Map<String, Value>, name: &str, depth: usize) -> Map<String, Value> {
    let Some(Value::Object(kind)) = kinds.get(name) else {
        return Map::new();
    };
    if depth > DEPTH {
        return Map::new();
    }
    let parent = kind.get("extends").and_then(Value::as_str).unwrap_or("");
    let base = resolve_kind(kinds, parent, depth + 1);
    let mut own = kind.clone();
    own.remove("extends");
    merge_map(&base, &own)
}

/// `shorthands.inheritance.maps`: deep merge, the page wins, `null` removes what is inherited;
/// named lists merge by name ([`merge_named`]).
fn merge_map(base: &Map<String, Value>, over: &Map<String, Value>) -> Map<String, Value> {
    let mut out = base.clone();
    for (key, value) in over {
        let merged = match (out.get(key), value) {
            (_, Value::Null) => None,
            (Some(Value::Object(b)), Value::Object(o)) => Some(Value::Object(merge_map(b, o))),
            (Some(Value::Array(b)), Value::Array(o)) if named(b) || named(o) => {
                Some(Value::Array(merge_named(b, o)))
            }
            _ => Some(value.clone()),
        };
        match merged {
            Some(merged) => {
                out.insert(key.clone(), merged);
            }
            None => {
                out.remove(key);
            }
        }
    }
    out
}

fn name_of(entry: &Value) -> Option<&str> {
    entry.get("name").and_then(Value::as_str)
}

fn named(entries: &[Value]) -> bool {
    !entries.is_empty() && entries.iter().all(|e| name_of(e).is_some())
}

/// `shorthands.inheritance.named_lists`: matched by name, inherited entries in their order then
/// new ones; `{name, remove: true}` removes; an entry naming another `component` replaces.
fn merge_named(base: &[Value], over: &[Value]) -> Vec<Value> {
    let mut out: Vec<Value> = base.to_vec();
    for entry in over {
        let Some(name) = name_of(entry) else {
            out.push(entry.clone());
            continue;
        };
        let at = out.iter().position(|e| name_of(e) == Some(name));
        let removes = entry.get("remove") == Some(&Value::Bool(true));
        match (at, removes) {
            (Some(i), true) => {
                out.remove(i);
            }
            (None, true) => {}
            (Some(i), false) => {
                let replaces = ["component", "primitive"].iter().any(|k| {
                    entry
                        .get(*k)
                        .is_some_and(|v| out[i].get(*k).is_some_and(|w| w != v))
                });
                out[i] = match (&out[i], entry) {
                    (Value::Object(b), Value::Object(o)) if !replaces => {
                        Value::Object(merge_map(b, o))
                    }
                    _ => entry.clone(),
                };
            }
            (None, false) => out.push(entry.clone()),
        }
    }
    out
}

/// Replaces each `same_as` overlay under `holder.overlays` by the page overlay it names, with its
/// own keys merged over it.
fn resolve_overlays(holder: &mut Value, pages: &Map<String, Value>) {
    let Some(Value::Object(overlays)) = holder.get_mut("overlays") else {
        return;
    };
    for overlay in overlays.values_mut() {
        *overlay = resolve_same_as(overlay, pages, 0);
    }
}

fn resolve_same_as(overlay: &Value, pages: &Map<String, Value>, depth: usize) -> Value {
    let Value::Object(local) = overlay else {
        return overlay.clone();
    };
    let Some(target) = local.get("same_as").and_then(Value::as_str) else {
        return overlay.clone();
    };
    if depth > DEPTH {
        return overlay.clone();
    }
    let (page, name) = target.rsplit_once('.').unwrap_or((target, ""));
    let Some(found) = pages
        .get(page)
        .and_then(|p| p.get("overlays"))
        .and_then(|o| o.get(name))
    else {
        return overlay.clone();
    };
    let Value::Object(base) = resolve_same_as(found, pages, depth + 1) else {
        return overlay.clone();
    };
    let mut own = local.clone();
    own.remove("same_as");
    let mut base = base;
    base.remove("same_as");
    Value::Object(merge_map(&base, &own))
}

/// The maps a value holds: a map counts one and what it holds; a list what its entries hold.
fn size(value: &Value) -> u64 {
    match value {
        Value::Object(map) => map.values().fold(1, |n, v| n.saturating_add(size(v))),
        Value::Array(items) => items.iter().fold(0, |n, v| n.saturating_add(size(v))),
        _ => 0,
    }
}

/// The size of `value` once each whole-string `args.<param>` in it is replaced by the bound
/// value, whose size `bound` gives.
fn substituted(value: &Value, bound: &BTreeMap<String, u64>) -> u64 {
    match value {
        Value::String(text) => text
            .strip_prefix("args.")
            .and_then(|param| bound.get(param))
            .copied()
            .unwrap_or(0),
        Value::Object(map) => map
            .values()
            .fold(1, |n, v| n.saturating_add(substituted(v, bound))),
        Value::Array(items) => items
            .iter()
            .fold(0, |n, v| n.saturating_add(substituted(v, bound))),
        _ => 0,
    }
}

/// A widget named by a map's `component`: any name that is no composite kind.
fn widget_of(map: &Map<String, Value>) -> Option<&str> {
    map.get("component")
        .and_then(Value::as_str)
        .filter(|c| CompositeKind::parse(c).is_none())
}

struct Walk<'a> {
    widgets: &'a Map<String, Value>,
    memo: HashMap<(String, Vec<(String, u64)>), u64>,
    stack: Vec<String>,
    total: u64,
    largest: Option<(String, u64)>,
}

impl Walk<'_> {
    /// Adds every widget use under `value` (at ESS path `at`) to the total.
    fn uses(&mut self, value: &Value, at: &str) {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    if !OPAQUE.contains(&key.as_str()) {
                        self.uses(child, &format!("{at}/{key}"));
                    }
                }
                if let Some(widget) = widget_of(map) {
                    let args: BTreeMap<String, u64> = map
                        .get("args")
                        .and_then(Value::as_object)
                        .map(|args| args.iter().map(|(k, v)| (k.clone(), size(v))).collect())
                        .unwrap_or_default();
                    let weight = self.instance(widget, &args);
                    self.total = self.total.saturating_add(weight);
                    if self.largest.as_ref().is_none_or(|(_, w)| weight > *w) {
                        self.largest = Some((at.to_owned(), weight));
                    }
                }
            }
            Value::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    let step = name_of(item).map_or_else(|| i.to_string(), str::to_owned);
                    self.uses(item, &format!("{at}/{step}"));
                }
            }
            _ => {}
        }
    }

    /// The weight of one use of `widget` whose arguments weigh `args`: its body with the
    /// arguments substituted, and every widget use in that body.
    fn instance(&mut self, widget: &str, args: &BTreeMap<String, u64>) -> u64 {
        let Some(definition) = self.widgets.get(widget).and_then(Value::as_object) else {
            return 0;
        };
        if self.stack.iter().any(|w| w == widget) {
            return 0;
        }
        let mut bound = BTreeMap::new();
        if let Some(Value::Object(params)) = definition.get("params") {
            for (param, spec) in params {
                let weight = args
                    .get(param)
                    .copied()
                    .or_else(|| spec.get("default").map(size))
                    .unwrap_or(0);
                bound.insert(param.clone(), weight);
            }
        }
        let key = (
            widget.to_owned(),
            bound.iter().map(|(k, v)| (k.clone(), *v)).collect(),
        );
        if let Some(weight) = self.memo.get(&key) {
            return *weight;
        }
        let body = definition.get("body").cloned().unwrap_or(Value::Null);
        self.stack.push(widget.to_owned());
        let weight = substituted(&body, &bound).saturating_add(self.nested(&body, &bound));
        self.stack.pop();
        self.memo.insert(key, weight);
        weight
    }

    /// The weight of the widget uses inside a body whose arguments weigh `bound`.
    fn nested(&mut self, value: &Value, bound: &BTreeMap<String, u64>) -> u64 {
        match value {
            Value::Object(map) => {
                let mut weight = map
                    .iter()
                    .filter(|(k, _)| !OPAQUE.contains(&k.as_str()))
                    .fold(0u64, |n, (_, v)| n.saturating_add(self.nested(v, bound)));
                if let Some(widget) = widget_of(map) {
                    let args: BTreeMap<String, u64> = map
                        .get("args")
                        .and_then(Value::as_object)
                        .map(|args| {
                            args.iter()
                                .map(|(k, v)| (k.clone(), substituted(v, bound)))
                                .collect()
                        })
                        .unwrap_or_default();
                    weight = weight.saturating_add(self.instance(widget, &args));
                }
                weight
            }
            Value::Array(items) => items
                .iter()
                .fold(0, |n, v| n.saturating_add(self.nested(v, bound))),
            _ => 0,
        }
    }
}
