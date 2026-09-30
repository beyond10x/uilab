// generated from uilab v1
// model digest 671836a8392f9ff5e57e7fc2b988468dc076692b9031754b5c33c0622ff876cf
// contract digest 989bc1e79ee0eb257a75b80b6e076c650ce3b2d5d73883df0f19629de0ee462c
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
