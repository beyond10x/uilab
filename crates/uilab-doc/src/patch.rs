//! Patches: one insert, replace or remove at one node, and whether the result is admissible.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::check::{Finding, Severity, check, replace_drops};
use crate::model::{
    Composite, Document, NavPages, NavSection, Node, NodeBody, Overlay, Page, Region, Shell, Widget,
};
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
    /// Several patches admitted together, in order: what one instruction needs when it touches
    /// more than one node (a drawer and the row action that opens it). The checks run once, on
    /// the result of all of them.
    Batch {
        /// The node the operator pointed at.
        target: NodePath,
        /// The patches; none of them is a batch.
        patches: Vec<Patch>,
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
            Patch::Insert { target, .. }
            | Patch::Replace { target, .. }
            | Patch::Remove { target }
            | Patch::Batch { target, .. } => target,
        }
    }

    /// The operation's name, as the session domain spells it.
    pub fn op_name(&self) -> &'static str {
        match self {
            Patch::Insert { .. } => "Insert",
            Patch::Replace { .. } => "Replace",
            Patch::Remove { .. } => "Remove",
            Patch::Batch { .. } => "Batch",
        }
    }

    /// The path of the node that changed: the new child for an insert; for a batch, the nearest
    /// node that holds every change.
    pub fn changed_path(&self) -> NodePath {
        match self {
            Patch::Insert { target, child } => target.child(child.layer, &child.name),
            Patch::Replace { target, .. } | Patch::Remove { target } => target.clone(),
            Patch::Batch { patches, .. } => {
                let mut paths = patches.iter().map(Patch::changed_path);
                let first = paths.next().unwrap_or_default();
                paths.fold(first, |common, path| {
                    NodePath(
                        common
                            .0
                            .iter()
                            .zip(&path.0)
                            .take_while(|(a, b)| a == b)
                            .map(|(a, _)| a.clone())
                            .collect(),
                    )
                })
            }
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
        Refusal {
            check: check.to_owned(),
            message: message.into(),
        }
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
/// Returns the patched document and all of its findings, followed by a `replace_drops` warning
/// for what each replace, alone or in a batch, removes.
pub fn admit(doc: &Document, patch: &Patch) -> Result<(Document, Vec<Finding>), Refusal> {
    let before = check(doc);
    let mut next = doc.clone();
    apply_unchecked(&mut next, patch)?;
    if next == *doc {
        return Err(Refusal::new(
            "no_change",
            format!(
                "the patch at `{}` leaves the document as it is",
                patch.target()
            ),
        ));
    }
    let after = check(&next);
    if let Some(new) = after
        .iter()
        .find(|f| f.severity == Severity::Error && !before.contains(f))
    {
        return Err(Refusal::new(
            new.check,
            format!("{}: {}", new.path, new.message),
        ));
    }
    let mut findings = after;
    findings.extend(drops(doc, &next, patch));
    Ok((next, findings))
}

/// What the replaces of an admitted patch remove: at each replace target, what the stored
/// document has and the admitted one lacks. A batch is judged by its result, so what a later
/// patch puts back, or what an earlier insert added, is never named. A target under another
/// replaced target is covered by that one's comparison.
fn drops(doc: &Document, next: &Document, patch: &Patch) -> Vec<Finding> {
    let targets: Vec<&NodePath> = match patch {
        Patch::Replace { target, .. } => vec![target],
        Patch::Batch { patches, .. } => patches
            .iter()
            .filter_map(|p| match p {
                Patch::Replace { target, .. } => Some(target),
                _ => None,
            })
            .collect(),
        Patch::Insert { .. } | Patch::Remove { .. } => Vec::new(),
    };
    let mut out = Vec::new();
    for (i, target) in targets.iter().enumerate() {
        let covered = targets.iter().enumerate().any(|(j, other)| {
            (other.0.len() < target.0.len() && target.0.starts_with(&other.0))
                || (j < i && *other == *target)
        });
        if !covered {
            out.extend(replace_drops(doc, next, target));
        }
    }
    out
}

fn parse<T: serde::de::DeserializeOwned>(layer: Layer, node: &Value) -> Result<T, Refusal> {
    serde_json::from_value(node.clone())
        .map_err(|e| Refusal::new("node_shape", format!("not a valid {layer}: {e}")))
}

/// A body node as a patch carries it: the name is the child's name or the target's.
fn parse_node(doc: &Document, name: &str, fields: &Value, own: &str) -> Result<Node, Refusal> {
    let node = Node::named(name, fields)
        .map_err(|e| Refusal::new("node_shape", format!("not a valid node: {e}")))?;
    components_resolve(doc, Some(own), node.composite())?;
    Ok(node)
}

/// An item node as a patch carries it: the name is the child's name or the target's.
fn parse_item(doc: &Document, name: &str, fields: &Value) -> Result<Node, Refusal> {
    let node = Node::named(name, fields)
        .map_err(|e| Refusal::new("node_shape", format!("not a valid item: {e}")))?;
    components_resolve(doc, None, node.composite())?;
    Ok(node)
}

/// Refuses a composite, or one nested in it, whose `component` is neither a composite kind nor a
/// widget of the document. `own` is the widget being written, which its own body may name: the
/// `widget_recursion` check refuses that, with its own id.
fn components_resolve<'a>(
    doc: &Document,
    own: Option<&str>,
    composites: impl IntoIterator<Item = &'a Composite>,
) -> Result<(), Refusal> {
    let mut stack: Vec<&Composite> = composites.into_iter().collect();
    while let Some(composite) = stack.pop() {
        if let Some(name) = composite.component.widget()
            && !doc.widgets.contains_key(name)
            && own != Some(name)
        {
            return Err(Refusal::new(
                "node_shape",
                format!("`{name}` is neither a composite kind nor a declared widget"),
            ));
        }
        stack.extend(composite.widgets.values());
        stack.extend(composite.item_composites());
    }
    Ok(())
}

fn page_composites(page: &Page) -> impl Iterator<Item = &Composite> {
    page.sections
        .values()
        .flatten()
        .chain(page.overlays.values().flatten().map(|o| &o.body))
}

fn widget_composites(widget: &Widget) -> impl Iterator<Item = &Composite> {
    widget.body.iter().filter_map(Node::composite)
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
        Patch::Batch { patches, .. } => {
            if patches.is_empty() {
                return Err(Refusal::new(
                    "batch_shape",
                    "a batch holds at least one patch",
                ));
            }
            for (i, patch) in patches.iter().enumerate() {
                if matches!(patch, Patch::Batch { .. }) {
                    return Err(Refusal::new("batch_shape", "a batch cannot hold a batch"));
                }
                apply_unchecked(doc, patch).map_err(|r| Refusal {
                    check: r.check,
                    message: format!("patch {} of {}: {}", i + 1, patches.len(), r.message),
                })?;
            }
            Ok(())
        }
    }
}

