---
format: aep.planning-md/3
id: story:drafts-not-warnings
kind: story
status: implemented
title: Draft reads are data to model, not warnings
relations:
- decomposes: epic:ux-fidelity
- serves: vision:website-harness
scope:
- confidence: cited
  path: widget/src/components/SidebarPanel.vue
- confidence: cited
  path: widget/src/lib/sidebar.ts
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T12:43:38Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T12:43:38Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T13:44:06Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

Reads of `draft.` views are not warnings in the sidebar: the findings badge counts errors and warnings without `draft_read`, and a "Data to model" list names each draft view once with the sections that read it and a click that selects the first. The check still reports `draft_read` (the agent, the eval and `/api/state` see it).

## Found by

Operator, 2026-09-30, three times: "loading the page shows these warnings" — five `draft_read` lines for three views (`draft.LoansPerMonth` ×3, `draft.LoansByState`, `draft.ProfileSettings`). They are the document's open data needs, not faults, and the badge shows `⚠ 5` on every load.

## Acceptance

- lib tests (`widget/src/lib/sidebar.ts`): the badge ignores `draft_read`; a document with only draft reads shows ✓; the draft list groups by view in document order with its section paths.
- The sidebar shows the list collapsed with a count ("3 draft views"), expandable; clicking a section path selects it.
- Screenshot on a copy of the operator's document recorded as evidence.
