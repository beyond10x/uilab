---
format: aep.planning-md/3
id: review-result:item-list-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:item-list
relations:
- reviews: story:item-list
revision: 1
---
unit: story:item-list on unit/item-list, findings cover d655644 (my test commit f3dc4e9 on top)
verdict: NEEDS-CHANGE
cases: executed 45→51, red 1
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths, all deleted (part 6)
needs-coordinator: yes. Findings 2 and 3 are in widget/ and crates/uilab-app, outside this unit's surface

The unit does what its acceptance asks, but one check misses the new primitive item nodes, and two consumers outside uilab-doc still read `item` as a map.

**1. Diff stat** (`git --no-pager diff --stat d655644 HEAD`)
```
 crates/uilab-doc/tests/adversary_item_list.rs | 280 ++++++++++++++++++++++++++
 1 file changed, 280 insertions(+)
```
Only a test file changed. Committed as f3dc4e9 through `b10x-gates bot`. Author and committer are both b10x-bot[bot].

**2. Cases added** (in `crates/uilab-doc/tests/adversary_item_list.rs`)

| case | asserts | now |
|---|---|---|
| `a_button_item_that_opens_a_missing_overlay_is_reported` | `opens_resolves` fires for a button item that opens `nowhere`. The same button under `children` is the control. `admit` refuses the insert | **red** |
| `item_nodes_are_removed_and_replaced_at_every_position` | remove and replace at start, middle and end keep sibling order; insert and remove work in a shell-overlay record and a widget-body collection | green |
| `empty_item_forms_read_as_no_items` | `[]`, `{}` and `~` read as no items and round-trip | green |
| `a_malformed_item_entry_is_refused_with_its_reason` | entries with no name, an empty name, a scalar, both keys or neither key are refused with a reason | green |
| `numbers_in_item_nodes_survive_a_round_trip` | integer `2` and float `1.5` are still numbers after a write and re-read | green |
| `a_widget_containing_itself_through_an_item_is_refused` | the insert is refused with `widget_recursion` | green |

Red output from running this file alone, before the suite (line 130, which became 129 after rustfmt):
```
thread 'a_button_item_that_opens_a_missing_overlay_is_reported' panicked at crates/uilab-doc/tests/adversary_item_list.rs:130:5:
a button item opens `nowhere`, which neither the page nor its shell declares, and no check says so: []
test result: FAILED. 5 passed; 1 failed
```

**3. Suite runs, after the cases existed**

| command | result |
|---|---|
| `cargo test -p uilab-doc --no-fail-fast` | EXIT=101. adversary_item_list 5/1 failed; adversary_widget_model 9 ok; adversary_widget_model_p2 4 ok; doc 32 ok. The 45 before my file comes from this run's per-binary counts |
| `cargo check --workspace --all-targets` | EXIT=0 |
| `cargo test -p uilab-agent -p uilab-behaviour --no-fail-fast` | EXIT=0, 43 passed |
| `cargo fmt -p uilab-doc --check`, `cargo clippy -p uilab-doc --all-targets -- -D warnings` | clean |

**4. Findings** (all against d655644)

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| 1 | crates/uilab-doc/src/check.rs:677 | NEEDS-CHANGE | introduced | `check_composite` only recurses into item nodes that are composites. A primitive item's `action.opens` is never checked. The red case above shows it | an agent `Insert` of a primitive item. The patch schema's `node` def allows it, and `admit` lets it through. Before this unit, an item could not be a primitive |
| 2 | widget/src/lib/yamlblock.ts:117 | NEEDS-CHANGE | introduced | for `item:<name>` it looks for the key `<name>:`, but the server now writes `- name: <name>`. A scratch copy returned null for the list form and found the map form | `YamlView.vue:9` runs it on `to_yaml()` output (`app.rs:345`), so the YAML view can no longer highlight any selected item node |
| 3 | crates/uilab-app/src/eval.rs:248 | NEEDS-CHANGE | introduced | `holds` reads `item` with `as_object()`. On a scratch copy the list form returned `false` and the map form `true` | eval case `item-in-collection` (`evals/library.yaml:84`). When the agent answers with a Batch, it is now judged "batch holds no item metric" |
| 4 | crates/uilab-doc/src/check.rs:326 | CONFIRMED | introduced | the `names_unique` finding points at `item:<name>`, which `resolve` maps to the first node, not the duplicate. `children` lists the name twice, so the outline shows the first node twice | only a document that already fails `names_unique` |

Suggested fixes (not applied):
1. Run `opens()` over each primitive item's props in `check_composite`.
2. Treat `item` like `nav_section` in `yamlBlock`, through `namedEntry`.
3. Accept arrays in `holds`.

**5. Attacked and not broken**
- Map-to-list conversion keeps order; preserve_order is on in the workspace.
- Nested items, and items in records, shell overlays and widget-body collections.
- Replace and remove at every position.
- Duplicate-name inserts are refused with `name_unique`.
- Widget args and recursion checks over item instances.
- The patch schema: `node` for items, `named_node` for lists and bodies.
- Numbers under arbitrary_precision.
- The flipped pinned case, which now asserts that the list form parses and resolves and that the instance is checked.

**6. Written outside the worktree** (all under `/home/timo/.cache/uilab-wave-w2/item-list/`, all deleted)
- `probe-holds/`: a scratch Rust crate plus its own target, 59M
- `probe-yamlblock/`: copies of yamlblock.ts and outline.ts plus a probe test
- `ess-ui.schema.yaml`, `partner-ui.yaml`: reference copies read from ess `integrate/ess-ui-1`

I also used the build dir `/home/timo/.cache/b10x-target/uilab-w2-item-list`, as assigned. My lease `adversary-item-list` is released. Disk is at 14G free on `/`.

**7. Findings block**
```findings
- file: crates/uilab-doc/src/check.rs
  line: 677
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "opens_resolves skips primitive item nodes, so a button item opening an undeclared overlay passes check and admit"
- file: widget/src/lib/yamlblock.ts
  line: 117
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "yamlBlock looks up item:<name> as a map key, so it finds no item node in the list form the server now writes"
- file: crates/uilab-app/src/eval.rs
  line: 248
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "holds reads item with as_object, so a batch that inserts an item is judged as holding none"
- file: crates/uilab-doc/src/check.rs
  line: 326
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the names_unique finding path resolves to the first node of the name, not the duplicate it reports"
```
