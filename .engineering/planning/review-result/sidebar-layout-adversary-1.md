---
format: aep.planning-md/3
id: review-result:sidebar-layout-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:sidebar-layout
relations:
- reviews: story:sidebar-layout
revision: 1
---
unit: story:sidebar-layout, branch unit/sidebar-layout; findings cover c3152ef (adversary test at f60fee0)
verdict: NEEDS-CHANGE
cases: executed 111→115, red 0 on the tree (2 of 4 go red on a scratch mutant)
origin: introduced 3 / pre-existing 1 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w5/sidebar-layout/adversary/
needs-coordinator: none

Case added: `widget/src/lib/sidebar.adversary.test.ts` (4 cases on the parsed template via `vue/compiler-sfc`); on a mutant with the card below the activity feed and the rename input outside the chip, the unit's own tests stay 10/10 green and these go red 2 of 4.

Attacked and held: rename field keys (Enter commits, Esc cancels, view keys ignored), blank or same name sends nothing, the ✓ badge, goal panel placement, the path tooltip, no idle. Residue: with 9 operators, a 300-character instruction and 14 findings Accept is below the fold at 1024×700; the realistic case puts it at y 563–593.

```findings
- file: widget/src/lib/sidebar.test.ts
  line: 43
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "The unit's order and chip tests match source text, so they stay green when the proposal card is moved below the activity feed or the rename input out of the chip; the parsed-template cases in sidebar.adversary.test.ts go red on both."
- file: widget/src/components/SidebarPanel.vue
  line: 157
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "When disconnected, the status label squeezes the flex:1/min-width:0 chip strip and the local operator chip is drawn under the connection dot and label at 1024x700 and 1440x900."
- file: widget/src/components/PresenceStrip.vue
  line: 11
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A browser that starts without a connection shows no local chip and so cannot rename itself, where the base always showed the name input."
- file: widget/src/App.vue
  line: 55
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "Pressing Enter with keyboard focus on the Reject button or the findings badge accepts the waiting proposal."
```

Coordinator decisions for round 2: finding 1 is closed by the adversary's template cases; findings 2, 3 and 4 are fixed (App.vue joins the surface for 4).
