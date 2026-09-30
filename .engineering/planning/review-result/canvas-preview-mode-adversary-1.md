---
format: aep.planning-md/3
id: review-result:canvas-preview-mode-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:canvas-preview-mode
relations:
- reviews: story:canvas-preview-mode
revision: 1
---
unit: story:canvas-preview-mode, branch unit/canvas-preview-mode; findings cover cd07ea2 (adversary cases at 2079817)
verdict: NEEDS-CHANGE
cases: executed 237→242, red 2
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w7/canvas-preview-mode/adversary/
needs-coordinator: port 8787 held by an unrelated process; 8797/8798 used

Cases (`widget/src/lib/canvasmode.adversary.test.ts`, the first to render CanvasView and CompositeView via `vue/server-renderer`): preview hides a composite's view in every text (red); the overlay outlet hidden unless marked (green); account chrome name and selection (green); a draft account menu carries a sample marker (red); `accountName` uses `full_name` and trims (green; red against 2 mutants the unit's suite misses).

Held: structure mode markup identical to main on Overview and Members; Ctrl/Alt/Meta+P ignored; `p` inert with help open; corrupt storage falls back; the bell has an aria-label; chrome selection outline; no collision with keys.ts.

```findings
- file: widget/src/components/CompositeView.vue
  line: 32
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "preview hides the view name in the label and placeholder but the empty line still prints it (\"no data yet (members.All)\", \"loading members.All…\")"
- file: widget/src/components/CanvasView.vue
  line: 35
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "an account menu reading a draft view shows its made-up sample name as the signed-in user with no sample-data marker"
- file: widget/src/lib/canvasmode.test.ts
  line: 74
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the unit's suite stays 7/7 green when full_name is dropped from accountName's field list or its trim is removed; adversary case 5 catches both"
- file: widget/src/App.vue
  line: 54
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "p toggles and persists the canvas mode on the YAML, Docs and Components tabs where the mode button is hidden, so the canvas silently comes back in the other mode"
- file: widget/src/App.vue
  line: 132
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the mode button changes its label and aria-pressed together, announcing \"Structure p, not pressed\", and the account chrome's accessible name includes the avatar initial and the caret"
```

Coordinator decisions for round 2: fix 1, 2, 4 and 5; 3 is closed by the adversary's case.
