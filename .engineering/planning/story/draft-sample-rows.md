---
format: aep.planning-md/3
id: story:draft-sample-rows
kind: story
status: active
title: Draft views show sample rows in the browser
relations:
- decomposes: epic:ux-fidelity
- serves: vision:website-harness
scope:
- confidence: cited
  path: widget/src/components/CompositeView.vue
- confidence: cited
  path: widget/src/store.ts
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T12:08:58Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T12:08:58Z", actor: "human:timo", revision: 4}
---
## Outcome

A composite that reads a `draft.` view shows sample rows marked "sample data": a collection shows rows, a chart draws bars, a metric shows a number. It never shows "loading …" forever or an empty placeholder.

## Found by

The server answers `rows` for a draft view with `sample_rows` (`crates/uilab-app/src/app.rs:894`), but the browser never asks: `requestRows` returns early for a draft view (`widget/src/store.ts:404`). Screenshots: `chart · draft.LoansPerMonth` placeholder on Overview; `loading draft.MembersWithOverdue…` in a proposal preview.

## Acceptance

- lib/store test: `requestRows('draft.X')` sends a `rows` message.
- The collection, chart and metric renderers show the rows with a "sample data" tag.
- The `draft_read` warning stays: sample data is not a model binding.
