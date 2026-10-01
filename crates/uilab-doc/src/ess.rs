//! The bridge to ESS: what uilab derives from [`ess_ui::SCHEMA`], the mapping between uilab node
//! paths and ESS canonical paths, ESS's loader and ESS's checker run on a document.
//!
//! uilab keeps its own node paths (`page:loans/section:list`) as an editing layer. Each maps to
//! exactly one ESS canonical path (`pages/loans/sections/list`) and back ([`to_ess`],
//! [`from_ess`]); an ESS path below anything uilab addresses (a column, an action, a widget
//! instance's expanded body) is shown on the nearest uilab node that holds it ([`node_at`]).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

use serde_yaml::Value as Yaml;

use crate::check::{Finding, Severity};
use crate::model::Document;
use crate::path::{Layer, NodePath, resolve};

/// The schema, parsed once.
fn schema() -> &'static Yaml {
    static PARSED: OnceLock<Yaml> = OnceLock::new();
    PARSED.get_or_init(|| serde_yaml::from_str(ess_ui::SCHEMA).expect("the ESS schema is YAML"))
}

/// The construct a uilab layer's node is in the schema, for the layers whose children do not
/// depend on a composite kind.
fn construct_of(layer: Layer) -> Option<&'static str> {
    Some(match layer {
        Layer::Root => "Document",
        Layer::Shell => "Shell",
        Layer::Region => "Region",
        Layer::Nav => "Navigation",
        Layer::NavSection => "NavSection",
        Layer::Page => "Page",
        Layer::Component => "Widget",
        _ => return None,
    })
}

/// The layer uilab addresses the named nodes under `key` of `construct` with, when the schema
/// types that key as a list or a map of `element`. `None` for what uilab does not address (state,
/// guards, page kinds, fields, actions).
fn layer_of(construct: &str, key: &str, element: &str) -> Option<Layer> {
    Some(match (construct, key, element) {
        // `WidgetInstance.body` is written by expansion, never by an author.
        ("WidgetInstance", "body", _) => return None,
        ("Document", "shells", "Shell") => Layer::Shell,
        ("Document", "pages", "Page") => Layer::Page,
        ("Document", "widgets", "Widget") => Layer::Component,
        ("Shell", "regions", "Region") => Layer::Region,
        (_, "overlays", "overlay") => Layer::Overlay,
        ("Navigation", "sections", "NavSection") => Layer::NavSection,
        (_, "sections", "Section") => Layer::Section,
        ("Widget", "body", "Node") => Layer::Node,
        (_, "item", "Node") => Layer::Item,
        (_, "widgets", "Node") => Layer::Widget,
        (_, "children", "Node") => Layer::Child,
        (_, "parts", "Node") => Layer::Part,
        (_, "choices", "Node") => Layer::Choice,
        (_, "toolbar", "Node") => Layer::Tool,
        _ => return None,
    })
}

/// The layers of the named nodes a construct of the schema holds: every field typed as a list or
/// a map of a construct uilab addresses, in uilab's layer order.
fn node_layers(construct: &str) -> Vec<Layer> {
    static CACHE: OnceLock<std::sync::Mutex<BTreeMap<String, Vec<Layer>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(found) = cache.lock().expect("not poisoned").get(construct) {
        return found.clone();
    }
    let mut layers = Vec::new();
    if let Some(fields) = schema()["constructs"][construct]["fields"].as_mapping() {
        for (key, spec) in fields {
            let Some(key) = key.as_str() else { continue };
            let ty = &spec["type"];
            let element = ty
                .get("list")
                .or_else(|| ty.get("map").and_then(|map| map.get("value")))
                .map(|v| v.get("optional").unwrap_or(v))
                .and_then(Yaml::as_str);
            if let Some(layer) = element.and_then(|e| layer_of(construct, key, e)) {
                layers.push(layer);
            }
        }
    }
    layers.sort_by_key(|l| Layer::ALL.iter().position(|x| x == l));
    layers.dedup();
    cache
        .lock()
        .expect("not poisoned")
        .insert(construct.to_owned(), layers.clone());
    layers
}

/// What a node of `layer` holds, when that does not depend on a composite kind.
pub(crate) fn layers_under(layer: Layer) -> Vec<Layer> {
    construct_of(layer).map(node_layers).unwrap_or_default()
}

/// What a composite holds: its kind's named-node lists, or nothing for a widget instance; a
/// section adds the section's own (`children`).
pub(crate) fn layers_in_composite(kind: Option<&str>, section: bool) -> Vec<Layer> {
    let mut layers = node_layers(kind.unwrap_or("WidgetInstance"));
    if section {
        layers.extend(node_layers("Section"));
        layers.sort_by_key(|l| Layer::ALL.iter().position(|x| x == l));
        layers.dedup();
    }
    layers
}

/// The ESS container key a layer's nodes sit under.
fn container(layer: Layer) -> &'static str {
    match layer {
        Layer::Root => "",
        Layer::Shell => "shells",
        Layer::Region => "regions",
        Layer::Nav => "navigation",
        Layer::NavSection => "sections",
        Layer::Page => "pages",
        Layer::Section => "sections",
        Layer::Overlay => "overlays",
        Layer::Widget => "widgets",
        Layer::Item => "item",
        Layer::Child => "children",
        Layer::Part => "parts",
        Layer::Choice => "choices",
        Layer::Tool => "toolbar",
        Layer::Component => "widgets",
        Layer::Node => "body",
    }
}

