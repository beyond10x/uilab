---
format: aep.planning-md/3
id: epic:ess-ui-adoption
kind: epic
status: active
title: uilab reads, edits and writes only ESS ess-ui/1
relations:
- serves: vision:website-harness
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T11:11:13Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-01T11:11:13Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

uilab reads, edits and writes only ESS's published UI format, `ess-ui/1`. Every document uilab
loads or saves, and every patch it admits, is decided by ESS's own checker (`ess-ui-check`), and
`ess ui check` reports 0 errors on every document in the repository and on every document uilab
writes. uilab is the lab that shows what `ess-ui/1` can do; it owns no format of its own.

## Rule (operator, 2026-10-01)

"I want ess to have a spec format to specify ui. uilab shall use it. uilab may have some
intermediate thing which helps uilab to integrate better. But output must be ess provided spec.
Spec features must evolve in ../ess itself." A construct uilab needs and `ess-ui/1` lacks is filed
in ESS and waits for an ESS release; uilab never adds it to the document on its own.

## Why now

uilab was built on 2026-09-30 against `ui-spec/1`, a draft schema kept outside ESS. ESS formalised
the same draft as `ess-ui/1` an hour later (ess `51aee77`, released 0.47.0; 0.48.0 is current) and
uilab never followed (`crates/uilab-doc/src/lib.rs:13-15` says it would).

## Gap, measured with ess 0.48.0 `ess ui check`

On `examples/library/library.ui.yaml`, converted by hand until it loads:

| draft (`ui-spec/1`) | `ess-ui/1` |
|---|---|
| `format: ui-spec/1` | `format: ess-ui/1` |
| `pages.*.sections` is a map keyed by name | a list of nodes, each with `name` |
| `fixtures.views` maps view to file | `fixtures: {dir, index}`, the index file lists views |
| document-level `views:` with inline rows | no such key; rows live in fixture files |
| section `title` | no section title; `metric` has `label` |
| a `draft.<x>` view as a placeholder | `reads: {placeholder: <x>, fixture: <file>}`, warned as `unbound_placeholder` until bound |
| `dashboard_page` without a board | the kind merges a built-in `board` section that must `reads` |

On the operator's working copy (452 lines, built by the agent in uilab sessions), after the
mechanical conversion, ESS still refuses 19 findings uilab had admitted: sections naming widgets
that do not exist (`header`, `page_intro`), widget params without `note`, `board` without `reads`,
`filter_bar` with `reads`, `form` without `does`, `collection` with `title`, navigation naming
pages that the refusals removed. uilab's own checks are weaker than ESS's; that is the defect this
epic closes.

## Decisions

- D1. uilab edits the authored document (unexpanded, key order kept). ESS's `ess-ui` crate loads
  and expands it, and `ess-ui-check` checks it, both as git dependencies of `crates/uilab-doc` only, on ess tag `0.48.0`
  (`5dbda40be9d30687c347f54f2b617718a094c2eb`), re-exported from `uilab_doc` so no other crate adds
  them. A patch is refused when its result has an ESS
  error finding the document did not already have.
- D2. uilab's node paths (`page:loans/section:list`) stay on the wire as uilab's intermediate. A
  tested bijection maps them to ESS canonical paths (`pages/loans/sections/list`); every ESS finding
  is shown on its uilab node. ESS paths on the wire are not part of this epic.
- D3. The node shapes the agent may write come from `ess_ui::SCHEMA`, not from a hand-written list.
- D4. A uilab check survives only where ESS has no counterpart; each survivor is listed in
  story:essui-document with its reason, and filed in ESS when it is a fact about the format.
- D5. uilab pins ess 0.48.0 (`ess-inputs/2`, `requires: ess 0.48.0`) for its own specification and
  for `ess ui check` in `task check`.
- D6. The canvas shows what ESS renders: the expanded document, page-kind and widget contributions
  included and marked as inherited.
- D7. The demo server keeps running the current binary until the integration gate is green on the
  merged pull request (story:eval-round-5 carries the switch).

## Stories

| wave | story | what |
|---|---|---|
| 1 | story:essui-document | uilab-doc reads, patches and writes `ess-ui/1`; ESS decides admission; examples converted; ess pinned to 0.48.0, uilab's own specification regenerated |
| 2 | story:essui-library-model | an ESS specification for the library example; `ess ui check --model` is clean |
| 2 | story:essui-agent-schema | the agent's field shapes and prompt come from `ess-ui/1` |
| 2 | story:essui-app-widget | server and browser on `ess-ui/1`: the expanded document on the canvas, findings by node, placeholders |
| 3 | story:essui-docs | AGENTS.md, README and the docs site say `ess-ui/1`; screenshots retaken |
| 3 | story:eval-round-5 | the eval suite on `ess-ui/1` through the LLM agent; the demo server switches (D7) |

## ESS requirements filed

- beyond10x/ess#281 (https://github.com/beyond10x/ess/issues/281), filed 2026-10-01 by b10x-bot:
  - a section has no heading: uilab sections carry a `title`; `ess-ui/1` sections have none;
  - a page of a built-in kind cannot leave out a section the kind contributes (`dashboard_page`
    and `board`).
- Until ESS answers, uilab writes no section `title`, and the library overview uses a kind that
  loads.

## Not covered

- ESS paths on the wire (D2).
- Style tokens (epic:styles; ess story `ui-spec-style-tokens` is a draft).
- Converting the operator's 452-line working copy: it is archived, and the demo starts from the
  converted example.
