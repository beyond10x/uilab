// generated from uilab v1
// model digest c1c9563af89cf1f1f8ef7782b80ff93b32b7086e6e86f27333557738d9508f0c
// contract digest 80afaf984fa26a672bfaf0ddb52dc82a4b8cea5b476335aed27257b03acacaab
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
