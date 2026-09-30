// generated from uilab v1
// model digest b49ec22e0519143d410cde9e2fa36346e670414275339df6320419160b60ba3a
// contract digest 382360f755c7a7fe5f6fc4cb540326082dab33d91dbb381976dae34503c8a0e5
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