/// The ESS canonical path of a uilab node.
pub fn to_ess(path: &NodePath) -> String {
    let mut segments: Vec<&str> = Vec::new();
    for segment in &path.0 {
        segments.push(container(segment.layer));
        if segment.layer != Layer::Nav {
            segments.push(&segment.name);
        }
    }
    if segments.is_empty() {
        "/".to_owned()
    } else {
        segments.join("/")
    }
}

/// The uilab layer an ESS container key holds under a node of `parent`'s layer.
fn layer_under(parent: Layer, key: &str) -> Option<Layer> {
    use Layer::*;
    Some(match (parent, key) {
        (Root, "shells") => Shell,
        (Root, "navigation") => Nav,
        (Root, "pages") => Page,
        (Root, "widgets") => Component,
        (Shell, "regions") => Region,
        (Shell, "overlays") => Overlay,
        (Nav, "sections") => NavSection,
        (Page, "sections") => Section,
        (Page, "overlays") => Overlay,
        (Component, "body") => Node,
        (Section | Overlay | Widget | Item | Child | Part | Choice | Tool | Node, key) => match key
        {
            "widgets" => Widget,
            "item" => Item,
            "children" if parent == Section => Child,
            "parts" => Part,
            "choices" => Choice,
            "toolbar" => Tool,
            _ => return None,
        },
        _ => return None,
    })
}

/// Reads an ESS path as far as it names uilab nodes: the deepest uilab path it starts with, and
/// how many ESS segments that took.
fn read_ess(text: &str) -> (NodePath, usize) {
    let segments: Vec<&str> = text
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let mut path = NodePath::root();
    let mut used = 0;
    while used < segments.len() {
        let Some(layer) = layer_under(path.layer(), segments[used]) else {
            break;
        };
        if layer == Layer::Nav {
            path = path.child(Layer::Nav, "");
            used += 1;
            continue;
        }
        let Some(name) = segments.get(used + 1) else {
            break;
        };
        path = path.child(layer, name);
        used += 2;
    }
    (path, used)
}

/// The uilab path of an ESS canonical path that names a uilab node exactly; `None` for a path
/// that ends below or beside every uilab node.
pub fn from_ess(text: &str) -> Option<NodePath> {
    let total = text
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .count();
    let (path, used) = read_ess(text);
    (used == total).then_some(path)
}

/// The uilab node an ESS path is about: the deepest node of `doc` the path passes through. An
/// inherited section, a column or an action is shown on the page, section or composite that holds
/// it.
pub fn node_at(doc: &Document, text: &str) -> NodePath {
    let (path, _) = read_ess(text);
    path.lineage()
        .into_iter()
        .rev()
        .find(|p| resolve(doc, p).is_ok())
        .unwrap_or_default()
}

/// A document ESS's loader refuses, with ESS's path and message.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{path}: {message}")]
pub struct LoadError {
    /// The ESS canonical path of the node at fault (`/` for the document).
    pub path: String,
    /// ESS's message.
    pub message: String,
}

impl LoadError {
    pub(crate) fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        LoadError {
            path: path.into(),
            message: message.into(),
        }
    }
}

/// ESS's loader on the text: the document loads, or ESS's refusal.
pub(crate) fn load(text: &str) -> Result<(), LoadError> {
    ess_ui::load_str(text)
        .map(|_| ())
        .map_err(|e| LoadError::new(e.path().to_string(), e.message()))
}

/// ESS's checker on a document, with fixture paths read relative to the document's directory.
pub fn report(doc: &Document) -> ess_ui_check::Report {
    let text = doc.to_yaml().unwrap_or_default();
    let base = doc.base().unwrap_or(Path::new("."));
    ess_ui_check::check_source(&text, "document", base, None, &Default::default())
}

/// ESS's findings on a document, each on the uilab node it is about. A finding below that node
/// names its ESS path first.
pub(crate) fn findings(doc: &Document) -> Vec<Finding> {
    report(doc)
        .findings
        .into_iter()
        .map(|f| {
            let at = node_at(doc, &f.path);
            // Below a widget instance, the finding is in its expansion: name the widget too.
            let widget = match crate::path::resolve(doc, &at) {
                Ok(crate::path::NodeRef::Composite(c)) => c.component.widget(),
                _ => None,
            };
            let message = match widget {
                _ if to_ess(&at) == f.path => f.message,
                Some(widget) => format!("widget `{widget}`, `{}`: {}", f.path, f.message),
                None => format!("`{}`: {}", f.path, f.message),
            };
            Finding {
                check: check_id(&f.check),
                severity: match f.severity {
                    ess_ui_check::Severity::Error => Severity::Error,
                    ess_ui_check::Severity::Warning => Severity::Warning,
                },
                path: at.to_string(),
                message,
            }
        })
        .collect()
}

/// ESS's own spelling of a check id, from its list.
fn check_id(id: &str) -> &'static str {
    ess_ui_check::CHECKS
        .iter()
        .find(|c| c.id == id)
        .map_or("document_loads", |c| c.id)
}
