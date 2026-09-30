//! The `ui-spec/1` subset uilab edits.
//!
//! - [`model`]: the document, typed where uilab addresses, checks or renders it, with every other
//!   key kept in order.
//! - [`path`]: node paths, keyed by name, and what each node may hold.
//! - [`patch`]: one insert, replace or remove, and [`admit`], which refuses a patch whose result
//!   fails a check the document did not already fail.
//! - [`check`]: the document checks.
//! - [`schema`]: the JSON Schema of a patch at one node, for an agent's structured output.
//! - [`outline`]: the tree the operator sees and the context the agent is given.
//! - [`fixtures`]: rows per view, so a document renders without a backend.
//!
//! This is written by hand because `ess generate types` cannot yet read `ui-spec/1` as written
//! (ordered maps, inline composite props). `ess` is moving `ui-spec/1` to lists of named nodes and
//! inline union tags; this crate follows when that ships.

pub mod check;
pub mod fixtures;
pub mod model;
pub mod outline;
pub mod patch;
pub mod path;
pub mod schema;

pub use check::{CHECKS, Finding, Severity, check};
pub use fixtures::{Fixtures, ViewRows, field_findings, sample_rows};
pub use model::Document;
pub use outline::{
    NodeContext, OutlineNode, node_context, outline, outline_at, vocabulary, yaml_at,
};
pub use patch::{Child, Patch, Refusal, admit, apply};
pub use path::{Layer, NodePath, PathError, allowed_children, resolve};
pub use schema::patch_schema;
