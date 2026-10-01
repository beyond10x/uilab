---
format: aep.planning-md/3
id: review-result:essui-app-widget-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:essui-app-widget
relations:
- reviews: story:essui-app-widget
revision: 1
---
unit: story:essui-app-widget at a73f754 (working tree with my test additions)
verdict: NEEDS-CHANGE
cases: executed 565→574, red 5
origin: introduced 3 / pre-existing 1 / undecided 0
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: .agents/ appeared untracked in the worktree at 16:45:20, during my session-start; I did not write it and left it in place

The suite is red on 5 new cases, covering 4 findings. Three are new in this unit: the outline misses nodes ESS renders in two cases, and the browser is told to drop a node ESS still renders in a third. The fourth already reproduces at the base commit.

**1. `git --no-pager diff --stat`**
```
 crates/uilab-app/src/app.rs | 169 ++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 169 insertions(+)
?? crates/uilab-doc/tests/adversary_rendered_vs_ess.rs
?? widget/src/lib/divider.child.adversary.test.ts
?? .agents/   (not mine)
```
`app.rs` is the one path in the diff that is not a pure test file. All 169 lines are appended inside its existing `#[cfg(test)] mod tests` (hunk `@@ -3202,4 +3202,173 @@`), the same way earlier adversary passes added tests there. No code outside that module was touched.

**2. Cases added (each was run alone first; output is in the scratch logs)**

| file :: case | asserts | now |
|---|---|---|
| `crates/uilab-doc/tests/adversary_rendered_vs_ess.rs` :: `…_when_the_page_refines_a_kind_section` | the nodes in `rendered()` equal the nodes ESS lists (`ess_ui::load_str` + `nodes()`) | red: `ESS renders and rendered misses: ["page:loans/section:filters/choice:status"]` |
| same :: `…_for_a_kind_section_in_shorthand` | the same, for a declared kind whose section uses `reads: loans.Summary` | red: `misses: ["page:loans/section:count"]` |
| same :: `…_for_the_library`, `…_for_nested_and_repeated_instances`, `adversary_each_use_of_a_widget_answers_for_its_own_body` | the library example, nested and repeated widget instances, instances in items and section children | green |
| `crates/uilab-app/src/app.rs` :: `adversary_a_removed_section_the_page_kind_still_renders_stays_on_the_browser` | applying the server's messages after removing `page:loans/section:list` leaves the browser holding every node in `rendered()` | red: `the server renders and the browser was told to drop: ["page:loans/section:list"]` |
| same :: `adversary_batches_undos_and_rejects_leave_a_file_ess_checks_clean` | batch, undo of the batch, reject, a placeholder with a missing fixture, removing a section the page kind also contributes: each saved file has 0 `ess_ui_check` errors | green |
| `widget/src/lib/divider.child.adversary.test.ts` (structure and preview) | a `divider` among a section's `children`, drawn through CanvasView, appears as `<hr class="prim-rule">` | red ×2: it is drawn as a card holding `<div class="placeholder">divider</div>` |

All documents the cases build check clean with ESS 0.48.0 (`ess ui check`: 0 errors, 0 warnings).

**3. Suite runs (after the cases existed)**
- `cargo test -p uilab-app -p uilab-doc --locked --no-default-features --no-fail-fast`: EXIT=101; 228 cases, 3 failed. The new test names appear in the run output, so it ran this tree.
- `pnpm test` in `widget`: EXIT=1; 346 tests, 343 pass, 2 fail, 1 TODO that was already there.
- "Before" (565) is the same runs with my 9 cases subtracted, not a separate run.
- I used `--no-default-features` to skip the vulkan speech build.
- `cargo clippy … -D warnings` exits 0, `cargo fmt --check` exits 0, `vue-tsc --noEmit` exits 0.

**4. Findings (covering a73f754)**

