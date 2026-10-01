//! The `ess-ui/1` document as uilab edits it: authored, never expanded.
//!
//! Only what uilab addresses or renders is typed. Every other key a construct carries is kept in
//! its `extra`/`props` map in document order, so a document round-trips through uilab without
//! losing what uilab does not type. Whether a document is one is ESS's to say: [`Document::from_yaml`]
//! runs ESS's loader first, and [`crate::check`] runs ESS's checker.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The format marker every document carries: ESS's.
pub const FORMAT: &str = ess_ui::FORMAT;

/// Built-in page kinds of `ess-ui/1` (`PageKind.builtins`), in the schema's order.
pub const BUILTIN_PAGE_KINDS: [&str; 8] = [
    "list_page",
    "report_page",
    "detail_page",
    "settings_page",
    "dashboard_page",
    "editor_page",
    "form_page",
    "static_page",
];

/// Where a document came from, which is not part of it: the authored YAML, whose key order a
/// write keeps, and the directory its fixture paths are relative to. Two documents that differ
/// only here are equal.
#[derive(Clone, Default)]
pub struct Origin {
    authored: Option<std::sync::Arc<Value>>,
    base: Option<std::path::PathBuf>,
}

impl PartialEq for Origin {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl std::fmt::Debug for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Origin")
    }
}

/// The root of an `ess-ui/1` file describing one application, as authored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Format marker, `ess-ui/1`.
    pub format: String,
    /// Root of every fully qualified name.
    pub app: String,
    /// Human title of the application.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The ESS system views, commands and events resolve against.
    pub model: String,
    /// Default state placement for the whole document.
    pub placement_profile: PlacementProfile,
    /// Sample data so renderers run without a backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixtures: Option<FixtureIndex>,
    /// Application frames.
    pub shells: IndexMap<String, Shell>,
    /// Menu structure and home page.
    pub navigation: Navigation,
    /// App-defined page templates; kept as data, not addressed by uilab.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub page_kinds: IndexMap<String, Value>,
    /// App-defined composites, usable wherever a composite kind is.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub widgets: IndexMap<String, Widget>,
    /// Every route of the app.
    pub pages: IndexMap<String, Page>,
    /// Keys this subset does not type.
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
    /// Where the document came from; never written.
    #[serde(skip)]
    pub origin: Origin,
}

/// Where UI state lives by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementProfile {
    /// State held by the server.
    Thin,
    /// State held by the client.
    Fat,
    /// Decided per page or section.
    Hybrid,
}

/// An application frame.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Shell {
    /// Named areas of the frame.
    pub regions: IndexMap<String, Region>,
    /// Overlays reachable from every page of the shell.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub overlays: IndexMap<String, Overlay>,
    /// Keys this subset does not type (preload, guards, state, source).
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// One named area of a shell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Region {
    /// Role of the region.
    pub kind: RegionKind,
    /// Kind-specific options.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub props: IndexMap<String, Value>,
    /// Keys this subset does not type (visible).
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// Role of a shell region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionKind {
    /// The menu.
    Navigation,
    /// Where the current page renders.
    PageOutlet,
    /// Where drawers and dialogs appear.
    OverlayOutlet,
    /// Toasts.
    Notifications,
    /// A chat helper.
    Assistant,
    /// The signed-in actor's menu.
    AccountMenu,
}

impl RegionKind {
    /// Every region kind, in declaration order.
    pub const ALL: [RegionKind; 6] = [
        RegionKind::Navigation,
        RegionKind::PageOutlet,
        RegionKind::OverlayOutlet,
        RegionKind::Notifications,
        RegionKind::Assistant,
        RegionKind::AccountMenu,
    ];

    /// The name the document spells.
    pub fn as_str(self) -> &'static str {
        match self {
            RegionKind::Navigation => "navigation",
            RegionKind::PageOutlet => "page_outlet",
            RegionKind::OverlayOutlet => "overlay_outlet",
            RegionKind::Notifications => "notifications",
            RegionKind::Assistant => "assistant",
            RegionKind::AccountMenu => "account_menu",
        }
    }
}

/// The menu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Navigation {
    /// Page opened at the app root.
    pub home: String,
    /// Menu groups in display order.
    pub sections: Vec<NavSection>,
    /// Routed pages not shown in the menu.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden: Vec<String>,
    /// Keys this subset does not type (visibility, search, restrictions).
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// One group of the menu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavSection {
    /// Stable id of the section.
    pub name: String,
    /// Heading text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Semantic icon name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Fixed pages, or entries generated from a view.
    pub pages: NavPages,
}

/// The pages of a menu section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NavPages {
    /// A fixed list of page names.
    Fixed(Vec<String>),
    /// Entries generated from the rows of a view (`DynamicNavEntries`), kept as data.
    Dynamic(IndexMap<String, Value>),
}

