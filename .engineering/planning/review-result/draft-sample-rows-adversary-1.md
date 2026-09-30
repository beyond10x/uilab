---
format: aep.planning-md/3
id: review-result:draft-sample-rows-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:draft-sample-rows
relations:
- reviews: story:draft-sample-rows
revision: 1
---
unit: story:draft-sample-rows, unit/draft-sample-rows at 2b67e5f (adversary cases at b9259ec)
verdict: NEEDS-CHANGE
cases: executed 208→212, red 2
origin: introduced 4 / pre-existing 1 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w6/draft-sample-rows/adversary/
needs-coordinator: yes (findings 3 and 4 need a server change outside the story's scope lines)

Cases (`widget/src/lib/rows.store.adversary.test.ts`, the real store through a fake transport): request per view per revision (green); preview draft view asked at the current and next revision (green); a re-request in flight at disconnect is asked again on reconnect (red, `2 !== 3`); a view no composite reads is not asked on reconnect (red, `2 !== 1`).

Live probe (port 8785, two draft metrics added to a copy of the eval document): `from: on_loan` rendered `2026-10-03`, `from: members` rendered `members 1`, both tagged SAMPLE DATA.

```findings
- file: widget/src/store.ts
  line: 396
  category: concurrency
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "A draft view whose re-request is in flight when the connection drops keeps its old-shaped rows after a reconnect at the same revision, because the reconnect only clears requests for views with no rows."
- file: widget/src/store.ts
  line: 397
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "pendingViews is never pruned, so every reconnect after a revision change asks again for every draft view ever read, including ones no composite reads."
- file: widget/src/components/CompositeView.vue
  line: 56
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A metric over a draft view shows a non-number (live \"2026-10-03\" for from on_loan, \"members 1\" for from members) where the acceptance says a metric shows a number."
- file: crates/uilab-app/src/app.rs
  line: 894
  category: acceptance
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "Sample rows for a proposal preview are built from the current document, so a draft view only the preview reads gets name/value rows and renders as blank cells, a blank metric or blank chart labels tagged sample data."
- file: widget/src/lib/rows.ts
  line: 17
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "Every view without the draft. prefix is treated as fixture-backed, so such a view without a fixture shows made-up rows without the sample-data tag and never reshapes them, contrary to the new doc comment."
```

Coordinator decisions for round 2: fix 1 and 2 in the store; fix 3 at its cause (`crates/uilab-doc/src/fixtures.rs` sample values: a metric's `from` field is a number, and `on` at the start of a name is not a date); fix 4 on the server (sample rows are built from the document with the waiting proposal applied, when one waits); 5 stays, with the doc comment corrected.
