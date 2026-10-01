//! Node paths: where a node sits in a document, keyed by name and never by position.
//!
//! A path is `/`-separated segments of `<layer>:<name>`; the navigation is the single segment
//! `nav`, and the document root is `/`. Adding a sibling never changes an existing path.
//!
//! ```text
//! /
//! shell:app/region:nav
//! nav/nav_section:sales
//! page:loans/section:list/item:status
//! component:loan_card/node:title
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::model::{
    Composite, CompositeKind, Document, NavSection, Navigation, Node, NodeBody, Overlay, Page,
    Primitive, Region, Shell, Widget,
};

/// The layer a node belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// The document itself.
    Root,
    /// An application frame.
    Shell,
    /// A named area of a shell.
    Region,
    /// The menu; there is one.
    Nav,
    /// A group of the menu.
    NavSection,
    /// A route.
    Page,
    /// A region of a page with its own read.
    Section,
    /// A drawer or dialog of a page or shell.
    Overlay,
    /// A composite of a board, per widget kind.
    Widget,
    /// A named node nested in each row of a collection or record: a composite, a widget instance
    /// or a primitive.
    Item,
    /// An app-defined widget declared under the root.
    Component,
    /// One named node of a widget body: a composite, a widget instance or a primitive.
    Node,
    /// One of a section's extra named nodes (`children`).
    Child,
    /// One named node of a form (`parts`).
    Part,
    /// One choice of a filter bar (`choices`).
    Choice,
    /// One named node of a graph editor's toolbar (`toolbar`).
    Tool,
}

impl Layer {
    /// Every layer, in the order uilab lists a node's children.
    pub const ALL: [Layer; 16] = [
        Layer::Root,
        Layer::Shell,
        Layer::Region,
        Layer::Nav,
        Layer::NavSection,
        Layer::Page,
        Layer::Section,
        Layer::Overlay,
        Layer::Widget,
        Layer::Item,
        Layer::Part,
        Layer::Choice,
        Layer::Tool,
        Layer::Child,
        Layer::Component,
        Layer::Node,
    ];

    /// The name a path spells.
    pub fn as_str(self) -> &'static str {
        match self {
            Layer::Root => "root",
            Layer::Shell => "shell",
            Layer::Region => "region",
            Layer::Nav => "nav",
            Layer::NavSection => "nav_section",
            Layer::Page => "page",
            Layer::Section => "section",
            Layer::Overlay => "overlay",
            Layer::Widget => "widget",
            Layer::Item => "item",
            Layer::Component => "component",
            Layer::Node => "node",
            Layer::Child => "child",
            Layer::Part => "part",
            Layer::Choice => "choice",
            Layer::Tool => "tool",
        }
    }

    fn parse(text: &str) -> Option<Layer> {
        Layer::ALL
            .into_iter()
            .find(|l| *l != Layer::Root && l.as_str() == text)
    }

    /// Whether a node of this layer is, or may be, a composite (and may hold named nodes).
    pub fn is_composite(self) -> bool {
        matches!(
            self,
            Layer::Section
                | Layer::Overlay
                | Layer::Widget
                | Layer::Item
                | Layer::Node
                | Layer::Child
                | Layer::Part
                | Layer::Choice
                | Layer::Tool
        )
    }

    /// Whether nodes of this layer sit in one of a composite's lists of named nodes.
    pub fn is_node_list(self) -> bool {
        matches!(
            self,
            Layer::Item | Layer::Child | Layer::Part | Layer::Choice | Layer::Tool
        )
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One step of a path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Segment {
    /// Layer of the node this step reaches.
    pub layer: Layer,
    /// Its name; empty for `nav`.
    pub name: String,
}

/// Where a node sits in a document.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct NodePath(pub Vec<Segment>);

/// A path that does not parse, or names no node.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PathError {
    /// The text is not a path.
    #[error("`{0}` is not a node path: {1}")]
    Malformed(String, &'static str),
    /// A layer follows a layer it cannot sit under.
    #[error("`{path}`: a {child} cannot sit under a {parent}")]
    Misplaced {
        /// The whole path.
        path: String,
        /// Layer of the parent.
        parent: Layer,
        /// Layer of the child.
        child: Layer,
    },
    /// No node has this path.
    #[error("the document has no node at `{0}`")]
    NotFound(String),
}

impl NodePath {
    /// The document root, `/`.
    pub fn root() -> Self {
        NodePath(Vec::new())
    }

    /// The layer of the node this path reaches.
    pub fn layer(&self) -> Layer {
        self.0.last().map_or(Layer::Root, |s| s.layer)
    }

    /// The name of the node this path reaches; empty for the root and `nav`.
    pub fn name(&self) -> &str {
        self.0.last().map_or("", |s| s.name.as_str())
    }

    /// The path of the parent, or `None` at the root.
    pub fn parent(&self) -> Option<NodePath> {
        let mut segments = self.0.clone();
        segments.pop().map(|_| NodePath(segments))
    }

    /// This path extended by one child.
    pub fn child(&self, layer: Layer, name: &str) -> NodePath {
        let mut segments = self.0.clone();
        segments.push(Segment {
            layer,
            name: if layer == Layer::Nav {
                String::new()
            } else {
                name.to_owned()
            },
        });
        NodePath(segments)
    }

    /// Every path from the root down to this one, root first, this one last.
    pub fn lineage(&self) -> Vec<NodePath> {
        (0..=self.0.len())
            .map(|n| NodePath(self.0[..n].to_vec()))
            .collect()
    }
}

impl fmt::Display for NodePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("/");
        }
        for (i, segment) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str("/")?;
            }
            if segment.layer == Layer::Nav {
                f.write_str("nav")?;
            } else {
                write!(f, "{}:{}", segment.layer, segment.name)?;
            }
        }
        Ok(())
    }
}

