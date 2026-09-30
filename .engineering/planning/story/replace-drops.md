---
format: aep.planning-md/3
id: story:replace-drops
kind: story
status: draft
title: Warn when a replace drops what was there
relations:
- decomposes: epic:workbench
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-doc
revision: 2
---
## Outcome

A `replace` that removes existing children (sections, overlays, widgets, items, nodes) or existing entries of `columns` / `fields` / `row_actions` is flagged with a warning finding `replace_drops`, naming what goes, so the operator sees it on the proposal card before accepting.

## Found by

Wave 2026-09-30 w2, story:agent-widgets live check: "make a reusable card for a member ... use it on the members page" came back as a replace of `page:members` that renamed section `list` and dropped its 4 columns (name, joined, loans, standing); `admit` accepted it without a word.

## Acceptance

`cargo test -p uilab-doc` passes with cases: replacing a page that drops a section warns naming it; replacing a collection that drops a column warns naming it; a replace that only adds or changes warns nothing; the warning appears in `admit`'s findings.
