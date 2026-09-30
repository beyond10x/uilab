// generated from uilab v1
// model digest 55369e6e2fd022d062872af44b9c252bf6b80b0b82da849d828e32c4bf780a9f
// contract digest c7e76b5222a53b2b047350c46d9086d71ebcc13314a0ee6ce79ea111afb83928
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
