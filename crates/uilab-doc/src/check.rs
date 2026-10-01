//! The document checks: ESS's, and the few uilab still runs because ESS has no counterpart.
//!
//! [`check`] runs ESS's checker (`ess-ui-check`) on the document and shows each finding on the
//! uilab node it is about, then runs uilab's own checks ([`own`]). Each of uilab's checks is one
//! ESS does not make (story:essui-document `## Survivors`); the rest were ESS's all along and are
//! ESS's now. Ids are ESS's where the finding is ESS's.

use std::collections::HashSet;

use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value;

use crate::model::{Composite, CompositeKind, Document, NavPages, Node, NodeBody, Widget};
use crate::path::{Layer, NodePath, NodeRef, children, node_list, resolve};

/// How much a finding matters. An error refuses a patch that introduces it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The document is wrong.
    Error,
    /// The document is incomplete.
    Warning,
}

/// One check that does not hold, at one node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Finding {
    /// Id of the check: ESS's, or one of [`CHECKS`].
    pub check: &'static str,
    /// Error or warning.
    pub severity: Severity,
    /// The uilab node it is about.
    pub path: String,
    /// What is wrong.
    pub message: String,
}

/// Every check uilab runs itself, with its severity and what it holds: the checks ESS 0.48.0 does
/// not make (each filed in ESS when it is a fact about the format), and `replace_drops`, a rule
/// about a patch rather than a document.
pub const CHECKS: [(&str, Severity, &str); 4] = [
    (
        "nav_unique",
        Severity::Error,
        "every page is listed once and menu section names are unique",
    ),
    ("shell_refs", Severity::Error, "a page's shell exists"),
    (
        "page_outlet",
        Severity::Error,
        "a shell a page renders in has a page_outlet region",
    ),
    (
        "replace_drops",
        Severity::Warning,
        "patch-only, reported by `admit` and never by `check`: a replace keeps every child and \
         every `columns`, `fields`, `row_actions` and `actions` entry the node had",
    ),
];

/// The list props whose entries a replace is checked for dropping.
const LIST_PROPS: [&str; 4] = ["columns", "fields", "row_actions", "actions"];

/// The `replace_drops` findings of a replace at `target`: every child of the node, however deep
/// under children that stay, and every entry of its list props that `before` has and `after`
/// does not. One finding per node that loses something, at that node.
pub(crate) fn replace_drops(
    before: &Document,
    after: &Document,
    target: &NodePath,
) -> Vec<Finding> {
    let mut out = Findings(Vec::new());
    drops_at(&mut out, before, after, target);
    out.0
}

fn drops_at(out: &mut Findings, before: &Document, after: &Document, at: &NodePath) {
    let (Ok(old), Ok(new)) = (resolve(before, at), resolve(after, at)) else {
        return;
    };
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    let mut add = |what: &str, name: String| match groups.iter_mut().find(|(w, _)| w == what) {
        Some((_, names)) => names.push(name),
        None => groups.push((what.to_owned(), vec![name])),
    };

    let (stayed, gone) = compare(
        children(before, at).unwrap_or_default(),
        children(after, at).unwrap_or_default(),
    );
    for (layer, name) in gone {
        add(layer.as_str(), name);
    }
    let mut stayed: Vec<NodePath> = stayed
        .into_iter()
        .map(|(layer, name)| at.child(layer, &name))
        .collect();
    stayed.dedup();
    if let (NodeRef::NavSection(old), NodeRef::NavSection(new)) = (old, new)
        && let (NavPages::Fixed(old), NavPages::Fixed(new)) = (&old.pages, &new.pages)
    {
        for page in compare(old.clone(), new.clone()).1 {
            add("page", page);
        }
    }
    if let (Some(old), Some(new)) = (old.composite(), new.composite()) {
        for key in LIST_PROPS {
            let (_, gone) = compare(
                entries(old, key).map(identity).collect(),
                entries(new, key).map(identity).collect(),
            );
            for id in gone {
                add(key, id);
            }
        }
    }

    if !groups.is_empty() {
        let what = groups
            .iter()
            .map(|(what, names)| format!("{what} {}", names.join(", ")))
            .collect::<Vec<_>>()
            .join("; ");
        out.push("replace_drops", at, format!("replace at {at} drops {what}"));
    }
    for child in stayed {
        drops_at(out, before, after, &child);
    }
}