/// One route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    /// Template the page starts from.
    pub kind: String,
    /// Frame the page renders in; when absent, `shells.app`, else the only shell ([`Document::shell_of`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    /// Header title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Regions of the page by name, in layout order, written as the list of named nodes
    /// `ess-ui/1` declares. `None` is `{name: <n>, remove: true}`, which removes a section the
    /// page kind contributes.
    #[serde(default, skip_serializing_if = "Sections::unwritten")]
    pub sections: Sections,
    /// Drawers and dialogs of the page. `null` removes one inherited from the kind; `overlays:
    /// null` removes them all ([`Overlays`]).
    #[serde(default, skip_serializing_if = "Overlays::unwritten")]
    pub overlays: Overlays,
    /// Keys this subset does not type (nav, params, state, header, switch_to, …).
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// A page's sections by name, in layout order, written as the list of named nodes `ess-ui/1`
/// declares. `None` is `{name: <n>, remove: true}`, which removes a section the page kind
/// contributes. A page that writes no `sections` (it takes every section from its kind) is written
/// back without the key while it has none.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sections {
    map: IndexMap<String, Option<Composite>>,
    written: bool,
}

impl Sections {
    /// Whether the page writes no `sections` and has none: then the key is left out.
    pub fn unwritten(&self) -> bool {
        !self.written && self.map.is_empty()
    }
}

impl std::ops::Deref for Sections {
    type Target = IndexMap<String, Option<Composite>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl std::ops::DerefMut for Sections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl From<IndexMap<String, Option<Composite>>> for Sections {
    fn from(map: IndexMap<String, Option<Composite>>) -> Self {
        Sections { map, written: true }
    }
}

impl<'a> IntoIterator for &'a Sections {
    type Item = (&'a String, &'a Option<Composite>);
    type IntoIter = indexmap::map::Iter<'a, String, Option<Composite>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a> IntoIterator for &'a mut Sections {
    type Item = (&'a String, &'a mut Option<Composite>);
    type IntoIter = indexmap::map::IterMut<'a, String, Option<Composite>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

impl Serialize for Sections {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut list = Vec::with_capacity(self.map.len());
        for (name, section) in &self.map {
            let mut entry = serde_json::Map::new();
            entry.insert("name".into(), Value::String(name.clone()));
            match section {
                Some(composite) => match serde_json::to_value(composite) {
                    Ok(Value::Object(fields)) => entry.extend(fields),
                    Ok(_) => {}
                    Err(e) => return Err(serde::ser::Error::custom(e)),
                },
                None => {
                    entry.insert("remove".into(), Value::Bool(true));
                }
            }
            list.push(Value::Object(entry));
        }
        list.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Sections {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let entries = match Value::deserialize(deserializer)? {
            Value::Null => Vec::new(),
            Value::Array(entries) => entries,
            _ => {
                return Err(D::Error::custom(
                    "`sections` is a list of named sections, each `{name: <name>, component: …}`",
                ));
            }
        };
        let mut map = IndexMap::new();
        for entry in entries {
            let Value::Object(mut fields) = entry else {
                return Err(D::Error::custom("a section is a map"));
            };
            let name = match fields.remove("name") {
                Some(Value::String(name)) if !name.is_empty() => name,
                _ => return Err(D::Error::custom("a section carries its `name`")),
            };
            if map.contains_key(&name) {
                return Err(D::Error::custom(format!("two sections are named `{name}`")));
            }
            let removed = fields.len() == 1 && fields.get("remove") == Some(&Value::Bool(true));
            let section = if removed {
                None
            } else {
                Some(
                    serde_json::from_value(Value::Object(fields))
                        .map_err(|e| D::Error::custom(format!("section `{name}`: {e}")))?,
                )
            };
            map.insert(name, section);
        }
        Ok(Sections { map, written: true })
    }
}

/// A page's overlays by name. An overlay set to `null` removes the one the page kind contributes
/// of that name; `overlays: null` removes every overlay the kind contributes
/// (`inheritance.maps.null_value: remove_inherited`), and is written back as `null` while the page
/// adds none.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Overlays {
    map: IndexMap<String, Option<Overlay>>,
    written: Written,
}

/// How a page writes `overlays`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Written {
    /// Not at all.
    #[default]
    Absent,
    /// As `null`.
    Null,
    /// As a map.
    Map,
}

impl Overlays {
    /// Whether the page writes no `overlays` and has none: then the key is left out.
    pub fn unwritten(&self) -> bool {
        self.written == Written::Absent && self.map.is_empty()
    }

