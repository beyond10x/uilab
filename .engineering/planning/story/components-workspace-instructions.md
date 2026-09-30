---
format: aep.planning-md/3
id: story:components-workspace-instructions
kind: story
status: implemented
title: Instructions on the Components tab build widgets
relations:
- decomposes: epic:component-library
- serves: vision:website-harness
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T08:09:01Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-30T08:09:02Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-30T08:09:02Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

An instruction or goal given on the Components tab lands at the root, or at the selected widget or body node, and the agent is told the operator is building the component library, so it declares widgets under `widgets:` instead of adding page sections. The proposal card lists only the findings the proposal brings; the document's own findings stay in the sidebar.

## Found by

Operator report, 2026-09-30: "put some basic set of components" spoken on the Components tab became three sections of `page:overview` (session journal 1790749266, `proposed` target `page:overview`). Every proposal card repeated the document's five `draft_read` warnings.

## Acceptance

- `an_instruction_from_the_components_workspace_lands_among_the_widgets`, `a_goal_from_the_components_workspace_is_planned_among_the_widgets` (crates/uilab-app).
- `an_instruction_from_the_components_workspace_asks_for_widgets` (crates/uilab-agent).
- `widget/src/lib/workspace.test.ts` (5 cases).
- `a_proposal_card_names_only_the_findings_the_proposal_brings` (crates/uilab-app).
- Live replay on a copy of the operator's document: target `/`, a batch of five widget inserts.

Implemented at 54342bf, merged at 732bd4e.