/// The entries of a composite's list prop; nothing when it is absent or not a list.
fn entries<'a>(composite: &'a Composite, key: &str) -> impl Iterator<Item = &'a Value> {
    composite
        .props
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// Matches `old` against `new` as multisets, in `old`'s order: each item of `new` answers for one
/// item of `old` only. Returns the items of `old` that are matched and those that are not.
fn compare<T: PartialEq>(old: Vec<T>, mut new: Vec<T>) -> (Vec<T>, Vec<T>) {
    let mut kept = Vec::new();
    let mut gone = Vec::new();
    for item in old {
        match new.iter().position(|n| *n == item) {
            Some(i) => {
                new.swap_remove(i);
                kept.push(item);
            }
            None => gone.push(item),
        }
    }
    (kept, gone)
}

/// What an entry of a list prop is known by: its `field`, else its `name`, else its `label`,
/// else the whole value (a string as written, anything else as JSON). Only the value counts, not
/// the key it came from, so `due` and `{field: due}` are the same field, as the renderer and
/// [`crate::fixtures`] read them.
fn identity(entry: &Value) -> String {
    ["field", "name", "label"]
        .into_iter()
        .find_map(|k| entry.get(k).and_then(Value::as_str).map(str::to_owned))
        .unwrap_or_else(|| match entry {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        })
}

fn severity_of(id: &str) -> Severity {
    CHECKS
        .iter()
        .chain(&GUARDS)
        .find(|(c, _, _)| *c == id)
        .map_or(Severity::Error, |(_, s, _)| *s)
}

struct Findings(Vec<Finding>);

impl Findings {
    fn push(&mut self, check: &'static str, path: impl ToString, message: impl Into<String>) {
        self.0.push(Finding {
            check,
            severity: severity_of(check),
            path: path.to_string(),
            message: message.into(),
        });
    }
}

/// Every finding on the document: ESS's, each on the uilab node it is about, then uilab's own.
/// A document over [`EXPANSION_LIMIT`] is not handed to ESS: its one finding is `expansion_bound`.
pub fn check(doc: &Document) -> Vec<Finding> {
    if let Some(over) = expansion(doc) {
        return vec![over];
    }
    let mut out = crate::ess::findings(doc);
    out.extend(own(doc));
    out
}

/// The checks uilab runs itself ([`CHECKS`] but `replace_drops`), without ESS's.
pub fn own(doc: &Document) -> Vec<Finding> {
    let mut out = Findings(Vec::new());
    let root = NodePath::root();
    let nav = root.child(Layer::Nav, "");

    // Every page listed once, every menu section name once.
    let mut listed: Vec<&str> = Vec::new();
    let mut section_names = HashSet::new();
    for section in &doc.navigation.sections {
        let at = nav.child(Layer::NavSection, &section.name);
        if !section_names.insert(section.name.as_str()) {
            out.push(
                "nav_unique",
                &at,
                format!("menu section `{}` is declared twice", section.name),
            );
        }
        match &section.pages {
            NavPages::Fixed(pages) => listed.extend(pages.iter().map(String::as_str)),
            NavPages::Dynamic(entries) => {
                listed.extend(entries.get("page").and_then(Value::as_str));
            }
        }
    }
    listed.extend(doc.navigation.hidden.iter().map(String::as_str));
    let mut seen = HashSet::new();
    for page in &listed {
        if !seen.insert(*page) {
            out.push(
                "nav_unique",
                &nav,
                format!("page `{page}` is listed more than once"),
            );
        }
    }

    // Each page's shell exists and has an outlet to render it in.
    for (name, page) in &doc.pages {
        let at = root.child(Layer::Page, name);
        let shell = doc
            .shell_of(page)
            .and_then(|s| doc.shells.get(s).map(|shell| (s, shell)));
        match (&page.shell, shell) {
            (Some(named), None) => {
                out.push("shell_refs", &at, format!("shell `{named}` does not exist"))
            }
            (_, Some((shell_name, shell))) => {
                if !shell
                    .regions
                    .values()
                    .any(|r| r.kind == crate::model::RegionKind::PageOutlet)
                {
                    out.push(
                        "page_outlet",
                        &at,
                        format!("shell `{shell_name}` has no page_outlet region to render it in"),
                    );
                }
            }
            (None, None) => out.push("shell_refs", &at, "the document declares no shell"),
        }
    }
    out.0
}