    /// Whether the page writes `overlays: null`, removing every overlay its kind contributes.
    pub fn removes_inherited(&self) -> bool {
        self.written == Written::Null
    }
}

impl std::ops::Deref for Overlays {
    type Target = IndexMap<String, Option<Overlay>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl std::ops::DerefMut for Overlays {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl<'a> IntoIterator for &'a Overlays {
    type Item = (&'a String, &'a Option<Overlay>);
    type IntoIter = indexmap::map::Iter<'a, String, Option<Overlay>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a> IntoIterator for &'a mut Overlays {
    type Item = (&'a String, &'a mut Option<Overlay>);
    type IntoIter = indexmap::map::IterMut<'a, String, Option<Overlay>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

impl Serialize for Overlays {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.written == Written::Null && self.map.is_empty() {
            serializer.serialize_none()
        } else {
            self.map.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Overlays {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(
            match Option::<IndexMap<String, Option<Overlay>>>::deserialize(deserializer)? {
                None => Overlays {
                    map: IndexMap::new(),
                    written: Written::Null,
                },
                Some(map) => Overlays {
                    map,
                    written: Written::Map,
                },
            },
        )
    }
}

/// A board's node per widget kind, as `ess-ui/1` writes it: `{map: {key: name, value: Node}}`.
/// A key written `null` removes the widget the page kind gives under that name (ESS's
/// `inheritance.maps.null_value: remove_inherited`); it is no node, so it is kept apart and
/// written back where the author wrote it. Derefs to the nodes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BoardWidgets {
    map: IndexMap<String, Node>,
    /// Names written `null`.
    removed: Vec<String>,
    /// Every key as written, in order: where [`Self::removed`] go when written back.
    order: Vec<String>,
}

impl BoardWidgets {
    /// Whether the board writes no `widgets` at all: then the key is left out.
    pub fn unwritten(&self) -> bool {
        self.map.is_empty() && self.removed.is_empty()
    }

    /// The names written `null`: the page kind's widgets the board removes.
    pub fn removed(&self) -> &[String] {
        &self.removed
    }
}

impl std::ops::Deref for BoardWidgets {
    type Target = IndexMap<String, Node>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl std::ops::DerefMut for BoardWidgets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl From<IndexMap<String, Node>> for BoardWidgets {
    fn from(map: IndexMap<String, Node>) -> Self {
        BoardWidgets {
            order: map.keys().cloned().collect(),
            map,
            removed: Vec::new(),
        }
    }
}

impl<'a> IntoIterator for &'a BoardWidgets {
    type Item = (&'a String, &'a Node);
    type IntoIter = indexmap::map::Iter<'a, String, Node>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a> IntoIterator for &'a mut BoardWidgets {
    type Item = (&'a String, &'a mut Node);
    type IntoIter = indexmap::map::IterMut<'a, String, Node>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

impl Serialize for BoardWidgets {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let node = |node: &Node| {
            let mut fields: serde_json::Map<String, Value> = node.clone().into();
            fields.remove("name");
            Value::Object(fields)
        };
        let mut out = serde_json::Map::new();
        for name in &self.order {
            if let Some(found) = self.map.get(name) {
                out.insert(name.clone(), node(found));
            } else if self.removed.contains(name) {
                out.insert(name.clone(), Value::Null);
            }
        }
        for (name, found) in &self.map {
            if !out.contains_key(name) {
                out.insert(name.clone(), node(found));
            }
        }
        out.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BoardWidgets {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let written =
            Option::<IndexMap<String, Value>>::deserialize(deserializer)?.unwrap_or_default();
        let mut out = BoardWidgets::default();
        for (name, fields) in written {
            out.order.push(name.clone());
            // `k: rich_text` is the Composite shorthand for `k: {component: rich_text}`; a map
            // with neither `component` nor `primitive` refines the widget a page kind gives;
            // `null` removes it.
            let mut fields = match fields {
                Value::Null => {
                    out.removed.push(name);
                    continue;
                }
                Value::String(kind) => {
                    serde_json::Map::from_iter([("component".to_owned(), Value::String(kind))])
                }
                Value::Object(fields) => fields,
                _ => return Err(D::Error::custom(format!("board widget `{name}` is a node"))),
            };
            fields.insert("name".into(), Value::String(name.clone()));
            let node = Node::in_list(fields).map_err(D::Error::custom)?;
            out.map.insert(name, node);
        }
        Ok(out)
    }
}

/// A composite: what a section or an overlay renders, and what nests inside another composite.
///
/// A section is a composite with section keys (`load`, `depends_on`, `states`, `live`) among its
/// props; `ess-ui/1` writes them inline in the same map. A composite whose `component` names a
/// widget is a widget instance, and its `args` prop supplies the widget's params. The named-node
/// lists `ess-ui/1` declares are typed, each a uilab layer: a board's `widgets`, `item`, a
/// section's `children`, a form's `parts`, a filter bar's `choices` and a graph editor's
/// `toolbar`. Which of them a composite may hold is ESS's to say ([`crate::allowed_children`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Composite {
    /// Kind of the composite, or the widget it instantiates.
    #[serde(default, skip_serializing_if = "Component::is_inherited")]
    pub component: Component,
    /// The composite's data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reads: Option<Reads>,
    /// A board's node per widget kind (`{map: {key: name, value: Node}}`): a composite, a widget
    /// instance or a primitive, each named by its key.
    #[serde(default, skip_serializing_if = "BoardWidgets::unwritten")]
    pub widgets: BoardWidgets,
    /// A collection's or record's named nodes per row, in order: composites, widget instances and
    /// primitives. Written as the list `ess-ui/1` declares; the map of name to composite that
    /// older documents carry is read too. A name written twice is kept for ESS's `names_unique`
    /// to report.
    #[serde(
        default,
        deserialize_with = "item_nodes",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub item: Vec<Node>,
    /// A form's named nodes, in order.
    #[serde(
        default,
        deserialize_with = "item_nodes",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub parts: Vec<Node>,
    /// A filter bar's choices, in order.
    #[serde(
        default,
        deserialize_with = "item_nodes",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub choices: Vec<Node>,
    /// A graph editor's nodes above the canvas, left to right.
    #[serde(
        default,
        deserialize_with = "item_nodes",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub toolbar: Vec<Node>,
    /// A section's extra named nodes, rendered after its composite.
    #[serde(
        default,
        deserialize_with = "item_nodes",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub children: Vec<Node>,
    /// Every other prop, in document order.
    #[serde(flatten)]
    pub props: IndexMap<String, Value>,
}

/// A drawer, dialog, fullscreen pane or popover.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Overlay {
    /// Presentation hint; left out where the overlay refines one its page kind contributes, or is
    /// `same_as` another, and takes it from there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<OverlayKind>,
    /// The composite inside the overlay, with its props inline.
    #[serde(flatten)]
    pub body: Composite,
}

/// Presentation of an overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlayKind {
    /// Slides in from a side.
    Drawer,
    /// Modal dialog.
    Dialog,
    /// Takes the whole screen.
    Fullscreen,
    /// Anchored to its opener.
    Popover,
}

/// The composite kinds of `ess-ui/1`: the members of its composite union
/// (`constructs.Composite.union.members`). A page header and an overlay are placed by position,
/// not by `component`, so neither is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositeKind {
    /// Rows of a view.
    Collection,
    /// One row shown as fields.
    Record,
    /// Input bound to a command.
    Form,
    /// A pick from options or a view.
    Choice,
    /// Search, window and filter inputs.
    FilterBar,
    /// A confirmation before a command.
    Confirm,
    /// One number.
    Metric,
    /// A series.
    Chart,
    /// A grid of widgets.
    Board,
    /// Nodes and edges edited as a whole.
    GraphEditor,
    /// Formatted text.
    RichText,
    /// What uses a record.
    References,
}

impl CompositeKind {
    /// Every composite kind, in declaration order.
    pub const ALL: [CompositeKind; 12] = [
        CompositeKind::Collection,
        CompositeKind::Record,
        CompositeKind::Form,
        CompositeKind::Choice,
        CompositeKind::FilterBar,
        CompositeKind::Confirm,
        CompositeKind::Metric,
        CompositeKind::Chart,
        CompositeKind::Board,
        CompositeKind::GraphEditor,
        CompositeKind::RichText,
        CompositeKind::References,
    ];

    /// The name the document spells.
    pub fn as_str(self) -> &'static str {
        match self {
            CompositeKind::Collection => "collection",
            CompositeKind::Record => "record",
            CompositeKind::Form => "form",
            CompositeKind::Choice => "choice",
            CompositeKind::FilterBar => "filter_bar",
            CompositeKind::Confirm => "confirm",
            CompositeKind::Metric => "metric",
            CompositeKind::Chart => "chart",
            CompositeKind::Board => "board",
            CompositeKind::GraphEditor => "graph_editor",
            CompositeKind::RichText => "rich_text",
            CompositeKind::References => "references",
        }
    }

    /// One line on when to use the kind, for people reading help and documentation.
    pub fn summary(self) -> &'static str {
        match self {
            CompositeKind::Collection => {
                "rows of a view as a table, cards, a list or a tree, with columns and row actions"
            }
            CompositeKind::Record => "one row shown as labelled fields",
            CompositeKind::Form => "inputs bound to a command (`does`)",
            CompositeKind::Choice => "a pick from fixed options or from a view",
            CompositeKind::FilterBar => "search, time window and filter inputs above a collection",
            CompositeKind::Confirm => "a confirmation step before a command runs",
            CompositeKind::Metric => "one number (`from` a field of the first row)",
            CompositeKind::Chart => "a series over time or categories (`x`, `series`)",
            CompositeKind::Board => "a grid of widgets, each an ordinary composite",
            CompositeKind::GraphEditor => "nodes and edges edited as a whole",
            CompositeKind::RichText => "formatted text",
            CompositeKind::References => "what uses a record",
        }
    }

    /// The kind a document spells `name`, if it is one.
    pub fn parse(name: &str) -> Option<CompositeKind> {
        CompositeKind::ALL.into_iter().find(|k| k.as_str() == name)
    }
}

/// What a composite's `component` names: a built-in kind, or a widget the document declares.
///
/// A name that is a composite kind is always the kind; any other name is a widget, whether or not
/// the document declares it (ESS's `widget_expands` decides that).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Component {
    /// A member of the composite union.
    Builtin(CompositeKind),
    /// An app-defined widget, by name.
    Widget(String),
    /// Not written: where `ess-ui/1` lets it be left out, a page section that refines the section
    /// its kind contributes, and an overlay that is `same_as` another, take it from there.
    #[default]
    Inherited,
}

impl Component {
    /// The built-in kind, for a built-in.
    pub fn kind(&self) -> Option<CompositeKind> {
        match self {
            Component::Builtin(kind) => Some(*kind),
            Component::Widget(_) | Component::Inherited => None,
        }
    }

