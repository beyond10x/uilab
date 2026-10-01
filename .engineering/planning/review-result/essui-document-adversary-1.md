---
format: aep.planning-md/3
id: review-result:essui-document-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:essui-document
relations:
- reviews: story:essui-document
revision: 1
---
unit: story:essui-document, commit 55e2c8d plus one untracked test file in the worktree
verdict: NEEDS-CHANGE
cases: executed 124→133, red 7
origin: introduced 6 / pre-existing 0 / undecided 1
wrote-outside-worktree: 23 files, all under the assigned scratch directory; 0 build directories beyond the assigned one
needs-coordinator: 2 items. D1 lets uilab save a file `ess ui check` fails, which contradicts the brief's "every document uilab writes has 0 errors". The `fixtures.views` acceptance line contradicts ESS 0.48.0.

**1. Diff**

`git --no-pager diff --stat` is empty: nothing tracked changed. `git status --short` shows one untracked file, a test file:
```
?? crates/uilab-doc/tests/adversary_essui_document.rs
```
No implementation file was touched.

**2. Cases added** (`crates/uilab-doc/tests/adversary_essui_document.rs`). Each was run alone before the suite (`red.log`, `red-sections.log`).

| line | case | asserts | now |
|---|---|---|---|
| 56 | `a_page_section_that_overrides_its_kinds_section_is_read` | a `list_page` with `{name: list, columns: [title]}` (no `component`) is read and saved as written | red |
| 65 | `an_overlay_with_same_as_is_read` | an overlay `{kind: drawer, same_as: a.edit}` is read | red |
| 74 | `a_board_widget_that_is_a_primitive_is_read` | `board.widgets.hint: {primitive: text}` is read | red |
| 83 | `the_reads_shorthand_is_read` | `reads: x.All` is read | red |
| 93 | `a_page_that_inherits_every_section_is_written_as_authored` | a page without `sections` is saved without them | red |
| 104 | `a_page_without_shell_renders_in_shells_app_not_the_first_shell` | uilab's `check` reports no error ESS does not | red |
| 142 | `expansion_bound_counts_a_page_kinds_widget_once_per_page` | a document past the expansion limit is refused within 10s | red |
| 318 | `every_layer_maps_to_ess_and_back_and_is_written_as_authored` | path round trip and key order on a document that uses every layer | green |
| 395 | `fixtures_views_is_written_back_unchanged` | known gap | green |

Every case first checks that ESS 0.48.0 finds 0 errors in the document. The red output, verbatim:
```
uilab refuses an ess-ui/1 document ESS reads clean: /: pages.a: section `list`: missing field `component` at line 14 column 5
uilab refuses an ess-ui/1 document ESS reads clean: /: pages.b.overlays.edit: missing field `component` at line 24 column 13
uilab refuses an ess-ui/1 document ESS reads clean: /: pages.a: section `board`: missing field `component` at line 14 column 5
uilab refuses an ess-ui/1 document ESS reads clean: /: pages.a: section `list`: invalid type: string "x.All", expected struct Reads at line 14 column 5
written back as authored  left: … "a": {"kind": "list_page", "title": "Loans", "sections": Sequence []}  right: … "a": {"kind": "list_page", "title": "Loans"}
uilab reports an error ESS does not …: Finding { check: "page_outlet", severity: Error, path: "page:a", message: "shell `auth` has no page_outlet region to render it in" }
not refused within 10s: the bound let the document through and ESS is expanding 2 × 98302 nodes
```

Two ignored probes back these up:
- The expansion probe (line 171) reads the same document in 33.17s with no error, so the bound really does let it through.
- The example probe (`ESS_UI_EXAMPLE`) feeds in ESS 0.48.0's own reference example, `examples/partner-portal/ui.yaml`. ESS finds 0 errors; uilab refuses it: `pages.partners.list: section `filters`: missing field `component``.

**3. Suite run**, after the cases existed: `cargo test --locked -p uilab-doc --no-fail-fast`
```
Running tests/adversary_essui_document.rs … test result: FAILED. 2 passed; 7 failed; 1 ignored
every other uilab-doc binary: ok (5+3+2+6+12+6+6+3+9+4+7+6+43+9+3)
EXIT=101
```
- 124 is the uilab-doc count from the implementer's `gate-test.log`.
- The second ignored probe was added after this run. It is not a case.

**4. Findings**

