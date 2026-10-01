---
format: aep.planning-md/3
id: review-result:essui-agent-schema-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:essui-agent-schema
relations:
- reviews: story:essui-agent-schema
revision: 1
---
unit: story:essui-agent-schema at `0c14c0d` (worktree `~/.local/state/worktree/trees/b10x/uilab/uilab-w10-essui-agent-schema`, plus 1 untracked test file)
verdict: red
cases: executed 249→254, red 5
origin: introduced 3, pre-existing 0, undecided 1
wrote-outside-worktree: `~/.cache/uilab-wave-w10/essui-agent-schema/adversary-2/{probe/,red.log,suite.log}`, plus the assigned build dir `~/.cache/b10x-target/uilab-essui-agent-schema`
needs-coordinator: yes. ESS 0.48.0 checks `chart: donut` with 0 findings, but its own schema's enum leaves `donut` out. That is another case of ESS's schema and loader disagreeing, for beyond10x/ess#305. It is not a uilab defect.

## 1. What I changed

`git --no-pager diff --stat` is empty. `git status --short` shows one file:
```
?? crates/uilab-agent/tests/adversary_agent_schema_p2.rs
```
It is a test file. No implementation file was touched. I also wrote a probe test (`adversary_agent_schema_p2_probe.rs`), ran it, and deleted it before writing the cases.

## 2. Cases added (all in `crates/uilab-agent/tests/adversary_agent_schema_p2.rs`)

| case | asserts | now |
|---|---|---|
| `the_schema_refusal_of_an_inserted_node_names_the_value_at_fault` | when the schema refuses an inserted node (metric `label: 5`, chart `donut`, badge `tone: purple`), the message names the bad value, as it already does for a replace (`5 is not of type "string"`) | red |
| `the_agent_is_told_which_field_its_inserted_section_got_wrong` | the same mistake run through the real `Proposer` loop with a scripted model: the tool result the model reads names the fault | red |
| `a_node_carrying_both_tags_is_refused_by_the_schema` | a node with both `component` and `primitive` is refused by the schema, as a section, an overlay and an item | red |
| `a_node_without_its_tag_is_refused_where_nothing_is_inherited` | an untagged node is refused where no page kind contributes anything: a replace of `section:on_loan`, an item, a shell overlay | red |
| `an_inserted_section_keeps_the_name_the_patch_gives_it_when_written` | an insert with `child.name: copies` and `node.name: other` still has `section:copies` after write and reload | red |

The cases were run alone first (`red.log`), with 4 red and 1 green. The green one was the naming case, which at that point only checked the document in memory, where `section:copies` does resolve. I rewrote it to check the written document, then ran it alone:
```
admitted as `section:copies` (in memory: true), written and reloaded as `section:other` (true)
```
The other 4, verbatim from that run:
```
the agent is not told what is wrong with its answer: "the arguments of `answer` do not match its published schema: {\"layer\":\"section\",\"name\":\"copies\",\"node\":{\"component\":\"metric\",\"label\":5,...}} is not valid under any of the schemas listed in the 'oneOf' keyword; call it again with a valid object"
```
```
the schema accepts an untagged node where nothing is inherited:
"replace" "page:overview/section:on_loan": admission says document_loads: page:overview/section:on_loan: a node needs `component` or `primitive`
"insert" "page:loans/section:list": admission says node_shape: not a valid item: node `due` has neither `component` nor `primitive`; a node has exactly one
"insert" "shell:app": admission says document_loads: shell:app/overlay:help: a node needs `component` or `primitive`
```
```
the schema accepts a node with both tags:
"page:overview" "section": admission says document_loads: page:overview/section:both: a node has exactly one of `component` and `primitive`
"page:loans" "overlay": admission says document_loads: page:loans/overlay:both: a node has exactly one of `component` and `primitive`
"page:loans/section:list" "item": admission says node_shape: not a valid item: node `both` has both `component` and `primitive`; a node has exactly one
```
```
5 at "page:overview":  schema: {...} is not valid under any of the schemas listed in the 'oneOf' keyword
  admission would say: document_loads: ...: `metric`: invalid type: integer `5`, expected a string
"donut" at "page:overview":  schema: {...} is not valid under any of the schemas listed in the 'oneOf' keyword
  admission would say: admitted
"purple" at "page:loans/section:list":  schema: {...} is not valid under any of the schemas listed in the 'oneOf' keyword
```

## 3. Suite run (after the cases existed)

