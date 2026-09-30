//! The document checks: the `checks` of `ui-spec/1` that apply to this subset, plus the ones the
//! subset adds. Ids that exist in `ui-spec/1` keep its spelling.

use std::collections::HashSet;

use serde::Serialize;
use serde_json::Value;

use crate::model::{
    BUILTIN_PAGE_KINDS, Composite, CompositeKind, Document, FORMAT, NavPages, Node, Widget,
};
use crate::path::{Layer, NodePath};

/// How much a finding matters. An error refuses a patch that introduces it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The document is wrong.
    Error,
    /// The document is incomplete.
    Warning,
}

/// One check that does not hold, at one node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// Id of the check.
    pub check: &'static str,
    /// Error or warning.
    pub severity: Severity,
    /// The node it is about.
    pub path: String,
    /// What is wrong.
    pub message: String,
}

/// Every check id, with its severity and what it holds.
pub const CHECKS: [(&str, Severity, &str); 16] = [
    (
        "format_marker",
        Severity::Error,
        "the document says `format: ui-spec/1`",
    ),
    (
        "nav_resolves",
        Severity::Error,
        "the home page, every menu page and every hidden page exist",
    ),
    (
        "page_reachable",
        Severity::Error,
        "every page is in the menu or hidden",
    ),
    (
        "nav_unique",
        Severity::Error,
        "every page is listed once and menu section names are unique",
    ),
    ("shell_refs", Severity::Error, "a page's shell exists"),
    (
        "page_kind_known",
        Severity::Error,
        "a page's kind is built in or declared in page_kinds",
    ),
    (
        "page_outlet",
        Severity::Error,
        "a shell a page renders in has a page_outlet region",
    ),
    (
        "opens_resolves",
        Severity::Error,
        "`opens` names an overlay of the page or of its shell",
    ),
    (
        "section_refs",
        Severity::Error,
        "`depends_on` names a sibling section",
    ),
    (
        "fixture_per_view",
        Severity::Warning,
        "a read's view has a fixture",
    ),
    (
        "draft_read",
        Severity::Warning,
        "a read is a `draft.` placeholder with no model binding yet",
    ),
    (
        "unmapped_reported",
        Severity::Warning,
        "a value is an `UNMAPPED:` marker",
    ),
    (
        "widget_named_like_builtin",
        Severity::Error,
        "no widget is named like a built-in composite kind",
    ),
    (
        "widget_args",
        Severity::Error,
        "a widget instance supplies every required param and no arg the widget does not declare",
    ),
    (
        "widget_recursion",
        Severity::Error,
        "no widget contains itself, directly or through another widget",
    ),
    (
        "widget_resolves",
        Severity::Error,
        "a `component` that is not a composite kind names a declared widget",
    ),
];

