//! The document checks: the `checks` of `ui-spec/1` that apply to this subset, plus the ones the
//! subset adds. Ids that exist in `ui-spec/1` keep its spelling.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value;

use crate::model::{
    BUILTIN_PAGE_KINDS, Composite, CompositeKind, Document, FORMAT, NavPages, Node, NodeBody,
    Widget,
};
use crate::path::{Layer, NodePath, NodeRef, children, resolve};

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
pub const CHECKS: [(&str, Severity, &str); 18] = [
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
    (
        "names_unique",
        Severity::Error,
        "no two nodes of an `item` list share a name",
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
    widget_opens(&mut out, doc);

    // Item lists: a node is addressed by its name, so two siblings cannot share one.
    for (path, composite) in composites(doc) {
        for (i, node) in composite.item.iter().enumerate() {
            if composite.item[..i].iter().any(|n| n.name == node.name) {
                out.push(
                    "names_unique",
                    path.child(Layer::Item, &node.name),
                    format!("two nodes of the `item` list are named `{}`", node.name),
                );
            }
        }
    }

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
    for (path, instance) in widget_uses(doc) {
        let name = instance.widget;
        let at = instance
            .key
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

/// `opens_resolves` inside widget bodies, as ess WidgetInstance expansion says: at each use site
/// [`widget_uses`] finds on a page or in a shell overlay, every `opens` of the used widget's body,
/// through nested widgets, against the overlays of that page and its shell (or of that shell),
/// at the instance's own path, a [`NodePath`] that selects it; `<instance path>/body/<node>` is not
/// one, so the message names the body trail, and the instance's trail through untyped data when it
/// has one. `args.<param>` is substituted first (see [`Opened::at`]). A use in another widget's
/// body counts only through that widget's own uses, and a use in `page_kinds` sits on no page, so
/// neither reports by itself.
fn widget_opens(out: &mut Findings, doc: &Document) {
    let mut expander = Expander::new(doc);
    for (path, instance) in widget_uses(doc) {
        let Some((reachable, declarer)) = overlays_at(doc, &path) else {
            continue;
        };
        let widget = doc.widgets.get(instance.widget);
        let expanded = expander.body(instance.widget);
        let at = match &instance.key {
            Some(key) => format!(" at `{key}`"),
            None => String::new(),
        };
        for (node, opened) in expanded.iter() {
            let Some(opened) = opened.at(widget, instance.args) else {
                continue;
            };
            if !reachable.contains(&opened.as_str()) {
                out.push(
                    "opens_resolves",
                    &path,
                    format!(
                        "widget `{}`{at}, body `body/{node}`, opens `{opened}`, which {declarer}",
                        instance.widget
                    ),
                );
            }
        }
    }
}

/// An `opens` in a widget body: an overlay as written, or `args.<param>`, which each instance binds.
#[derive(Debug, Clone)]
enum Opened {
    Overlay(String),
    Arg(String),
}

impl Opened {
    /// What a body's `opens` names; `None` for `args.<param>.<field>`, which no static check can
    /// judge.
    fn written(text: String) -> Option<Self> {
        match text.strip_prefix("args.") {
            None => Some(Opened::Overlay(text)),
            Some(param) if !param.is_empty() && !param.contains('.') => {
                Some(Opened::Arg(param.to_owned()))
            }
            Some(_) => None,
        }
    }

    /// The overlay it names at a use of `widget` on a page or shell: [`Opened::bind`], where an
    /// `args.<param>` still left has no holder to bind it.
    fn at(&self, widget: Option<&Widget>, args: Option<&Value>) -> Option<String> {
        match self.bind(widget, args)? {
            Opened::Overlay(name) => Some(name),
            Opened::Arg(_) => None,
        }
    }

    /// What it names at an instance of `widget` with `args`: `args.<param>` is the value the
    /// instance binds, else the param's `default`. A bound `args.<param>` passes the holder's own
    /// param through, for the holder's instance to bind. `None` when that cannot be judged
    /// statically: the param is unbound with no default, or bound to a non-string, a field of an
    /// arg, or a runtime reference (`row`, `rows`, or a path under one).
    fn bind(&self, widget: Option<&Widget>, args: Option<&Value>) -> Option<Opened> {
        let param = match self {
            Opened::Overlay(name) => return Some(Opened::Overlay(name.clone())),
            Opened::Arg(param) => param,
        };
        let bound = args
            .and_then(|a| a.as_object())
            .and_then(|a| a.get(param))
            .or_else(|| widget?.params.get(param)?.default.as_ref())?;
        match bound {
            Value::String(text) if text.starts_with("args.") => Opened::written(text.clone()),
            Value::String(name) if !is_reference(name) => Some(Opened::Overlay(name.clone())),
            _ => None,
        }
    }
}

/// The shallower of two places on the expansion stack, either of which may be absent.
fn shallowest(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

/// Whether a bound value is an expression read at run time rather than a name as written.
fn is_reference(text: &str) -> bool {
    ["row", "rows", "args"].into_iter().any(|root| {
        text.strip_prefix(root)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
    })
}

/// The overlays an `opens` in a node at `path` may name, with the end of the message that says
/// who does not declare one: a page's and its shell's, or a shell's own. `None` off every page
/// and shell.
fn overlays_at<'a>(doc: &'a Document, path: &NodePath) -> Option<(Vec<&'a str>, String)> {
    let first = path.0.first()?;
    match first.layer {
        Layer::Page => {
            let page = doc.pages.get(&first.name)?;
            let mut reachable: Vec<&str> = page.overlays.keys().map(String::as_str).collect();
            if let Some(shell) = doc.shell_of(page).and_then(|s| doc.shells.get(s)) {
                reachable.extend(shell.overlays.keys().map(String::as_str));
            }
            Some((reachable, "neither the page nor its shell declares".into()))
        }
        Layer::Shell => {
            let shell = doc.shells.get(&first.name)?;
            Some((
                shell.overlays.keys().map(String::as_str).collect(),
                format!("shell `{}` does not declare", first.name),
            ))
        }
        _ => None,
    }
}

/// Each `opens` of a body node: the node's path relative to the body, and what it opens, with the
/// `args.<param>` of the body itself left for each instance to bind.
type BodyOpens = Rc<Vec<(String, Opened)>>;

/// Expands widget bodies for [`widget_opens`]: every `opens` a widget's body holds, through the
/// widgets it uses, each nested instance's `args` substituted. A widget already being expanded is
/// not expanded again, so a body that recurs (a `widget_recursion` error) stops where it recurs.
/// An expansion is kept for reuse unless it stopped at a widget expanded around it: where such a
/// recursion stops depends on the widgets being expanded outside it, so reusing it under another
/// use would make the findings depend on the order of the uses. A recursion that closes inside the
/// expansion (at the widget itself or one below it) stops the same way wherever it is expanded.
struct Expander<'a> {
    doc: &'a Document,
    done: HashMap<&'a str, BodyOpens>,
    expanding: Vec<&'a str>,
    /// The shallowest place on `expanding` at which an expansion under way stopped at a recursion.
    stopped_at: Option<usize>,
}

impl<'a> Expander<'a> {
    fn new(doc: &'a Document) -> Self {
        Expander {
            doc,
            done: HashMap::new(),
            expanding: Vec::new(),
            stopped_at: None,
        }
    }

    /// The expanded `opens` of widget `name`'s body; nothing for an undeclared widget.
    fn body(&mut self, name: &'a str) -> BodyOpens {
        if let Some(done) = self.done.get(name) {
            return Rc::clone(done);
        }
        let Some(widget) = self.doc.widgets.get(name) else {
            return Rc::default();
        };
        if self.expanding.contains(&name) {
            let depth = self.expanding.iter().position(|w| *w == name);
            self.stopped_at = shallowest(self.stopped_at, depth);
            return Rc::default();
        }
        let outer = self.stopped_at.take();
        let depth = self.expanding.len();
        self.expanding.push(name);
        let mut found = Vec::new();
        for node in &widget.body {
            self.node(node.name.clone(), node, &mut found);
        }
        self.expanding.pop();
        let found = Rc::new(found);
        let inner = self.stopped_at.filter(|at| *at < depth);
        if inner.is_none() {
            self.done.insert(name, Rc::clone(&found));
        }
        self.stopped_at = shallowest(outer, inner);
        found
    }

    fn node(&mut self, at: String, node: &'a Node, found: &mut Vec<(String, Opened)>) {
        match &node.body {
            NodeBody::Composite(c) => self.composite(at, c, found),
            NodeBody::Primitive(p) => {
                self.props(&at, &p.props, instances_in_props(&p.props), found)
            }
        }
    }

    /// A composite of a body: its props and the widget it may be, then its board widgets and
    /// items, however deep.
    fn composite(
        &mut self,
        at: String,
        composite: &'a Composite,
        found: &mut Vec<(String, Opened)>,
    ) {
        self.props(&at, &composite.props, instances_in(composite), found);
        for (name, widget) in &composite.widgets {
            self.composite(format!("{at}/{}:{name}", Layer::Widget), widget, found);
        }
        for node in &composite.item {
            self.node(format!("{at}/{}:{}", Layer::Item, node.name), node, found);
        }
    }

    /// The `opens` in one node's props, and the expanded bodies of the widgets it instantiates,
    /// each with the `args` its instance binds substituted; what an instance cannot bind
    /// statically is dropped.
    fn props(
        &mut self,
        at: &str,
        props: &IndexMap<String, Value>,
        instances: Vec<Instance<'a>>,
        found: &mut Vec<(String, Opened)>,
    ) {
        let value = Value::Object(props.clone().into_iter().collect());
        found.extend(
            opens(&value)
                .into_iter()
                .filter_map(Opened::written)
                .map(|o| (at.to_owned(), o)),
        );
        for instance in instances {
            let used = match &instance.key {
                Some(key) => format!("{at}/{key}"),
                None => at.to_owned(),
            };
            let widget = self.doc.widgets.get(instance.widget);
            for (inner, opened) in self.body(instance.widget).iter() {
                if let Some(bound) = opened.bind(widget, instance.args) {
                    found.push((format!("{used}/body/{inner}"), bound));
                }
            }
        }
    }
}

/// A widget instance: a typed composite, or one written in untyped data.
pub(crate) struct Instance<'a> {
    /// From the node that holds it to the instance through untyped data (`children/badge`,
    /// `header/metrics/due`); `None` for a typed composite, which is the node itself.
    pub(crate) trail: Option<String>,
    /// The trail as findings name it, so that putting an entry before it in a list does not move
    /// it: an unnamed list entry is the widget's name when it is the instance itself, else `*`.
    pub(crate) key: Option<String>,
    /// The widget it names.
    pub(crate) widget: &'a str,
    /// Its `args`.
    pub(crate) args: Option<&'a Value>,
}

/// Every widget instance of the document with the path of the node that holds it, in document
/// order. This is the one walk behind the widget checks, the in-use refusal of a removal (through
/// `widget_resolves`) and the use sites the docs list.
///
/// It covers every place `ui-spec/1` puts a Node:
/// - each composite [`composites`] lists (sections, overlays, board widgets, items, widget-body
///   composites), itself and its untyped props (see [`instances_in`]);
/// - each primitive of an `item` list or a widget body, its props (an action's `choice`), held
///   at the primitive;
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
            walk_value(header, Trail::new("header"), &mut found);
            let at = root.child(Layer::Page, name);
            out.extend(found.into_iter().map(|i| (at.clone(), i)));
        }
    }
    for (kind, value) in &doc.page_kinds {
        let mut found = Vec::new();
        walk_value(value, Trail::new(&format!("page_kinds/{kind}")), &mut found);
        out.extend(found.into_iter().map(|i| (root.clone(), i)));
    }
    out
}

