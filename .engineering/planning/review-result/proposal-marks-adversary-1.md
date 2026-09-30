---
format: aep.planning-md/3
id: review-result:proposal-marks-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:proposal-marks
relations:
- reviews: story:proposal-marks
revision: 1
---
unit: story:proposal-marks, branch unit/proposal-marks, findings cover 358d368 (adversary cases at 9d384c5)
verdict: NEEDS-CHANGE
cases: executed 111→119, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 7 paths under ~/.cache/uilab-wave-w5/proposal-marks/adversary/
needs-coordinator: yes. Do the derived props `uses` and nav `pages` count as "changed"?

Cases (`widget/src/lib/marks.adversary.test.ts`, fixture shapes from a real `/api/state`): 1 insert of a widget instance marks only the instance (red); 2 a page removal still lists the page in the canvas menu (red); 3 a page removal marks only the page (red); 4–8 rename, body node removal, array vs key order, reorder plus removal, 1000-leaf performance (green).

```
not ok 1 - inserting a widget instance marks only the new instance, not the widget it instantiates
    +   'component:member_card': 'changed',
not ok 2 - a page removal still lists the removed page in the canvas menu
    -   'page:loans'
not ok 3 - a page removal marks nothing but the removed page, also when a nav section lists it
    +   'nav/nav_section:circulation': 'changed',
```

```findings
- file: widget/src/lib/marks.ts
  line: 22
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "nodeDiffers compares a widget's server-derived props.uses, so inserting, removing or renaming any widget instance marks the widget itself changed"
- file: widget/src/store.ts
  line: 126
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "shownOutline keeps the proposal's nav section, whose pages list no longer names a removed page, so a plain page Remove no longer shows that page struck through in the canvas menu"
- file: widget/src/lib/marks.ts
  line: 41
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a page Remove also marks the nav section that lists the page as changed"
- file: widget/src/store.ts
  line: 133
  category: concurrency
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "marks diff the pending proposal against the current document, so a document change that does not close the card (another client's undo) is shown as added or removed by the proposal"
```

Coordinator decisions for round 2: derived props (`uses` on component nodes, `pages` on nav sections) do not make a node changed; a removed page stays listed in the shown menu (take the document's nav section when it lists a removed page); marks diff against the document outline the proposal was shown against, kept with the proposal.
