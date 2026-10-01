//! What the operator sees and what the agent is told about one node.
//!
//! [`outline`] is the document as the author wrote it: what the agent is given and what node
//! paths address. [`rendered`] is what ESS renders (epic:ess-ui-adoption D6): the same tree with
//! every node ESS's loader yields that the author did not write (what a page kind contributes,
//! merged by name into the page, and the body of every widget instance) marked `inherited`. The
//! browser is shown [`rendered`]; an instruction at an inherited node lands on the nearest node
//! the author wrote ([`authored_at`]).

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::check::{Instance, reaches, widget_uses};
use crate::model::{CompositeKind, Document, Node, NodeBody, Overlay};
use crate::path::{Layer, NodePath, NodeRef, PathError, allowed_children, children, resolve};

/// One node of the document tree, with its children in document order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutlineNode {
    /// Where the node sits.
    pub path: String,
    /// Its layer.
    pub layer: Layer,
    /// Its name; empty for the root and `nav`.
    pub name: String,
    /// Its kind: page kind, region kind, composite kind.
    pub kind: String,
    /// Its title or label, where it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The view it reads: a composite's or an overlay's `reads`, a region's `props.reads`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    /// A composite's props other than `component`, `reads`, `widgets` and `item`: what a renderer
    /// needs to draw it (columns, title, from, fields, a widget instance's args). A primitive's
    /// props; a widget's params, arrangement and `uses`: each instance of it as `{path, trail?}`,
    /// the node that holds it and the way through that node's untyped data, as the widget checks
    /// and the docs find them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Value>,
    /// ESS renders the node but the author did not write it: a section, an overlay or a named
    /// node a page kind contributes, or a node of a widget instance's body (at
    /// `<instance>/node:<name>`). Its fields are the ones written where it comes from (the kind,
    /// the widget's declaration); the instance's args bind a body's. Only [`rendered`] holds such
    /// nodes.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub inherited: bool,
    /// Its children.
    pub children: Vec<OutlineNode>,
}

/// The whole document as the author wrote it, from the root.
pub fn outline(doc: &Document) -> OutlineNode {
    Walk::new(doc)
        .authored(&NodePath::root())
        .expect("the root resolves")
}

/// The whole document as ESS renders it, from the root: [`outline`] with every node ESS's loader
/// yields under `pages/` and `shells/` that the author did not write, marked `inherited`. Named
/// lists (sections, items, choices, a body, …) are in the order ESS renders them. A document
/// ESS's loader refuses is shown as written.
pub fn rendered(doc: &Document) -> OutlineNode {
    let mut root = outline(doc);
    if let Some(expanded) = Expanded::of(doc) {
        expanded.add_to(&mut root);
    }
    root
}

/// The subtree of [`rendered`] at `path`, an inherited node's included; `None` when ESS renders
/// no node there.
pub fn rendered_at(doc: &Document, path: &str) -> Option<OutlineNode> {
    let root = rendered(doc);
    lineage(&root, path).and_then(|chain| chain.last().map(|n| (*n).clone()))
}

/// The node the author wrote that a node of [`rendered`] is, or is contributed to: the node
/// itself when the author wrote it; for an inherited node, the nearest one above it the author
/// wrote (the page for a section of its kind, the instance for a node of a widget's body). `None`
/// when ESS renders no node at `path`.
pub fn authored_at(doc: &Document, path: &str) -> Option<NodePath> {
    let root = rendered(doc);
    let chain = lineage(&root, path)?;
    chain
        .iter()
        .rev()
        .find(|n| !n.inherited)
        .and_then(|n| n.path.parse().ok())
}

/// Whether `target` is the node at `at` or sits below it, both with surrounding slashes trimmed.
fn within(target: &str, at: &str) -> bool {
    target == at || target.strip_prefix(at).is_some_and(|r| r.starts_with('/'))
}

/// The nodes from `root` down to the one at `path`, compared as written with surrounding slashes
/// trimmed; `None` when the tree has no node there.
fn lineage<'o>(root: &'o OutlineNode, path: &str) -> Option<Vec<&'o OutlineNode>> {
    let target = path.trim().trim_matches('/');
    let mut chain = vec![root];
    let mut node = root;
    while node.path.trim_matches('/') != target {
        node = node
            .children
            .iter()
            .find(|c| within(target, c.path.trim_matches('/')))?;
        chain.push(node);
    }
    Some(chain)
}

/// [`lineage`]'s last node, to change.
fn node_mut<'o>(node: &'o mut OutlineNode, target: &str) -> Option<&'o mut OutlineNode> {
    if node.path.trim_matches('/') == target {
        return Some(node);
    }
    let next = node
        .children
        .iter_mut()
        .find(|c| within(target, c.path.trim_matches('/')))?;
    node_mut(next, target)
}

