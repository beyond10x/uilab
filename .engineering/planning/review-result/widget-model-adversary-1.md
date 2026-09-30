---
format: aep.planning-md/3
id: review-result:widget-model-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:widget-model
relations:
- reviews: story:widget-model
revision: 1
---
unit: story:widget-model, branch unit/widget-model, findings cover b22de9d (implementation) plus 5e4fefc (my test file only)
verdict: NEEDS-CHANGE
cases: executed 24→33 (`uilab-doc`), red 5
origin: introduced 4 / pre-existing 1 / undecided 0
wrote-outside-worktree: 2 paths
needs-coordinator: yes. Someone has to decide whether ess `item: [Node]` (a list) belongs to this story or to its own story, because it reproduces at base 8eb6405.

**1. Diff stat** (`git --no-pager diff --stat b22de9d HEAD`)
```
 crates/uilab-doc/tests/adversary_widget_model.rs | 262 +++++++++++++++++++++++
 1 file changed, 262 insertions(+)
```
The only file touched is a test file. The commit author and committer are both `b10x-bot[bot]`.

**2. Cases added** in `crates/uilab-doc/tests/adversary_widget_model.rs`. The red output below was captured when I first ran this file on its own. rustfmt moved lines afterwards; the `:NN` values in the output are from before that.

| test | asserts | now |
|---|---|---|
| `a_param_key_the_subset_does_not_type_is_kept_or_refused` :70 | a param key the model does not type is either refused or kept | red |
| `an_explicit_null_param_default_survives_a_round_trip` :88 | `default: null` survives a round trip | red |
| `an_ess_collection_item_list_with_a_widget_instance_parses_and_resolves` :103 | the ess example `item: [{name: card, component: <widget>, args}]` parses | red |
| `the_agent_context_offers_declared_widgets_where_a_composite_can_go` :120 | `node_context(page).composite_kinds` includes declared widgets | red |
| `removing_a_widget_a_section_child_still_uses_is_refused` :133 | a Remove of a widget that ess section `children: [Node]` still uses is refused | red |
| `a_number_in_a_body_node_round_trips_as_a_number` :156 | with `arbitrary_precision` on, numbers in nodes stay numbers | green |
| `indirect_recursion_through_a_board_node_is_found_at_both_instances` :174 | recursion through a board node is reported at both paths | green |
| `a_board_node_in_a_body_takes_a_widget_instance_by_patch` :207 | insert under `component:w/node:board`, and `widget_args` there | green |
| `replacing_a_widget_to_drop_a_param_its_instance_passes_is_refused` :238 | `widget_args` fires at `component:loan_card/node:state` | green |

Red output, verbatim:
```
the param's `label` was dropped on the round trip:
summary: s
params:
  state:
    type: string
    required: true
body: []
---
`default: null` was dropped on the round trip: ... params: note: type: string
---
the ess item shape parses: Error("pages.overview.sections.list.item: invalid type: sequence, expected a map", line: 41, column: 15)
---
a page's context omits declared widgets: ["collection", "record", "form", "choice", "filter_bar", "header", "overlay", "confirm", "metric", "chart", "board", "graph_editor", "rich_text", "references"]
---
assertion `left == right` failed: a widget still used by a section child was removed
  left: Ok(())
 right: Err("widget_resolves")
test result: FAILED. 4 passed; 5 failed
```

**3. Suite runs, after the cases existed**
- `cargo test -p uilab-doc --no-fail-fast`: the lib tests ran 0 cases. `adversary_widget_model` passed 4 and failed 5. `doc.rs` passed 24 of 24. There were no doc-tests. Exit 101. The "before" count of 24 is `doc.rs` alone, with my file left out.
- `cargo check --workspace --all-targets`: exit 0, no warnings. The change to the `Composite.component` type compiles everywhere.
- `cargo test -p uilab-behaviour -p uilab-agent -p uilab-app --no-fail-fast`: 26 passed, 0 failed, exit 0.
- `cargo fmt -p uilab-doc --check` exited 0, and `cargo clippy -p uilab-doc --all-targets -- -D warnings` passed.
- To settle the origin of the item finding, I rebuilt uilab-doc as it was at base 8eb6405 in scratch and ran a probe with `item: [{name: count, component: metric}]`. It fails with the same `invalid type: sequence, expected a map`, so that finding is pre-existing.