/// The widget instances a composite holds without going through its typed `widgets` and `item`:
/// the composite itself when it is one, then every instance in its untyped props, however deep.
///
/// `ui-spec/1` expects a Node in `children`, `parts`, `expand`, `choices`, `metrics`, `toolbar`, a
/// field's `choice` and a tab's `form`; this subset keeps those as props.
fn instances_in(composite: &Composite) -> Vec<Instance<'_>> {
    let mut out = Vec::new();
    if let Some(name) = composite.component.widget() {
        out.push(Instance {
            trail: None,
            key: None,
            widget: name,
            args: composite.args(),
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
            walk_value(value, Trail::new(key), &mut out);
        }
    }
    out
}

/// Where [`walk_value`] is in untyped data: the trail the docs show, which names an unnamed list
/// entry by its index, and the key findings use, which does not (see [`Instance::key`]).
struct Trail {
    shown: String,
    key: String,
}

impl Trail {
    fn new(start: &str) -> Self {
        Trail {
            shown: start.to_owned(),
            key: start.to_owned(),
        }
    }

    fn child(&self, shown: &str, key: &str) -> Self {
        Trail {
            shown: format!("{}/{shown}", self.shown),
            key: format!("{}/{key}", self.key),
        }
    }
}

/// The widget an untyped object instantiates: its `component`, when that is not a composite kind.
fn widget_named(value: &Value) -> Option<&str> {
    value
        .get("component")
        .and_then(Value::as_str)
        .filter(|name| CompositeKind::parse(name).is_none())
}

