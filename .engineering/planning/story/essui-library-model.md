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
revision: 11
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

## State (paused 2026-10-21)

Branch `unit/essui-library-model`. beyond10x/ess#286 shipped in ess 0.53.0 (ess CHANGELOG, 0.53.0
section); dependency-blocker:ess-view-read-grant is cleared.

Done, uncommitted gate:

- ess pin 0.48.0 -> 0.67.0: `ESS_VERSION` in `.github/workflows/check.yml`, `requires:` in
  `ess/ess-inputs.yaml` and `examples/library/model/ess-inputs.yaml`, the `ess-ui` and
  `ess-ui-check` git rev in `crates/uilab-doc/Cargo.toml` (4579d80, tag 0.67.0), `Cargo.lock`,
  `AGENTS.md`. `generated/` and `widget/src/generated/` regenerated with ess 0.67.0.
- Model `format: ess/22` (a view grant requires it); `library.staff.Staff` may read
  `loans.All`, `loans.Summary`, `members.All`, `staff.Me`.
- The overview's `params: {size: 5}` on `loans.All` is dropped from the document: ess 0.67.0
  reports it as `read_params`, and a `size` parameter no filter reads is refused as
  `unobservable_fact`. The gap is an `UNMAPPED:` marker on the view in `loans.yaml`.
- `task ui` gains `ess ui check --model`; with ess 0.67.0 `task validate drift ui` passes and the
  `--model` check prints `0 error(s), 0 warning(s)`.
- `crates/uilab-doc/src/outline.rs`: ess-ui 0.67 holds page kinds as `serde_norway` values; the
  `json` helper is generic over `Serialize`. `cargo check --workspace --all-targets
  --no-default-features` passes.
- Tests adjusted: `agent_schema.rs` (ess 0.67.0 declares a section `title`; `subtitle` is the
  undeclared field now), `doc.rs` (the number round-trip puts `size: 5` back in the read).

Open:

- The full `cargo test --workspace --no-fail-fast --no-default-features` has not run after the
  two test edits (the first run failed only `schema_patches_pass_ess`, and stopped there).
- The Acceptance section still names ess 0.48.0 and the document as committed; revise it to
  ess 0.67.0, `format: ess/22`, and the dropped `size` read, then copy `grep -rn 'UNMAPPED:'
  examples/library/model` into `## Unmapped`.
- `task check` (both feature sets), one adversarial pass, PR, merge, release v0.1.6. The
  release's live agent step is a paid model call and needs its own approval.

First step on resume: in this branch's worktree, run the test command above through
`b10x-build-slot` with ess 0.67.0 on `PATH`.
