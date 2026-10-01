---
format: aep.planning-md/3
id: review-result:canvas-shows-labels-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:canvas-shows-labels
relations:
- reviews: story:canvas-shows-labels
revision: 1
---
unit: story:canvas-shows-labels, worktree ~/.local/state/worktree/trees/b10x/uilab/uilab-w10-canvas-labels at a582f42 plus 2 untracked adversary test files
verdict: NEEDS-CHANGE
cases: executed 563→575 (uilab-doc 170→172, widget 393→403), red 12
origin: introduced 0 / pre-existing 7 / undecided 0
wrote-outside-worktree: 2 paths (~/.cache/uilab-wave-w10/canvas-labels/adversary-2, ~/.cache/b10x-target/uilab-canvas-labels-adv2-base)
needs-coordinator: yes. The merge matches ESS on every case I tried. All 7 findings are pre-existing. Five are texts the canvas still drops, which the acceptance bullet "any the canvas drops is listed and fixed" covers. You decide whether they come back to this unit or go to new stories.

**1. Diff.** `git --no-pager diff --stat` is empty because both files are untracked. `git status --short`:
```
?? crates/uilab-doc/tests/adversary_canvas_labels_p2.rs        (494 lines)
?? widget/src/lib/canvas.labels.p2.adversary.test.ts           (184 lines)
```
Both are test files. No implementation file was touched. `widget/node_modules` was missing, so I ran `pnpm install --frozen-lockfile`; the directory is gitignored.

**2. Cases added.** Each file was run alone before any suite run.

`adversary_canvas_labels_p2.rs` has a differential harness: every section, overlay and nested node that `ess_ui::load_str(..).nodes()` renders, with its texts (overlay title; labels of columns, fields, tabs, groups, options and actions; metric and button captions; primitive text), is compared with `outline::rendered`. Both fixtures pass `ess ui check` 0.48.0 with 0 errors.

| case | now | red output |
|---|---|---|
| `a_board_widget_the_page_removes_with_null_is_opened_and_not_shown` | red | `uilab refuses a document ESS loads: Some(LoadError { path: "/", message: "pages.wall: section `board`: board widget `late` is a node at line 26 column 5" })` |
| `an_entry_a_page_removes_from_a_nested_list_is_not_shown` | red | `the browser is shown what the page removes: ["page:front/section:filters/choice:branch (kind inherited, props Some(Object {"remove": Bool(true)}))", "page:front/section:list/item:lend (kind inherited, props …)"]` |

`canvas.labels.p2.adversary.test.ts` has 5 cases, each run in preview and in structure mode, all red:
- `the selectable columns' labels are not on the canvas: body · collection · loans.All`
- `the form's record field label is not on the canvas: … Member settings Contact Email Submit`
- `the form group's action label is not on the canvas: …` (the group heading control passes)
- `the header metric's widget body is not on the canvas: overview · static_page Overview due · due_tag …`
- `the header's help link is not on the canvas: …`

Logs are in the scratch directory: `rust-red-final.log`, `rust-red-nested.log`, `widget-red.log`.

**3. Suite runs** (after the cases existed):
- `cargo test -p uilab-doc --no-default-features --no-fail-fast`: passed 170, failed 2, ignored 3, EXIT=101, `error: 1 target failed`. The 2 failures are exactly mine.
- `pnpm test`: `# tests 403 # pass 392 # fail 10 # todo 1`, EXIT=1. The 10 failures are exactly mine.
- Before-counts: the widget suite with my file left out ran 393 tests. For uilab-doc, 170 is this run's total minus my binary.
- Clippy `-D warnings`, `rustfmt` on my file only, and `pnpm typecheck` are all clean.

**4. Findings** (covering a582f42)

