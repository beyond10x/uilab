---
format: aep.planning-md/3
id: story:essui-agent-schema
kind: story
status: active
title: The agent's field shapes and prompt come from ESS's schema
relations:
- decomposes: epic:ess-ui-adoption
- depends_on: story:essui-document
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-agent
- confidence: cited
  path: crates/uilab-doc/src/schema.rs
- confidence: cited
  path: crates/uilab-doc/tests/agent_schema.rs
- confidence: cited
  path: crates/uilab-doc/tests/eval_suite.rs
- confidence: cited
  path: evals/library.yaml
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T11:11:20Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-10-01T11:11:20Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":5}}}
---
## Outcome

The JSON Schema the agent's structured output is held to (`crates/uilab-doc/src/schema.rs`) takes
every field shape from ESS's embedded schema, `ess_ui::SCHEMA`: composite kinds and their fields,
primitive kinds and their fields, overlay and region kinds, widget instances and their params. It
builds on the child layers story:essui-document derives from the same schema (`children_from_ess`)
and does not derive them again. No shape is hand-listed in uilab (epic decision D3). The agent's
prompt teaches `ess-ui/1`; this story owns the prompt text; story:eval-round-5 changes it afterwards only for a failure its
round names.

## Acceptance

Tests in `crates/uilab-doc/tests/agent_schema.rs` unless named otherwise.

- `schema_from_ess`: for every composite kind and every primitive kind in `ess_ui::SCHEMA`, the
  patch schema's variant offers exactly the ESS fields (plus `name` and `component`/`primitive`),
  marks ESS's required fields required, and offers no field ESS does not declare. The test reads
  the kinds from `ess_ui::SCHEMA`, so an ESS release that adds a field fails it until uilab follows.
- `schema_patches_pass_ess`: one generated patch per composite and primitive kind, valid against
  the patch schema, is admitted, and the resulting document has 0 `ess-ui-check` errors.
- In `crates/uilab-agent/tests/propose.rs`, one test per prompt outcome:
  `prompt_names_ess_ui` (the prompt says `ess-ui/1` and never `ui-spec/1`),
  `prompt_teaches_placeholder_reads` (it shows `reads: {placeholder, fixture}`),
  `prompt_has_no_draft_views` (no `draft.` view prefix anywhere in the prompt),
  `prompt_offers_no_section_title` (no section `title` in the prompt or the schema it sends).
- `evals/library.yaml` is stated in `ess-ui/1` terms. `eval_suite_is_ess_ui` in
  `crates/uilab-doc/tests/eval_suite.rs` (new file; it reads the YAML directly, since `uilab-app` is
  a binary crate) asserts: every case's `target` resolves in the converted library example; no
  `expect` value contains `draft.`; no case's `say` contains `title` unless its target is a page or
  an overlay.
- The cases `the_library_suite_loads_with_its_move_cases` pins in `crates/uilab-app/src/eval.rs`
  (`retarget-new-page`, `retarget-navigate`, `retarget-none`) keep their ids, targets and `expect`
  fields; only a `say` may change (`retarget-none` asks for a section title today, which `ess-ui/1`
  cannot hold, so its `say` becomes another in-place change to that collection). `eval.rs` is not
  edited by this story.
- `task check` exits 0.

- `batch_patches_carry_nodes` (demo gap, 2026-10-01): a `batch` whose patches hold no node (round 4
  case `creative`: "batch holds no node") is not valid against the patch schema: every insert and
  replace inside a batch requires its `child`/`node`; a test feeds the round-4 answer and asserts
  the schema refuses it, and `schema_patches_pass_ess` covers a valid batch.

## Scope

- crates/uilab-doc/src/schema.rs, crates/uilab-doc/tests/agent_schema.rs (cited)
- crates/uilab-agent/** (cited: prompt in `src/lib.rs`, bins `uilab-plan`, `uilab-propose`, tests)
- evals/library.yaml (cited)
- crates/uilab-doc/tests/eval_suite.rs (new)

No Cargo manifest changes: the ESS crates come through `uilab_doc`'s re-export.

Depends on story:essui-document.
