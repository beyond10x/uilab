---
format: aep.planning-md/3
id: review-result:draft-sample-rows-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:draft-sample-rows
relations:
- reviews: story:draft-sample-rows
revision: 1
---
unit: story:draft-sample-rows, unit/draft-sample-rows at c5d71a4 (adversary cases at afc7ea9)
verdict: NEEDS-CHANGE
cases: executed 396→402, red 5
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w6/draft-sample-rows/adversary2/
needs-coordinator: no

Cases: `widget/src/lib/rows.store.adversary2.test.ts` (a preview adding a metric over an answered draft view, a proposal replaced by another, a reject after preview rows — all red `1 !== 2`); `crates/uilab-doc/tests/adversary_sample_rows_p2.rs` (`date_…` names stay dates — red; one bar per x label — red; integers for every quantity over one draft view — green).

Name-rule comparison over every field name in examples/, the eval document, its fixtures and code: 2 names changed there, both better (`date_format`, `on_loan`); among plausible names 7 regress (`date_of_birth`, `date_created`, `date_added`, `date_returned`, `sms_notify`, `overdue_notify`, `week_start`) and 12 improve.

```findings
- file: widget/src/lib/rows.ts
  line: 20
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "rowsDue keys a draft view's request by document revision alone while the server now shapes the answer by the waiting proposal, so a preview reading an already-answered draft view through another field, a proposal replaced by another, or a reject after rows were answered for a preview all keep rows shaped for a different outline."
- file: crates/uilab-doc/src/fixtures.rs
  line: 284
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Reading the kind from the last word only turns date_of_birth, date_created, date_added and date_returned, dates at the base commit, into text; sms_notify, overdue_notify and week_start lose their flag or week shape the same way."
- file: crates/uilab-doc/src/fixtures.rs
  line: 291
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "A chart with x: state gets five sample rows over three state values, so the operator's draft.LoansByState chart draws two bars for one category."
```

Coordinator correction d30aba4 (+ 4bc79d1 formatting): requests keyed by revision plus the proposal shown; `date` first word is a date, `week` first word a week unless counted, `notify` last word a flag; a chart's `x` gets one row per category. Verified: `pnpm check` 218/218; `cargo test -p uilab-doc --no-fail-fast` every lane green; clippy (CARGO_INCREMENTAL=0, disk at 340M free before cleanup) and fmt exit 0.