fn severity_of(id: &str) -> Severity {
    CHECKS
        .iter()
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

/// Runs every check over the document.
pub fn check(doc: &Document) -> Vec<Finding> {
    let mut out = Findings(Vec::new());
    let root = NodePath::root();
    let nav = root.child(Layer::Nav, "");

    if doc.format != FORMAT {
        out.push(
            "format_marker",
            &root,
            format!("format is `{}`, not `{FORMAT}`", doc.format),
        );
    }

    // Navigation.
    if !doc.pages.contains_key(&doc.navigation.home) {
        out.push(
            "nav_resolves",
            &nav,
            format!("home page `{}` does not exist", doc.navigation.home),
        );
    }
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
            NavPages::Fixed(pages) => {
                for page in pages {
                    if !doc.pages.contains_key(page) {
                        out.push("nav_resolves", &at, format!("page `{page}` does not exist"));
                    }
                    listed.push(page);
                }
            }
            NavPages::Dynamic(entries) => {
                if let Some(page) = entries.get("page").and_then(Value::as_str) {
                    if !doc.pages.contains_key(page) {
                        out.push("nav_resolves", &at, format!("page `{page}` does not exist"));
                    }
                    listed.push(page);
                }
            }
        }
    }
    for page in &doc.navigation.hidden {
        if !doc.pages.contains_key(page) {
            out.push(
                "nav_resolves",
                &nav,
                format!("hidden page `{page}` does not exist"),
            );
        }
        listed.push(page);
    }
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

    // Shells.
    for (name, shell) in &doc.shells {
        let at = root.child(Layer::Shell, name);
        let overlays: Vec<&str> = shell.overlays.keys().map(String::as_str).collect();
        for (overlay_name, overlay) in &shell.overlays {
            let path = at.child(Layer::Overlay, overlay_name);
            check_composite(&mut out, &path, &overlay.body, &overlays, &[]);
        }
        for (region_name, region) in &shell.regions {
            let path = at.child(Layer::Region, region_name);
            for opened in opens(&serde_json::to_value(region).unwrap_or_default()) {
                if !overlays.contains(&opened.as_str()) {
                    out.push(
                        "opens_resolves",
                        &path,
                        format!("opens `{opened}`, which shell `{name}` does not declare"),
                    );
                }
            }
        }
    }

    // Pages.
    for (name, page) in &doc.pages {
        let at = root.child(Layer::Page, name);
        if !listed.contains(&name.as_str()) {
            out.push(
                "page_reachable",
                &at,
                format!("page `{name}` is neither in the menu nor hidden"),
            );
        }
        if !BUILTIN_PAGE_KINDS.contains(&page.kind.as_str())
            && !doc.page_kinds.contains_key(&page.kind)
        {
            out.push(
                "page_kind_known",
                &at,
                format!("page kind `{}` is not built in and not declared", page.kind),
            );
        }
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

        let mut reachable: Vec<&str> = page.overlays.keys().map(String::as_str).collect();
        if let Some((_, shell)) = shell {
            reachable.extend(shell.overlays.keys().map(String::as_str));
        }
        let siblings: Vec<&str> = page.sections.keys().map(String::as_str).collect();
        for opened in opens(&Value::Object(page.extra.clone().into_iter().collect())) {
            if !reachable.contains(&opened.as_str()) {
                out.push(
                    "opens_resolves",
                    &at,
                    format!("opens `{opened}`, which neither the page nor its shell declares"),
                );
            }
        }
        for (section_name, section) in &page.sections {
            if let Some(section) = section {
                check_composite(
                    &mut out,
                    &at.child(Layer::Section, section_name),
                    section,
                    &reachable,
                    &siblings,
                );
            }
        }
        for (overlay_name, overlay) in &page.overlays {
            if let Some(overlay) = overlay {
                check_composite(
                    &mut out,
                    &at.child(Layer::Overlay, overlay_name),
                    &overlay.body,
                    &reachable,
                    &siblings,
                );
            }
        }
    }

    check_widgets(&mut out, doc);

    // Fixtures.
    let fixtures: HashSet<&str> = doc
        .fixtures
        .iter()
        .flat_map(|f| f.views.keys().map(String::as_str))
        .collect();
    for (path, composite) in composites(doc) {
        if let Some(reads) = &composite.reads {
            if reads.is_draft() {
                out.push(
                    "draft_read",
                    &path,
                    format!(
                        "reads `{}`, a placeholder with no model binding yet",
                        reads.view
                    ),
                );
            } else if !fixtures.contains(reads.view.as_str()) {
                out.push(
                    "fixture_per_view",
                    &path,
                    format!("view `{}` has no fixture", reads.view),
                );
            }
        }
    }

    // Retrofit gaps.
    let whole = serde_json::to_value(doc).unwrap_or_default();
    let mut markers = Vec::new();
    unmapped(&whole, &mut markers);
    for marker in markers {
        out.push("unmapped_reported", &root, marker);
    }

    out.0
}

/// Widget declarations and every widget instance, wherever it sits.
fn check_widgets(out: &mut Findings, doc: &Document) {
    let root = NodePath::root();
    for name in doc.widgets.keys() {
        if CompositeKind::parse(name).is_some() {
            out.push(
                "widget_named_like_builtin",
                root.child(Layer::Component, name),
                format!(
                    "widget `{name}` is named like a built-in composite kind; `component: {name}` \
                     always means the built-in"
                ),
            );
        }
    }
    for (path, composite) in composites(doc) {
        for instance in instances_in(composite) {
            let name = instance.widget;
            let at = instance
                .trail
                .as_ref()
                .map_or(String::new(), |t| format!("`{t}`: "));
            let Some(widget) = doc.widgets.get(name) else {
                out.push(
                    "widget_resolves",
                    &path,
                    format!("{at}`{name}` is neither a composite kind nor a declared widget"),
                );
                continue;
            };
            check_args(out, &path, &at, name, widget, instance.args);
            if let Some(Layer::Component) = path.0.first().map(|s| s.layer) {
                let within = &path.0[0].name;
                if name == within || reaches(doc, name, within) {
                    out.push(
                        "widget_recursion",
                        &path,
                        format!("{at}widget `{within}` contains itself through `{name}`"),
                    );
                }
            }
        }
    }
}

/// A widget instance held by a composite: the composite itself, or one written in its untyped
/// props.
struct Instance<'a> {
    /// From the composite to the instance through its props (`children/badge`); `None` for the
    /// composite itself.
    trail: Option<String>,
    /// The widget it names.
    widget: &'a str,
    /// Its `args`.
    args: Option<&'a Value>,
}

