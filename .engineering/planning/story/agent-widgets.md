---
format: aep.planning-md/3
id: story:agent-widgets
kind: story
status: active
title: The agent builds and uses widgets
relations:
- decomposes: epic:component-library
- serves: vision:website-harness
- depends_on: story:widget-model
scope:
- confidence: cited
  path: crates/uilab-agent/src/lib.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T04:02:56Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T04:02:56Z", actor: "human:timo", revision: 4}
---
## Outcome

The agent creates and edits widgets by voice and uses them in pages: the prompt explains widgets, params, args, primitives and when to make a widget instead of repeating composites.

## Acceptance

`cargo test -p uilab-agent` passes with a scripted case where an answer inserting a widget and an instance of it is admitted; one live check creates a widget from "make a reusable card for a member with name and standing and use it in the members table".