/// A widget instance: a typed composite, or one written in untyped data.
pub(crate) struct Instance<'a> {
    /// From the node that holds it to the instance through untyped data (`children/badge`,
    /// `header/metrics/due`); `None` for a typed composite, which is the node itself.
    pub(crate) trail: Option<String>,
    /// The widget it names.
    pub(crate) widget: &'a str,
}

/// Every widget instance of the document with the path of the node that holds it, in document
/// order: the use sites the outline and the docs list.
///
/// It covers every place `ess-ui/1` puts a Node:
/// - each composite [`composites`] lists (sections, overlays, board widgets, listed nodes,
///   widget-body composites), itself and its untyped props (see [`instances_in`]);
/// - each primitive of a list or a widget body, its props (an action's `choice`), held at the
///   primitive;
/// - a page's untyped `header` (`metrics`, an action's `choice`), held at the page;
/// - every page kind in `page_kinds` (its `sections` and `header`), held at the root with a trail
///   from `page_kinds/<kind>`.
///
/// Shells and regions hold no Node outside their typed overlays.
pub(crate) fn widget_uses(doc: &Document) -> Vec<(NodePath, Instance<'_>)> {
    let mut out = Vec::new();
    for (path, node) in nodes(doc) {
        let found = match node {
            NodeRef::Composite(composite) => instances_in(composite),
            NodeRef::Primitive(primitive) => instances_in_props(&primitive.props),
            _ => Vec::new(),
        };
        out.extend(found.into_iter().map(|i| (path.clone(), i)));
    }
    let root = NodePath::root();
    for (name, page) in &doc.pages {
        if let Some(header) = page.extra.get("header") {
            let mut found = Vec::new();
            walk_value(header, "header".to_owned(), &mut found);
            let at = root.child(Layer::Page, name);
            out.extend(found.into_iter().map(|i| (at.clone(), i)));
        }
    }
    for (kind, value) in &doc.page_kinds {
        let mut found = Vec::new();
        walk_value(value, format!("page_kinds/{kind}"), &mut found);
        out.extend(found.into_iter().map(|i| (root.clone(), i)));
    }
    out
}

/// The widget instances a composite holds without going through its typed lists: the composite
/// itself when it is one, then every instance in its untyped props, however deep.
fn instances_in(composite: &Composite) -> Vec<Instance<'_>> {
    let mut out = Vec::new();
    if let Some(name) = composite.component.widget() {
        out.push(Instance {
            trail: None,
            widget: name,
        });
    }
    out.extend(instances_in_props(&composite.props));
    out
}

/// Every widget instance in a node's untyped props, however deep; `args` is data and is skipped.
fn instances_in_props(props: &IndexMap<String, Value>) -> Vec<Instance<'_>> {
    let mut out = Vec::new();
    for (key, value) in props {
        if key != "args" {
            walk_value(value, key.clone(), &mut out);
        }
    }
    out
}

/// The widget an untyped object instantiates: its `component`, when that is not a composite kind.
fn widget_named(value: &Value) -> Option<&str> {
    value
        .get("component")
        .and_then(Value::as_str)
        .filter(|name| CompositeKind::parse(name).is_none())
}

