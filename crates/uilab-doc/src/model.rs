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
/// props; `ui-spec/1` writes them inline in the same map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Composite {
    /// Kind of the composite.
    pub component: CompositeKind,
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