    /// The widget's name, for a widget instance.
    pub fn widget(&self) -> Option<&str> {
        match self {
            Component::Builtin(_) | Component::Inherited => None,
            Component::Widget(name) => Some(name),
        }
    }

    /// The name the document spells; `inherited` where it spells none.
    pub fn as_str(&self) -> &str {
        match self {
            Component::Builtin(kind) => kind.as_str(),
            Component::Widget(name) => name,
            Component::Inherited => "inherited",
        }
    }

    /// Whether the document leaves `component` out, to be taken from what the node refines.
    pub fn is_inherited(&self) -> bool {
        matches!(self, Component::Inherited)
    }
}

impl From<CompositeKind> for Component {
    fn from(kind: CompositeKind) -> Self {
        Component::Builtin(kind)
    }
}

impl PartialEq<CompositeKind> for Component {
    fn eq(&self, other: &CompositeKind) -> bool {
        self.kind() == Some(*other)
    }
}

impl Serialize for Component {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        if name.is_empty() {
            return Err(serde::de::Error::custom("`component` is empty"));
        }
        Ok(CompositeKind::parse(&name).map_or(Component::Widget(name), Component::Builtin))
    }
}

impl Composite {
    /// A widget instance's arguments: the `args` prop.
    pub fn args(&self) -> Option<&Value> {
        self.props.get("args")
    }

