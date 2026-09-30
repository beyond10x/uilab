---
format: aep.planning-md/3
id: review-result:components-gallery-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:components-gallery
relations:
- reviews: story:components-gallery
revision: 1
---
unit: story:components-gallery, branch unit/components-gallery; findings cover 1379e4a (e3d970d plus two adversary test files)
verdict: INFEASIBLE
cases: executed 217→223 (widget 115→118, uilab-doc 102→105), red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths under ~/.cache/uilab-wave-w5/components-gallery/adversary2/
needs-coordinator: none

The amended pass-1 fixture holds: the server sends the `account` region as `{…, view: "staff.Me"}` (`adversary_components_gallery_p2.rs`). Attacked and held: region reads without a view or with a draft, `outline_at` on a region, plural edge cases, a shell overlay under another shell, a removed overlay's link.

```findings
- file: widget/src/lib/components.ts
  line: 166
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "the irregular plural replaces the regular one instead of joining it, so Person no longer maps to a persons.All view that b0acee8 matched"
```

Coordinator correction f8300eb: stems are the irregular plural, the regular plural, then the name. Verified: `pnpm check` 118/118, `pnpm build` ok.
