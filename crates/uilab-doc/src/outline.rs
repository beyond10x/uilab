//! What the operator sees and what the agent is told about one node.
//!
//! [`outline`] is the document as the author wrote it: what the agent is given and what node
//! paths address. [`rendered`] is what ESS renders (epic:ess-ui-adoption D6): the same tree with
//! the sections and overlays each page's kind contributes and the body of every widget instance,
//! each of those nodes marked `inherited`. The browser is shown [`rendered`]; an instruction at an
//! inherited node lands on the nearest node the author wrote ([`authored_at`]).

use std::sync::OnceLock;

use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value;

use crate::check::{Instance, reaches, widget_uses};
use crate::model::{Composite, CompositeKind, Document, Node, NodeBody, Overlay};
use crate::path::{
    Layer, NodePath, NodeRef, PathError, allowed_children, children, node_list, resolve,
};

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
    /// ESS renders the node but the author did not write it: a section or an overlay the page's
    /// kind contributes, or a node of a widget instance's body (at `<instance>/node:<name>`, with
    /// the props the widget declares; the instance's args bind them). Only [`rendered`] holds
    /// such nodes.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub inherited: bool,
    /// Its children.
    pub children: Vec<OutlineNode>,
}

/// The whole document as the author wrote it, from the root.
pub fn outline(doc: &Document) -> OutlineNode {
    Walk::new(doc, false)
        .authored(&NodePath::root())
        .expect("the root resolves")
}

/// The whole document as ESS renders it, from the root: [`outline`] with what each page's kind
/// contributes and the body of each widget instance, those nodes marked `inherited`. Sections are
/// in the order ESS renders them. A document ESS's loader refuses gets no page-kind contributions.
pub fn rendered(doc: &Document) -> OutlineNode {
    Walk::new(doc, true)
        .authored(&NodePath::root())
        .expect("the root resolves")
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

/// The nodes from `root` down to the one at `path`, compared as written with surrounding slashes
/// trimmed; `None` when the tree has no node there.
fn lineage<'o>(root: &'o OutlineNode, path: &str) -> Option<Vec<&'o OutlineNode>> {
    let target = path.trim().trim_matches('/');
    let mut chain = vec![root];
    let mut node = root;
    while node.path.trim_matches('/') != target {
        node = node.children.iter().find(|c| {
            let at = c.path.trim_matches('/');
            target == at || target.strip_prefix(at).is_some_and(|r| r.starts_with('/'))
        })?;
        chain.push(node);
    }
    Some(chain)
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

/// What a page kind contributes, read as uilab's model: its sections and overlays by name.
#[derive(Default)]
struct KindParts {
    sections: IndexMap<String, Composite>,
    overlays: IndexMap<String, Overlay>,
}

impl KindParts {
    /// The kind's `sections` (a list of named sections) and `overlays` (a map), as ESS resolved
    /// them. An entry uilab's model cannot read is left out.
    fn read(sections: Option<&serde_yaml::Value>, overlays: Option<&serde_yaml::Value>) -> Self {
        let json = |v: Option<&serde_yaml::Value>| v.and_then(|v| serde_json::to_value(v).ok());
        let mut parts = KindParts::default();
        for entry in json(sections)
            .as_ref()
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(mut fields) = entry.as_object().cloned() else {
                continue;
            };
            let Some(Value::String(name)) = fields.remove("name") else {
                continue;
            };
            if fields.get("remove") == Some(&Value::Bool(true)) {
                continue;
            }
            if let Ok(composite) = serde_json::from_value::<Composite>(Value::Object(fields)) {
                parts.sections.insert(name, composite);
            }
        }
        for (name, value) in json(overlays)
            .as_ref()
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            if let Ok(overlay) = serde_json::from_value::<Overlay>(value.clone()) {
                parts.overlays.insert(name.clone(), overlay);
            }
        }
        parts
    }
}

/// What ESS's loader renders of a document, as far as the outline needs it.
struct Rendered {
    /// Per page, the names of its sections in the order ESS renders them, and of its overlays.
    pages: IndexMap<String, (Vec<String>, Vec<String>)>,
    /// Per page kind a page uses, what it contributes, resolved over the kinds it extends.
    kinds: IndexMap<String, KindParts>,
}

impl Rendered {
    /// ESS's loader on the document as written; `None` when it refuses the document.
    fn of(doc: &Document) -> Option<Self> {
        let text = doc.to_yaml().ok()?;
        let ess = ess_ui::load_str(&text).ok()?;
        let pages = ess
            .pages
            .iter()
            .map(|(name, page)| {
                let sections = page.sections.iter().map(|s| s.name.clone()).collect();
                let overlays = page.overlays.keys().cloned().collect();
                (name.clone(), (sections, overlays))
            })
            .collect();
        let mut kinds = IndexMap::new();
        for page in doc.pages.values() {
            if kinds.contains_key(&page.kind) {
                continue;
            }
            // A declared kind is resolved over the kinds it extends by the loader; a built-in one
            // is the schema's.
            let parts = match ess.page_kinds.get(&page.kind) {
                Some(kind) => KindParts::read(kind.sections.as_ref(), kind.overlays.as_ref()),
                None => {
                    let builtin =
                        &schema()["constructs"]["PageKind"]["builtins"][page.kind.as_str()];
                    KindParts::read(builtin.get("sections"), builtin.get("overlays"))
                }
            };
            kinds.insert(page.kind.clone(), parts);
        }
        Some(Rendered { pages, kinds })
    }
}