/// Every widget instance under untyped data, however deep: any object whose `component` is not a
/// composite kind. An `args` value is data, not nodes, and is not searched.
fn walk_value<'a>(value: &'a Value, trail: Trail, out: &mut Vec<Instance<'a>>) {
    match value {
        Value::Object(map) => {
            if let Some(name) = widget_named(value) {
                out.push(Instance {
                    trail: Some(trail.shown.clone()),
                    key: Some(trail.key.clone()),
                    widget: name,
                    args: map.get("args"),
                });
            }
            for (key, v) in map {
                if key != "args" {
                    walk_value(v, trail.child(key, key), out);
                }
            }
        }
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                let child = match item.get("name").and_then(Value::as_str) {
                    Some(name) => trail.child(name, name),
                    None => trail.child(&i.to_string(), widget_named(item).unwrap_or("*")),
                };
                walk_value(item, child, out);
            }
        }
        _ => {}
    }
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

/// The widgets a widget's body instantiates, however deep, typed or held in the props of a
/// composite or a primitive.
fn uses(widget: &Widget) -> Vec<&str> {
    let mut stack: Vec<&Node> = widget.body.iter().collect();
    let mut out = Vec::new();
    while let Some(node) = stack.pop() {
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
            composites.extend(composite.widgets.values());
            stack.extend(&composite.item);
        }
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
    nodes(doc)
        .into_iter()
        .filter_map(|(path, node)| match node {
            NodeRef::Composite(c) => Some((path, c)),
            _ => None,
        })
        .collect()
}