| file:line | verdict | origin | what it is | what reaches it |
|---|---|---|---|---|
| model.rs:311 | NEEDS-CHANGE | introduced | `Composite.component` is required. In ess-ui/1 a page section that refines its kind's section leaves `component` out, and so does a `same_as` overlay (model.rs:368). | ESS's own reference example. The format docs also say to declare only what differs from the page kind. |
| check.rs:313 | NEEDS-CHANGE | introduced | `expansion` counts a widget used in `page_kinds` once. ESS expands it again on every page of that kind, so 2 pages get past the 100 000 limit. | Any page kind that uses a widget, on many pages. The 98k-node widget chain is one I built. |
| model.rs:317 | CONFIRMED | introduced | `board.widgets` is typed as a `Composite`. The schema allows any node there, including a primitive. | Nothing found in the repo. The schema allows it. |
| model.rs:314 | CONFIRMED | introduced | The `Reads` shorthand (`reads: <view>`) is not accepted. | Nothing found in the repo. It is listed in the schema's shorthands. |
| model.rs:1030 | CONFIRMED | undecided | When a page names no shell, `shell_of` picks the first shell. ESS uses `shells.app`, or the only shell. The surviving `page_outlet` check then raises a false error, and `admit` refuses valid patches. | A sign-in shell declared before `app`. The schema itself suggests one shell per frame. |
| model.rs:219 | CONFIRMED | introduced | A page saved without `sections` comes back with `sections: []`, a key the author never wrote. | A page that inherits every section from its kind. |
| patch.rs `admit` | INFEASIBLE | undecided | Judgement, no test. `from_yaml` only runs ESS's loader, not its checker. A file that already has checker errors (for example `home: ghost`) still loads, patches are accepted, and the saved file fails `ess ui check`. This follows D1 but contradicts the brief's requirement. | Any file opened with checker errors. The coordinator decides. |

**5. Attacked and could not break**
- **Path round trip:** shell overlay items, board widget items, `children`, `parts`, `choices`, `toolbar`, widget body items, the nav section, and names `Big2` and `v2.list` all hold. Every ESS node lands on a node uilab has.
- **Write-back:** key order and unknown keys survive on that document, including `actor`, `types`, `placement_defaults`, region `props` with `ratio: 1.5` and `'007'`, `load` and `nav`.
- **The four survivors:** the `ess` 0.48.0 CLI reports `0 error(s), 0 warning(s)` on the `nav_unique`, `shell_refs` and `page_outlet` documents. `replace_drops` is covered by the existing test.
- **`fixtures.views`:** ESS 0.48.0 reads it. uilab reads it and writes it back unchanged. This contradicts the acceptance line, as already known.
- **Rewritten older tests:** the number of `assert`s per file did not go down (doc.rs 236→240). The check ids changed to ESS's `widget_expands` and `document_loads`, which is the expected format change.
- **Tests outside uilab-doc:** only the document text changed. One scenario was narrowed: `adversary_components_workspace.rs` gave its two same-named header metrics different names, so it no longer tests same-named instances. The implementer says ESS refuses same-named metrics; I did not verify that.
- **Names:** a name containing `/` is refused by ESS's loader with ESS's path.
- I took no session lease.

**6. Paths written outside the worktree**
- `~/.cache/uilab-wave-w9/essui-document/adversary-1/`: `red.log`, `red-sections.log`, `expansion-probe.log`, `example-probe.log`, `suite.log`, plus 18 probe documents in `surv/`.
- Build output went to the assigned `~/.cache/b10x-target/uilab-essui-document`.

```findings
[
  {"file": "crates/uilab-doc/src/model.rs", "line": 311, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Composite requires component, so uilab refuses ess-ui/1 page sections that refine their kind's section and same_as overlays, including ESS 0.48.0's own partner-portal example (red: adversary_essui_document.rs:56, :65)"},
  {"file": "crates/uilab-doc/src/check.rs", "line": 313, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "expansion_bound weighs a page_kinds widget use once while ESS expands it on every page of that kind, so two pages pass 196604 nodes to ESS (33s per load in debug) (red: :142)"},
  {"file": "crates/uilab-doc/src/model.rs", "line": 317, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "board.widgets is typed as Composite, so a primitive board widget that ESS reads clean is refused (red: :74)"},
  {"file": "crates/uilab-doc/src/model.rs", "line": 314, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "the Reads shorthand reads: <view> from the ess-ui/1 shorthand index is refused (red: :83)"},
  {"file": "crates/uilab-doc/src/model.rs", "line": 1030, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "shell_of defaults to the first shell instead of shells.app or the only shell, so the surviving page_outlet check raises a false error on a document ESS checks clean (red: :104)"},
  {"file": "crates/uilab-doc/src/model.rs", "line": 219, "category": "acceptance", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "a page that inherits every section is saved with sections: [], a key the author never wrote (red: :93)"},
  {"file": "crates/uilab-doc/src/patch.rs", "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "undecided", "message": "from_yaml runs only ESS's loader, so a file with ESS checker errors is opened, edited and saved still failing ess ui check, which follows D1 but contradicts the brief's 0-errors-on-write requirement"}
]
```