    /// The item node named `name`: the first, if the name is written twice.
    pub fn item_node(&self, name: &str) -> Option<&Node> {
        self.item.iter().find(|n| n.name == name)
    }

    /// The composites among the item nodes, in order.
    pub fn item_composites(&self) -> impl Iterator<Item = &Composite> {
        self.item.iter().filter_map(Node::composite)
    }
}

/// An `item` as `ui-spec/1` writes it, a list of named nodes, or as older documents wrote it, a
/// map of name to node, in the order written.
fn item_nodes<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<Node>, D::Error> {
    use serde::de::Error;
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(Vec::new()),
        Value::Array(nodes) => nodes
            .into_iter()
            .map(|node| match node {
                Value::Object(map) => Node::in_list(map).map_err(D::Error::custom),
                _ => Err(D::Error::custom("an `item` node is a map")),
            })
            .collect(),
        Value::Object(map) => map
            .iter()
            .map(|(name, fields)| Node::named(name, fields).map_err(D::Error::custom))
            .collect(),
        _ => Err(D::Error::custom(
            "`item` is a list of named nodes, or a map of name to node",
        )),
    }
}

/// An app-defined composite with typed parameters, usable wherever a composite kind is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Widget {
    /// One line shown in pickers and docs.
    pub summary: String,
    /// Longer description for authors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Typed parameters, supplied by an instance's `args`.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub params: IndexMap<String, Param>,
    /// How body nodes are arranged; `column` when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrange: Option<Arrange>,
    /// Named nodes, in order; each may read `args.<param>`.
    #[serde(deserialize_with = "unique_nodes")]
    pub body: Vec<Node>,
    /// Keys this subset does not type.
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

impl Widget {
    /// How the body is arranged.
    pub fn arrangement(&self) -> Arrange {
        self.arrange.unwrap_or(Arrange::Column)
    }

    /// The body node named `name`.
    pub fn node(&self, name: &str) -> Option<&Node> {
        self.body.iter().find(|n| n.name == name)
    }
}

fn unique_nodes<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<Node>, D::Error> {
    let nodes = Vec::<Node>::deserialize(deserializer)?;
    for (i, node) in nodes.iter().enumerate() {
        if nodes[..i].iter().any(|n| n.name == node.name) {
            return Err(serde::de::Error::custom(format!(
                "two nodes of the body are named `{}`",
                node.name
            )));
        }
    }
    Ok(nodes)
}

/// One parameter of a widget.
///
/// What the document writes is kept as written: an explicit `required: false` and an explicit
/// `default: null` survive a round trip, and keys this subset does not type stay in `extra`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    /// Its type: a primitive name, a constructor map or a named type.
    #[serde(rename = "type")]
    pub ty: Value,
    /// Whether every instance must supply it, as written; absent means not required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Value used when an instance does not supply it. `Some(Value::Null)` is an explicit
    /// `default: null`; `None` is no default.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub default: Option<Value>,
    /// Author remark.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Keys this subset does not type, in document order.
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

impl Param {
    /// Whether every instance must supply it.
    pub fn is_required(&self) -> bool {
        self.required.unwrap_or(false)
    }
}

/// A field that is present, whatever its value: `null` included.
fn present<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<Value>, D::Error> {
    Value::deserialize(deserializer).map(Some)
}

/// How a widget arranges its body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arrange {
    /// Side by side.
    Row,
    /// One below the other.
    Column,
    /// In a grid.
    Grid,
}

