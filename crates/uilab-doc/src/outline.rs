//! What the operator sees and what the agent is told about one node.

use serde::Serialize;

use crate::model::{CompositeKind, Document};
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
    /// The view it reads, for a composite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    /// A composite's props other than `component`, `reads`, `widgets` and `item`: what a renderer
    /// needs to draw it (columns, title, from, fields).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Value>,
    /// Its children.
    pub children: Vec<OutlineNode>,
}

/// The whole document as a tree, from the root.
pub fn outline(doc: &Document) -> OutlineNode {
    node(doc, &NodePath::root()).expect("the root resolves")
}

fn node(doc: &Document, path: &NodePath) -> Result<OutlineNode, PathError> {
    let found = resolve(doc, path)?;
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
        _ => found
            .composite()
            .filter(|c| !c.props.is_empty())
            .map(|c| serde_json::Value::Object(c.props.clone().into_iter().collect())),
    };
    let (title, view) = match found {
        NodeRef::Root(d) => (d.title.clone(), None),
        NodeRef::Page(p) => (p.title.clone(), None),
        NodeRef::NavSection(s) => (s.label.clone(), None),
        NodeRef::Overlay(o) => (
            o.body
                .props
                .get("title")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            o.body.reads.as_ref().map(|r| r.view.clone()),
        ),
        NodeRef::Composite(c) => (
            c.props
                .get("title")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            c.reads.as_ref().map(|r| r.view.clone()),
        ),
        _ => (None, None),
    };
    let mut kids = Vec::new();
    for (layer, name) in children(doc, path)? {
        kids.push(node(doc, &path.child(layer, &name))?);
    }
    Ok(OutlineNode {
        path: path.to_string(),
        layer: path.layer(),
        name: path.name().to_owned(),
        kind: found.kind_label(),
        title,
        view,
        props,
        children: kids,
    })
}

/// What an agent needs to know to propose a patch at one node.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NodeContext {
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
    /// Composite kinds a new composite child can be.
    pub composite_kinds: Vec<&'static str>,
    /// Existing children, as `layer:name`.
    pub children: Vec<String>,
    /// The node itself, as YAML.
    pub yaml: String,
}

/// The context of the node at `path`.
pub fn node_context(doc: &Document, path: &NodePath) -> Result<NodeContext, PathError> {
    let found = resolve(doc, path)?;
    let allowed = allowed_children(doc, path)?;
    let composite_kinds = if allowed.iter().any(|l| l.is_composite()) {
        CompositeKind::ALL.iter().map(|k| k.as_str()).collect()
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
