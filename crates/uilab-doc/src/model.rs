//! The `ui-spec/1` subset uilab reads and writes.
//!
//! Only what uilab addresses, checks or renders is typed. Every other key a construct carries is
//! kept in its `extra`/`props` map in document order, so a document round-trips through uilab
//! without losing what this subset does not understand.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The format marker every document carries.
pub const FORMAT: &str = "ui-spec/1";

/// Built-in page kinds of `ui-spec/1` (`PageKind.builtins`).
pub const BUILTIN_PAGE_KINDS: [&str; 6] = [
    "list_page",
    "report_page",
    "settings_page",
    "dashboard_page",
    "editor_page",
    "form_page",
];

/// The root of a `ui-spec/1` file describing one application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Format marker, `ui-spec/1`.
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
    /// Frame the page renders in; the first shell when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    /// Header title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Regions of the page, in layout order. `null` removes one inherited from the kind.
    #[serde(default)]
    pub sections: IndexMap<String, Option<Composite>>,
    /// Drawers and dialogs of the page. `null` removes one inherited from the kind.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub overlays: IndexMap<String, Option<Overlay>>,
    /// Keys this subset does not type (nav, params, state, header, switch_to, …).
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// A composite: what a section or an overlay renders, and what nests inside another composite.
///
/// A section is a composite with section keys (`load`, `depends_on`, `states`, `live`) among its
/// props; `ui-spec/1` writes them inline in the same map. A composite whose `component` names a
/// widget is a widget instance, and its `args` prop supplies the widget's params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Composite {
    /// Kind of the composite, or the widget it instantiates.
    pub component: Component,
    /// The composite's data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reads: Option<Reads>,
    /// A board's composite per widget kind.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub widgets: IndexMap<String, Composite>,
    /// A collection's nested composites per row.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub item: IndexMap<String, Composite>,
    /// Every other prop, in document order.
    #[serde(flatten)]
    pub props: IndexMap<String, Value>,
}

/// A drawer, dialog, fullscreen pane or popover.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Overlay {
    /// Presentation hint.
    pub kind: OverlayKind,
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

/// The 14 composite kinds of `ui-spec/1`.
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
    /// Title, total, actions.
    Header,
    /// A nested overlay.
    Overlay,
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
    pub const ALL: [CompositeKind; 14] = [
        CompositeKind::Collection,
        CompositeKind::Record,
        CompositeKind::Form,
        CompositeKind::Choice,
        CompositeKind::FilterBar,
        CompositeKind::Header,
        CompositeKind::Overlay,
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
            CompositeKind::Header => "header",
            CompositeKind::Overlay => "overlay",
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
            CompositeKind::Header => "a page's title, total and actions",
            CompositeKind::Overlay => "an overlay opened from inside a composite",
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
/// the document declares it (the `widget_resolves` check decides that).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Component {
    /// A member of the composite union.
    Builtin(CompositeKind),
    /// An app-defined widget, by name.
    Widget(String),
}

impl Component {
    /// The built-in kind, for a built-in.
    pub fn kind(&self) -> Option<CompositeKind> {
        match self {
            Component::Builtin(kind) => Some(*kind),
            Component::Widget(_) => None,
        }
    }

    /// The widget's name, for a widget instance.
    pub fn widget(&self) -> Option<&str> {
        match self {
            Component::Builtin(_) => None,
            Component::Widget(name) => Some(name),
        }
    }

    /// The name the document spells.
    pub fn as_str(&self) -> &str {
        match self {
            Component::Builtin(kind) => kind.as_str(),
            Component::Widget(name) => name,
        }
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

/// One named node of a widget body: a composite, a widget instance or a primitive.
///
/// Exactly one of `component` and `primitive` is written; a node in a list carries `name`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    try_from = "serde_json::Map<String, Value>",
    into = "serde_json::Map<String, Value>"
)]
pub struct Node {
    /// Its name, unique in the body.
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
            _ => {
                return Err(format!(
                    "node `{name}` has exactly one of `component` and `primitive`"
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

/// How a composite reads an ESS view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reads {
    /// ESS view name. A `draft.` prefix marks a placeholder with no model binding yet.
    pub view: String,
    /// Keys this subset does not type (params, paging, debounce, …).
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// The view prefix of a placeholder read (requirement R3 to `ess`, pending).
pub const DRAFT_VIEW_PREFIX: &str = "draft.";

impl Reads {
    /// Whether this read is a placeholder with no model binding yet.
    pub fn is_draft(&self) -> bool {
        self.view.starts_with(DRAFT_VIEW_PREFIX)
    }
}

/// Sample data per view.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FixtureIndex {
    /// Fixture directory relative to the document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    /// File, relative to `dir`, holding the `views` map.
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
    /// Parses a document from YAML.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }

    /// Writes the document as YAML.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        to_yaml(self)
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
    /// The shell a page renders in: its own, or the first shell of the document.
    pub fn shell_of<'a>(&'a self, page: &'a Page) -> Option<&'a str> {
        page.shell
            .as_deref()
            .or_else(|| self.shells.keys().next().map(String::as_str))
    }
}