/// One named node of a widget body or of an `item` list: a composite, a widget instance or a
/// primitive.
///
/// Exactly one of `component` and `primitive` is written; a node in a list carries `name`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    try_from = "serde_json::Map<String, Value>",
    into = "serde_json::Map<String, Value>"
)]
pub struct Node {
    /// Its name, unique among its siblings.
    pub name: String,
    /// What it is.
    pub body: NodeBody,
}

/// What a node is.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeBody {
    /// A built-in composite or a widget instance.
    Composite(Box<Composite>),
    /// A renderer-neutral leaf.
    Primitive(Primitive),
}

impl Node {
    /// The composite, for a composite or a widget instance.
    pub fn composite(&self) -> Option<&Composite> {
        match &self.body {
            NodeBody::Composite(c) => Some(c.as_ref()),
            NodeBody::Primitive(_) => None,
        }
    }

    /// The primitive, for a primitive.
    pub fn primitive(&self) -> Option<&Primitive> {
        match &self.body {
            NodeBody::Composite(_) => None,
            NodeBody::Primitive(p) => Some(p),
        }
    }

    /// Reads a node's fields without its name, as a patch carries them, and names it `name`.
    pub fn named(name: &str, fields: &Value) -> Result<Node, String> {
        let mut map = fields
            .as_object()
            .cloned()
            .ok_or_else(|| "a node is a map".to_owned())?;
        map.insert("name".into(), Value::String(name.to_owned()));
        Node::try_from(map)
    }
}

impl Node {
    /// A node of a composite's named list. An entry with neither `component` nor `primitive`
    /// refines the entry of its name that a page kind contributes (`inheritance.named_lists`,
    /// matched by name) and takes `component` from it; where nothing is inherited, ESS's loader
    /// refuses it.
    pub(crate) fn in_list(map: serde_json::Map<String, Value>) -> Result<Node, String> {
        if map.contains_key("component") || map.contains_key("primitive") {
            return Node::try_from(map);
        }
        let mut map = map;
        let name = match map.remove("name") {
            Some(Value::String(name)) if !name.is_empty() => name,
            Some(_) => return Err("a node's `name` is a non-empty string".into()),
            None => return Err("a node in a list carries `name`".into()),
        };
        let composite = serde_json::from_value(Value::Object(map))
            .map_err(|e| format!("node `{name}`: {e}"))?;
        Ok(Node {
            name,
            body: NodeBody::Composite(composite),
        })
    }
}

impl TryFrom<serde_json::Map<String, Value>> for Node {
    type Error = String;

    fn try_from(mut map: serde_json::Map<String, Value>) -> Result<Self, Self::Error> {
        let name = match map.remove("name") {
            Some(Value::String(name)) if !name.is_empty() => name,
            Some(_) => return Err("a node's `name` is a non-empty string".into()),
            None => return Err("a node in a list carries `name`".into()),
        };
        let body = match (map.contains_key("component"), map.contains_key("primitive")) {
            (true, false) => NodeBody::Composite(
                serde_json::from_value(Value::Object(map))
                    .map_err(|e| format!("node `{name}`: {e}"))?,
            ),
            (false, true) => NodeBody::Primitive(
                serde_json::from_value(Value::Object(map))
                    .map_err(|e| format!("node `{name}`: {e}"))?,
            ),
            (true, true) => {
                return Err(format!(
                    "node `{name}` has both `component` and `primitive`; a node has exactly one"
                ));
            }
            (false, false) => {
                return Err(format!(
                    "node `{name}` has neither `component` nor `primitive`; a node has exactly one"
                ));
            }
        };
        Ok(Node { name, body })
    }
}

impl From<Node> for serde_json::Map<String, Value> {
    fn from(node: Node) -> Self {
        let mut map = serde_json::Map::new();
        map.insert("name".into(), Value::String(node.name));
        let fields = match node.body {
            NodeBody::Composite(c) => serde_json::to_value(c),
            NodeBody::Primitive(p) => serde_json::to_value(p),
        };
        if let Ok(Value::Object(fields)) = fields {
            map.extend(fields);
        }
        map
    }
}

/// A renderer-neutral leaf: a caption, a badge, a button, an image.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Primitive {
    /// Kind of primitive.
    pub primitive: PrimitiveKind,
    /// Its props (`text`, `src`, `action`, `visible`, …), in document order.
    #[serde(flatten)]
    pub props: IndexMap<String, Value>,
}

/// The nine primitive kinds of `ui-spec/1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveKind {
    /// A run of text.
    Text,
    /// A short value in a toned pill.
    Badge,
    /// A semantic icon.
    Icon,
    /// A button that runs one action.
    Button,
    /// Text that navigates.
    Link,
    /// A single free input.
    Input,
    /// An on/off switch.
    Toggle,
    /// An image with alternative text.
    Image,
    /// A visual separator.
    Divider,
}

