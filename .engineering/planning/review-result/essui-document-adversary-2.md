---
format: aep.planning-md/3
id: review-result:essui-document-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:essui-document
relations:
- reviews: story:essui-document
revision: 1
---
unit: story:essui-document, commit 0d731f9 plus one untracked test file
verdict: NEEDS-CHANGE
cases: executed 134→146, red 9 (the suite run also had 1 red existing timing test, not mine; see part 3)
origin: introduced 6 / pre-existing 0 / undecided 2
wrote-outside-worktree: 20 files, all under the assigned scratch directory; build output in the assigned build dir
needs-coordinator: yes. An existing timing test failed at load average 70 (part 3). It needs a re-run on a quiet machine before anyone attributes it.

**1. Diff**

`git --no-pager diff --stat` is empty: no tracked file changed. `git status --short` shows one untracked file, and it is a test file:
```
?? crates/uilab-doc/tests/adversary_essui_document_p2.rs
```
I did not touch any implementation file. I ran `rustfmt` on my own file only.

**2. Cases added** (`crates/uilab-doc/tests/adversary_essui_document_p2.rs`)

I ran this file on its own before the suite. Every read case first asserts that ESS 0.48.0 finds 0 errors in the document. Red output, verbatim (`red.log`):

| line | case | now |
|---|---|---|
| 66 | `overlays: null` (removes every overlay the page kind brings) is read and written back | red |
| 77 | a board widget that refines its kind's widget (`k: {label: Total}`) is read | red |
| 88 | a board widget written as the Composite shorthand (`k: rich_text`) is read | red |
| 98 | a shell overlay that is `same_as` a page overlay | green |
| 108 | a refined `item` entry plus a `choices` `{name, remove: true}` | green |
| 118 | `props: {ratio: .inf}` is written back unchanged | red |
| 138 | inserting an item into a section that refines `list_page`'s collection | red |
| 168 | replacing and inserting section refinements: written as patched, and ESS reads the result clean | green |
| 257 | `expansion_bound`: 3 `same_as` copies of a 49,150-node overlay | red |
| 277 | `expansion_bound`: `args` substitution doubling over 20 levels | red |
| 299 | `expansion_bound`: a widget use under `pages.p0.nav` | red |
| 316 | `expansion_bound` must not refuse a document where `k2 extends k1` replaces k1's widget section | red |

```
uilab refuses an ess-ui/1 document ESS reads clean: /: pages.a.overlays: invalid type: unit value, expected a map at line 18 column 15
uilab refuses an ess-ui/1 document ESS reads clean: /: pages.a: section `board`: node `k` has exactly one of `component` and `primitive` at line 23 column 5
uilab refuses an ess-ui/1 document ESS reads clean: /: pages.a: section `board`: a node is a map at line 14 column 5
written back as authored … "props": Mapping {"ratio": Null} … vs … "props": Mapping {"ratio": Number(.inf)}
uilab refuses an insert whose result ESS reads clean: layer_allowed: a item cannot be added under `page:a/section:list`; allowed: child
uilab refuses a document ESS reads clean in milliseconds: pages/p0: the document's widget uses expand to 393208 nodes, more than the 100000 uilab hands to ESS; this use alone expands to 393208
not refused within 10s: the bound let the document through and ESS is expanding 4 × 49150 widget nodes through same_as
not refused within 10s: the bound let the document through and ESS is expanding 2^17 levels of widget under pages.p0.nav
not refused within 10s: the bound let the document through and ESS is substituting a 2^20-leaf argument
test result: FAILED. 3 passed; 9 failed
```
After that run I reworded one panic message (line 303) to "about 2^18 widget nodes". The assertion is unchanged.

**What ESS itself does with the expansion documents** (release `ess` 0.48.0 CLI on the same documents):

| document | ESS result | time |
|---|---|---|
| `same_as` copies | 0 errors | 7.06s |
| `args` doubling | 0 errors, about ×2 per level | depth 18: 7.26s |
| widget under `nav` | refused after full expansion | 45.6s |
| `k2` replaces k1's widget section | 0 errors | 0.04s |

**3. Suite run**, made after the cases existed: `cargo test --locked -p uilab-doc --no-fail-fast` (`suite.log`)
```
adversary_essui_document_p2.rs  FAILED. 3 passed; 9 failed
adversary_widget_opens.rs       FAILED. 6 passed; 1 failed   (a_chain_doubling_at_each_of_twelve_levels_is_checked_in_bounded_time)
every other binary ok: 5+3+9+2+6+12+6+6+3+9+4+6+43+10+3
EXIT=101
```
- **The before count:** 134 is the 146 cases that ran minus my 12. That matches pass 1's 133 plus `ess_reference_example_loads`.
- **The `widget_opens` failure is a timing test.** The run measured `check 19.5s` against a 5s bound. Run alone it still failed (`check 10.1s, admit 8.8s`) at load average 70–73. It passed in pass 1. I can't separate machine load from a slowdown in 0d731f9, so it needs a re-run on a quiet machine.

**4. Findings** (they cover 0d731f9)

