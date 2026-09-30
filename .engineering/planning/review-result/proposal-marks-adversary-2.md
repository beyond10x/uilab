---
format: aep.planning-md/3
id: review-result:proposal-marks-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:proposal-marks
relations:
- reviews: story:proposal-marks
revision: 1
---
unit: story:proposal-marks, branch unit/proposal-marks; findings cover c687fc2 (cases in 0a25e38)
verdict: NEEDS-CHANGE
cases: executed 124→136, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths under ~/.cache/uilab-wave-w5/proposal-marks/adversary2/
needs-coordinator: yes. Round 2 ruled a menu section's `pages` never counts; it is the section's own list, so moves and reorders mark nothing.

Cases: `marks.adversary2.test.ts` (a Batch moving a page between sections and a Replace reordering one — red; relabel plus removal, two removals from one section, remove and re-add elsewhere, dynamic sections — green); `marks.store.adversary2.test.ts` drives the real store through a fake transport (base on arrival, other operators' changes, re-send after reconnect, refused accept, second proposal, accept clears — all green; 2 go red on a mutant reading `state.doc`).

```findings
- file: widget/src/lib/marks.ts
  line: 28
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "DERIVED_PROPS drops a nav section's own authored pages list, so a proposal that moves a page between menu sections or reorders one marks no node at all"
- file: widget/src/store.ts
  line: 130
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the existing suite stayed 124/124 green with the store diffing against state.doc instead of state.proposalBase; the new store cases now catch it"
```

Coordinator correction 62e5f61: the round-2 ruling is narrowed. A nav section's `pages` is compared less the pages the proposal removes (document side) and adds (proposal side); DERIVED_PROPS keeps `uses` only. The unit's two cases that asserted the old ruling now expect the section marks. Verified: `pnpm check` 136/136, `pnpm build` ok.
