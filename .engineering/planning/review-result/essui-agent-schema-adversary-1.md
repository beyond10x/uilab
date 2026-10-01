---
format: aep.planning-md/3
id: review-result:essui-agent-schema-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:essui-agent-schema
relations:
- reviews: story:essui-agent-schema
revision: 1
---
unit: story:essui-agent-schema at 3b42aa2 (worktree ~/.local/state/worktree/trees/b10x/uilab/uilab-w10-essui-agent-schema, plus 2 untracked test files)
verdict: red (NEEDS-CHANGE)
cases: executed 239→249, red 4
origin: introduced 3 / pre-existing 1 / undecided 1
wrote-outside-worktree: 5 paths under ~/.cache/uilab-wave-w10/essui-agent-schema/adversary-1/ plus the assigned build dir
needs-coordinator: yes (2 findings trace to ESS 0.48.0's YAML schema disagreeing with ESS's own loader, so they may need an ESS issue rather than a uilab fix)

## 1. What I changed

`git --no-pager diff --stat` is empty: no tracked file changed. `git status --short` shows two new, untracked test files and nothing else.
```
?? crates/uilab-agent/tests/adversary_agent_schema.rs
?? crates/uilab-agent/tests/adversary_agent_schema_examples.rs
```
I ran `rustfmt` on those two files only.

## 2. Cases added

Each case was run alone first; the red output below is from that run.

| case | asserts | now |
|---|---|---|
| `sorting_the_loans_table_as_ess_admits_is_valid_against_the_schema` | the pinned eval case `retarget-none` ("sort this table by the due date") answered as a replace with `sort: {by: due}`. uilab admits it, and `ess ui check` reports 0 findings. | red: `the schema refuses it: "allowed" is a required property` |
| `nodes_ess_admits_are_valid_against_their_schema_when_written_back` | an overlay with `visible` and `degrades: {no_drawer: dialog}`, and a nested `text` primitive with `degrades` and `state`. ESS 0.48.0 reports 0 findings on both. Writing either node back unchanged must pass the schema. | red: `page:loans/overlay:edit: Additional properties are not allowed ('visible' was unexpected)`; `item:hint ... is not valid under any of the schemas listed in the 'oneOf' keyword` |
| `a_widget_declared_as_the_prompt_teaches_is_admitted_in_a_batch` | a batch that declares a widget with params written as the prompt teaches them (`{type: …, required: true}`) and then uses it | red: `left: Err("document_loads: component:member_card: missing field `note`")` |
| `an_empty_node_is_refused_by_the_schema_at_the_top_level_too` | `{}` as a section insert, an item insert and a section replace. The schema refuses `{}` inside a batch, so it should refuse it here too. | red: the schema accepts all 3. uilab then refuses them as `document_loads: … a node needs component or primitive` and `node_shape: … node more has exactly one of component and primitive`; the second message is wrong, because the node has neither. |
| `the_prompts_placeholder_example_is_admitted` | the prompt's own `reads: {placeholder: loans.Overdue, fixture: …}` example | green |
| `a_batch_declaring_a_widget_and_using_it_is_admitted` | a batch that declares a widget (with `note`) and uses it | green |
| `an_overlay_same_as_another_without_kind_is_admitted` | `{component: form, same_as: loans.edit}` | green |
| `a_section_refining_an_inherited_one_is_admitted` | `filters` on a `list_page` written as `{reset: true}` | green |
| `every_ess_example_is_valid_against_the_schema` | each of 51 construct `example`s in `ess_ui::SCHEMA` validates against the matching `$defs` entry (schema built over partner-portal) | green |
| `the_schema_grows_linearly_in_declared_widgets` | schema size at `page:loans`: 61,722 bytes with 0 widgets, 84,385 with 10, 106,405 with 20 (about 2.2 KB per widget) | green |

## 3. Suite run (after the cases existed)

Command: `CARGO_TARGET_DIR=~/.cache/b10x-target/uilab-essui-agent-schema CARGO_INCREMENTAL=0 cargo test --no-fail-fast -p uilab-doc -p uilab-agent --no-default-features`

Result: `EXIT=101`, 249 executed, 245 passed, 4 failed. All 4 failures are in `adversary_agent_schema`. Every existing test is green.

The before count (239) comes from this same run: 249 minus the 8 + 2 cases in my two test binaries.

## 4. Findings

