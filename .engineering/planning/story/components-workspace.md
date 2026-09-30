---
format: aep.planning-md/3
id: story:components-workspace
kind: story
status: active
title: Components workspace in the browser
relations:
- decomposes: epic:component-library
- serves: vision:website-harness
- depends_on: story:widget-model
scope:
- confidence: cited
  path: crates/uilab-app
- confidence: cited
  path: widget
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T05:00:08Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T05:00:08Z", actor: "human:timo", revision: 4}
---
## Outcome

A Components workspace in the browser: a fourth canvas tab listing the document's widgets with summary, params and use sites, a search box, and a preview of each rendered with sample args; selecting one selects its node so instructions land on it.

## Acceptance

`pnpm check` passes with tests for widget listing, search and sample args; the server outline carries widgets; live, a widget created by voice appears in the tab.
