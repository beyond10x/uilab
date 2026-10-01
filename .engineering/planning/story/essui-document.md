---
format: aep.planning-md/3
id: story:essui-document
kind: story
status: active
title: uilab-doc on ess-ui/1, admission decided by ESS
relations:
- decomposes: epic:ess-ui-adoption
- serves: vision:website-harness
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: inferred
  path: crates/uilab-agent/src
- confidence: inferred
  path: crates/uilab-agent/tests
- confidence: inferred
  path: crates/uilab-app/src
- confidence: inferred
  path: crates/uilab-behaviour/src/source.rs
- confidence: cited
  path: crates/uilab-doc/Cargo.toml
- confidence: cited
  path: crates/uilab-doc/src/check.rs
- confidence: cited
  path: crates/uilab-doc/src/docs.rs
- confidence: cited
  path: crates/uilab-doc/src/fixtures.rs
- confidence: cited
  path: crates/uilab-doc/src/lib.rs
- confidence: cited
  path: crates/uilab-doc/src/model.rs
- confidence: cited
  path: crates/uilab-doc/src/patch.rs
- confidence: cited
  path: crates/uilab-doc/src/path.rs
- confidence: cited
  path: crates/uilab-doc/tests
- confidence: cited
  path: ess/domains/session.yaml
- confidence: cited
  path: ess/ess-inputs.yaml
- confidence: cited
  path: ess/system.yaml
- confidence: cited
  path: examples/empty.ui.yaml
- confidence: cited
  path: examples/library/fixtures
- confidence: cited
  path: examples/library/library.ui.yaml
- confidence: cited
  path: generated
- confidence: cited
  path: widget/src/generated
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T11:11:19Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":9}}}
- {from: "proposed", to: "active", at: "2026-10-01T11:11:19Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":9}}}
---
## Outcome

`crates/uilab-doc` reads, patches and writes `ess-ui/1` and nothing else. ESS decides: the `ess-ui`
crate loads and expands a document, `ess-ui-check` checks it, both git dependencies of
`crates/uilab-doc` only, on ess tag `0.48.0` (`5dbda40be9d30687c347f54f2b617718a094c2eb`), and
re-exported from `uilab_doc` so no other crate adds them. uilab keeps its own editing layer (node
paths, patches, outline, sample rows) on top. This story also moves uilab's toolchain and its own
specification to ess 0.48.0. Epic decisions D1, D2, D4 and D5.

## Acceptance

Each test named here lives in `crates/uilab-doc/tests/ess_ui.rs` unless it says otherwise.

- `loads_only_ess_ui`: a document with `format: ui-spec/1` is refused, and the message names
  `ess-ui/1`; a document ESS's loader refuses is refused with ESS's message and ESS path.
- `sections_are_named_lists`: page sections are read from and written as lists of `{name, …}`;
  insert, replace, remove and reorder keep every other node's path, and the written YAML keeps the
  authored key order.
- `saves_authored_only`: after every patch kind (insert, replace, remove, batch) on the library
  example, the written file holds no node a page kind or widget contributes (no `filters` section
  on a `list_page` that did not author one, no widget body under an instance); the test compares
  the saved file with the authored document plus the patch.
- `fixtures_index`: `fixtures: {dir, index}` with an index file is read; a read's sample rows come
  from the file the index names; a document carrying the draft's document-level `views:` key or
  `fixtures.views` is refused by ESS's loader and uilab relays it.
- `placeholder_reads`: `reads: {placeholder: <name>, fixture: <file>}` replaces every `draft.<x>`
  view; a placeholder whose fixture file exists yields its rows, one whose file does not yields
  generated rows marked as samples. `grep -rn '"draft\.' crates/uilab-doc/src crates/uilab-app/src`
  prints nothing and exits 1.
- `ess_decides_admission`: one case per class below, each a patch that introduces exactly that
  defect, is refused naming the ESS check id and the uilab node; a patch that adds no new ESS error
  is admitted:
  1. a section naming a widget that does not exist (`component: page_intro`);
  2. a widget param without `note`;
  3. `component: board` without `reads`;
  4. `component: filter_bar` with `reads`;
  5. `component: form` without `does`;
  6. `component: collection` with `title`;
  7. a navigation entry naming a page that does not exist.
- `children_from_ess`: what each node may hold (`path.rs`, `allowed_children`) is derived from
  `ess_ui::SCHEMA`: for every composite kind, page kind and region kind the schema declares, the
  child layers uilab offers equal the schema's. story:essui-agent-schema builds the field shapes on
  this, it does not derive children again.