/// The use sites of the widget `name`, one per instance, in the order [`widget_uses`] finds them.
fn uses_of(uses: &[(NodePath, Instance<'_>)], name: &str) -> serde_json::Value {
    let mut out: Vec<serde_json::Value> = Vec::new();
    for (path, instance) in uses.iter().filter(|(_, i)| i.widget == name) {
        let site = match &instance.trail {
            Some(trail) => serde_json::json!({"path": path.to_string(), "trail": trail}),
            None => serde_json::json!({"path": path.to_string()}),
        };
        out.push(site);
    }
    serde_json::Value::Array(out)
}

/// The schema ESS embeds, parsed once: the built-in page kinds are read from it.
fn schema() -> &'static serde_yaml::Value {
    static PARSED: OnceLock<serde_yaml::Value> = OnceLock::new();
    PARSED.get_or_init(|| serde_yaml::from_str(ess_ui::SCHEMA).expect("the ESS schema is YAML"))
}

/// The uilab layer an ESS container key holds under a node of `parent`'s layer; a widget
/// instance's expanded `body` is uilab's `node:` at the instance. `None` for what uilab does not
/// address (fields, actions, states, tabs, a header).
fn layer_in(parent: Layer, key: &str) -> Option<Layer> {
    use Layer::*;
    Some(match (parent, key) {
        (Root, "pages") => Page,
        (Root, "shells") => Shell,
        (Shell, "regions") => Region,
        (Shell, "overlays") => Overlay,
        (Page, "sections") => Section,
        (Page, "overlays") => Overlay,
        (Section | Overlay | Widget | Item | Child | Part | Choice | Tool | Node, key) => match key
        {
            "body" => Node,
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

/// The uilab path of an ESS canonical path, when every step names a uilab node.
fn uilab_of(segments: &[String]) -> Option<NodePath> {
    let mut path = NodePath::root();
    for step in segments.chunks(2) {
        let [key, name] = step else { return None };
        path = path.child(layer_in(path.layer(), key)?, name);
    }
    Some(path)
}

/// Whether nodes of `layer` sit in a list, whose order ESS keeps; the others sit in maps.
fn listed(layer: Layer) -> bool {
    matches!(
        layer,
        Layer::Section
            | Layer::Item
            | Layer::Part
            | Layer::Choice
            | Layer::Tool
            | Layer::Child
            | Layer::Node
    )
}

/// The keys of a written node that are not among its props: what [`describe`] reads elsewhere.
const NOT_PROPS: [&str; 10] = [
    "name",
    "component",
    "primitive",
    "reads",
    "widgets",
    "item",
    "parts",
    "choices",
    "toolbar",
    "children",
];

/// What ESS's loader renders of a document, and where each node's fields are written.
struct Expanded<'a> {
    doc: &'a Document,
    uses: Vec<(NodePath, Instance<'a>)>,
    /// Every node ESS renders under `pages/` and `shells/` that names a uilab node, in ESS's
    /// order (parents before children), with its ESS canonical path.
    nodes: Vec<(NodePath, Vec<String>)>,
    /// What ESS renders each section, overlay and node of [`Self::nodes`] as, by uilab path: its
    /// kind label as uilab spells it and the view it reads ([`ess_fields`]).
    fields: HashMap<String, (String, Option<String>)>,
    /// The document as written, as JSON.
    written: Value,
    /// Per page kind a page uses, its `sections` and `overlays` as ESS resolves the kind over
    /// those it extends (a built-in kind as the schema writes it), as JSON.
    kinds: Map<String, Value>,
    /// Per page, by name, its `header` and `nav` as ESS renders them ([`header_json`],
    /// [`nav_json`]); a key ESS renders none for is absent.
    pages: HashMap<String, Map<String, Value>>,
}

impl<'a> Expanded<'a> {
    /// ESS's loader on the document as written; `None` when it refuses the document.
    fn of(doc: &'a Document) -> Option<Self> {
        let text = doc.to_yaml().ok()?;
        let ess = ess_ui::load_str(&text).ok()?;
        let mut nodes = Vec::new();
        let mut fields = HashMap::new();
        for located in ess.nodes() {
            let segments = located.path.segments();
            let top = segments.first().map(String::as_str);
            if !matches!(top, Some("pages" | "shells")) {
                continue;
            }
            let Some(path) = uilab_of(segments) else {
                continue;
            };
            if let Some(found) = ess_fields(located.node) {
                fields.insert(path.to_string(), found);
            }
            nodes.push((path, segments.to_vec()));
        }
        let json = |v: Option<&serde_yaml::Value>| {
            v.and_then(|v| serde_json::to_value(v).ok())
                .unwrap_or(Value::Null)
        };
        let mut kinds = Map::new();
        for page in doc.pages.values() {
            if kinds.contains_key(&page.kind) {
                continue;
            }
            let kind = match ess.page_kinds.get(&page.kind) {
                Some(kind) => serde_json::json!({
                    "sections": json(kind.sections.as_ref()),
                    "overlays": json(kind.overlays.as_ref()),
                }),
                None => json(Some(
                    &schema()["constructs"]["PageKind"]["builtins"][page.kind.as_str()],
                )),
            };
            kinds.insert(page.kind.clone(), kind);
        }
        Some(Expanded {
            doc,
            uses: widget_uses(doc),
            nodes,
            fields,
            written: serde_json::to_value(doc).ok()?,
            kinds,
            pages: ess
                .pages
                .iter()
                .map(|(name, page)| {
                    let mut shown = Map::new();
                    if let Some(header) = &page.header {
                        shown.insert("header".into(), header_json(header));
                    }
                    if let Some(nav) = &page.nav {
                        shown.insert("nav".into(), nav_json(nav));
                    }
                    (name.clone(), shown)
                })
                .collect(),
        })
    }

    /// Adds to the authored tree every node ESS renders that it does not hold, inherited, under
    /// its parent; then puts each list's nodes in ESS's order. Nodes of a map (shells, regions,
    /// overlays, pages, board widgets) keep the author's order, inherited ones after. Last, every
    /// section, overlay and node, the author's included, takes the kind and the view ESS renders
    /// it with: a section refining its page kind's, an overlay `same_as` another; and every page
    /// takes the header and menu entry ESS renders for it.
    fn add_to(&self, root: &mut OutlineNode) {
        self.add_nodes(root);
        self.merge_fields(root);
        self.merge_pages(root);
    }

    /// Each page node's `header` and `nav` props as ESS renders them: the page kind's header
    /// merged in, `title: from_page` and a missing `nav.label` taken from the page's title. A
    /// header's `metrics` stay as the page writes them.
    fn merge_pages(&self, root: &mut OutlineNode) {
        for page in root.children.iter_mut().filter(|c| c.layer == Layer::Page) {
            let Some(shown) = self.pages.get(&page.name) else {
                continue;
            };
            let mut props = match page.props.take() {
                Some(Value::Object(props)) => props,
                _ => Map::new(),
            };
            let metrics = props.get("header").and_then(|h| h.get("metrics")).cloned();
            for key in ["header", "nav"] {
                match shown.get(key) {
                    Some(value) => props.insert(key.into(), value.clone()),
                    None => props.remove(key),
                };
            }
            if let (Some(metrics), Some(Value::Object(header))) = (metrics, props.get_mut("header"))
            {
                header.insert("metrics".into(), metrics);
            }
            page.props = (!props.is_empty()).then_some(Value::Object(props));
        }
    }

    /// Each node's kind and view as ESS renders it, where ESS gives them; a view only where ESS
    /// reads one.
    fn merge_fields(&self, node: &mut OutlineNode) {
        if let Some((kind, view)) = self.fields.get(&node.path) {
            node.kind.clone_from(kind);
            if view.is_some() {
                node.view.clone_from(view);
            }
        }
        for child in &mut node.children {
            self.merge_fields(child);
        }
    }

    /// [`Self::add_to`]'s first part: the nodes, in ESS's order.
    fn add_nodes(&self, root: &mut OutlineNode) {
        let mut order: HashMap<String, usize> = HashMap::new();
        for (i, (path, segments)) in self.nodes.iter().enumerate() {
            let text = path.to_string();
            order.insert(text.clone(), i);
            if lineage(root, &text).is_some() {
                continue;
            }
            let parent = path.parent().unwrap_or_default().to_string();
            let node = self.inherited(path, segments);
            if let Some(holder) = node_mut(root, parent.trim_matches('/')) {
                holder.children.push(node);
            }
        }
        in_order(root, &order);
    }

    /// A node the author did not write, read from where it is written: the page kind, or the
    /// declaration of the widget an instance uses. Without children; [`Self::add_to`] adds them.
    fn inherited(&self, path: &NodePath, segments: &[String]) -> OutlineNode {
        let written = self.written_at(segments);
        let typed = written.and_then(|fields| match path.layer() {
            Layer::Overlay => serde_json::from_value::<Overlay>(fields.clone())
                .ok()
                .map(|o| {
                    describe(
                        self.doc,
                        &self.uses,
                        path,
                        NodeRef::Overlay(&o),
                        vec![],
                        true,
                    )
                }),
            _ => Node::named(path.name(), fields)
                .ok()
                .map(|node| describe(self.doc, &self.uses, path, node_ref(&node), vec![], true)),
        });
        typed.unwrap_or_else(|| as_written(path, written))
    }

    /// The fields written for the node at an ESS path: the author's where the author wrote it,
    /// else the page kind's (named lists merge by name, so a node of a refined section is the
    /// kind's), a widget instance's `body` read from the widget's declaration.
    fn written_at(&self, segments: &[String]) -> Option<&Value> {
        let (top, name) = (segments.first()?, segments.get(1)?);
        let mut candidates: Vec<&Value> = vec![self.written.get(top.as_str())?.get(name.as_str())?];
        if top == "pages"
            && let Some(kind) = self
                .doc
                .pages
                .get(name.as_str())
                .and_then(|page| self.kinds.get(&page.kind))
        {
            candidates.push(kind);
        }
        for step in segments.get(2..)?.chunks(2) {
            let [key, child] = step else { return None };
            let found: Vec<&Value> = candidates
                .into_iter()
                .filter_map(|c| self.child_of(c, key, child))
                .collect();
            candidates = Vec::new();
            for value in found {
                self.with_same_as(value, &mut candidates, 0);
            }
        }
        candidates.first().copied()
    }

    /// `value`, then, where it is an overlay written `same_as: <page>.<overlay>`, the overlay it
    /// copies (as the author wrote it, then as that page's kind contributes it), as ESS copies it
    /// (`resolve_same_as`, at most 8 deep).
    fn with_same_as<'v>(&'v self, value: &'v Value, out: &mut Vec<&'v Value>, depth: usize) {
        out.push(value);
        let Some((page, overlay)) = value
            .get("same_as")
            .and_then(Value::as_str)
            .and_then(|target| target.rsplit_once('.'))
        else {
            return;
        };
        if depth >= 8 {
            return;
        }
        let kind = self
            .doc
            .pages
            .get(page)
            .and_then(|p| self.kinds.get(&p.kind));
        let sources = [self.written.get("pages").and_then(|p| p.get(page)), kind];
        for source in sources.into_iter().flatten() {
            if let Some(copied) = source.get("overlays").and_then(|o| o.get(overlay)) {
                self.with_same_as(copied, out, depth + 1);
            }
        }
    }

    /// The written child `name` under `key` of `value`: an entry of a list by its `name`, a value
    /// of a map by its key; under `body`, the node of the declaration of the widget `value` uses.
    fn child_of<'v>(&'v self, value: &'v Value, key: &str, name: &str) -> Option<&'v Value> {
        let holder = if key == "body" {
            let widget = value.get("component")?.as_str()?;
            self.written.get("widgets")?.get(widget)?.get("body")?
        } else {
            value.get(key)?
        };
        match holder {
            Value::Array(entries) => entries
                .iter()
                .find(|e| e.get("name").and_then(Value::as_str) == Some(name)),
            Value::Object(entries) => entries.get(name),
            _ => None,
        }
    }
}

/// A page header as ESS renders it, as JSON: its title, total, filters, switch, live channels,
/// help, and each action's name, text and what it runs or opens. ESS's types do not serialize, so
/// this carries what a renderer draws, not every field of an action; `metrics` are left to the
/// page as written ([`Expanded::merge_pages`]).
fn header_json(header: &ess_ui::Header) -> Value {
    let mut out = Map::new();
    let mut text = |key: &str, value: &Option<String>| {
        if let Some(v) = value {
            out.insert(key.into(), Value::from(v.as_str()));
        }
    };
    text("title", &header.title);
    text("total", &header.total);
    text("filters", &header.filters);
    if let Some(help) = &header.help {
        let mut h = Map::new();
        for (key, value) in [("text", &help.text), ("link", &help.link)] {
            if let Some(v) = value {
                h.insert(key.into(), Value::from(v.as_str()));
            }
        }
        out.insert("help".into(), Value::Object(h));
    }
    for (key, list) in [("switch", &header.switch), ("live", &header.live)] {
        if !list.is_empty() {
            out.insert(key.into(), serde_json::json!(list));
        }
    }
    if !header.actions.is_empty() {
        let actions = header
            .actions
            .iter()
            .map(|a| {
                let mut m = Map::new();
                m.insert("name".into(), Value::from(a.name.as_str()));
                for (key, value) in [("label", &a.label), ("does", &a.does), ("opens", &a.opens)] {
                    if let Some(v) = value {
                        m.insert(key.into(), Value::from(v.as_str()));
                    }
                }
                Value::Object(m)
            })
            .collect();
        out.insert("actions".into(), Value::Array(actions));
    }
    Value::Object(out)
}

/// A page's menu entry as ESS renders it, as JSON: its label (the page's title when not written)
/// and synonyms.
fn nav_json(nav: &ess_ui::NavEntry) -> Value {
    let mut out = Map::new();
    if let Some(label) = &nav.label {
        out.insert("label".into(), Value::from(label.as_str()));
    }
    if !nav.synonyms.is_empty() {
        out.insert("synonyms".into(), serde_json::json!(nav.synonyms));
    }
    Value::Object(out)
}

/// What ESS renders a section, an overlay or a node as: its kind label as uilab spells it (a
/// composite kind, the widget an instance uses, a primitive kind; an overlay's presentation before
/// its body's, `drawer form`) and the view its `reads` names. `None` for any other node.
fn ess_fields(node: ess_ui::NodeRef<'_>) -> Option<(String, Option<String>)> {
    match node {
        ess_ui::NodeRef::Section(section) => Some(body_fields(&section.body)),
        ess_ui::NodeRef::Node(node) => Some(body_fields(&node.body)),
        ess_ui::NodeRef::Overlay(overlay) => {
            let (kind, view) = body_fields(&overlay.body);
            let presentation = match &overlay.kind {
                ess_ui::OverlayKind::Drawer => "drawer",
                ess_ui::OverlayKind::Dialog => "dialog",
                ess_ui::OverlayKind::Fullscreen => "fullscreen",
                ess_ui::OverlayKind::Popover => "popover",
                ess_ui::OverlayKind::Unmapped(_) => return Some((kind, view)),
            };
            Some((format!("{presentation} {kind}"), view))
        }
        _ => None,
    }
}

/// [`ess_fields`] of what a section, an overlay or a node holds.
fn body_fields(body: &ess_ui::Body) -> (String, Option<String>) {
    use ess_ui::Composite as C;
    use ess_ui::Primitive as P;
    let named = |reads: Option<&ess_ui::Reads>| {
        reads.and_then(|r| r.view.clone().or_else(|| r.placeholder.clone()))
    };
    match body {
        ess_ui::Body::Widget(instance) => (instance.component.clone(), None),
        ess_ui::Body::Primitive(primitive) => {
            let kind = match primitive {
                P::Text(_) => "text",
                P::Badge(_) => "badge",
                P::Icon(_) => "icon",
                P::Button(_) => "button",
                P::Link(_) => "link",
                P::Input(_) => "input",
                P::Toggle(_) => "toggle",
                P::Image(_) => "image",
                P::Divider(_) => "divider",
            };
            (kind.to_owned(), None)
        }
        ess_ui::Body::Composite(composite) => {
            let (kind, view) = match composite {
                C::Collection(c) => ("collection", named(c.reads.as_ref())),
                C::Record(c) => ("record", named(c.reads.as_ref())),
                C::Form(_) => ("form", None),
                C::Choice(c) => ("choice", named(c.reads.as_ref())),
                C::FilterBar(_) => ("filter_bar", None),
                C::Confirm(_) => ("confirm", None),
                C::Metric(c) => ("metric", named(c.reads.as_ref())),
                C::Chart(c) => ("chart", named(Some(&c.reads))),
                C::Board(c) => ("board", named(Some(&c.reads))),
                C::GraphEditor(c) => ("graph_editor", named(Some(&c.reads))),
                C::RichText(_) => ("rich_text", None),
                C::References(c) => ("references", named(Some(&c.reads))),
            };
            (kind.to_owned(), view)
        }
    }
}

/// Each list's nodes in ESS's order (`order`); map layers keep theirs, layer groups stay put.
fn in_order(node: &mut OutlineNode, order: &HashMap<String, usize>) {
    let mut groups: Vec<Layer> = Vec::new();
    for child in &node.children {
        if !groups.contains(&child.layer) {
            groups.push(child.layer);
        }
    }
    node.children.sort_by_key(|c| {
        let group = groups.iter().position(|l| *l == c.layer);
        let place = if listed(c.layer) {
            order.get(&c.path).copied().unwrap_or(usize::MAX)
        } else {
            0
        };
        (group, place)
    });
    for child in &mut node.children {
        in_order(child, order);
    }
}

/// An inherited node read from its written fields as they are, for a shape uilab's model does not
/// read (a `Reads` shorthand in a page kind): kind from `component` or `primitive`, view from
/// `reads`, every other field a prop.
fn as_written(path: &NodePath, written: Option<&Value>) -> OutlineNode {
    let text = |key: &str| {
        written
            .and_then(|w| w.get(key))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    let overlay = path.layer() == Layer::Overlay;
    let composite = text("component")
        .or_else(|| text("primitive"))
        .unwrap_or_else(|| "node".to_owned());
    let kind = match text("kind") {
        Some(presentation) if overlay => format!("{presentation} {composite}"),
        _ => composite,
    };
    let view = written
        .and_then(|w| w.get("reads"))
        .and_then(|reads| match reads {
            Value::String(view) => Some(view.clone()),
            other => other
                .get("view")
                .or_else(|| other.get("placeholder"))
                .and_then(Value::as_str)
                .map(str::to_owned),
        });
    let props: Map<String, Value> = written
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(k, _)| !NOT_PROPS.contains(&k.as_str()) && !(overlay && *k == "kind"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    OutlineNode {
        path: path.to_string(),
        layer: path.layer(),
        name: path.name().to_owned(),
        kind,
        title: text("title"),
        view,
        props: (!props.is_empty()).then_some(Value::Object(props)),
        inherited: true,
        children: Vec::new(),
    }
}

/// One walk of the document as written.
struct Walk<'a> {
    doc: &'a Document,
    uses: Vec<(NodePath, Instance<'a>)>,
}

impl<'a> Walk<'a> {
    fn new(doc: &'a Document) -> Self {
        Walk {
            doc,
            uses: widget_uses(doc),
        }
    }

    /// The node the author wrote at `path`, with its children.
    fn authored(&self, path: &NodePath) -> Result<OutlineNode, PathError> {
        let found = resolve(self.doc, path)?;
        let mut kids = Vec::new();
        for (layer, name) in children(self.doc, path)? {
            kids.push(self.authored(&path.child(layer, &name))?);
        }
        Ok(describe(self.doc, &self.uses, path, found, kids, false))
    }
}

fn node_ref(node: &Node) -> NodeRef<'_> {
    match &node.body {
        NodeBody::Composite(c) => NodeRef::Composite(c),
        NodeBody::Primitive(p) => NodeRef::Primitive(p),
    }
}

/// The outline node of `found` at `path`, holding `children`.
fn describe(
    doc: &Document,
    uses: &[(NodePath, Instance<'_>)],
    path: &NodePath,
    found: NodeRef<'_>,
    children: Vec<OutlineNode>,
    inherited: bool,
) -> OutlineNode {
    // Composites carry their own props. The menu, its sections and pages carry what a renderer
    // needs to place them: the home page, a section's pages, a page's shell.
    let props = match found {
        NodeRef::Nav(n) => Some(serde_json::json!({"home": n.home})),
        NodeRef::NavSection(s) => Some(match &s.pages {
            crate::model::NavPages::Fixed(pages) => serde_json::json!({"pages": pages}),
            crate::model::NavPages::Dynamic(entries) => {
                serde_json::json!({"from_view": entries.get("from_view"), "page": entries.get("page")})
            }
        }),
        NodeRef::Page(p) => {
            let mut props = Map::new();
            if let Some(shell) = doc.shell_of(p) {
                props.insert("shell".into(), serde_json::json!(shell));
            }
            for key in ["header", "nav"] {
                if let Some(value) = p.extra.get(key) {
                    props.insert(key.into(), value.clone());
                }
            }
            (!props.is_empty()).then_some(Value::Object(props))
        }
        NodeRef::Component(w) => Some(serde_json::json!({
            "params": w.params,
            "arrange": w.arrangement(),
            "uses": uses_of(uses, path.name()),
        })),
        NodeRef::Primitive(p) => Some(serde_json::Value::Object(
            p.props.clone().into_iter().collect(),
        )),
        _ => found
            .composite()
            .filter(|c| !c.props.is_empty())
            .map(|c| serde_json::Value::Object(c.props.clone().into_iter().collect())),
    };
    let (title, view) = match found {
        NodeRef::Root(d) => (d.title.clone(), None),
        NodeRef::Page(p) => (p.title.clone(), None),
        NodeRef::NavSection(s) => (s.label.clone(), None),
        NodeRef::Component(w) => (Some(w.summary.clone()), None),
        NodeRef::Overlay(o) => (
            o.body
                .props
                .get("title")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            o.body.reads.as_ref().map(|r| r.name().to_owned()),
        ),
        NodeRef::Composite(c) => (
            c.props
                .get("title")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            c.reads.as_ref().map(|r| r.name().to_owned()),
        ),
        NodeRef::Region(r) => (
            None,
            r.props
                .get("reads")
                .and_then(|reads| reads.get("view"))
                .and_then(|v| v.as_str())
                .map(str::to_owned),
        ),
        _ => (None, None),
    };
    OutlineNode {
        path: path.to_string(),
        layer: path.layer(),
        name: path.name().to_owned(),
        kind: found.kind_label(),
        title,
        view,
        props,
        inherited,
        children,
    }
}

/// What an agent needs to know to propose a patch at one node.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NodeContext<'a> {
    /// The node.
    pub path: String,
    /// Its layer.
    pub layer: Layer,
    /// Its kind.
    pub kind: String,
    /// From the root down to its parent, each as `path (kind)`.
    pub ancestors: Vec<String>,
    /// Layers a new child can take here.
    pub allowed_children: Vec<Layer>,
    /// What a new composite child can be: every built-in kind, then every declared widget that
    /// would not make a widget contain itself.
    pub composite_kinds: Vec<&'a str>,
    /// Existing children, as `layer:name`.
    pub children: Vec<String>,
    /// The node itself, as YAML.
    pub yaml: String,
}

/// The context of the node at `path`.
pub fn node_context<'a>(doc: &'a Document, path: &NodePath) -> Result<NodeContext<'a>, PathError> {
    let found = resolve(doc, path)?;
    let allowed = allowed_children(doc, path)?;
    let composite_kinds = if allowed.iter().any(|l| l.is_composite()) {
        let within = path
            .0
            .first()
            .filter(|s| s.layer == Layer::Component)
            .map(|s| s.name.as_str());
        CompositeKind::ALL
            .iter()
            .map(|k| k.as_str())
            .chain(
                doc.widgets
                    .keys()
                    .map(String::as_str)
                    .filter(|w| within.is_none_or(|own| *w != own && !reaches(doc, w, own))),
            )
            .collect()
    } else {
        Vec::new()
    };
    let mut ancestors = Vec::new();
    for ancestor in path
        .lineage()
        .into_iter()
        .rev()
        .skip(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let kind = resolve(doc, &ancestor)?.kind_label();
        ancestors.push(format!("{ancestor} ({kind})"));
    }
    let children = children(doc, path)?
        .into_iter()
        .map(|(layer, name)| {
            if layer == Layer::Nav {
                "nav".to_owned()
            } else {
                format!("{layer}:{name}")
            }
        })
        .collect();
    Ok(NodeContext {
        path: path.to_string(),
        layer: path.layer(),
        kind: found.kind_label(),
        ancestors,
        allowed_children: allowed,
        composite_kinds,
        children,
        yaml: node_yaml(found),
    })
}

/// The node at `path` as YAML, or `None` when no node has that path.
pub fn yaml_at(doc: &Document, path: &NodePath) -> Option<String> {
    resolve(doc, path).ok().map(node_yaml)
}

fn node_yaml(node: NodeRef<'_>) -> String {
    let text = match node {
        NodeRef::Root(d) => crate::model::to_yaml(&serde_json::json!({
            "app": d.app, "title": d.title, "model": d.model,
            "shells": d.shells.keys().collect::<Vec<_>>(),
            "pages": d.pages.keys().collect::<Vec<_>>(),
        })),
        NodeRef::Shell(s) => crate::model::to_yaml(s),
        NodeRef::Region(r) => crate::model::to_yaml(r),
        NodeRef::Nav(n) => crate::model::to_yaml(n),
        NodeRef::NavSection(s) => crate::model::to_yaml(s),
        NodeRef::Page(p) => crate::model::to_yaml(p),
        NodeRef::Overlay(o) => crate::model::to_yaml(o),
        NodeRef::Composite(c) => crate::model::to_yaml(c),
        NodeRef::Component(w) => crate::model::to_yaml(w),
        NodeRef::Primitive(p) => crate::model::to_yaml(p),
    };
    text.unwrap_or_default()
}

/// Words the speech model should expect at this node: layer and kind names and the names of the
/// node's ancestors and children. A speech model given these as a prompt spells them as written.
pub fn vocabulary(doc: &Document, path: &NodePath) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut add = |w: &str| {
        let w = w.replace('_', " ");
        if !w.is_empty() && !words.contains(&w) {
            words.push(w);
        }
    };
    if let Ok(context) = node_context(doc, path) {
        context
            .allowed_children
            .iter()
            .for_each(|l| add(l.as_str()));
        context.composite_kinds.iter().for_each(|k| add(k));
        for child in &context.children {
            add(child.split_once(':').map_or(child.as_str(), |(_, n)| n));
        }
    }
    for segment in &path.0 {
        add(&segment.name);
    }
    for page in doc.pages.keys() {
        add(page);
    }
    words
}

/// The outline of the subtree at `path`, or `None` when no node has that path.
pub fn outline_at(doc: &Document, path: &NodePath) -> Option<OutlineNode> {
    Walk::new(doc).authored(path).ok()
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::{Fixtures, docs_markdown};

    const WIDGETS: &str = "widgets:
  loan_card:
    summary: A loan as a card.
    params:
      loan: {type: Loan, required: true, note: the loan row}
    body:
      - {name: title, primitive: text, text: args.loan.title, style: heading}
      - {name: due, primitive: badge, text: args.loan.due}
  badge:
    summary: A toned tag.
    params:
      label: {type: string, required: true, note: the tag text}
    body:
      - {name: tag, primitive: badge, text: args.label}
  unused:
    summary: Nothing uses it.
    body:
      - {name: tag, primitive: text, text: nothing}
pages:
";

    /// The library example with the widgets `loan_card`, `badge` and `unused`, none of them used.
    fn library() -> Document {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/library/library.ui.yaml");
        let text = std::fs::read_to_string(file).unwrap();
        assert!(text.contains("\npages:\n"), "fixture anchor is missing");
        Document::from_yaml(&text.replacen("\npages:\n", &format!("\n{WIDGETS}"), 1)).unwrap()
    }

    /// The `uses` prop of the widget `name`'s outline node.
    fn uses(doc: &Document, name: &str) -> Value {
        let root = outline(doc);
        let widget = root
            .children
            .iter()
            .find(|c| c.path == format!("component:{name}"))
            .unwrap_or_else(|| panic!("no widget `{name}` in the outline"));
        widget.props.as_ref().expect("a widget has props")["uses"].clone()
    }

    #[test]
    fn a_widget_used_only_in_a_page_header_carries_that_use() {
        let mut doc = library();
        doc.pages["overview"].extra.insert(
            "header".into(),
            json!({"metrics": [{"name": "due", "component": "loan_card", "args": {"loan": "rows.first"}}]}),
        );
        assert_eq!(
            uses(&doc, "loan_card"),
            json!([{"path": "page:overview", "trail": "header/metrics/due"}])
        );
    }

    #[test]
    fn a_widget_used_only_in_a_page_kind_carries_that_use_at_the_root() {
        let mut doc = library();
        doc.page_kinds.insert(
            "board_page".into(),
            json!({"header": {"metrics": [{"name": "due", "component": "badge", "args": {"label": "row.state"}}]}}),
        );
        assert_eq!(
            uses(&doc, "badge"),
            json!([{"path": "/", "trail": "page_kinds/board_page/header/metrics/due"}])
        );
    }

    /// A primitive `badge` has kind `badge` in the outline, as an instance of the widget `badge`
    /// does; only the instance is a use.
    #[test]
    fn a_widget_named_like_a_primitive_carries_only_its_instances() {
        let mut doc = library();
        let latest: crate::model::Composite =
            serde_json::from_value(json!({"component": "badge", "args": {"label": "rows.first"}}))
                .unwrap();
        doc.pages["overview"]
            .sections
            .insert("latest".into(), Some(latest));
        assert_eq!(
            uses(&doc, "badge"),
            json!([{"path": "page:overview/section:latest"}])
        );
        assert_eq!(uses(&doc, "unused"), json!([]));
    }

    /// A shell region that reads a view (the library's `account` menu reads `staff.Me` through its
    /// props) carries that view in the outline, as a composite does; a region that reads none
    /// carries none.
    #[test]
    fn a_region_carries_the_view_its_props_read() {
        let root = outline(&library());
        let shell = root
            .children
            .iter()
            .find(|c| c.path == "shell:app")
            .expect("the library has the shell `app`");
        let view = |region: &str| {
            shell
                .children
                .iter()
                .find(|c| c.path == format!("shell:app/region:{region}"))
                .unwrap_or_else(|| panic!("no region `{region}`"))
                .view
                .clone()
        };
        assert_eq!(view("account"), Some("staff.Me".to_owned()));
        assert_eq!(view("nav"), None);
        assert_eq!(view("main"), None);
    }

    /// Every widget's `uses` are the use sites the docs list for it, in the same order: the
    /// outline and `/api/docs.md` read one walk.
    #[test]
    fn every_widget_carries_the_use_sites_the_docs_list() {
        let mut doc = library();
        doc.pages["overview"].extra.insert(
            "header".into(),
            json!({"metrics": [
                {"name": "due", "component": "loan_card", "args": {"loan": "rows.first"}},
                {"name": "tag", "component": "badge", "args": {"label": "rows.first"}}
            ]}),
        );
        doc.page_kinds.insert(
            "board_page".into(),
            json!({"sections": {"s": {"component": "loan_card", "args": {"loan": "row"}}}}),
        );
        let docs = docs_markdown(&doc, &Fixtures::default(), &[]);
        for name in doc.widgets.keys() {
            let listed: Vec<String> = uses(&doc, name)
                .as_array()
                .unwrap()
                .iter()
                .map(|u| match u.get("trail") {
                    Some(trail) => format!(
                        "`{}` (`{}`)",
                        u["path"].as_str().unwrap(),
                        trail.as_str().unwrap()
                    ),
                    None => format!("`{}`", u["path"].as_str().unwrap()),
                })
                .collect();
            let line = if listed.is_empty() {
                "Not used yet.".to_owned()
            } else {
                format!("Used at: {}", listed.join(", "))
            };
            let section = docs.split(&format!("### {name}\n")).nth(1).unwrap();
            assert!(
                section.lines().any(|l| l == line),
                "`{name}`: the outline lists {line:?}, the docs do not"
            );
        }
    }

    /// The library with the widgets of [`WIDGETS`] and one instance of `loan_card`, the overview's
    /// section `latest`, written before `on_loan`.
    fn library_with_instance() -> Document {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/library/library.ui.yaml");
        let text = std::fs::read_to_string(file).unwrap();
        assert!(text.contains("\npages:\n") && text.contains("      - name: on_loan\n"));
        let text = text.replacen("\npages:\n", &format!("\n{WIDGETS}"), 1).replacen(
            "      - name: on_loan\n",
            "      - {name: latest, component: loan_card, args: {loan: rows.first}}\n      - name: on_loan\n",
            1,
        );
        Document::from_yaml(&text).unwrap()
    }

    /// The child of `node` at `path`.
    fn child_at<'o>(node: &'o OutlineNode, path: &str) -> &'o OutlineNode {
        node.children
            .iter()
            .find(|c| c.path == path)
            .unwrap_or_else(|| panic!("no child `{path}` in `{}`", node.path))
    }

    /// story:essui-app-widget `outline_shows_inherited`: the outline the browser is shown holds the
    /// `filters` section the Loans page's kind (`list_page`) contributes, inherited, in the order
    /// ESS renders it, and a widget instance holds its widget's body nodes, inherited, with the
    /// props the widget declares.
    #[test]
    fn outline_shows_inherited() {
        let doc = library_with_instance();
        let root = rendered(&doc);

        let loans = child_at(&root, "page:loans");
        let sections: Vec<(&str, bool)> = loans
            .children
            .iter()
            .filter(|c| c.layer == Layer::Section)
            .map(|c| (c.path.as_str(), c.inherited))
            .collect();
        assert_eq!(
            sections,
            [
                ("page:loans/section:filters", true),
                ("page:loans/section:list", false)
            ]
        );
        let filters = child_at(loans, "page:loans/section:filters");
        assert_eq!(filters.kind, "filter_bar");
        assert_eq!(
            serde_json::to_value(filters).unwrap()["inherited"],
            json!(true),
            "the wire carries the mark"
        );
        assert!(!loans.inherited);
        assert!(
            !child_at(loans, "page:loans/overlay:edit").inherited,
            "the author's overlay is the author's"
        );

        let latest = child_at(
            child_at(&root, "page:overview"),
            "page:overview/section:latest",
        );
        assert!(!latest.inherited);
        let body: Vec<(&str, &str, bool)> = latest
            .children
            .iter()
            .map(|c| (c.path.as_str(), c.kind.as_str(), c.inherited))
            .collect();
        assert_eq!(
            body,
            [
                ("page:overview/section:latest/node:title", "text", true),
                ("page:overview/section:latest/node:due", "badge", true),
            ]
        );
        assert_eq!(
            child_at(latest, "page:overview/section:latest/node:title").props,
            Some(json!({"text": "args.loan.title", "style": "heading"}))
        );
        // A widget's own declaration is not expanded.
        let card = child_at(&root, "component:loan_card");
        assert!(card.children.iter().all(|c| !c.inherited));
    }

    /// The outline the agent is given and node paths address stays the document as written: no
    /// node in it is inherited, and every one resolves.
    #[test]
    fn the_outline_as_written_holds_nothing_inherited() {
        let doc = library_with_instance();
        fn walk(doc: &Document, node: &OutlineNode) {
            assert!(!node.inherited, "{} is inherited", node.path);
            let at: NodePath = node.path.parse().unwrap();
            assert!(resolve(doc, &at).is_ok(), "{} does not resolve", node.path);
            node.children.iter().for_each(|c| walk(doc, c));
        }
        walk(&doc, &outline(&doc));
        assert!(
            outline_at(&doc, &"page:overview/section:latest".parse().unwrap())
                .unwrap()
                .children
                .is_empty()
        );
    }

    /// An inherited node is answered for by the nearest node the author wrote above it: the page
    /// for a section of its kind, the instance for a node of a widget's body; a node the author
    /// wrote answers for itself, and a path ESS renders nothing at for nothing.
    #[test]
    fn an_inherited_node_is_written_at_the_nearest_node_the_author_wrote() {
        let doc = library_with_instance();
        let at = |p: &str| authored_at(&doc, p).map(|n| n.to_string());
        assert_eq!(
            at("page:loans/section:filters").as_deref(),
            Some("page:loans")
        );
        assert_eq!(
            at("page:overview/section:latest/node:due").as_deref(),
            Some("page:overview/section:latest")
        );
        assert_eq!(
            at("page:loans/section:list").as_deref(),
            Some("page:loans/section:list")
        );
        assert_eq!(at("/").as_deref(), Some("/"));
        assert_eq!(at("page:loans/section:ghost"), None);
        assert_eq!(at("page:overview/section:latest/node:ghost"), None);
        assert_eq!(
            rendered_at(&doc, "page:overview/section:latest/node:title").map(|n| n.inherited),
            Some(true)
        );
    }

    /// story:canvas-shows-labels: a page node carries its `header` and its menu entry `nav`, as
    /// written in the outline the agent is given and as ESS renders them in the one the browser
    /// is shown: the page kind's header merged in, `title: from_page` and a missing `nav.label`
    /// both taken from the page's title.
    #[test]
    fn a_page_carries_the_header_and_nav_entry_ess_renders() {
        let mut doc = library();
        let header = json!({
            "title": "Today at the desk",
            "help": {"text": "What is out and what is due back"},
            "actions": [{"name": "lend", "does": "loans.Lend", "label": "Lend a copy"}]
        });
        doc.pages["overview"]
            .extra
            .insert("header".into(), header.clone());
        doc.pages["overview"]
            .extra
            .insert("nav".into(), json!({"label": "Desk"}));
        doc.pages["members"]
            .extra
            .insert("nav".into(), json!({"synonyms": ["patrons"]}));
        let props = |root: &OutlineNode, page: &str| {
            child_at(root, &format!("page:{page}"))
                .props
                .clone()
                .unwrap_or_default()
        };

        let written = outline(&doc);
        assert_eq!(props(&written, "overview")["header"], header);
        assert_eq!(props(&written, "overview")["nav"], json!({"label": "Desk"}));

        let shown = rendered(&doc);
        let overview = props(&shown, "overview");
        assert_eq!(overview["header"]["title"], json!("Today at the desk"));
        assert_eq!(
            overview["header"]["help"]["text"],
            json!("What is out and what is due back")
        );
        assert_eq!(
            overview["header"]["actions"][0]["label"],
            json!("Lend a copy")
        );
        assert_eq!(overview["nav"]["label"], json!("Desk"));
        assert_eq!(
            props(&shown, "loans")["header"]["title"],
            json!("Loans"),
            "list_page's header `title: from_page` is the page's title"
        );
        assert_eq!(
            props(&shown, "members")["nav"]["label"],
            json!("Members"),
            "a nav entry without a label is labelled with the page's title"
        );
        assert_eq!(props(&shown, "overview")["shell"], json!("app"));
    }

    /// A section the page removes (`{name: board, remove: true}` on the overview, a
    /// `dashboard_page`) is not rendered, so it is not shown as inherited either.
    #[test]
    fn a_section_the_page_removes_is_not_shown() {
        let root = rendered(&library_with_instance());
        let overview = child_at(&root, "page:overview");
        assert!(
            overview
                .children
                .iter()
                .all(|c| c.path != "page:overview/section:board"),
            "{:?}",
            overview
                .children
                .iter()
                .map(|c| &c.path)
                .collect::<Vec<_>>()
        );
    }
}