/// One walk of the document: as written, or as ESS renders it.
struct Walk<'a> {
    doc: &'a Document,
    uses: Vec<(NodePath, Instance<'a>)>,
    /// Add every widget instance's body.
    expand: bool,
    /// What ESS renders; `None` for the document as written, or one ESS's loader refuses.
    ess: Option<Rendered>,
}

impl<'a> Walk<'a> {
    fn new(doc: &'a Document, expand: bool) -> Self {
        Walk {
            doc,
            uses: widget_uses(doc),
            expand,
            ess: if expand { Rendered::of(doc) } else { None },
        }
    }

    /// The node the author wrote at `path`, with its children.
    fn authored(&self, path: &NodePath) -> Result<OutlineNode, PathError> {
        let found = resolve(self.doc, path)?;
        let mut kids = Vec::new();
        for (layer, name) in children(self.doc, path)? {
            kids.push(self.authored(&path.child(layer, &name))?);
        }
        if path.layer() == Layer::Page {
            self.inherit(path, &mut kids);
        }
        self.instance_body(path, found, &mut kids, &mut Vec::new());
        Ok(describe(self.doc, &self.uses, path, found, kids, false))
    }

    /// A node the author did not write, with what it holds.
    fn inherited(
        &self,
        path: &NodePath,
        found: NodeRef<'_>,
        within: &mut Vec<String>,
    ) -> OutlineNode {
        let mut kids = Vec::new();
        for (layer, name, child) in held(found) {
            kids.push(self.inherited(&path.child(layer, name), child, within));
        }
        self.instance_body(path, found, &mut kids, within);
        describe(self.doc, &self.uses, path, found, kids, true)
    }

    /// The page's children as ESS renders them: its sections in ESS's order, each the author's
    /// where the author wrote it and the kind's otherwise, then the author's overlays and those
    /// of the kind's the author did not write.
    fn inherit(&self, path: &NodePath, kids: &mut Vec<OutlineNode>) {
        let Some(ess) = &self.ess else { return };
        let (Some((sections, overlays)), Some(page)) =
            (ess.pages.get(path.name()), self.doc.pages.get(path.name()))
        else {
            return;
        };
        let kind = ess.kinds.get(&page.kind);
        let (mut own, mut rest): (Vec<OutlineNode>, Vec<OutlineNode>) = std::mem::take(kids)
            .into_iter()
            .partition(|k| k.layer == Layer::Section);
        let mut out = Vec::new();
        for name in sections {
            if let Some(i) = own.iter().position(|k| &k.name == name) {
                out.push(own.remove(i));
            } else if let Some(section) = kind.and_then(|k| k.sections.get(name)) {
                let at = path.child(Layer::Section, name);
                out.push(self.inherited(&at, NodeRef::Composite(section), &mut Vec::new()));
            }
        }
        out.extend(own);
        for name in overlays {
            if rest
                .iter()
                .any(|k| k.layer == Layer::Overlay && &k.name == name)
            {
                continue;
            }
            if let Some(overlay) = kind.and_then(|k| k.overlays.get(name)) {
                let at = path.child(Layer::Overlay, name);
                rest.push(self.inherited(&at, NodeRef::Overlay(overlay), &mut Vec::new()));
            }
        }
        out.extend(rest);
        *kids = out;
    }

    /// The body of the widget the node at `path` instantiates, as inherited children at
    /// `<path>/node:<name>`. Not in a widget's own declaration, which ESS does not expand, and not
    /// for a widget already being expanded above it.
    fn instance_body(
        &self,
        path: &NodePath,
        found: NodeRef<'_>,
        kids: &mut Vec<OutlineNode>,
        within: &mut Vec<String>,
    ) {
        if !self.expand {
            return;
        }
        let Some(name) = found.composite().and_then(|c| c.component.widget()) else {
            return;
        };
        let declared = path.0.first().is_some_and(|s| s.layer == Layer::Component);
        if declared || within.iter().any(|w| w == name) {
            return;
        }
        let Some(widget) = self.doc.widgets.get(name) else {
            return;
        };
        within.push(name.to_owned());
        for node in &widget.body {
            let at = path.child(Layer::Node, &node.name);
            kids.push(self.inherited(&at, node_ref(node), within));
        }
        within.pop();
    }
}

fn node_ref(node: &Node) -> NodeRef<'_> {
    match &node.body {
        NodeBody::Composite(c) => NodeRef::Composite(c),
        NodeBody::Primitive(p) => NodeRef::Primitive(p),
    }
}

/// The named nodes a composite holds, in the order [`children`] lists them: board widgets, then
/// each list of named nodes.
fn held(found: NodeRef<'_>) -> Vec<(Layer, &str, NodeRef<'_>)> {
    let Some(composite) = found.composite() else {
        return Vec::new();
    };
    let mut out: Vec<(Layer, &str, NodeRef<'_>)> = composite
        .widgets
        .iter()
        .map(|(name, w)| (Layer::Widget, name.as_str(), NodeRef::Composite(w)))
        .collect();
    for layer in Layer::ALL.into_iter().filter(|l| l.is_node_list()) {
        for node in node_list(composite, layer).into_iter().flatten() {
            out.push((layer, node.name.as_str(), node_ref(node)));
        }
    }
    out
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
        NodeRef::Page(p) => doc
            .shell_of(p)
            .map(|shell| serde_json::json!({"shell": shell})),
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
    Walk::new(doc, false).authored(path).ok()
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
