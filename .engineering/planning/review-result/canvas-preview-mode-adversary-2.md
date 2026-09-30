---
format: aep.planning-md/3
id: review-result:canvas-preview-mode-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:canvas-preview-mode
relations:
- reviews: story:canvas-preview-mode
revision: 1
---
unit: story:canvas-preview-mode, branch unit/canvas-preview-mode; findings cover 9a2f297 (cases at 56047c3)
verdict: INFEASIBLE
cases: executed 248→251, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w7/canvas-preview-mode/adversary2/
needs-coordinator: none

Cases (appended to `widget/src/lib/canvasmode.adversary.test.ts`): a draft account's accessible name says "sample data" (red, browser ARIA `button "name 1"`); 4 kinds × 6 row states never name the view in preview (green); the bell is announced by its tooltip title (red).

Held: structure mode against main on Overview, Loans, Showcase, Members (one leading space in an overlay button text, no visible change); `p` across tab changes; the mode button across tabs; draft tag plus selection outline; per-row menu entries in preview; the full accessible-name list in preview.

```findings
- file: widget/src/components/CanvasView.vue
  line: 84
  category: acceptance
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "the draft account's aria-label is the name alone and overrides the visible sample-data tag, so a screen reader hears the made-up name with no marker"
- file: widget/src/components/CanvasView.vue
  line: 97
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "the bell's tooltip reads the region title but its aria-label is always \"notifications\"; unreachable today because the outline gives regions no title"
```

Coordinator correction: the draft account is labelled "<name>, sample data"; the bell's aria-label follows its title; HelpModal's typing line corrected (review A of the docs site found it wrong: App.vue returns early for Esc too). Verified: `pnpm check` 251 (250 pass, 1 todo, 0 fail), `pnpm build` ok.
