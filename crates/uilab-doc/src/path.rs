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
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::model::{Composite, CompositeKind, Document, NavSection, Navigation, Overlay, Page, Region, Shell};

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
    /// A composite nested in each row of a collection.
    Item,
}

impl Layer {
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
        }
    }

    fn parse(text: &str) -> Option<Layer> {
        Some(match text {
            "shell" => Layer::Shell,
            "region" => Layer::Region,
            "nav" => Layer::Nav,
            "nav_section" => Layer::NavSection,
            "page" => Layer::Page,
            "section" => Layer::Section,
            "overlay" => Layer::Overlay,
            "widget" => Layer::Widget,
            "item" => Layer::Item,
            _ => return None,
        })
    }

    /// Whether a node of this layer is a composite (and may hold widgets or items).
    pub fn is_composite(self) -> bool {
        matches!(self, Layer::Section | Layer::Overlay | Layer::Widget | Layer::Item)
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
            name: if layer == Layer::Nav { String::new() } else { name.to_owned() },
        });
        NodePath(segments)
    }

    /// Every path from the root down to this one, root first, this one last.
    pub fn lineage(&self) -> Vec<NodePath> {
        (0..=self.0.len()).map(|n| NodePath(self.0[..n].to_vec())).collect()
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
                let (layer, name) = part
                    .split_once(':')
                    .ok_or(PathError::Malformed(text.to_owned(), "a segment is not `<layer>:<name>`"))?;
                let layer = Layer::parse(layer)
                    .filter(|l| *l != Layer::Nav)
                    .ok_or(PathError::Malformed(text.to_owned(), "unknown layer"))?;
                if name.is_empty() {
                    return Err(PathError::Malformed(text.to_owned(), "a segment has no name"));
                }
                (layer, name)
            };
            let parent = path.layer();
            if !may_contain(parent, layer) {
                return Err(PathError::Misplaced { path: text.to_owned(), parent, child: layer });
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

/// Whether a node of layer `child` can sit under one of layer `parent`, by the `layers` table of
/// `ui-spec/1`. Whether a composite holds widgets or items also depends on its kind; see
/// [`allowed_children`].
pub fn may_contain(parent: Layer, child: Layer) -> bool {
    use Layer::*;
    match parent {
        Root => matches!(child, Shell | Nav | Page),
        Shell => matches!(child, Region | Overlay),
        Nav => child == NavSection,
        Page => matches!(child, Section | Overlay),
        Section | Overlay | Widget | Item => matches!(child, Widget | Item),
        Region | NavSection => false,
    }
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
    /// A section, widget or item.
    Composite(&'a Composite),
}

impl<'a> NodeRef<'a> {
    /// The composite this node renders, for sections, overlays, widgets and items.
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
            NodeRef::Overlay(o) => format!("{:?} {}", o.kind, o.body.component.as_str()).to_lowercase(),
            NodeRef::Composite(c) => c.component.as_str().into(),
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
            (NodeRef::Root(d), Layer::Shell) => NodeRef::Shell(d.shells.get(name).ok_or_else(not_found)?),
            (NodeRef::Root(d), Layer::Nav) => NodeRef::Nav(&d.navigation),
            (NodeRef::Root(d), Layer::Page) => NodeRef::Page(d.pages.get(name).ok_or_else(not_found)?),
            (NodeRef::Shell(s), Layer::Region) => NodeRef::Region(s.regions.get(name).ok_or_else(not_found)?),
            (NodeRef::Shell(s), Layer::Overlay) => NodeRef::Overlay(s.overlays.get(name).ok_or_else(not_found)?),
            (NodeRef::Nav(n), Layer::NavSection) => {
                NodeRef::NavSection(n.sections.iter().find(|s| s.name == name).ok_or_else(not_found)?)
            }
            (NodeRef::Page(p), Layer::Section) => {
                NodeRef::Composite(p.sections.get(name).and_then(Option::as_ref).ok_or_else(not_found)?)
            }
            (NodeRef::Page(p), Layer::Overlay) => {
                NodeRef::Overlay(p.overlays.get(name).and_then(Option::as_ref).ok_or_else(not_found)?)
            }
            (parent, Layer::Widget) => {
                NodeRef::Composite(parent.composite().and_then(|c| c.widgets.get(name)).ok_or_else(not_found)?)
            }
            (parent, Layer::Item) => {
                NodeRef::Composite(parent.composite().and_then(|c| c.item.get(name)).ok_or_else(not_found)?)
            }
            _ => return Err(not_found()),
        };
    }
    Ok(node)
}

/// The layers a node at `path` can take a new child in, given what the node is.
///
/// Composites follow their kind: only a `board` holds widgets and only a `collection` holds items.
pub fn allowed_children(doc: &Document, path: &NodePath) -> Result<Vec<Layer>, PathError> {
    let node = resolve(doc, path)?;
    Ok(match node {
        NodeRef::Root(_) => vec![Layer::Shell, Layer::Page],
        NodeRef::Shell(_) => vec![Layer::Region, Layer::Overlay],
        NodeRef::Nav(_) => vec![Layer::NavSection],
        NodeRef::Page(_) => vec![Layer::Section, Layer::Overlay],
        NodeRef::Region(_) | NodeRef::NavSection(_) => vec![],
        NodeRef::Overlay(_) | NodeRef::Composite(_) => match node.composite().map(|c| c.component) {
            Some(CompositeKind::Board) => vec![Layer::Widget],
            Some(CompositeKind::Collection) => vec![Layer::Item],
            _ => vec![],
        },
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
            out.extend(names(Layer::Page, d.pages.keys().collect()));
            out
        }
        NodeRef::Shell(s) => names(Layer::Region, s.regions.keys().collect())
            .chain(names(Layer::Overlay, s.overlays.keys().collect()))
            .collect(),
        NodeRef::Nav(n) => n.sections.iter().map(|s| (Layer::NavSection, s.name.clone())).collect(),
        NodeRef::Page(p) => names(Layer::Section, p.sections.iter().filter(|(_, v)| v.is_some()).map(|(k, _)| k).collect())
            .chain(names(Layer::Overlay, p.overlays.iter().filter(|(_, v)| v.is_some()).map(|(k, _)| k).collect()))
            .collect(),
        NodeRef::Region(_) | NodeRef::NavSection(_) => vec![],
        node @ (NodeRef::Overlay(_) | NodeRef::Composite(_)) => {
            let c = node.composite().expect("overlays and composites carry a composite");
            names(Layer::Widget, c.widgets.keys().collect()).chain(names(Layer::Item, c.item.keys().collect())).collect()
        }
    })
}
