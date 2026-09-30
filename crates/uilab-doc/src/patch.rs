//! Patches: one insert, replace or remove at one node, and whether the result is admissible.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::check::{Finding, Severity, check};
use crate::model::{Composite, Document, NavPages, NavSection, Overlay, Page, Region, Shell};
use crate::path::{Layer, NodePath, allowed_children, children, resolve};

/// One change at one node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Patch {
    /// Adds a child under `target`.
    Insert {
        /// Path of the parent.
        target: NodePath,
        /// The new child.
        child: Child,
    },
    /// Replaces the node at `target`, keeping its name and place.
    Replace {
        /// Path of the node.
        target: NodePath,
        /// The new node, in the shape of the target's layer.
        node: Value,
    },
    /// Removes the node at `target`.
    Remove {
        /// Path of the node.
        target: NodePath,
    },
}

/// A node inserted by a patch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Child {
    /// Layer of the new node.
    pub layer: Layer,
    /// Its name, unique among its siblings of that layer.
    pub name: String,
    /// The node, in the shape of its layer.
    pub node: Value,
    /// For a page: the menu section to list it in. Without one the page is listed as hidden.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nav_section: Option<String>,
}

impl Patch {
    /// The node the patch acts on: the parent for an insert.
    pub fn target(&self) -> &NodePath {
        match self {
            Patch::Insert { target, .. } | Patch::Replace { target, .. } | Patch::Remove { target } => target,
        }
    }

    /// The operation's name, as the session domain spells it.
    pub fn op_name(&self) -> &'static str {
        match self {
            Patch::Insert { .. } => "Insert",
            Patch::Replace { .. } => "Replace",
            Patch::Remove { .. } => "Remove",
        }
    }

    /// The path of the node that changed: the new child for an insert.
    pub fn changed_path(&self) -> NodePath {
        match self {
            Patch::Insert { target, child } => target.child(child.layer, &child.name),
            Patch::Replace { target, .. } | Patch::Remove { target } => target.clone(),
        }
    }
}

/// Why a patch cannot be applied or would leave the document failing a check.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{check}: {message}")]
pub struct Refusal {
    /// Id of the check or rule that refuses it.
    pub check: String,
    /// What is wrong, naming the node.
    pub message: String,
}

impl Refusal {
    fn new(check: &str, message: impl Into<String>) -> Self {
        Refusal { check: check.to_owned(), message: message.into() }
    }
}

/// Applies the patch in place. The document is unchanged when it returns an error.
pub fn apply(doc: &mut Document, patch: &Patch) -> Result<(), Refusal> {
    let mut next = doc.clone();
    apply_unchecked(&mut next, patch)?;
    *doc = next;
    Ok(())
}

/// Applies the patch to a copy and runs every check on the result.
///
/// Refused when the patch does not apply, or when the result has an error finding the document
/// did not have before; a document that already fails a check can still be edited elsewhere.
/// Returns the patched document and all of its findings.
pub fn admit(doc: &Document, patch: &Patch) -> Result<(Document, Vec<Finding>), Refusal> {
    let before = check(doc);
    let mut next = doc.clone();
    apply_unchecked(&mut next, patch)?;
    let after = check(&next);
    if let Some(new) = after.iter().find(|f| f.severity == Severity::Error && !before.contains(f)) {
        return Err(Refusal::new(new.check, format!("{}: {}", new.path, new.message)));
    }
    Ok((next, after))
}

fn parse<T: serde::de::DeserializeOwned>(layer: Layer, node: &Value) -> Result<T, Refusal> {
    serde_json::from_value(node.clone())
        .map_err(|e| Refusal::new("node_shape", format!("not a valid {layer}: {e}")))
}

/// A menu section as a patch carries it: the name is the child's name or the target's.
#[derive(Deserialize)]
struct NavSectionBody {
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default = "no_pages")]
    pages: NavPages,
}

fn no_pages() -> NavPages {
    NavPages::Fixed(Vec::new())
}

fn apply_unchecked(doc: &mut Document, patch: &Patch) -> Result<(), Refusal> {
    let target = patch.target();
    resolve(doc, target).map_err(|e| Refusal::new("path_resolves", e.to_string()))?;
    match patch {
        Patch::Insert { target, child } => insert(doc, target, child),
        Patch::Replace { target, node } => replace(doc, target, node),
        Patch::Remove { target } => remove(doc, target),
    }
}

