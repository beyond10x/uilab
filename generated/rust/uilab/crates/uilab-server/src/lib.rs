// generated from uilab v1
// model digest 1a75f0421d837e2ee884066ba29e42b368e91a0d2fb46630399bf48ac98fa437
// contract digest 1419f72b14d91b0b3ee89f6e567db8160920e8728d07d1772d1e9b0cc5848690
// do not edit: regenerate with `ess synthesize`

//! The HTTP surface of `uilab` v1, synthesised.
//!
//! One module per component the specification declares is reached over a network, each holding
//! that component's route table, its listener and the two documents it publishes about itself.
//! The routes are the ones the committed `OpenAPI` document declares, from the same mapping, so a
//! path served here and a path published there cannot be two different answers.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent is absent by
//! decision — no framework, no runtime, no second protocol, no concurrency, no authentication —
//! and each absence is argued in the `TARGET.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod http;
pub mod json;
pub mod wire;
pub mod uilab_session;