- `paths_round_trip`: for every node of every document in `examples/`, the uilab path maps to the
  ESS canonical path and back unchanged; every ESS finding on those documents maps to an existing
  uilab node.
- `uilab_checks_survivors`: of the 18 checks `crates/uilab-doc/src/check.rs` runs today
  (`format_marker`, `nav_resolves`, `page_reachable`, `nav_unique`, `shell_refs`,
  `page_kind_known`, `page_outlet`, `opens_resolves`, `section_refs`, `fixture_per_view`,
  `draft_read`, `unmapped_reported`, `widget_named_like_builtin`, `widget_args`,
  `widget_recursion`, `widget_resolves`, `names_unique`, `replace_drops`), the rule is mechanical:
  the test holds one document (or, for `replace_drops`, one patch) per uilab check that fails that
  check and nothing else. Where `ess_ui_check` reports an error on it, the uilab check is covered
  and is deleted; where ESS reports none, it survives. The test asserts both halves for all 18, so
  the survivor list is what ESS answers, not a choice. `replace_drops` is a rule about a patch, not
  a document, and is expected to survive. This story's `## Survivors` section lists each survivor
  with the document the test used; a survivor other than `replace_drops` is a fact about the format
  and is filed in ESS (the coordinator files it; the issue number goes into `## Survivors`).
- Examples: `examples/library/library.ui.yaml` and `examples/empty.ui.yaml` are `ess-ui/1`;
  `ess ui check --path <each>` with ess 0.48.0 prints `0 error(s), 0 warning(s)`; the examples read
  views only (no `reads.placeholder`, which ESS always warns on until bound). The overview page
  keeps its metric and recent-loans list; `## Survivors` records which page kind it uses and why
  (beyond10x/ess#281).
- Toolchain: `ess/ess-inputs.yaml` is `ess-inputs/2` with `requires: ess 0.48.0`; uilab's own
  specification names `ess-ui/1` where it named `ui-spec/1` (`ess/system.yaml:3`,
  `ess/domains/session.yaml:26,107,159`); `task generate` under 0.48.0 is committed and
  `task drift` is clean.
- `task check` runs `ess ui check` on every `examples/**/*.ui.yaml` and fails on any error; it
  exits 0. In `crates/uilab-app`, `crates/uilab-agent` and `crates/uilab-behaviour`, existing tests
  change only in their document text (format marker, section lists, placeholder reads, fixtures
  index); no assertion is removed or weakened. The unit report lists every test changed there with
  the line that changed, and the adversary checks that list against `git diff`. The agent's prompt
  text is not changed here (story:essui-agent-schema).

## Scope

- crates/uilab-doc/** except `src/schema.rs` and `src/outline.rs` beyond what compiles (cited)
- Cargo.toml, Cargo.lock (cited: the git dependencies; no other story in the epic edits them)
- examples/library/library.ui.yaml, examples/library/fixtures/**, examples/empty.ui.yaml (cited)
- ess/ess-inputs.yaml, ess/system.yaml, ess/domains/session.yaml, generated/**,
  widget/src/generated/** (cited: the pin, the `ui-spec/1` text, `state.json` records producer
  ess 0.44.0)
- Taskfile.yml (cited: the `check` task gains `ess ui check`)
- crates/uilab-app/src/**, crates/uilab-agent/src/** except the prompt text, crates/uilab-agent/tests/**,
  crates/uilab-behaviour/src/source.rs (inferred: API call sites only)

## Survivors

Recorded by the coordinator from the implementor report, 2026-10-01 (unit commit 55e2c8d).

- Survivors, each tested on its document in `SURVIVOR_CASES` (`crates/uilab-doc/tests/ess_ui.rs`),
  ESS 0.48.0 reporting nothing on it: `nav_unique`, `shell_refs`, `page_outlet`; and
  `replace_drops`, tested on its patch (a rule about a patch, not a document).
- Guard, not a check: `expansion_bound` refuses a document whose widget uses would expand past
  100,000 nodes, because ESS expands nested widgets exponentially (beyond10x/ess#300).
- Overview page: `dashboard_page` with `{name: board, remove: true}`, which the ESS schema's
  named-list merge supports (`named_lists.remove`; correction posted on beyond10x/ess#281).
- Gap against the acceptance: ESS 0.48.0 accepts `fixtures.views`; the `fixtures_index` test
  relays whatever ESS answers, and uilab writes `{dir, index}`.
- Filed in ESS: beyond10x/ess#303 (the three document survivors), 2026-10-01.