fn insert(doc: &mut Document, target: &NodePath, child: &Child) -> Result<(), Refusal> {
    let allowed = allowed_children(doc, target).map_err(|e| Refusal::new("path_resolves", e.to_string()))?;
    if !allowed.contains(&child.layer) {
        return Err(Refusal::new(
            "layer_allowed",
            format!("a {} cannot be added under `{target}`; allowed: {}", child.layer, list(&allowed)),
        ));
    }
    if !valid_name(&child.name) {
        return Err(Refusal::new("name_valid", format!("`{}` is not a name: use a-z, 0-9, `_`, `-` and `.`", child.name)));
    }
    let taken = children(doc, target)
        .map_err(|e| Refusal::new("path_resolves", e.to_string()))?
        .into_iter()
        .any(|(layer, name)| layer == child.layer && name == child.name);
    // A page section or overlay set to `null` still occupies its name.
    let nulled = match resolve(doc, target) {
        Ok(crate::path::NodeRef::Page(p)) => match child.layer {
            Layer::Section => p.sections.contains_key(&child.name),
            Layer::Overlay => p.overlays.contains_key(&child.name),
            _ => false,
        },
        _ => false,
    };
    if taken || nulled {
        return Err(Refusal::new("name_unique", format!("`{target}` already has a {} named `{}`", child.layer, child.name)));
    }

    let name = child.name.clone();
    match child.layer {
        Layer::Shell => {
            doc.shells.insert(name, parse::<Shell>(child.layer, &child.node)?);
        }
        Layer::Page => {
            let page: Page = parse(child.layer, &child.node)?;
            match &child.nav_section {
                Some(section) => {
                    let entry = doc.navigation.sections.iter_mut().find(|s| &s.name == section).ok_or_else(|| {
                        Refusal::new("nav_resolves", format!("the menu has no section `{section}`"))
                    })?;
                    match &mut entry.pages {
                        NavPages::Fixed(pages) => pages.push(name.clone()),
                        NavPages::Dynamic(_) => {
                            return Err(Refusal::new(
                                "nav_resolves",
                                format!("menu section `{section}` lists pages from a view and takes no fixed page"),
                            ));
                        }
                    }
                }
                None => doc.navigation.hidden.push(name.clone()),
            }
            doc.pages.insert(name, page);
        }
        Layer::NavSection => {
            let body: NavSectionBody = parse(child.layer, &child.node)?;
            doc.navigation.sections.push(NavSection { name, label: body.label, icon: body.icon, pages: body.pages });
        }
        Layer::Region => {
            let region: Region = parse(child.layer, &child.node)?;
            shell_mut(doc, target)?.regions.insert(name, region);
        }
        Layer::Overlay => {
            let overlay: Overlay = parse(child.layer, &child.node)?;
            match target.layer() {
                Layer::Shell => {
                    shell_mut(doc, target)?.overlays.insert(name, overlay);
                }
                _ => {
                    page_mut(doc, target)?.overlays.insert(name, Some(overlay));
                }
            }
        }
        Layer::Section => {
            let section: Composite = parse(child.layer, &child.node)?;
            page_mut(doc, target)?.sections.insert(name, Some(section));
        }
        Layer::Widget => {
            let widget: Composite = parse(child.layer, &child.node)?;
            composite_mut(doc, target)?.widgets.insert(name, widget);
        }
        Layer::Item => {
            let item: Composite = parse(child.layer, &child.node)?;
            composite_mut(doc, target)?.item.insert(name, item);
        }
        Layer::Root | Layer::Nav => unreachable!("never an allowed child"),
    }
    Ok(())
}

fn replace(doc: &mut Document, target: &NodePath, node: &Value) -> Result<(), Refusal> {
    let layer = target.layer();
    let name = target.name().to_owned();
    match layer {
        Layer::Root | Layer::Nav => {
            return Err(Refusal::new("op_allowed", format!("`{target}` cannot be replaced, only edited below")));
        }
        Layer::Shell => *doc.shells.get_mut(&name).expect("resolved") = parse(layer, node)?,
        Layer::Page => *doc.pages.get_mut(&name).expect("resolved") = parse(layer, node)?,
        Layer::NavSection => {
            let body: NavSectionBody = parse(layer, node)?;
            let entry = doc.navigation.sections.iter_mut().find(|s| s.name == name).expect("resolved");
            *entry = NavSection { name, label: body.label, icon: body.icon, pages: body.pages };
        }
        Layer::Region => {
            let parent = target.parent().expect("a region has a shell");
            let region = parse(layer, node)?;
            *shell_mut(doc, &parent)?.regions.get_mut(&name).expect("resolved") = region;
        }
        Layer::Overlay => {
            let parent = target.parent().expect("an overlay has a parent");
            let overlay: Overlay = parse(layer, node)?;
            match parent.layer() {
                Layer::Shell => *shell_mut(doc, &parent)?.overlays.get_mut(&name).expect("resolved") = overlay,
                _ => *page_mut(doc, &parent)?.overlays.get_mut(&name).expect("resolved") = Some(overlay),
            }
        }
        Layer::Section => {
            let parent = target.parent().expect("a section has a page");
            let section = parse(layer, node)?;
            *page_mut(doc, &parent)?.sections.get_mut(&name).expect("resolved") = Some(section);
        }
        Layer::Widget | Layer::Item => {
            let parent = target.parent().expect("a nested composite has a parent");
            let composite = parse(layer, node)?;
            let holder = composite_mut(doc, &parent)?;
            let map = if layer == Layer::Widget { &mut holder.widgets } else { &mut holder.item };
            *map.get_mut(&name).expect("resolved") = composite;
        }
    }
    Ok(())
}

