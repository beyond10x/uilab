---
format: aep.planning-md/3
id: story:help-accuracy
kind: story
status: implemented
title: Help and docs describe uilab as it is
relations:
- decomposes: epic:ux-fidelity
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-doc/src/docs.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T12:08:57Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T12:08:58Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T12:39:37Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

The help and the generated docs describe uilab as it is.

## Found by

- `crates/uilab-doc/src/docs.rs:371`: help says "there is no multi-step goal yet"; goals shipped in wave 2.
- `crates/uilab-doc/src/docs.rs:22`: the placement profile prints through `{:?}` as `Fat`, where the document says `fat`.
- Help lists no key for the Components tab's workspace behaviour (instructions there build widgets).

## Acceptance

- Tests in `crates/uilab-doc/tests/doc.rs` on `help_markdown` (mentions goals and the Components workspace, not "no multi-step goal") and on the docs header (`placement \`fat\``).