**4. Findings** (tree 5e4fefc)

| # | file:line | finding | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| 1 | `crates/uilab-doc/src/model.rs:470` | `Param` has no `extra` field and no `deny_unknown_fields`, so unknown keys are silently dropped. The module docs promise a lossless round trip, and ess closes this record, which means refusing. | any YAML file or Insert/Replace patch whose param carries an extra key | NEEDS-CHANGE | introduced |
| 2 | `crates/uilab-doc/src/model.rs:478` | `default: Option<Value>` reads `null` as absent, so `default: null` is lost. ess types this field as `{optional: json}`. | a param written `default: null` | CONFIRMED | introduced |
| 3 | `crates/uilab-doc/src/model.rs:217` | `Composite.item` is a map. On integrate/ess-ui-1, `collection.item` and `record.item` are `{list: Node}`, and the ess example of an instance in an item is exactly the list form. The unit's acceptance only holds for uilab's own map form. | any document written to the ess example | CONFIRMED | pre-existing |
| 4 | `crates/uilab-doc/src/outline.rs:129` | `composite_kinds` lists only the 14 built-in kinds. `uilab-agent/src/lib.rs:421` sends it to the model as "Composite kinds a new composite child can be", which contradicts the patch schema that accepts widget names. | every agent request at a page, a composite or a widget body | NEEDS-CHANGE | introduced |
| 5 | `crates/uilab-doc/src/check.rs:471` | `composites()` and `components_resolve` only walk `widgets` and `item`. Widget instances inside ess Node lists (`children`, `parts`, `expand`, `toolbar`) are never resolved, arg-checked or recursion-checked, and removing a widget they use is admitted. | the ess section docs: "`children` adds widgets or primitives" | CONFIRMED | introduced |

**5. Attacked and not broken**
- The four checks and their paths, including recursion through a board node and args with defaults.
- Paths `component:<w>` and `component:<w>/node:<n>` through resolve, children, allowed children and `composite_mut`.
- Removing a widget that is in use at a section, a board widget or an item is refused.
- Replacing a widget so that it drops a param an instance still passes is refused.
- Numbers in nodes stay numbers under `arbitrary_precision`.
- Node order, a node with both `component` and `primitive`, and duplicate node names.
- The dependent crates still compile and their tests pass.
- An unresolved instance in a patch is refused as `node_shape`, not `widget_resolves`. That keeps the pre-existing handling of a mistyped kind, so I did not raise it.

**6. Paths written outside the worktree**
- `/home/timo/.cache/uilab-wave-w1/widget-model/adv-base/` (117M): the base 8eb6405 copy of uilab-doc, its probe test, and `target/`.
- `/home/timo/.cache/uilab-wave-w1/widget-model/adv-ess-ui.schema.yaml`: an extract of the ess schema.
- I also wrote a commit message file in that directory and have deleted it. I added test output to the existing build directory `~/.cache/b10x-target/uilab-w1-widget-model`.

```findings
- file: crates/uilab-doc/src/model.rs
  line: 470
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Param silently drops keys it does not type, which is neither the module's lossless round-trip promise nor the ess closed-record refusal
- file: crates/uilab-doc/src/model.rs
  line: 478
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "an explicit `default: null` on a param is read as no default and vanishes on write"
- file: crates/uilab-doc/src/model.rs
  line: 217
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: collection/record `item` is read as a map while ess integrate/ess-ui-1 declares a list of named Nodes, so the ess example of an instance in an item does not parse
- file: crates/uilab-doc/src/outline.rs
  line: 129
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: node_context.composite_kinds omits declared widgets, and the agent prompt presents that list as every kind a new composite child can be
- file: crates/uilab-doc/src/check.rs
  line: 471
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: widget instances inside ess Node lists kept as untyped props (children, parts, expand, toolbar) escape resolve, args and recursion checks, so a widget they use can be removed
```