| file:line | verdict | origin | what | what reaches it |
|---|---|---|---|---|
| check.rs:463 | NEEDS-CHANGE | introduced | Same root cause in all three slips: `expansion` weighs widget nodes only where uilab types them. Example: a `same_as` overlay counts 0, but ESS copies it before it expands widgets. | Constructed documents. The ESS-clean ones (`same_as`, `args`) hang uilab's loader and `admit`. |
| check.rs:399 | NEEDS-CHANGE | introduced | `weigh` counts nodes. ESS's `substitute` copies `args` values, which can double at every level while each level writes one node. | Constructed. |
| check.rs:302 | CONFIRMED | introduced | `widget_uses` covers typed composites, `header` and `page_kinds`. ESS expands every non-`OPAQUE` key under `shells` and `pages` before it reads the model. | Constructed; ESS refuses this document anyway, but only after the hang. |
| check.rs:476 | CONFIRMED | introduced | The `extends` walk charges a base kind's widget to pages whose kind replaced that section. | Constructed. Errs toward refusing, not toward a hang. |
| model.rs:373 | NEEDS-CHANGE | introduced | Board `widgets` values must carry `component` or `primitive`. A refinement and the Composite shorthand are both refused. | Any page that refines a user kind's board, which is the documented "declare only what differs". |
| model.rs:223 | CONFIRMED | introduced | `overlays: null` (remove all inherited) is refused. | ESS's `Kinds::merge` honours it. |
| path.rs:482 | CONFIRMED | introduced | A section with `Component::Inherited` is offered only `child`. Its item, part and choice lists can't be inserted into. | Any refined section, e.g. `{name: list, columns: […]}`. |
| model.rs:117 | CONFIRMED | undecided | Props are held as JSON, so `.inf` and `.nan` are written back as `null`. | Constructed. |

**Judgement, no new case:** `ess_ui.rs:1001`, `:1025` and `:1240` cannot fail. `node_at` falls back to the root path, and the root path always resolves (`ess.rs:229`), so "every ESS node maps to a uilab node" holds by construction. A mutant `node_at = root` would leave those lines green; line 989 is the one that would catch it.

**5. Attacked and could not break**
- **Round trips:** shell-overlay `same_as`, refined `item` entries and `choices` removal round-trip; refinement replace and insert are written as patched and ESS reads them clean.
- **Survivors:** `SURVIVOR_CASES` did not change between 55e2c8d and 0d731f9, so pass 1's ESS verification still holds.
- **Tests outside uilab-doc:** 0d731f9 touches no file outside `crates/uilab-doc`.
- **`fixtures.views` (known gap):** pass 1's case `fixtures_views_is_written_back_unchanged` is still green. uilab reads it and writes it back unchanged.
- **Shapes ESS refuses:** `sections: null`, an overlay written as a bare composite name, and integer map keys are all refused by ESS, so uilab refusing them is correct.
- I took no session lease.

**6. Paths written outside the worktree**
- `~/.cache/uilab-wave-w9/essui-document/adversary-2/`:
  - `red.log`, `suite.log`, `schema.yaml`
  - `probe/` with 17 YAML documents and `gen-args.sh`, `gen-chain.sh`
- Build output went to the assigned `~/.cache/b10x-target/uilab-essui-document`.

```findings
[
  {"file": "crates/uilab-doc/src/check.rs", "line": 463, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "expansion_bound weighs a same_as overlay as 0 while ESS copies it before expanding widgets, so 4 x 49150 nodes reach ESS from an ESS-clean document (red: adversary_essui_document_p2.rs:257)"},
  {"file": "crates/uilab-doc/src/check.rs", "line": 399, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "weigh counts nodes only, but ESS substitute copies args values, so a 21-node ESS-clean document whose args double per level hangs the loader (red: adversary_essui_document_p2.rs:277)"},
  {"file": "crates/uilab-doc/src/check.rs", "line": 302, "category": "boundary", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "widget_uses misses widget uses ESS expands under any non-OPAQUE key of shells and pages, e.g. pages.p0.nav, so ESS expands 2^18 nodes before refusing (red: adversary_essui_document_p2.rs:299)"},
  {"file": "crates/uilab-doc/src/check.rs", "line": 476, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "expansion_bound charges a base kind's widget section to pages of a kind that replaces it, refusing a document ESS reads clean in 0.04s (red: adversary_essui_document_p2.rs:316)"},
  {"file": "crates/uilab-doc/src/model.rs", "line": 373, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "board widgets require component or primitive, so a refinement of a kind's board widget and the Composite shorthand k: rich_text are refused though ESS reads both clean (red: adversary_essui_document_p2.rs:77, :88)"},
  {"file": "crates/uilab-doc/src/model.rs", "line": 223, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "overlays: null, which removes every inherited overlay in ESS's merge, is refused by uilab (red: adversary_essui_document_p2.rs:66)"},
  {"file": "crates/uilab-doc/src/path.rs", "line": 482, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "allowed_children reads no kind from Component::Inherited, so an item cannot be inserted into a section refining list_page's collection though ESS reads the result clean (red: adversary_essui_document_p2.rs:138)"},
  {"file": "crates/uilab-doc/src/model.rs", "line": 117, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "undecided", "message": "props held as serde_json Value write a YAML .inf or .nan back as null (red: adversary_essui_document_p2.rs:118)"}
]
```