fn remove(doc: &mut Document, target: &NodePath) -> Result<(), Refusal> {
    let layer = target.layer();
    let name = target.name().to_owned();
    match layer {
        Layer::Root | Layer::Nav => {
            return Err(Refusal::new("op_allowed", format!("`{target}` cannot be removed")));
        }
        Layer::Shell => {
            doc.shells.shift_remove(&name);
        }
        Layer::Page => {
            doc.pages.shift_remove(&name);
            doc.navigation.hidden.retain(|p| p != &name);
            for section in &mut doc.navigation.sections {
                if let NavPages::Fixed(pages) = &mut section.pages {
                    pages.retain(|p| p != &name);
                }
            }
        }
        Layer::NavSection => doc.navigation.sections.retain(|s| s.name != name),
        Layer::Region => {
            let parent = target.parent().expect("a region has a shell");
            shell_mut(doc, &parent)?.regions.shift_remove(&name);
        }
        Layer::Overlay => {
            let parent = target.parent().expect("an overlay has a parent");
            match parent.layer() {
                Layer::Shell => {
                    shell_mut(doc, &parent)?.overlays.shift_remove(&name);
                }
                _ => {
                    page_mut(doc, &parent)?.overlays.shift_remove(&name);
                }
            }
        }
        Layer::Section => {
            let parent = target.parent().expect("a section has a page");
            page_mut(doc, &parent)?.sections.shift_remove(&name);
        }
        Layer::Widget | Layer::Item => {
            let parent = target.parent().expect("a nested composite has a parent");
            let holder = composite_mut(doc, &parent)?;
            let map = if layer == Layer::Widget { &mut holder.widgets } else { &mut holder.item };
            map.shift_remove(&name);
        }
    }
    Ok(())
}

fn missing(path: &NodePath) -> Refusal {
    Refusal::new("path_resolves", format!("the document has no node at `{path}`"))
}

fn shell_mut<'a>(doc: &'a mut Document, path: &NodePath) -> Result<&'a mut Shell, Refusal> {
    doc.shells.get_mut(path.name()).ok_or_else(|| missing(path))
}

fn page_mut<'a>(doc: &'a mut Document, path: &NodePath) -> Result<&'a mut Page, Refusal> {
    doc.pages.get_mut(path.name()).ok_or_else(|| missing(path))
}

/// The composite at a section, overlay, widget or item path.
fn composite_mut<'a>(doc: &'a mut Document, path: &NodePath) -> Result<&'a mut Composite, Refusal> {
    let mut segments = path.0.iter();
    let first = segments.next().ok_or_else(|| missing(path))?;
    let second = segments.next().ok_or_else(|| missing(path))?;
    let mut current: &mut Composite = match (first.layer, second.layer) {
        (Layer::Page, Layer::Section) => doc
            .pages
            .get_mut(&first.name)
            .and_then(|p| p.sections.get_mut(&second.name))
            .and_then(Option::as_mut)
            .ok_or_else(|| missing(path))?,
        (Layer::Page, Layer::Overlay) => doc
            .pages
            .get_mut(&first.name)
            .and_then(|p| p.overlays.get_mut(&second.name))
            .and_then(Option::as_mut)
            .map(|o| &mut o.body)
            .ok_or_else(|| missing(path))?,
        (Layer::Shell, Layer::Overlay) => doc
            .shells
            .get_mut(&first.name)
            .and_then(|s| s.overlays.get_mut(&second.name))
            .map(|o| &mut o.body)
            .ok_or_else(|| missing(path))?,
        _ => return Err(missing(path)),
    };
    for segment in segments {
        let map = match segment.layer {
            Layer::Widget => &mut current.widgets,
            Layer::Item => &mut current.item,
            _ => return Err(missing(path)),
        };
        current = map.get_mut(&segment.name).ok_or_else(|| missing(path))?;
    }
    Ok(current)
}

/// Whether `name` can name a node: lower-case letters, digits, `_`, `-` and `.`, starting with a letter.
pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.'))
}

fn list(layers: &[Layer]) -> String {
    if layers.is_empty() {
        return "nothing".into();
    }
    layers.iter().map(|l| l.as_str()).collect::<Vec<_>>().join(", ")
}
