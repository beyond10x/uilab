---
format: aep.planning-md/3
id: story:proposal-marks
kind: story
status: implemented
title: Proposal preview marks added, changed and removed nodes
relations:
- decomposes: epic:ux-fidelity
- serves: vision:website-harness
scope:
- confidence: cited
  path: widget/src/lib/outline.ts
- confidence: cited
  path: widget/src/store.ts
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T12:08:57Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T12:08:57Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T12:39:37Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome

A proposal preview marks each node by what the proposal does to it: added, changed or removed. A batch that adds a dialog and a row action shows the dialog as added, the section that gains the row action as changed, and nothing as removed.

## Found by

`widget/src/store.ts:135` maps every op other than Insert and Replace to `remove`, so every Batch proposal draws the changed node and its whole subtree in red strikethrough (screenshot `proposal-proposal.png`: a batch adding an `extend_loan` dialog struck through the whole Members page).

## Acceptance

- Marks come from comparing the proposal's outline with the document's: a path only in the proposal is added, only in the document is removed, in both with different props, title, kind or view is changed.
- lib tests (node --test): a Batch of insert + replace marks one node added and one changed; a Remove marks the subtree removed; an Insert marks only the new subtree.
- The canvas and the tree use the same marks.
