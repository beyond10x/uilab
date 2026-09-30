---
format: aep.planning-md/3
id: story:item-list
kind: story
status: active
title: Collection item as a list of named nodes (ess)
relations:
- decomposes: epic:component-library
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-doc
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T04:03:10Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T04:03:10Z", actor: "human:timo", revision: 4}
---
## Outcome

Collection and record `item` follow ess ui-spec/1 on integrate/ess-ui-1: a list of named nodes (`{list: Node}`), not a map, so documents written to the ess example parse; paths, patches, checks and outline follow.

## Found by

review-result:widget-model-adversary-1 finding 3 (pre-existing at 8eb6405; `crates/uilab-doc/src/model.rs:217`).

## Acceptance

`cargo test -p uilab-doc` passes with the ess example `item: [{name: card, component: <widget>, args}]` parsing and resolving; the map form is read for old documents and written as a list.