fn insert(doc: &mut Document, target: &NodePath, child: &Child) -> Result<(), Refusal> {
    let allowed =
        allowed_children(doc, target).map_err(|e| Refusal::new("path_resolves", e.to_string()))?;
    if !allowed.contains(&child.layer) {
        return Err(Refusal::new(
            "layer_allowed",
            format!(
                "a {} cannot be added under `{target}`; allowed: {}",
                child.layer,
                list(&allowed)
            ),
        ));
    }
    if !valid_name(&child.name) {
        return Err(Refusal::new(
            "name_valid",
            format!(
                "`{}` is not a name: use a-z, 0-9, `_`, `-` and `.`",
                child.name
            ),
        ));
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
        return Err(Refusal::new(
            "name_unique",
            format!(
                "`{target}` already has a {} named `{}`",
                child.layer, child.name
            ),
        ));
    }

    let name = child.name.clone();
    match child.layer {
        Layer::Shell => {
            let shell: Shell = parse(child.layer, &child.node)?;
            components_resolve(doc, None, shell.overlays.values().map(|o| &o.body))?;
            doc.shells.insert(name, shell);
        }
        Layer::Component => {
            let widget: Widget = parse(child.layer, &child.node)?;
            components_resolve(doc, Some(&name), widget_composites(&widget))?;
            doc.widgets.insert(name, widget);
        }
        Layer::Node => {
            let node = parse_node(doc, &name, &child.node, target.name())?;
            widget_mut(doc, target)?.body.push(node);
        }
        Layer::Page => {
            let page: Page = parse(child.layer, &child.node)?;
            components_resolve(doc, None, page_composites(&page))?;
            match &child.nav_section {
                Some(section) => {
                    let entry = doc
                        .navigation
                        .sections
                        .iter_mut()
                        .find(|s| &s.name == section)
                        .ok_or_else(|| {
                            Refusal::new(
                                "nav_resolves",
                                format!("the menu has no section `{section}`"),
                            )
                        })?;
                    match &mut entry.pages {
                        NavPages::Fixed(pages) => pages.push(name.clone()),
                        NavPages::Dynamic(_) => {
                            return Err(Refusal::new(
                                "nav_resolves",
                                format!(
                                    "menu section `{section}` lists pages from a view and takes no fixed page"
                                ),
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
            doc.navigation.sections.push(NavSection {
                name,
                label: body.label,
                icon: body.icon,
                pages: body.pages,
            });
        }
        Layer::Region => {
            let region: Region = parse(child.layer, &child.node)?;
            shell_mut(doc, target)?.regions.insert(name, region);
        }
        Layer::Overlay => {
            let overlay: Overlay = parse(child.layer, &child.node)?;
            components_resolve(doc, None, [&overlay.body])?;
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
            components_resolve(doc, None, [&section])?;
            page_mut(doc, target)?.sections.insert(name, Some(section));
        }
        Layer::Widget => {
            let widget: Composite = parse(child.layer, &child.node)?;
            components_resolve(doc, None, [&widget])?;
            composite_mut(doc, target)?.widgets.insert(name, widget);
        }
        Layer::Item => {
            let node = parse_item(doc, &name, &child.node)?;
            composite_mut(doc, target)?.item.push(node);
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
            return Err(Refusal::new(
                "op_allowed",
                format!("`{target}` cannot be replaced, only edited below"),
            ));
        }
        Layer::Shell => {
            let shell: Shell = parse(layer, node)?;
            components_resolve(doc, None, shell.overlays.values().map(|o| &o.body))?;
            *doc.shells.get_mut(&name).expect("resolved") = shell;
        }
        Layer::Page => {
            let page: Page = parse(layer, node)?;
            components_resolve(doc, None, page_composites(&page))?;
            *doc.pages.get_mut(&name).expect("resolved") = page;
        }
        Layer::Component => {
            let widget: Widget = parse(layer, node)?;
            components_resolve(doc, Some(&name), widget_composites(&widget))?;
            *doc.widgets.get_mut(&name).expect("resolved") = widget;
        }
        Layer::Node => {
            let parent = target.parent().expect("a node has a widget");
            let replacement = parse_node(doc, &name, node, parent.name())?;
            let slot = widget_mut(doc, &parent)?
                .body
                .iter_mut()
                .find(|n| n.name == name)
                .expect("resolved");
            *slot = replacement;
        }
        Layer::NavSection => {
            let body: NavSectionBody = parse(layer, node)?;
            let entry = doc
                .navigation
                .sections
                .iter_mut()
                .find(|s| s.name == name)
                .expect("resolved");
            *entry = NavSection {
                name,
                label: body.label,
                icon: body.icon,
                pages: body.pages,
            };
        }
        Layer::Region => {
            let parent = target.parent().expect("a region has a shell");
            let region = parse(layer, node)?;
            *shell_mut(doc, &parent)?
                .regions
                .get_mut(&name)
                .expect("resolved") = region;
        }
        Layer::Overlay => {
            let parent = target.parent().expect("an overlay has a parent");
            let overlay: Overlay = parse(layer, node)?;
            components_resolve(doc, None, [&overlay.body])?;
            match parent.layer() {
                Layer::Shell => {
                    *shell_mut(doc, &parent)?
                        .overlays
                        .get_mut(&name)
                        .expect("resolved") = overlay
                }
                _ => {
                    *page_mut(doc, &parent)?
                        .overlays
                        .get_mut(&name)
                        .expect("resolved") = Some(overlay)
                }
            }
        }
        Layer::Section => {
            let parent = target.parent().expect("a section has a page");
            let section: Composite = parse(layer, node)?;
            components_resolve(doc, None, [&section])?;
            *page_mut(doc, &parent)?
                .sections
                .get_mut(&name)
                .expect("resolved") = Some(section);
        }
        Layer::Widget => {
            let parent = target.parent().expect("a nested composite has a parent");
            let composite: Composite = parse(layer, node)?;
            components_resolve(doc, None, [&composite])?;
            *composite_mut(doc, &parent)?
                .widgets
                .get_mut(&name)
                .expect("resolved") = composite;
        }
        Layer::Item => {
            let parent = target.parent().expect("an item has a parent");
            let replacement = parse_item(doc, &name, node)?;
            let slot = composite_mut(doc, &parent)?
                .item
                .iter_mut()
                .find(|n| n.name == name)
                .expect("resolved");
            *slot = replacement;
        }
    }
    Ok(())
}

fn remove(doc: &mut Document, target: &NodePath) -> Result<(), Refusal> {
    let layer = target.layer();
    let name = target.name().to_owned();
    match layer {
        Layer::Root | Layer::Nav => {
            return Err(Refusal::new(
                "op_allowed",
                format!("`{target}` cannot be removed"),
            ));
        }
        Layer::Shell => {
            doc.shells.shift_remove(&name);
        }
        Layer::Component => {
            doc.widgets.shift_remove(&name);
        }
        Layer::Node => {
            let parent = target.parent().expect("a node has a widget");
            widget_mut(doc, &parent)?.body.retain(|n| n.name != name);
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
        Layer::Widget => {
            let parent = target.parent().expect("a nested composite has a parent");
            composite_mut(doc, &parent)?.widgets.shift_remove(&name);
        }
        Layer::Item => {
            let parent = target.parent().expect("an item has a parent");
            let items = &mut composite_mut(doc, &parent)?.item;
            let at = items.iter().position(|n| n.name == name).expect("resolved");
            items.remove(at);
        }
    }
    Ok(())
}

fn missing(path: &NodePath) -> Refusal {
    Refusal::new(
        "path_resolves",
        format!("the document has no node at `{path}`"),
    )
}

fn shell_mut<'a>(doc: &'a mut Document, path: &NodePath) -> Result<&'a mut Shell, Refusal> {
    doc.shells.get_mut(path.name()).ok_or_else(|| missing(path))
}

fn page_mut<'a>(doc: &'a mut Document, path: &NodePath) -> Result<&'a mut Page, Refusal> {
    doc.pages.get_mut(path.name()).ok_or_else(|| missing(path))
}

fn widget_mut<'a>(doc: &'a mut Document, path: &NodePath) -> Result<&'a mut Widget, Refusal> {
    doc.widgets
        .get_mut(path.name())
        .ok_or_else(|| missing(path))
}

/// The composite at a section, overlay, board widget, item or composite body-node path.
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
        (Layer::Component, Layer::Node) => doc
            .widgets
            .get_mut(&first.name)
            .and_then(|w| w.body.iter_mut().find(|n| n.name == second.name))
            .and_then(|n| match &mut n.body {
                NodeBody::Composite(c) => Some(c.as_mut()),
                NodeBody::Primitive(_) => None,
            })
            .ok_or_else(|| missing(path))?,
        _ => return Err(missing(path)),
    };
    for segment in segments {
        current = match segment.layer {
            Layer::Widget => current.widgets.get_mut(&segment.name),
            Layer::Item => current
                .item
                .iter_mut()
                .find(|n| n.name == segment.name)
                .and_then(|n| match &mut n.body {
                    NodeBody::Composite(c) => Some(c.as_mut()),
                    NodeBody::Primitive(_) => None,
                }),
            _ => None,
        }
        .ok_or_else(|| missing(path))?;
    }
    Ok(current)
}

/// Whether `name` can name a node: lower-case letters, digits, `_`, `-` and `.`, starting with a letter.
pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.'))
}

fn list(layers: &[Layer]) -> String {
    if layers.is_empty() {
        return "nothing".into();
    }
    layers
        .iter()
        .map(|l| l.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