/// Every widget instance under untyped data, however deep: any object whose `component` is not a
/// composite kind. An `args` value is data, not nodes, and is not searched. A list entry is named
/// by its `name`, else by its index.
fn walk_value<'a>(value: &'a Value, trail: String, out: &mut Vec<Instance<'a>>) {
    match value {
        Value::Object(map) => {
            if let Some(name) = widget_named(value) {
                out.push(Instance {
                    trail: Some(trail.clone()),
                    widget: name,
                });
            }
            for (key, v) in map {
                if key != "args" {
                    walk_value(v, format!("{trail}/{key}"), out);
                }
            }
        }
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                let step = match item.get("name").and_then(Value::as_str) {
                    Some(name) => name.to_owned(),
                    None => i.to_string(),
                };
                walk_value(item, format!("{trail}/{step}"), out);
            }
        }
        _ => {}
    }
}

/// The widgets a widget's body instantiates, however deep, typed or held in the props of a
/// composite or a primitive.
fn uses(widget: &Widget) -> Vec<&str> {
    weigh(widget).1
}

/// The nodes a widget's body writes itself, and the widgets it instantiates (see [`uses`]).
fn weigh(widget: &Widget) -> (u64, Vec<&str>) {
    let mut stack: Vec<&Node> = widget.body.iter().collect();
    let mut out = Vec::new();
    let mut nodes = 0u64;
    while let Some(node) = stack.pop() {
        nodes += 1;
        let composite = match &node.body {
            NodeBody::Primitive(p) => {
                out.extend(instances_in_props(&p.props).into_iter().map(|i| i.widget));
                continue;
            }
            NodeBody::Composite(c) => c.as_ref(),
        };
        let mut composites = vec![composite];
        while let Some(composite) = composites.pop() {
            out.extend(instances_in(composite).into_iter().map(|i| i.widget));
            nodes += composite.widgets.len() as u64;
            composites.extend(composite.widgets.values());
            for layer in Layer::ALL.into_iter().filter(|l| l.is_node_list()) {
                stack.extend(node_list(composite, layer).into_iter().flatten());
            }
        }
    }
    (nodes, out)
}

/// The guards uilab runs before it hands a document to ESS: limits on work, not rules of the
/// format, so neither a check of ESS's nor one of [`CHECKS`]. `expansion_bound` stands until ESS
/// expands a widget once rather than at every use (beyond10x/ess#300).
pub const GUARDS: [(&str, Severity, &str); 1] = [(
    "expansion_bound",
    Severity::Error,
    "the widget uses of a document expand to at most EXPANSION_LIMIT nodes: ESS expands every use \
     in full, so a widget that uses another twice at each of n levels costs 2^n nodes",
)];

/// The most nodes the widget uses of a document may expand to ([`GUARDS`]).
pub const EXPANSION_LIMIT: u64 = 100_000;

/// The `expansion_bound` finding of a document whose widget uses expand to more than
/// [`EXPANSION_LIMIT`] nodes, at the use site that expands to the most; `None` within the limit.
/// Each widget is weighed once, so this costs one pass over the declarations and the uses.
pub(crate) fn expansion(doc: &Document) -> Option<Finding> {
    fn size<'a>(
        doc: &'a Document,
        name: &'a str,
        memo: &mut std::collections::HashMap<&'a str, u64>,
    ) -> u64 {
        if let Some(n) = memo.get(name) {
            return *n;
        }
        // A widget that contains itself is ESS's to refuse (`widget_expands`); weigh it as empty.
        memo.insert(name, 0);
        let n = doc.widgets.get(name).map_or(0, |widget| {
            let (own, used) = weigh(widget);
            used.into_iter()
                .fold(own, |n, u| n.saturating_add(size(doc, u, memo)))
        });
        memo.insert(name, n);
        n
    }
    let mut memo = std::collections::HashMap::new();
    let mut total = 0u64;
    let mut largest: Option<(NodePath, u64)> = None;
    // A use inside a widget body is weighed with that widget; only uses outside every body expand.
    let outside = widget_uses(doc)
        .into_iter()
        .filter(|(at, _)| at.0.first().is_none_or(|s| s.layer != Layer::Component));
    for (at, instance) in outside {
        let n = size(doc, instance.widget, &mut memo);
        total = total.saturating_add(n);
        if largest.as_ref().is_none_or(|(_, m)| n > *m) {
            largest = Some((at, n));
        }
    }
    let (at, n) = largest?;
    (total > EXPANSION_LIMIT).then(|| Finding {
        check: GUARDS[0].0,
        severity: GUARDS[0].1,
        path: at.to_string(),
        message: format!(
            "the document's widget uses expand to {} nodes, more than the {EXPANSION_LIMIT} uilab \
             hands to ESS; this use alone expands to {n}",
            if total == u64::MAX {
                "over 2^64".to_owned()
            } else {
                total.to_string()
            },
        ),
    })
}