impl PrimitiveKind {
    /// Every primitive kind, in declaration order.
    pub const ALL: [PrimitiveKind; 9] = [
        PrimitiveKind::Text,
        PrimitiveKind::Badge,
        PrimitiveKind::Icon,
        PrimitiveKind::Button,
        PrimitiveKind::Link,
        PrimitiveKind::Input,
        PrimitiveKind::Toggle,
        PrimitiveKind::Image,
        PrimitiveKind::Divider,
    ];

    /// The name the document spells.
    pub fn as_str(self) -> &'static str {
        match self {
            PrimitiveKind::Text => "text",
            PrimitiveKind::Badge => "badge",
            PrimitiveKind::Icon => "icon",
            PrimitiveKind::Button => "button",
            PrimitiveKind::Link => "link",
            PrimitiveKind::Input => "input",
            PrimitiveKind::Toggle => "toggle",
            PrimitiveKind::Image => "image",
            PrimitiveKind::Divider => "divider",
        }
    }

    /// One line on when to use the kind.
    pub fn summary(self) -> &'static str {
        match self {
            PrimitiveKind::Text => "a caption, heading or formatted value (`text` or `field`)",
            PrimitiveKind::Badge => "a short value in a toned pill (`tone` or `tone_by`)",
            PrimitiveKind::Icon => "a semantic icon with an accessible `label`",
            PrimitiveKind::Button => "a button that runs one `action`",
            PrimitiveKind::Link => "text that opens a page (`to`) or an address (`href`)",
            PrimitiveKind::Input => "one free input bound to state (`binds`)",
            PrimitiveKind::Toggle => "an on/off switch bound to state or running an action",
            PrimitiveKind::Image => "an image with required `alt` text",
            PrimitiveKind::Divider => "a visual separator",
        }
    }
}

/// What a composite reads: an ESS view, or, while the model has no such view yet, a named
/// placeholder answered by a fixture file (`ess-ui/1` `Reads`; exactly one of `view` and
/// `placeholder`, which ESS's loader holds).
///
/// `reads: <view>` is the shorthand `ess-ui/1` declares for `reads: {view: <view>}`
/// (`shorthands.index`, `Reads`); a read written that way is written back that way.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "ReadsRepr", into = "ReadsRepr")]
pub struct Reads {
    /// ESS view name.
    pub view: Option<String>,
    /// A view name not yet bound to the model.
    pub placeholder: Option<String>,
    /// The fixture file answering the placeholder, relative to the document.
    pub fixture: Option<String>,
    /// Keys this subset does not type (params, paging, debounce, …).
    pub extra: IndexMap<String, Value>,
    /// Whether the document writes it as the shorthand `reads: <view>`.
    pub shorthand: bool,
}

/// [`Reads`] as written: the shorthand string, or the map.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum ReadsRepr {
    Shorthand(String),
    Map(ReadsMap),
}

#[derive(Serialize, Deserialize)]
struct ReadsMap {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    view: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fixture: Option<String>,
    #[serde(flatten)]
    extra: IndexMap<String, Value>,
}

impl From<ReadsRepr> for Reads {
    fn from(repr: ReadsRepr) -> Self {
        match repr {
            ReadsRepr::Shorthand(view) => Reads {
                view: Some(view),
                placeholder: None,
                fixture: None,
                extra: IndexMap::new(),
                shorthand: true,
            },
            ReadsRepr::Map(map) => Reads {
                view: map.view,
                placeholder: map.placeholder,
                fixture: map.fixture,
                extra: map.extra,
                shorthand: false,
            },
        }
    }
}

impl From<Reads> for ReadsRepr {
    fn from(reads: Reads) -> Self {
        match reads {
            Reads {
                view: Some(view),
                placeholder: None,
                fixture: None,
                extra,
                shorthand: true,
            } if extra.is_empty() => ReadsRepr::Shorthand(view),
            reads => ReadsRepr::Map(ReadsMap {
                view: reads.view,
                placeholder: reads.placeholder,
                fixture: reads.fixture,
                extra: reads.extra,
            }),
        }
    }
}

impl Reads {
    /// A read of `view`, written as a map.
    pub fn view(view: impl Into<String>) -> Self {
        Reads {
            view: Some(view.into()),
            placeholder: None,
            fixture: None,
            extra: IndexMap::new(),
            shorthand: false,
        }
    }

    /// The name of what is read: the view, or the placeholder.
    pub fn name(&self) -> &str {
        self.view
            .as_deref()
            .or(self.placeholder.as_deref())
            .unwrap_or("")
    }

    /// Whether this read is a placeholder with no model binding yet.
    pub fn is_placeholder(&self) -> bool {
        self.view.is_none() && self.placeholder.is_some()
    }
}

/// Sample data per view (`ess-ui/1` `FixtureIndex`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FixtureIndex {
    /// Fixture directory relative to the document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    /// File, relative to the document, holding the `views` map.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
    /// View to fixture file, relative to `dir`.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub views: IndexMap<String, String>,
    /// Keys this subset does not type (derived, scripts).
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