| # | file:line | verdict | origin | what reaches it |
|---|---|---|---|---|
| 1 | crates/uilab-doc/src/outline.rs:313 | CONFIRMED | pre-existing | A `{name, remove: true}` in a nested named list (`choices`, `item`) stays in the browser's tree as a node of kind `inherited` with props `{remove: true}`. ESS renders no such node. `add_to` only adds ESS's nodes and never drops authored ones ESS does not render. ESS's own `examples/partner-portal` page `users.list` writes `{name: tags, remove: true}`. That the canvas then draws a placeholder for it is inferred, not run. |
| 2 | widget/src/lib/outline.ts:174 | NEEDS-CHANGE | pre-existing | `columnsOf` reads only list columns. User-selectable columns (`{binds, all}`) draw no table, so "Book" and "Due back" are lost. The server sends the labels: the merge case for this passed in the worktree and failed on base. No document in ESS or uilab found that uses this form; reached through the acceptance wording "ESS lets the author write". |
| 3 | widget/src/components/CompositeView.vue:195 | NEEDS-CHANGE | pre-existing | A form's `record` node ("read-only fields above the inputs") is never drawn. No example uses it; same acceptance basis. |
| 4 | widget/src/lib/outline.ts:226 | NEEDS-CHANGE | pre-existing | `groupsOf` keeps a group's fields and drops its `actions`. ESS's own example writes one ("Rotate key" in settings group `api`). Groups have only been drawn since a582f42; on base the whole group was missing. |
| 5 | widget/src/components/CanvasView.vue:181 | NEEDS-CHANGE | pre-existing | A header metric that is a widget instance draws nothing in preview: only `label`, and a value when `component === 'metric'`. uilab's own fixture at outline.rs:1242 builds exactly this. |
| 6 | widget/src/lib/outline.ts:270 | CONFIRMED | pre-existing | `headerOf` reads `help.text` only, so `help.link` is dropped. ESS's example writes both. |
| 7 | crates/uilab-doc/src/model.rs:474 | INFEASIBLE | pre-existing | `board_widgets` refuses `widgets: {late: null}`, which ESS merges away (`null_value: remove_inherited`), so uilab cannot open the document at all. I built this case; no document was found that writes it. |

**Origin.** Every case was run against a `git archive e2bd06f` copy with its own target directory, and all of them fail there too.

**The fix covers the earlier gaps.** On base, the merge cases showed 6 + 1 + 1 disagreements (kind columns, same_as titles, header metrics, selectable columns). On a582f42 they show none.

**Shared-target incident.** A probe of a HEAD copy built into the base target directory first ran base's library: cargo judged it fresh from file mtimes, and the results matched base exactly. I discarded that run, touched the copy's sources, and re-ran.

**5. Attacked and could not break.** All of these are run green by the harness, which is kept in scratch as `adversary_canvas_labels_p2.full.rs` because the brief asks for failing tests only:
- Kinds extending kinds (three levels) agree with ESS on every text of every node.
- Named-list order agrees, including page sections added before the kind's.
- `{name, remove: true}` on sections, columns and header metrics.
- `null` on an overlay and on an overlay title.
- A collection replaced by a record.
- `submit` maps and `{binds, all}` merged deep.
- A three-step `same_as` chain, including a shell overlay.
- Actions merged by the names ESS derives from `opens`, `does` and `navigate.to`, and inputs and columns written bare.
- Header metrics relabelled or removed by name.
- The agent outline stays as written: nothing inherited, refinements and `null` kept.
- ESS's `examples/partner-portal`: no disagreement apart from finding 1 and the widget-body `args.*` nodes, which the canvas binds by design.

**6. Paths written outside the worktree:**
- `~/.cache/uilab-wave-w10/canvas-labels/adversary-2/` (16M): logs, fixtures, `base/` and `head/` source snapshots (`base/widget/node_modules` is a symlink into the worktree), the schema copy, the full harness, and the partner-portal copy. A schema file was first written one level up by mistake and moved into this directory; nothing is left there.
- `~/.cache/b10x-target/uilab-canvas-labels-adv2-base/` (402M): build directory for the base and HEAD copies.
- `~/.cache/b10x-target/uilab-canvas-labels/`: my test binaries were added to the warm build directory.
- Inside the worktree: `widget/node_modules`, gitignored, from `pnpm install`.

**7. Findings block**

```findings
[
  {"file": "crates/uilab-doc/src/outline.rs", "line": 313, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "an entry a page removes from a nested named list with {name, remove: true} stays in the browser's tree as a node of kind inherited, though ESS renders none (ESS's own partner-portal example writes one)"},
  {"file": "widget/src/lib/outline.ts", "line": 174, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "pre-existing", "message": "user-selectable columns ({binds, all}) are not read, so the collection draws no table and its column labels are lost"},
  {"file": "widget/src/components/CompositeView.vue", "line": 195, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "pre-existing", "message": "a form's record node, the read-only fields above its inputs, is never drawn, so its field labels are lost"},
  {"file": "widget/src/lib/outline.ts", "line": 226, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "pre-existing", "message": "a form group's actions are dropped, so a label such as ESS's own example's Rotate key never reaches the canvas"},
  {"file": "widget/src/components/CanvasView.vue", "line": 181, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "pre-existing", "message": "a header metric that is a widget instance draws nothing in preview, so the widget body's text is lost"},
  {"file": "widget/src/lib/outline.ts", "line": 270, "category": "acceptance", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "the header's help link is dropped; only help.text is drawn"},
  {"file": "crates/uilab-doc/src/model.rs", "line": 474, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "pre-existing", "message": "a page removing its kind's board widget with null, which ESS accepts, makes uilab refuse to open the whole document; no such document was found"}
]
```