| file:line | verdict | origin | what was measured / what reaches it |
|---|---|---|---|
| `crates/uilab-doc/src/outline.rs:282` | NEEDS-CHANGE | introduced | **Measured:** when a page refines a page-kind section, the outline shows only the author's children; the kind's `status` choice, which ESS merges by name, is missing. **Reaches it:** any document with `page_kinds` opened via `uilab serve --doc` (the brief's "a page kind section the page refines"). |
| `crates/uilab-doc/src/outline.rs:155` | NEEDS-CHANGE | introduced | **Measured:** `KindParts::read` silently drops any kind section uilab's model cannot parse, so a section written with ESS's `Reads` shorthand vanishes. **Reaches it:** a hand-written `page_kinds` entry; ESS's schema defines this shorthand. **Fix for both rows:** build inherited nodes from ESS's expanded page (`ess.pages[..].sections`) instead of the raw kind YAML. |
| `crates/uilab-app/src/app.rs:1170` | NEEDS-CHANGE | introduced | **Measured:** a `Remove` delta goes out for a section that the page kind still renders, and `collab.ts` `applyChange` drops it. **Reaches it:** removing `page:loans/section:list` from the shipped library example; this is admitted, ESS reports 0 errors. **Fix:** send a snapshot when `rendered_at(doc, changed)` is still `Some` after a Remove. |
| `widget/src/components/CompositeView.vue:38` | CONFIRMED | pre-existing | **Measured:** non-item children always go through CompositeView, and `isPrimitive` (`components.ts:325`) accepts only layers `node` and `item`. So a primitive in a section's `children` (layer `child`) never reaches the new rule drawing. **Reaches it:** ESS's Section docs: "children adds widgets or primitives". **Base:** `git show 55e2c8d` has the same `others` line and no divider branch. The `divider_renders_as_rule` acceptance in the brief is this unit's, so it is unmet on this path. |

Not raised as a row:
- `nodeActions` (`canvasmode.ts:45`) is called by no component, and the widget has no remove action at all. `canvasmode.inherited.test.ts` therefore tests a function nothing uses: it passes and proves nothing about the canvas.

**5. Attacked and held**
- Session writes: batch, undo, reject, a placeholder with a missing fixture, removing a section the kind also contributes. All left 0 errors.
- A widget used twice, nested instances, instances in items and section children: both the rendered nodes and `authored_at` are correct.
- Selecting an inherited node and then instructing resolves to the author's node (the unit's own test, read and not contradicted).
- `/api/act` uses the same handler as the browser, so it adds no separate route to an inherited node.
- `unbound_placeholder` message parsing in the sidebar.
- The `sample` flag on the wire.
- An empty file and a refused file on `serve`: the error path reads the file and quotes ESS's message.

**6. Paths written outside the worktree**
- `~/.cache/uilab-wave-w10/essui-app-widget/adversary-1/`: logs, `probe/` copies of the library and variants, `ess-ui.schema.yaml`.
- `~/.cache/b10x-target/uilab-essui-app-widget/`: the assigned build directory, which did not exist and was built cold.
- Inside the worktree but ignored by git: `widget/node_modules` (created by `pnpm install --frozen-lockfile`), plus pnpm's shared store.

```findings
[
  {"file": "crates/uilab-doc/src/outline.rs", "line": 282, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a page section that refines a page-kind section is shown with only the author's children, missing named nodes ESS merges in from the kind (choice status)"},
  {"file": "crates/uilab-doc/src/outline.rs", "line": 155, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "KindParts::read silently drops a kind section uilab's Composite cannot parse, so a section written with ESS's Reads shorthand is rendered by ESS and missing from the canvas"},
  {"file": "crates/uilab-app/src/app.rs", "line": 1170, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "removing an authored section that the page kind also contributes sends a Remove delta, so the browser drops page:loans/section:list while ESS still renders the kind's list"},
  {"file": "widget/src/components/CompositeView.vue", "line": 38, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "a divider among a section's children (layer child) is drawn as a placeholder card, not a rule, because only node and item layers reach PrimitiveView"}
]
```
