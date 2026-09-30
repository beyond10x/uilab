// generated from uilab v1
// model digest 1a75f0421d837e2ee884066ba29e42b368e91a0d2fb46630399bf48ac98fa437
// contract digest 1419f72b14d91b0b3ee89f6e567db8160920e8728d07d1772d1e9b0cc5848690
// do not edit: regenerate with `ess synthesize`

//! Semantic types synthesised from the `uilab` specification, v1.
//!
//! A voice-driven editor for one UI document. The operator holds the selection and the decision; the agent holds only the proposal.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent — behaviour, queries,
//! escalations — is listed with reasons in the `PLAN.md` beside this workspace, and every entry
//! there is owed through a typed seam in an `obligations` module here.

// `deny`, not the source workspace's lint set: this crate must hold on its own, and an undocumented
// public item here is an emitter defect worth failing the gate over.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod json;
pub mod obligation;
pub mod primitives;
pub mod session;
pub mod wire;