/// Every composite and every primitive of the document with its path, in document order: the
/// order of [`composites`], each primitive of an `item` list or a widget body in its place. A
/// check that reads a node's props reads them from here, so a primitive node is not skipped.
pub(crate) fn nodes(doc: &Document) -> Vec<(NodePath, NodeRef<'_>)> {
    fn walk<'a>(path: NodePath, composite: &'a Composite, out: &mut Vec<(NodePath, NodeRef<'a>)>) {
        out.push((path.clone(), NodeRef::Composite(composite)));
        for (name, widget) in &composite.widgets {
            walk(path.child(Layer::Widget, name), widget, out);
        }
        for node in &composite.item {
            walk_node(path.child(Layer::Item, &node.name), node, out);
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

fn check_composite(
    out: &mut Findings,
    path: &NodePath,
    composite: &Composite,
    overlays: &[&str],
    siblings: &[&str],
) {
    opens_resolve(out, path, &composite.props, overlays);
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
    for node in &composite.item {
        let at = path.child(Layer::Item, &node.name);
        match &node.body {
            NodeBody::Composite(item) => check_composite(out, &at, item, overlays, &[]),
            NodeBody::Primitive(item) => opens_resolve(out, &at, &item.props, overlays),
        }
    }
}

/// `opens_resolves` over one node's props.
fn opens_resolve(
    out: &mut Findings,
    path: &NodePath,
    props: &IndexMap<String, Value>,
    overlays: &[&str],
) {
    let props = Value::Object(props.clone().into_iter().collect());
    for opened in opens(&props) {
        if !overlays.contains(&opened.as_str()) {
            out.push(
                "opens_resolves",
                path,
                format!("opens `{opened}`, which neither the page nor its shell declares"),
            );
        }
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
