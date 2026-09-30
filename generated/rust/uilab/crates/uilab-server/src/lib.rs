// generated from uilab v1
// model digest b49ec22e0519143d410cde9e2fa36346e670414275339df6320419160b60ba3a
// contract digest 382360f755c7a7fe5f6fc4cb540326082dab33d91dbb381976dae34503c8a0e5
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
