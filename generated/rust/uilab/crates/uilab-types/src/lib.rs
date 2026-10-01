// generated from uilab v1
// model digest 8bbec934f18fca6258713253bcfca181cac2a1249bf16684012986ad1b427455
// contract digest c789fcd30e3ffcc51487d00315741eca272a88a53a2be1e3be26a919b30c4bfb
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

pub mod actor;
pub mod behaviour;
pub mod json;
pub mod obligation;
pub mod primitives;
pub mod session;
pub mod wire;