impl FromStr for NodePath {
    type Err = PathError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let trimmed = text.trim().trim_matches('/');
        if trimmed.is_empty() {
            return Ok(NodePath::root());
        }
        let mut path = NodePath::root();
        for part in trimmed.split('/') {
            let (layer, name) = if part == "nav" {
                (Layer::Nav, "")
            } else {
                let (layer, name) = part.split_once(':').ok_or(PathError::Malformed(
                    text.to_owned(),
                    "a segment is not `<layer>:<name>`",
                ))?;
                let layer = Layer::parse(layer)
                    .filter(|l| *l != Layer::Nav)
                    .ok_or(PathError::Malformed(text.to_owned(), "unknown layer"))?;
                if name.is_empty() {
                    return Err(PathError::Malformed(
                        text.to_owned(),
                        "a segment has no name",
                    ));
                }
                (layer, name)
            };
            let parent = path.layer();
            if !may_contain(parent, layer) {
                return Err(PathError::Misplaced {
                    path: text.to_owned(),
                    parent,
                    child: layer,
                });
            }
            path = path.child(layer, name);
        }
        Ok(path)
    }
}

impl Serialize for NodePath {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for NodePath {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// Whether a node of layer `child` can sit under one of layer `parent` in some document: the
/// grammar of a path. Which children a node holds also depends on its kind; see
/// [`allowed_children`], which reads that from ESS's schema.
pub fn may_contain(parent: Layer, child: Layer) -> bool {
    use Layer::*;
    match parent {
        Root => matches!(child, Shell | Nav | Page | Component),
        Shell => matches!(child, Region | Overlay),
        Nav => child == NavSection,
        Page => matches!(child, Section | Overlay),
        Section => matches!(child, Widget | Item | Child | Part | Choice | Tool),
        Overlay | Widget | Item | Node | Child | Part | Choice | Tool => {
            matches!(child, Widget | Item | Part | Choice | Tool)
        }
        Component => child == Node,
        Region | NavSection => false,
    }
}

/// A composite's list of named nodes of `layer`: `item`, `children`, `parts`, `choices` or
/// `toolbar`.
pub(crate) fn node_list(composite: &Composite, layer: Layer) -> Option<&Vec<Node>> {
    Some(match layer {
        Layer::Item => &composite.item,
        Layer::Child => &composite.children,
        Layer::Part => &composite.parts,
        Layer::Choice => &composite.choices,
        Layer::Tool => &composite.toolbar,
        _ => return None,
    })
}

/// [`node_list`], to change.
pub(crate) fn node_list_mut(composite: &mut Composite, layer: Layer) -> Option<&mut Vec<Node>> {
    Some(match layer {
        Layer::Item => &mut composite.item,
        Layer::Child => &mut composite.children,
        Layer::Part => &mut composite.parts,
        Layer::Choice => &mut composite.choices,
        Layer::Tool => &mut composite.toolbar,
        _ => return None,
    })
}

/// A borrowed node of a document.
#[derive(Debug, Clone, Copy)]
pub enum NodeRef<'a> {
    /// The document.
    Root(&'a Document),
    /// A shell.
    Shell(&'a Shell),
    /// A region.
    Region(&'a Region),
    /// The navigation.
    Nav(&'a Navigation),
    /// A menu section.
    NavSection(&'a NavSection),
    /// A page.
    Page(&'a Page),
    /// A page overlay or shell overlay.
    Overlay(&'a Overlay),
    /// A section, board widget, item, or a composite or widget instance in a widget body.
    Composite(&'a Composite),
    /// A widget declaration.
    Component(&'a Widget),
    /// A primitive in a widget body or an item list.
    Primitive(&'a Primitive),
}

impl<'a> NodeRef<'a> {
    /// The composite this node renders, for sections, overlays, board widgets, items and
    /// composite nodes of a widget body.
    pub fn composite(self) -> Option<&'a Composite> {
        match self {
            NodeRef::Overlay(o) => Some(&o.body),
            NodeRef::Composite(c) => Some(c),
            _ => None,
        }
    }

    /// A short label: the kind of the node where it has one.
    pub fn kind_label(self) -> String {
        match self {
            NodeRef::Root(_) => "document".into(),
            NodeRef::Shell(_) => "shell".into(),
            NodeRef::Region(r) => r.kind.as_str().into(),
            NodeRef::Nav(_) => "navigation".into(),
            NodeRef::NavSection(_) => "nav_section".into(),
            NodeRef::Page(p) => p.kind.clone(),
            NodeRef::Overlay(o) => {
                format!("{:?} {}", o.kind, o.body.component.as_str()).to_lowercase()
            }
            NodeRef::Composite(c) => c.component.as_str().into(),
            NodeRef::Component(_) => "widget".into(),
            NodeRef::Primitive(p) => p.primitive.as_str().into(),
        }
    }
}

/// Finds the node at `path`.
pub fn resolve<'a>(doc: &'a Document, path: &NodePath) -> Result<NodeRef<'a>, PathError> {
    let not_found = || PathError::NotFound(path.to_string());
    let mut node = NodeRef::Root(doc);
    for segment in &path.0 {
        let name = segment.name.as_str();
        node = match (node, segment.layer) {
            (NodeRef::Root(d), Layer::Shell) => {
                NodeRef::Shell(d.shells.get(name).ok_or_else(not_found)?)
            }
            (NodeRef::Root(d), Layer::Nav) => NodeRef::Nav(&d.navigation),
            (NodeRef::Root(d), Layer::Page) => {
                NodeRef::Page(d.pages.get(name).ok_or_else(not_found)?)
            }
            (NodeRef::Root(d), Layer::Component) => {
                NodeRef::Component(d.widgets.get(name).ok_or_else(not_found)?)
            }
            (NodeRef::Component(w), Layer::Node) => node_ref(w.node(name).ok_or_else(not_found)?),
            (NodeRef::Shell(s), Layer::Region) => {
                NodeRef::Region(s.regions.get(name).ok_or_else(not_found)?)
            }
            (NodeRef::Shell(s), Layer::Overlay) => {
                NodeRef::Overlay(s.overlays.get(name).ok_or_else(not_found)?)
            }
            (NodeRef::Nav(n), Layer::NavSection) => NodeRef::NavSection(
                n.sections
                    .iter()
                    .find(|s| s.name == name)
                    .ok_or_else(not_found)?,
            ),
            (NodeRef::Page(p), Layer::Section) => NodeRef::Composite(
                p.sections
                    .get(name)
                    .and_then(Option::as_ref)
                    .ok_or_else(not_found)?,
            ),
            (NodeRef::Page(p), Layer::Overlay) => NodeRef::Overlay(
                p.overlays
                    .get(name)
                    .and_then(Option::as_ref)
                    .ok_or_else(not_found)?,
            ),
            (parent, Layer::Widget) => node_ref(
                parent
                    .composite()
                    .and_then(|c| c.widgets.get(name))
                    .ok_or_else(not_found)?,
            ),
            (parent, layer) if layer.is_node_list() => node_ref(
                parent
                    .composite()
                    .and_then(|c| node_list(c, layer))
                    .and_then(|nodes| nodes.iter().find(|n| n.name == name))
                    .ok_or_else(not_found)?,
            ),
            _ => return Err(not_found()),
        };
    }
    Ok(node)
}

fn node_ref(node: &Node) -> NodeRef<'_> {
    match &node.body {
        NodeBody::Composite(c) => NodeRef::Composite(c),
        NodeBody::Primitive(p) => NodeRef::Primitive(p),
    }
}

/// The layers a node at `path` can take a new child in, given what the node is, as ESS's schema
/// declares it: each field of the node's construct that holds a list or a map of nodes uilab
/// addresses.
///
/// A composite follows its kind (a `board` holds widgets, a `collection` or a `record` items, a
/// `form` parts, a `filter_bar` choices, a `graph_editor` its toolbar) and a section adds its
/// `children`; a widget instance and a primitive hold nothing an author writes. A widget takes
/// nodes in its body.
pub fn allowed_children(doc: &Document, path: &NodePath) -> Result<Vec<Layer>, PathError> {
    let node = resolve(doc, path)?;
    Ok(match node {
        NodeRef::Root(_)
        | NodeRef::Shell(_)
        | NodeRef::Nav(_)
        | NodeRef::NavSection(_)
        | NodeRef::Page(_)
        | NodeRef::Region(_)
        | NodeRef::Component(_) => crate::ess::layers_under(path.layer()),
        NodeRef::Primitive(_) => vec![],
        NodeRef::Overlay(_) | NodeRef::Composite(_) => {
            let kind = node.composite().and_then(|c| c.component.kind());
            crate::ess::layers_in_composite(
                kind.map(CompositeKind::as_str),
                path.layer() == Layer::Section,
            )
        }
    })
}

/// The names of the node's existing children, per layer, in document order.
pub fn children(doc: &Document, path: &NodePath) -> Result<Vec<(Layer, String)>, PathError> {
    fn names(layer: Layer, keys: Vec<&String>) -> impl Iterator<Item = (Layer, String)> + '_ {
        keys.into_iter().map(move |k| (layer, k.clone()))
    }
    Ok(match resolve(doc, path)? {
        NodeRef::Root(d) => {
            let mut out: Vec<_> = names(Layer::Shell, d.shells.keys().collect()).collect();
            out.push((Layer::Nav, String::new()));
            out.extend(names(Layer::Component, d.widgets.keys().collect()));
            out.extend(names(Layer::Page, d.pages.keys().collect()));
            out
        }
        NodeRef::Component(w) => w
            .body
            .iter()
            .map(|n| (Layer::Node, n.name.clone()))
            .collect(),
        NodeRef::Shell(s) => names(Layer::Region, s.regions.keys().collect())
            .chain(names(Layer::Overlay, s.overlays.keys().collect()))
            .collect(),
        NodeRef::Nav(n) => n
            .sections
            .iter()
            .map(|s| (Layer::NavSection, s.name.clone()))
            .collect(),
        NodeRef::Page(p) => names(
            Layer::Section,
            p.sections
                .iter()
                .filter(|(_, v)| v.is_some())
                .map(|(k, _)| k)
                .collect(),
        )
        .chain(names(
            Layer::Overlay,
            p.overlays
                .iter()
                .filter(|(_, v)| v.is_some())
                .map(|(k, _)| k)
                .collect(),
        ))
        .collect(),
        NodeRef::Region(_) | NodeRef::NavSection(_) | NodeRef::Primitive(_) => vec![],
        node @ (NodeRef::Overlay(_) | NodeRef::Composite(_)) => {
            let c = node
                .composite()
                .expect("overlays and composites carry a composite");
            let mut out: Vec<(Layer, String)> =
                names(Layer::Widget, c.widgets.keys().collect()).collect();
            for layer in Layer::ALL.into_iter().filter(|l| l.is_node_list()) {
                let nodes = node_list(c, layer).into_iter().flatten();
                out.extend(nodes.map(|n| (layer, n.name.clone())));
            }
            out
        }
    })
}
