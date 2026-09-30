---
format: aep.planning-md/3
id: story:session-behaviour
kind: story
status: draft
title: Session obligations held to the synthesized conformance suite
relations:
- decomposes: epic:voice-editing
- depends_on: story:doc-model
scope:
- confidence: cited
  path: crates/uilab-behaviour
revision: 2
---
## Outcome

`crates/uilab-behaviour` fills the nine obligations of the generated `uilab-session` component (six
command behaviours, three view queries) over `uilab-doc`, reading and writing document files.

## Acceptance

The conformance suite ESS synthesizes from `ess/` (`generated/suite.json`, 27 scenarios) replays
against the generated port with every scenario passing and no step kind skipped
(`cargo test -p uilab-behaviour --test conformance`). Unit tests cover propose → accept → file saved
→ undo restores, a refused patch, and a stale accept.