impl Document {
    /// Reads a document from YAML. ESS's loader decides whether the text is an `ess-ui/1`
    /// document; what it refuses is refused with its path and message.
    ///
    /// A document whose widget expansion would hold more than [`crate::check::EXPANSION_LIMIT`]
    /// maps is refused before ESS sees it (`expansion_bound`), at the ESS path of the use that
    /// weighs most; one ESS does not read within [`crate::ess::deadline`] is refused at `/`.
    pub fn from_yaml(text: &str) -> Result<Self, crate::LoadError> {
        let authored = serde_yaml::from_str::<Value>(text).ok();
        if let Some((at, message)) = authored.as_ref().and_then(crate::check::expansion_of) {
            return Err(crate::LoadError::new(at, message));
        }
        crate::ess::load(text)?;
        let mut doc: Document =
            serde_yaml::from_str(text).map_err(|e| crate::LoadError::new("/", e.to_string()))?;
        doc.origin.authored = authored.map(std::sync::Arc::new);
        Ok(doc)
    }

    /// [`Document::from_yaml`] for a document in `dir`, which its fixture paths are relative to.
    pub fn from_yaml_in(text: &str, dir: &std::path::Path) -> Result<Self, crate::LoadError> {
        let mut doc = Self::from_yaml(text)?;
        doc.origin.base = Some(dir.to_path_buf());
        Ok(doc)
    }

    /// The directory the document's fixture paths are relative to, where it is known.
    pub fn base(&self) -> Option<&std::path::Path> {
        self.origin.base.as_deref()
    }

    /// Writes the document as YAML: what the author wrote, in the order they wrote it, with the
    /// edits; never what a page kind or a widget contributes.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        let json =
            serde_json::to_value(self).map_err(<serde_yaml::Error as serde::ser::Error>::custom)?;
        let json = match &self.origin.authored {
            Some(authored) => in_authored_order(json, authored),
            None => json,
        };
        serde_yaml::to_string(&yaml_of(json))
    }
}

/// `value` with each map's keys in the order `authored` has them, then the keys it adds. List
/// entries are matched by `name`, an entry without one by its place.
fn in_authored_order(value: Value, authored: &Value) -> Value {
    match (value, authored) {
        (Value::Object(mut map), Value::Object(old)) => {
            let mut out = serde_json::Map::new();
            for (key, old_value) in old {
                if let Some(value) = map.remove(key) {
                    out.insert(key.clone(), in_authored_order(value, old_value));
                }
            }
            out.extend(map);
            Value::Object(out)
        }
        (Value::Array(items), Value::Array(old)) => Value::Array(
            items
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    let name = item.get("name").and_then(Value::as_str);
                    let template = match name {
                        Some(name) => old
                            .iter()
                            .find(|o| o.get("name").and_then(Value::as_str) == Some(name)),
                        None => old.get(i).filter(|o| o.get("name").is_none()),
                    };
                    match template {
                        Some(template) => in_authored_order(item, template),
                        None => item,
                    }
                })
                .collect(),
        ),
        // The Composite shorthand (`shorthands.index`): a bare kind where a node goes, as written.
        (Value::Object(map), Value::String(kind))
            if map.len() == 1 && map.get("component") == Some(&Value::String(kind.clone())) =>
        {
            Value::String(kind.clone())
        }
        (value, _) => value,
    }
}

/// Writes a value as YAML through a `serde_json::Value`, so numbers stay numbers whatever features
/// the build unified onto `serde_json`: with `arbitrary_precision` (the generated wire crate turns
/// it on) serde_yaml writes a `serde_json::Number` as a private map instead of a number.
pub fn to_yaml<T: Serialize>(value: &T) -> Result<String, serde_yaml::Error> {
    let json =
        serde_json::to_value(value).map_err(<serde_yaml::Error as serde::ser::Error>::custom)?;
    serde_yaml::to_string(&yaml_of(json))
}

fn yaml_of(value: Value) -> serde_yaml::Value {
    use serde_yaml::Value as Y;
    match value {
        Value::Null => Y::Null,
        Value::Bool(b) => Y::Bool(b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Y::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                Y::Number(u.into())
            } else if let Some(f) = n.as_f64() {
                Y::Number(f.into())
            } else {
                Y::String(n.to_string())
            }
        }
        Value::String(s) => Y::String(s),
        Value::Array(items) => Y::Sequence(items.into_iter().map(yaml_of).collect()),
        Value::Object(map) => Y::Mapping(
            map.into_iter()
                .map(|(k, v)| (Y::String(k), yaml_of(v)))
                .collect(),
        ),
    }
}

impl Document {
    /// The shell a page renders in, as ESS resolves `Page.shell` (`first_present: [shells.app,
    /// only_shell]`): its own, else the shell named `app`, else the only shell; `None` when the
    /// document has several shells and none is `app`.
    pub fn shell_of<'a>(&'a self, page: &'a Page) -> Option<&'a str> {
        page.shell.as_deref().or_else(|| {
            if let Some((name, _)) = self.shells.get_key_value("app") {
                Some(name.as_str())
            } else if self.shells.len() == 1 {
                self.shells.keys().next().map(String::as_str)
            } else {
                None
            }
        })
    }
}
