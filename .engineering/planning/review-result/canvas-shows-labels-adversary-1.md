---
format: aep.planning-md/3
id: review-result:canvas-shows-labels-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:canvas-shows-labels
relations:
- reviews: story:canvas-shows-labels
revision: 1
---
unit: story:canvas-shows-labels, worktree ~/.local/state/worktree/trees/b10x/uilab/uilab-w10-canvas-labels at 01dee79 plus 3 untracked adversary test files
verdict: NEEDS-CHANGE
cases: executed 548→563 (widget 385→393, uilab-doc 163→170), red 13
origin: introduced 1 / pre-existing 6 / undecided 0
write-outside-worktree: 2 paths (~/.cache/uilab-wave-w10/canvas-labels/adversary-1, ~/.cache/b10x-target/uilab-canvas-labels-adv-base); one base build also wrote into ~/.cache/b10x-target/uilab-canvas-labels
needs-coordinator: yes. Six of the seven findings are pre-existing but fall inside the acceptance bullet "any the canvas drops is listed and fixed". You decide whether they come back to this unit or go to new stories.

**1. Diff** (`git --no-pager diff --stat` is empty because every change is an untracked file; `git status --short`):
```
?? crates/uilab-doc/tests/adversary_canvas_labels.rs          (212 lines)
?? crates/uilab-doc/tests/adversary_canvas_labels_merged.rs   (132 lines)
?? widget/src/lib/canvas.labels.adversary.test.ts             (166 lines)
```
All three are test files. I also ran `pnpm install --frozen-lockfile` in `widget/` because `node_modules` was missing. It is gitignored.

**2. Cases added.** Each file was run alone before any suite run.

