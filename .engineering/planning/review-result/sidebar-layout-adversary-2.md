---
format: aep.planning-md/3
id: review-result:sidebar-layout-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:sidebar-layout
relations:
- reviews: story:sidebar-layout
revision: 1
---
unit: story:sidebar-layout, branch unit/sidebar-layout; findings cover 177ff75 (adversary cases at 76f70c4)
verdict: NEEDS-CHANGE
cases: executed 118→121, red 3
origin: introduced 3 / pre-existing 1 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w5/sidebar-layout/adversary2/
needs-coordinator: none

Cases (`widget/src/lib/sidebar.adversary2.test.ts`, read the parsed templates): every tab-order element with its own Enter keeps it (red: ActivityFeed `<summary>`); an inert canvas control (`tabindex="-1"`) leaves Enter to the proposal (red: CompositeView `<button>`, PrimitiveView `<a>`, `<button>`); the local chip keeps its `:key` when presence lists it (red: `'h-2' !== 'self'`).

Attacked and held: children of buttons, tree rows, roles and shadow DOM (none present), contenteditable ancestors, rename while connecting, goal plus proposal layout (Accept at y 526–556 and 567–597), pass-1 template cases still bite on a mutant.

```findings
- file: widget/src/lib/keys.ts
  line: 8
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "With keyboard focus on the activity feed's <summary>, Enter accepts the waiting proposal and the feed does not toggle, because SUMMARY is missing from OWN_ENTER_TAGS."
- file: widget/src/lib/keys.ts
  line: 17
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "Clicking an inert canvas preview control (tabindex=-1, e.g. the Loans row action) to select it leaves focus on it, so Enter re-selects instead of accepting the proposal."
- file: widget/src/components/PresenceStrip.vue
  line: 43
  category: concurrency
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The chip key changes from 'self' to the operator id when presence lists the browser, which remounts an open rename: the half-typed name is committed and later keystrokes fire global shortcuts."
- file: widget/src/lib/keys.ts
  line: 17
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "After a mouse click on a canvas nav link or the mode toggle, Enter no longer accepts, and :focus-visible cannot tell this apart from keyboard focus because Chrome reports it true at keydown."
```

Coordinator correction: SUMMARY is a control; a control out of the tab order or focused by a pointer press (within 500 ms, tracked in App.vue) leaves Enter to the proposal; fields always keep Enter; the local chip is keyed `'self'`. Verified: `pnpm check` 124/124 (the 3 red cases green, 3 new key cases red first), `pnpm build` ok.
