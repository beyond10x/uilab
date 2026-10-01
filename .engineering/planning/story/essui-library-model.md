---
format: aep.planning-md/3
id: story:essui-library-model
kind: story
status: active
title: An ESS model for the library example, checked with ess ui check --model
relations:
- decomposes: epic:ess-ui-adoption
- depends_on: story:essui-document
- serves: vision:website-harness
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/uilab-doc/tests/library_model.rs
- confidence: cited
  path: examples/library/model
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T11:11:20Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-10-01T11:11:20Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":5}}}
---
## Outcome

The library example names `model: library`, and no such model exists. This story writes it: an ESS
specification under `examples/library/model/` declaring every view and command the library
document names, so `ess ui check --model` checks the converted document against a real model, and
`task check` holds it there.

## Acceptance

- `ess specify validate --path examples/library/model` prints `valid` (ess 0.48.0, the version
  story:essui-document pins).
- `ess ui check --path examples/library/library.ui.yaml --model examples/library/model` prints
  `0 error(s), 0 warning(s)` on the document story:essui-document committed.
- The model declares the document's reads and does: views `loans.All`, `loans.Summary`,
  `members.All`, `staff.Me`; command `loans.ExtendLoan`; their fields are the keys of the fixture
  rows in `examples/library/fixtures/*.yaml`.
- `model_invents_nothing` in `crates/uilab-doc/tests/library_model.rs` (new file): it reads the
  model's YAML files and asserts (a) every view and command name the model declares occurs in
  `examples/library/library.ui.yaml`, and (b) every field of every view occurs as a key in at least
  one fixture row under `examples/library/fixtures/`. What the model needs and neither source says
  is an `UNMAPPED:` marker in the model file; `grep -rn 'UNMAPPED:' examples/library/model` is
  copied verbatim into this story's `## Unmapped` section by the coordinator before the move.
- `task check` runs `ess ui check` with `--model examples/library/model` for the library document
  and fails on any error; `task check` exits 0.

## Scope

- examples/library/model/** (new)
- crates/uilab-doc/tests/library_model.rs (new)
- Taskfile.yml (cited: the `ess ui check` line story:essui-document adds gains `--model`)

Depends on story:essui-document (it converts the document and adds the Taskfile line).

## Unmapped

Filled by the implementor.
