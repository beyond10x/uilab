// generated from uilab v1
// model digest 671836a8392f9ff5e57e7fc2b988468dc076692b9031754b5c33c0622ff876cf
// contract digest 989bc1e79ee0eb257a75b80b6e076c650ce3b2d5d73883df0f19629de0ee462c
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
