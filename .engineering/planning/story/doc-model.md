---
format: aep.planning-md/3
id: story:doc-model
kind: story
status: implemented
title: 'ui-spec/1 subset: model, paths, patches, checks, patch schema'
relations:
- decomposes: epic:voice-editing
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-doc
- confidence: cited
  path: examples
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T03:01:58Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T03:01:58Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T03:03:05Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`crates/uilab-doc` reads and writes the `ui-spec/1` subset uilab edits, addresses every node by a
path keyed by name, applies one insert/replace/remove patch, runs the document checks, and emits the
JSON Schema of a patch at one node.

Written by hand: `ess generate types` (ess 0.44.0) produces `BTreeMap` maps and `deny_unknown_fields`
structs, so it cannot read `ui-spec/1` as written (section order is layout order; composite props are
inline). Reported to the `ess` session as R7; `ess` is moving `ui-spec/1` to lists of named nodes and
inline union tags.

## Acceptance

`cargo test -p uilab-doc` passes, including: both examples pass every check and round-trip; section
order and untyped props survive a round trip; a node path survives a sibling insert; `patch_schema`
offers only the child layers the node can hold; `admit` refuses by check id; each of the 12 checks
fails on its own fixture.
