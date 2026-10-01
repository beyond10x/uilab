---
format: aep.planning-md/3
id: review-result:essui-app-widget-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:essui-app-widget
relations:
- reviews: story:essui-app-widget
revision: 1
---
unit: story:essui-app-widget at d825897 (worktree with my 2 untracked test files)
verdict: red
cases: executed 593→601, red 7
origin: introduced 4 / pre-existing 0 / undecided 1
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: yes. Finding 1 is in `crates/uilab-doc/src/path.rs` and finding 5 is in `crates/uilab-doc/src/ess.rs`. Both files belong to story:essui-document and reached this branch through merge af59f1f (5d0ae6a). You need to decide which story fixes them.

The suite is red on 7 new cases, covering 4 introduced findings. Separately, an existing case fails under machine load (finding 5).

**1. `git --no-pager diff --stat`**
```
(empty: no tracked file changed)
?? crates/uilab-doc/tests/adversary_rendered_fields_p2.rs
?? widget/src/lib/nested.instance.p2.adversary.test.ts
```
Both paths are test files, and I changed no implementation file. `widget/node_modules` came from `pnpm install --frozen-lockfile`; git ignores it.

**2. Cases added (each run alone first; logs are `doc-red.log` and `widget-red.log` in scratch)**

| case | asserts | red output |
|---|---|---|
| `adversary_an_overlay_kind_reads_as_the_document_spells_it_p2` | the shipped library's `page:loans/overlay:edit` has kind `drawer form` | `left: "some(drawer) form"` |
| `adversary_a_refining_section_shows_the_component_its_kind_gives_p2` | a `filters` section with no `component`, refining the kind's section, has kind `filter_bar` | `left: "inherited"` |
| `adversary_a_refining_section_reads_the_view_its_kind_gives_p2` | a `list` that refines a declared kind's `list` reads `loans.All` | `left: None` |
| `adversary_a_same_as_overlay_shows_the_overlay_it_copies_p2` | an overlay written as `same_as: loans.edit` has kind `drawer form` | `left: "none inherited"` |
| `adversary_a_part_of_a_same_as_overlay_shows_its_kind_p2` | the copied `part:hint` has kind `text` | `left: "node"` |
| `adversary_a_nested_body_is_sent_as_its_declaration_writes_it_p2` | shows that finding 4's widget outline is what the server actually sends | green |
| `nested.instance.p2.adversary.test.ts` (structure and preview) | a nested body node `line` shows "Dune Messiah", as ESS renders it | red ×2: `<span>Dune</span>` |

Each Rust case first asserts that ESS 0.48.0 finds 0 errors in its document, so every document is one ESS accepts.

**3. Suite runs (after the cases existed)**
- `cargo test -p uilab-app -p uilab-doc --locked --no-default-features --no-fail-fast`: EXIT=101. 256 cases ran, 6 failed: my 5, plus `adversary_item_list_p2::admitting_beside_sixteen_thousand_body_findings_stays_near_the_cost_of_a_check` (`left: 0, right: 16384`). That case also failed when run alone, taking 35s at load average 14.9. It passed in the implementor's gate at 17:07.
- `pnpm test`: EXIT=1. 345 tests: 342 pass, 2 fail (mine), 1 TODO that was already there.
- "Before" is these runs minus my files: 250 + 343.
- `rustfmt --check` on the new file exits 0.

**4. Findings (covering d825897)**

| file:line | verdict | origin | measured / what reaches it |
|---|---|---|---|
| `crates/uilab-doc/src/path.rs:385` | NEEDS-CHANGE | introduced | `kind_label` formats `Option<OverlayKind>` with Debug, so the label reads `some(drawer) form`. **Reaches it:** every overlay of the shipped library, shown in the tree, on the card label and in the agent's `node_context` ancestors. At base 55e2c8d, `kind` was not an `Option`. |
| `crates/uilab-doc/src/outline.rs:451` | NEEDS-CHANGE | introduced | A node the author wrote is shown with only what the author wrote, not what ESS merges in. Measured three ways: a refining section shows kind `inherited` instead of `filter_bar`, has no view instead of `loans.All`, and a `same_as` overlay shows `none inherited`. **Reaches it:** wave 1's `Component::Inherited` is there so documents can write exactly these refinements. **Fix:** take the fields of every node, including authored ones, from the merged page. |
| `crates/uilab-doc/src/outline.rs:321` | NEEDS-CHANGE | introduced | `written_at` does not follow `same_as`, so a part ESS copies into an overlay is shown as an untyped `node`. **Reaches it:** any overlay written as `same_as` whose source has parts. |
| `widget/src/lib/instance.ts:138` | NEEDS-CHANGE | introduced | `previewNode` (`components.ts:295`) runs recursively and fills in the outer instance's args inside the nested body the server now sends. So the inner widget shows the outer loan's title. **Reaches it:** two widgets that share a param name (`loan`), one nested in the other. At base, `instanceBody` read only `widget.body` (`git show 55e2c8d:widget/src/lib/instance.ts:128-131`). |
| `crates/uilab-doc/src/ess.rs:290` | CONFIRMED | undecided | A 10s wall-clock deadline on ESS means `check()` returns 0 findings when the machine is under load. The existing case above went red twice. Not run at base. |

**5. Attacked and held**
- `announce_change`: `delta_holds` compares the whole flattened tree, and undo, reject, batch, page changes and widget changes all send a full snapshot. Insert always adds at the end, and so does ESS, so sibling order agreed in every case I tried.
- Inherited nodes for nested instances, instances inside a kind, and a declared kind over an `extends` chain all line up with ESS's node list.
- Primitives drawn in every `NODE_LAYERS` list. `PRIMITIVE_KINDS` matches ESS's 9 primitives.
- Selecting an inherited node, and selecting one whose author node was then removed, falls back to the session's selection.

**6. Paths written outside the worktree**
- `~/.cache/uilab-wave-w10/essui-app-widget/adversary-2/`: `doc-red.log`, `widget-red.log`, `suite-rust.log`, `suite-widget.log`, `rerun-sixteen.log`
- `~/.cache/b10x-target/uilab-essui-app-widget/`: the assigned build directory, reused
- pnpm's shared store (from the install)

```findings
[
  {"file": "crates/uilab-doc/src/path.rs", "line": 385, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "an overlay's kind label is built from Debug of Option<OverlayKind>, so every overlay reaches the wire and the agent as `some(drawer) form`"},
  {"file": "crates/uilab-doc/src/outline.rs", "line": 451, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "authored nodes in rendered() carry only the author's fields, so a refining section shows kind `inherited` and no view and a same_as overlay shows `none inherited`, where ESS renders filter_bar, loans.All and drawer form"},
  {"file": "crates/uilab-doc/src/outline.rs", "line": 321, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "written_at does not follow same_as, so a part ESS copies into a same_as overlay is shown with kind `node` instead of its primitive kind"},
  {"file": "widget/src/lib/instance.ts", "line": 138, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "previewNode fills in the outer instance's args inside the nested body the server now sends, so a nested widget sharing a param name draws the outer row's value instead of what ESS renders"},
  {"file": "crates/uilab-doc/src/ess.rs", "line": 290, "category": "concurrency", "severity": "note", "verdict": "CONFIRMED", "origin": "undecided", "message": "a 10s wall-clock ESS deadline makes check() findings depend on machine load; adversary_item_list_p2's sixteen-thousand case finds 0 instead of 16384 at load 14"}
]
```
