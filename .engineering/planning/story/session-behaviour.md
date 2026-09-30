---
format: aep.planning-md/3
id: story:session-behaviour
kind: story
status: implemented
title: Session obligations held to the synthesized conformance suite
relations:
- decomposes: epic:voice-editing
- depends_on: story:doc-model
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-behaviour
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T03:03:06Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T03:03:06Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T03:03:06Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`crates/uilab-behaviour` fills the nine obligations of the generated `uilab-session` component (six
command behaviours, three view queries) over `uilab-doc`, reading and writing document files.

## Acceptance

The conformance suite ESS synthesizes from `ess/` (`generated/suite.json`, 27 scenarios) replays
against the generated port with every scenario passing and no step kind skipped
(`cargo test -p uilab-behaviour --test conformance`). Unit tests cover propose → accept → file saved
→ undo restores, a refused patch, and a stale accept.