| file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|
| crates/uilab-doc/src/schema.rs:398 | `record()` makes every non-`optional` record field required. ESS's loader defaults `Sort.allowed` (ess model.rs:1040), so the schema refuses `sort: {by: due}`, which ESS admits. | NEEDS-CHANGE | introduced (the base schema at e2bd06f was open) | the pinned eval case `retarget-none`, rewritten by this unit |
| crates/uilab-doc/src/schema.rs:718 | Overlay and primitive variants are closed over ESS's YAML field list. ESS's loader also takes `visible`/`degrades` on an overlay and `state`/`degrades` on any node. A document ESS checks clean then has nodes the agent cannot replace. | CONFIRMED | introduced | any hand-written ESS-clean document; `no_drawer` lists `overlay` as the construct it applies to |
| crates/uilab-doc/src/schema.rs:756 | The "refines an inherited node" branch requires nothing, so `{}` is schema-valid wherever there is no tag. That includes items and shell overlays, which no built-in page kind contributes. This contradicts the batch's `minProperties: 1`. | CONFIRMED | introduced | a model answering with an empty node |
| crates/uilab-agent/src/lib.rs:1013 | The prompt teaches params as `{type: …, required: true}` without `note`. ESS requires `note`. Batch nodes are not shape-checked, so the declare-and-use batch the prompt prescribes is refused. | NEEDS-CHANGE | undecided (same text at base; not run there) | every widget the agent declares in a batch |
| crates/uilab-agent/src/lib.rs:998 | Judgement, no test: the prompt teaches metric `from: <field>` (a row field). ESS's note says `from` is "value from a channel field". ESS does not refuse it, because a bare word parses as a literal. | CONFIRMED | pre-existing (the library example at base writes `from: on_loan`) | the metric eval cases |

The first two findings come from ESS's YAML schema disagreeing with ESS's loader. Fixing them inside uilab means listing fields by hand, which conflicts with epic decision D3. The alternative is an ESS issue. That choice is why `needs-coordinator` is yes.

## 5. Attacked and could not break

- All 51 of ESS's construct examples validate against the schema.
- Placeholder reads are admitted even when the fixture file is missing.
- `same_as` overlays without `kind`, and refinement of inherited sections, are both admitted.
- A batch that declares a widget and uses it is admitted.
- The partner-portal fixture is byte-identical to ESS 0.48.0's `examples/partner-portal/ui.yaml`.
- Schema size grows linearly with declared widgets.
- `eval_suite_is_ess_ui` and the four prompt tests can each fail. The pinned cases keep their ids, targets and `expect` (only `retarget-none`'s `say` changed).

## 6. Paths written outside the worktree

- `~/.cache/uilab-wave-w10/essui-agent-schema/adversary-1/ess-ui.schema.yaml`
- `~/.cache/uilab-wave-w10/essui-agent-schema/adversary-1/ess-model.rs`
- `~/.cache/uilab-wave-w10/essui-agent-schema/adversary-1/probe/` (a copy of the library example plus `p2.ui.yaml` and `p3.ui.yaml`)
- `~/.cache/uilab-wave-w10/essui-agent-schema/adversary-1/suite.log`
- `~/.cache/b10x-target/uilab-essui-agent-schema` (the assigned build dir, reused)

I took a session lease on the worktree and have released it.

## 7. Findings block

```findings
[
  {
    "file": "crates/uilab-doc/src/schema.rs",
    "line": 398,
    "category": "contract-drift",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "record() requires Sort.allowed, which ESS's loader defaults, so the answer to the pinned eval case retarget-none (sort by due) that ESS admits with 0 findings is refused by the schema"
  },
  {
    "file": "crates/uilab-doc/src/schema.rs",
    "line": 718,
    "category": "contract-drift",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "overlay and primitive variants are closed over ESS's YAML field list, so nodes ESS 0.48.0 checks clean (overlay visible/degrades, a primitive's state/degrades) cannot be written back by the agent"
  },
  {
    "file": "crates/uilab-doc/src/schema.rs",
    "line": 756,
    "category": "boundary",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "the untagged refinement branch requires nothing, so an empty node {} is schema-valid at the top level (section, item, replace) although a batch refuses it, and the item refusal names the wrong reason"
  },
  {
    "file": "crates/uilab-agent/src/lib.rs",
    "line": 1013,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "undecided",
    "message": "the prompt teaches widget params without the note ESS requires, so the declare-and-use batch it prescribes is refused with missing field note"
  },
  {
    "file": "crates/uilab-agent/src/lib.rs",
    "line": 998,
    "category": "judgement",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "pre-existing",
    "message": "the prompt teaches metric from as a row field while ESS defines it as a channel field expression"
  }
]
```
