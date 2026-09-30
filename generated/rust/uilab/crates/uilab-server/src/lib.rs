// generated from uilab v1
// model digest 823a0dbdc48dcff7ae64379e3fd12563f56326f07f9fafab80645c14a42e2c24
// contract digest 1f7ac65ed8e829d046658cff8ae43891c3483669061f0d449be07d91d77ac6f9
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