/// The widget instances a composite holds without going through its typed `widgets` and `item`:
/// the composite itself when it is one, then every instance in its untyped props, however deep.
///
/// `ui-spec/1` expects a Node in `children`, `parts`, `expand`, `choices`, `metrics`, `toolbar`, a
/// field's `choice` and a tab's `form`; this subset keeps those as props. Any object there whose
/// `component` is not a composite kind is a widget instance. An `args` value is data, not nodes,
/// and is not searched.
fn instances_in(composite: &Composite) -> Vec<Instance<'_>> {
    fn walk<'a>(value: &'a Value, trail: String, out: &mut Vec<Instance<'a>>) {
        match value {
            Value::Object(map) => {
                if let Some(Value::String(name)) = map.get("component")
                    && CompositeKind::parse(name).is_none()
                {
                    out.push(Instance {
                        trail: Some(trail.clone()),
                        widget: name,
                        args: map.get("args"),
                    });
                }
                for (key, v) in map {
                    if key != "args" {
                        walk(v, format!("{trail}/{key}"), out);
                    }
                }
            }
            Value::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    let segment = item
                        .get("name")
                        .and_then(Value::as_str)
                        .map_or(i.to_string(), str::to_owned);
                    walk(item, format!("{trail}/{segment}"), out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    if let Some(name) = composite.component.widget() {
        out.push(Instance {
            trail: None,
            widget: name,
            args: composite.args(),
        });
    }
    for (key, value) in &composite.props {
        if key != "args" {
            walk(value, key.clone(), &mut out);
        }
    }
    out
}

fn check_args(
    out: &mut Findings,
    path: &NodePath,
    at: &str,
    name: &str,
    widget: &Widget,
    args: Option<&Value>,
) {
    let empty = serde_json::Map::new();
    let args = match args {
        None | Some(Value::Null) => &empty,
        Some(Value::Object(args)) => args,
        Some(_) => {
            out.push(
                "widget_args",
                path,
                format!("{at}the args of `{name}` are not a map of param to value"),
            );
            return;
        }
    };
    for arg in args.keys() {
        if !widget.params.contains_key(arg) {
            out.push(
                "widget_args",
                path,
                format!("{at}unknown arg `{arg}`: widget `{name}` declares no such param"),
            );
        }
    }
    for (param, declared) in &widget.params {
        if declared.is_required() && !args.contains_key(param) {
            out.push(
                "widget_args",
                path,
                format!("{at}missing required arg `{param}` of widget `{name}`"),
            );
        }
    }
}

/// The widgets a widget's body instantiates, however deep, typed or held in props.
fn uses(widget: &Widget) -> Vec<&str> {
    let mut stack: Vec<&Composite> = widget.body.iter().filter_map(Node::composite).collect();
    let mut out = Vec::new();
    while let Some(composite) = stack.pop() {
        out.extend(instances_in(composite).into_iter().map(|i| i.widget));
        stack.extend(composite.widgets.values());
        stack.extend(composite.item.values());
    }
    out
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
    fn walk<'a>(
        path: NodePath,
        composite: &'a Composite,
        out: &mut Vec<(NodePath, &'a Composite)>,
    ) {
        out.push((path.clone(), composite));
        for (name, widget) in &composite.widgets {
            walk(path.child(Layer::Widget, name), widget, out);
        }
        for (name, item) in &composite.item {
            walk(path.child(Layer::Item, name), item, out);
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
            if let Some(composite) = node.composite() {
                walk(at.child(Layer::Node, &node.name), composite, &mut out);
            }
        }
    }
    out
}

fn check_composite(
    out: &mut Findings,
    path: &NodePath,
    composite: &Composite,
    overlays: &[&str],
    siblings: &[&str],
) {
    let props = Value::Object(composite.props.clone().into_iter().collect());
    for opened in opens(&props) {
        if !overlays.contains(&opened.as_str()) {
            out.push(
                "opens_resolves",
                path,
                format!("opens `{opened}`, which neither the page nor its shell declares"),
            );
        }
    }
    if let Some(depends_on) = composite.props.get("depends_on").and_then(Value::as_str)
        && path.layer() == Layer::Section
        && (!siblings.contains(&depends_on) || depends_on == path.name())
    {
        out.push(
            "section_refs",
            path,
            format!("depends_on `{depends_on}`, which is not a sibling section"),
        );
    }
    for (name, widget) in &composite.widgets {
        check_composite(out, &path.child(Layer::Widget, name), widget, overlays, &[]);
    }
    for (name, item) in &composite.item {
        check_composite(out, &path.child(Layer::Item, name), item, overlays, &[]);
    }
}

/// Every `opens:` value under a JSON value, however deep.
fn opens(value: &Value) -> Vec<String> {
    let mut found = Vec::new();
    fn walk(value: &Value, found: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (key, v) in map {
                    match (key.as_str(), v) {
                        ("opens", Value::String(name)) => found.push(name.clone()),
                        _ => walk(v, found),
                    }
                }
            }
            Value::Array(items) => items.iter().for_each(|v| walk(v, found)),
            _ => {}
        }
    }
    walk(value, &mut found);
    found
}

fn unmapped(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) if s.starts_with("UNMAPPED: ") => out.push(s.clone()),
        Value::Object(map) => map.values().for_each(|v| unmapped(v, out)),
        Value::Array(items) => items.iter().for_each(|v| unmapped(v, out)),
        _ => {}
    }
}
