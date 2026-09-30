// generated from uilab v1
// model digest c1c9563af89cf1f1f8ef7782b80ff93b32b7086e6e86f27333557738d9508f0c
// contract digest 80afaf984fa26a672bfaf0ddb52dc82a4b8cea5b476335aed27257b03acacaab
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