Command: `CARGO_TARGET_DIR=~/.cache/b10x-target/uilab-essui-agent-schema CARGO_INCREMENTAL=0 cargo test --no-fail-fast -p uilab-doc -p uilab-agent --no-default-features`

Result: `EXIT=101`. 254 executed, 249 passed, 5 failed. All 5 failures are mine; every existing test is green.

The before count, 249, is the same run with my binary `adversary_agent_schema_p2` left out. The failing test names exist in this tree, so the shared build dir did not run another tree's binary.

## 4. Findings (cover `0c14c0d`)

| file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|
| crates/uilab-doc/src/schema.rs:66 | `child` picks its layer with `oneOf` (the same holds for `node` at :626). Any field mistake in an inserted section or item comes back as the whole child plus "not valid under any of the schemas listed in the 'oneOf' keyword", which names no field. Each such refusal costs one of the 4 turns in an attempt. Fix: choose the layer with `if layer const … then` (and choose `node` on whether `primitive` is present), so the validator reports the inner error, as the replace path already does. | NEEDS-CHANGE | introduced (base had the `oneOf` but almost no field constraints under it; `label: 5` passed the schema at base) | every insert at a page or a collection |
| crates/uilab-doc/src/schema.rs:715 | The open tagged variants accept the other tag, so a node with both tags passes the schema and only admission refuses it. The guard was added to the untagged refinement branch only. Fix: give each tagged variant the same `not: {anyOf: others}`. | CONFIRMED | introduced (the variants were closed before correction 1) | a model that fills both tags; the refusal is one it can act on |
| crates/uilab-doc/src/schema.rs:761 | The untagged refinement branch is offered for each position, not for each target. For a replace of a section no page kind contributes, for items, and for shell overlays, the schema tells the model to "write only what differs", and admission always refuses that. | CONFIRMED | introduced (base `composite` required `component`) | a replace answered with only the changed props |
| crates/uilab-doc/src/schema.rs:525 | The section position offers `name` inside the node. When it differs from `child.name`, admission keys the section `copies` but writes it as `other` (`Sections` serialize, model.rs:290). After a save, the section the patch named, and that a later plan step targets, is gone. | CONFIRMED | undecided (admission code is identical at base; not run there) | a model that writes `name` in the node |

## 5. Attacked and could not break

- The prompt's widget example as written (`type: Member`, `note`, `text: args.member.name`, `args: {member: row}`) is admitted in a batch.
- Unknown keys on a section, an overlay or an item (`title`, `colour`) are refused with clear messages, for example "unknown field `title`, expected one of `reads`, `from`, …". This is the coordinator's design working.
- `{}` is refused at every position. A primitive used as a section is refused by the schema.
- An overlay with only `same_as` is valid against the schema, and both ESS and uilab admit it.
- A retry loop cannot happen: an answer gets at most `MAX_ATTEMPTS` 2 × `DEFAULT_MAX_TURNS` 4.
- Schema refusals in the replace path name the value at fault.

## 6. Paths written outside the worktree

- `~/.cache/uilab-wave-w10/essui-agent-schema/adversary-2/probe/`: a copy of pass 1's probe example plus 12 variant `*.ui.yaml` files, checked with the ESS 0.48.0 binary
- `~/.cache/uilab-wave-w10/essui-agent-schema/adversary-2/red.log`
- `~/.cache/uilab-wave-w10/essui-agent-schema/adversary-2/suite.log`
- `~/.cache/b10x-target/uilab-essui-agent-schema` (the assigned build dir, reused)

I took a session lease on the worktree and have released it.

## 7. Findings block

```findings
[
  {"file": "crates/uilab-doc/src/schema.rs", "line": 66, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "child's oneOf over layers turns every field mistake in an inserted node into an opaque 'not valid under any of the schemas listed in the oneOf keyword' refusal that names no field, and that is what the agent is fed back"},
  {"file": "crates/uilab-doc/src/schema.rs", "line": 715, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "open tagged variants accept the other tag, so a node with both component and primitive is schema-valid at section, overlay and item and only admission refuses it"},
  {"file": "crates/uilab-doc/src/schema.rs", "line": 761, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the untagged refinement branch is offered per position, so a replace of a non-inherited section, an item and a shell overlay without a tag are schema-valid and always refused"},
  {"file": "crates/uilab-doc/src/schema.rs", "line": 525, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "the section node offers name, and a node name that differs from child.name is admitted under child.name but written and reloaded under the node's name"}
]
```