| file | asserts | now |
|---|---|---|
| `adversary_canvas_labels.rs` `a_page_without_a_header_is_shown_the_metrics_its_kind_contributes` | rendered header `metrics` of `front` (writes no header) equal ESS's | red: `left: [] right: ["out"]` |
| same file, `a_page_header_metric_is_merged_with_the_kinds_by_name` | `back` gets kind's metric plus its own | red: `left: ["overdue"] right: ["out", "overdue"]` |
| same file, `a_choice_over_an_enum_type_is_shown_the_options_ess_renders` | `options: LoanState` reaches the browser as ESS's `{value,label}` list | red: `left: Some(String("LoanState"))` |
| same file, two controls | title/help/actions through two document kinds equal `ess_ui::load_str`; agent outline keeps header as written | green |
| `adversary_canvas_labels_merged.rs` `a_section_the_page_names_again_keeps_the_column_labels_its_kind_gives` | kind's column labels survive a page re-naming `list` | red: `front's list is shown with props None` / `left: [] right: [Some("Book"), Some("Due back")]` |
| same file, `an_overlay_same_as_another_carries_the_title_ess_renders` | `same_as: front.edit` overlay has title "Extend loan" | red: `left: None right: Some("Extend loan")` |
| `canvas.labels.adversary.test.ts` (×2 modes) | header metric label "Copies out" is drawn | red: `the header metric's label is not on the canvas: Overview` |
| same file (×2) | `states.empty.message` / action label are drawn on empty rows | red: `...not on the canvas: title no data yet` (preview shows invented text instead of the author's) |
| same file (×2) | `toggle` primitive label is drawn | red: `...not on the canvas: toggle · notify` |
| same file (×2) | `input` primitive placeholder is drawn | red: `no input hints "Search the shelves": input · find` |

Both Rust fixtures pass `ess ui check` (0.48.0) with 0 errors (warnings are missing fixtures only). Logs are under `~/.cache/uilab-wave-w10/canvas-labels/adversary-1/` (`rust-red.log`, `rust-red-merged.log`, `widget-red.log`).

**3. Suite runs** (after the cases existed):
- `pnpm test` (widget): `# tests 393 # pass 384 # fail 8 # todo 1`, EXIT=1. The 8 failures are exactly mine.
- `cargo test -p uilab-doc --no-default-features --no-fail-fast`: passed 165, failed 5, ignored 3, EXIT=101, `error: 2 targets failed`. The log shows `Compiling uilab-doc (…/uilab-w10-canvas-labels/…)`, and the control case that fails on base passed, so the worktree's own code ran.

**4. Findings** (covering 01dee79)

| # | file:line | verdict | origin | what reaches it |
|---|---|---|---|---|
| 1 | crates/uilab-doc/src/outline.rs:321 | NEEDS-CHANGE | introduced | `merge_pages` puts back only the page's written `header.metrics`, so metrics a page kind contributes are dropped. ESS merges them by name. Reached by any document page kind with `header.metrics` (the check.rs tests already build one). On base this case fails as "no header at all", which is the defect this unit fixes. Fix: serialize ESS's `header.metrics` (or merge the kind's written metrics by name). |
| 2 | widget/src/components/CanvasView.vue:163 | NEEDS-CHANGE | pre-existing | Header metrics are never drawn, so a metric `label` (the story's own subject) is missing in header position. `headerOf` returns only title/help/actions. |
| 3 | widget/src/components/CompositeView.vue:64 | NEEDS-CHANGE | pre-existing | `states.empty.message` and `states.empty.action.label` are dropped. On empty rows, preview shows the invented "no data yet" instead. Reached by any read that returns 0 rows. The schema's own Section example writes `empty.message`. |
| 4 | widget/src/lib/components.ts:312 | NEEDS-CHANGE | pre-existing | `toggle` and `input` (and `icon`) primitives are drawn as the placeholder `kind · name`, so `label` and `placeholder` are lost. Reached by section `children` and widget bodies. |
| 5 | crates/uilab-doc/src/outline.rs:737 | CONFIRMED | pre-existing | `choice` with `options: <EnumType>` (the schema's own example form) reaches the browser as a string, and the canvas draws the placeholder "choice". |
| 6 | crates/uilab-doc/src/outline.rs:338 | CONFIRMED | pre-existing | Authored nodes take only kind/view from ESS. When a page names a kind's section again with only `reads`, the kind's columns (and their labels) vanish and no table is drawn. |
| 7 | crates/uilab-doc/src/outline.rs:745 | CONFIRMED | pre-existing | A `same_as` overlay's title comes from written props only, so the canvas shows the node name "extend" instead of ESS's "Extend loan". The acceptance names "`title` on an overlay". |

I checked origin by running every case against an extracted copy of base e2bd06f (`git archive`, nothing checked out). All 8 widget cases and all 5 Rust cases fail there.

Shared-target incident: my first base run used the warm build dir. Cargo reused the same test-binary hash and ran the worktree's library instead of base's: the control case passed, which is impossible on base. I discarded that result and reran base in its own dir. The later worktree suite recompiled from the worktree path.

**5. Tried and could not break:**
- title, help and actions merge through two of the document's own page kinds with `extends`, and match `ess_ui::load_str`.
- `title: from_page` resolves correctly.
- The agent outline keeps the header as the author wrote it; the agent prompt shows written nodes only.
- nav label from `nav_json`; a page with no header falls back to its title (the unit's own test covers this).

**6. Paths written outside the worktree:**
- `~/.cache/uilab-wave-w10/canvas-labels/adversary-1/` (7.8M: logs, `header-kinds.ui.yaml`, `probe.ui.yaml`, `base/` snapshot with a `node_modules` symlink into the worktree)
- `~/.cache/b10x-target/uilab-canvas-labels-adv-base/` (402M base build dir)
- `~/.cache/b10x-target/uilab-canvas-labels/`: the base `uilab-doc` test target was compiled into it once. The worktree suite has since rebuilt it from the worktree.
- Inside the worktree: `widget/node_modules` (gitignored, from pnpm install)

```findings
[
  {
    "file": "crates/uilab-doc/src/outline.rs",
    "line": 321,
    "category": "contract-drift",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "merge_pages keeps only the page's written header.metrics, so headline metrics a page kind contributes (and ESS merges by name) are missing from the browser's rendered header"
  },
  {
    "file": "widget/src/components/CanvasView.vue",
    "line": 163,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "pre-existing",
    "message": "the canvas draws no page-header metrics, so a header metric's label never appears in preview or structure mode"
  },
  {
    "file": "widget/src/components/CompositeView.vue",
    "line": 64,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "pre-existing",
    "message": "a section's states.empty message and action label are dropped and preview shows the invented \"no data yet\" instead"
  },
  {
    "file": "widget/src/lib/components.ts",
    "line": 312,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "pre-existing",
    "message": "toggle and input primitives draw as a kind·name placeholder, dropping the author's toggle label and input placeholder"
  },
  {
    "file": "crates/uilab-doc/src/outline.rs",
    "line": 737,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "pre-existing",
    "message": "a choice over a named enum type reaches the browser as the type name, so the canvas shows a placeholder instead of the option labels ESS renders"
  },
  {
    "file": "crates/uilab-doc/src/outline.rs",
    "line": 338,
    "category": "contract-drift",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "pre-existing",
    "message": "an authored section that a page kind also defines takes only kind and view from ESS, so the kind's column labels are lost"
  },
  {
    "file": "crates/uilab-doc/src/outline.rs",
    "line": 745,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "pre-existing",
    "message": "an overlay written as same_as another carries no title in the rendered outline, so the canvas shows its node name instead of ESS's title"
  }
]
```
