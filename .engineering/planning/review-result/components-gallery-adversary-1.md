---
format: aep.planning-md/3
id: review-result:components-gallery-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:components-gallery
relations:
- reviews: story:components-gallery
revision: 1
---
unit: story:components-gallery, branch unit/components-gallery; findings cover 33ee1e9 (b0acee8 plus one adversary test file)
verdict: NEEDS-CHANGE
cases: executed 109→112, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 8 paths under ~/.cache/uilab-wave-w5/components-gallery/adversary/
needs-coordinator: yes. Finding 2 needs the server outline (crates/uilab-doc/src/outline.rs) to carry a region's view.

Cases (`widget/src/lib/components.gallery.adversary.test.ts`), all red: a use site in a shell overlay opens that overlay; a `Staff` param previews the `staff.Me` row, a view only a shell region reads; `LoanRequest` maps to `loan_requests.All`.

Attacked and held: `Staff`/`Category`/`Box` stems, wrappers, drafts left out, rows arriving after render, junk rows, object defaults, header use sites, hidden pages, stale pages, search.

```findings
- file: widget/src/lib/components.ts
  line: 214
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "useSiteTarget drops the overlay for a shell-overlay use site and openUse returns early without a page, so the link never shows it in the UI tab"
- file: widget/src/lib/components.ts
  line: 176
  category: acceptance
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "viewsRead cannot see staff.Me, a declared fixture view read only by a shell region, because region nodes carry no view in the outline"
- file: widget/src/lib/components.ts
  line: 157
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "viewStems never matches a snake_case view for a multi-word entity (LoanRequest -> loan_requests.All) or an irregular plural"
```

Coordinator decisions for round 2: all three are fixed; the surface widens to `crates/uilab-doc/src/outline.rs` so a region node carries the view it reads.
