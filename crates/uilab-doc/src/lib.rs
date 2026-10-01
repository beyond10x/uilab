//! The `ess-ui/1` document as uilab edits it, and nothing else.
//!
//! ESS decides what a document is: [`ess_ui`] loads and expands it and [`ess_ui_check`] checks
//! it, both re-exported here so no other uilab crate depends on them. uilab keeps an editing
//! layer on top:
//!
//! - [`model`]: the authored document, typed where uilab addresses or renders it, with every
//!   other key kept in order; written back as authored, never expanded.
//! - [`path`]: node paths, keyed by name, and what each node may hold (from ESS's schema).
//! - [`ess`]: the bridge to ESS: uilab paths to ESS canonical paths and back, ESS's loader and
//!   checker, findings on uilab nodes.
//! - [`patch`]: one insert, replace or remove, and [`admit`], which refuses a patch whose result
//!   has an error the document did not already have.
//! - [`check`]: ESS's findings and the few checks uilab still runs.
//! - [`schema`]: the JSON Schema of a patch at one node, for an agent's structured output.
//! - [`outline`]: the tree the operator sees and the context the agent is given.
//! - [`fixtures`]: rows per view, so a document renders without a backend.

pub use ess_ui;
pub use ess_ui_check;

pub mod check;
mod cost;
pub mod docs;
pub mod ess;
pub mod fixtures;
pub mod model;
pub mod outline;
pub mod patch;
pub mod path;
pub mod schema;

pub use check::{CHECKS, EXPANSION_LIMIT, Finding, GUARDS, Severity, check};
pub use docs::{docs_markdown, help_markdown};
pub use ess::LoadError;
pub use fixtures::{Fixtures, ViewRows, field_findings, sample_rows};
pub use model::Document;
pub use outline::{
    NodeContext, OutlineNode, node_context, outline, outline_at, vocabulary, yaml_at,
};
pub use patch::{Child, Patch, Refusal, admit, apply};
pub use path::{Layer, NodePath, PathError, allowed_children, resolve};
pub use schema::patch_schema;