/// Whether widget `from` contains widget `to`, directly or through other widgets.
pub(crate) fn reaches(doc: &Document, from: &str, to: &str) -> bool {
    let mut seen = HashSet::new();
    let mut stack = vec![from];
    while let Some(name) = stack.pop() {
        if !seen.insert(name) {
            continue;
        }
        let Some(widget) = doc.widgets.get(name) else {
            continue;
        };
        for used in uses(widget) {
            if used == to {
                return true;
            }
            stack.push(used);
        }
    }
    false
}

/// Every composite of the document with its path, in document order: sections, overlays and what
/// nests in them, then the composites and widget instances of each widget body.
pub fn composites(doc: &Document) -> Vec<(NodePath, &Composite)> {
    nodes(doc)
        .into_iter()
        .filter_map(|(path, node)| match node {
            NodeRef::Composite(c) => Some((path, c)),
            _ => None,
        })
        .collect()
}

/// Every composite and every primitive of the document with its path, in document order: the
/// order of [`composites`], each primitive of a list or a widget body in its place.
pub(crate) fn nodes(doc: &Document) -> Vec<(NodePath, NodeRef<'_>)> {
    fn walk<'a>(path: NodePath, composite: &'a Composite, out: &mut Vec<(NodePath, NodeRef<'a>)>) {
        out.push((path.clone(), NodeRef::Composite(composite)));
        for (name, widget) in &composite.widgets {
            walk(path.child(Layer::Widget, name), widget, out);
        }
        for layer in Layer::ALL.into_iter().filter(|l| l.is_node_list()) {
            for node in node_list(composite, layer).into_iter().flatten() {
                walk_node(path.child(layer, &node.name), node, out);
            }
        }
    }
    fn walk_node<'a>(path: NodePath, node: &'a Node, out: &mut Vec<(NodePath, NodeRef<'a>)>) {
        match &node.body {
            NodeBody::Composite(c) => walk(path, c, out),
            NodeBody::Primitive(p) => out.push((path, NodeRef::Primitive(p))),
        }
    }
    let root = NodePath::root();
    let mut out = Vec::new();
    for (name, shell) in &doc.shells {
        for (overlay_name, overlay) in &shell.overlays {
            walk(
                root.child(Layer::Shell, name)
                    .child(Layer::Overlay, overlay_name),
                &overlay.body,
                &mut out,
            );
        }
    }
    for (name, page) in &doc.pages {
        let at = root.child(Layer::Page, name);
        for (section_name, section) in &page.sections {
            if let Some(section) = section {
                walk(at.child(Layer::Section, section_name), section, &mut out);
            }
        }
        for (overlay_name, overlay) in &page.overlays {
            if let Some(overlay) = overlay {
                walk(
                    at.child(Layer::Overlay, overlay_name),
                    &overlay.body,
                    &mut out,
                );
            }
        }
    }
    for (name, widget) in &doc.widgets {
        let at = root.child(Layer::Component, name);
        for node in &widget.body {
            walk_node(at.child(Layer::Node, &node.name), node, &mut out);
        }
    }
    out
}
