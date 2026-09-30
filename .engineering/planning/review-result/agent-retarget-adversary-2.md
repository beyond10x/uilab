---
format: aep.planning-md/3
id: review-result:agent-retarget-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:agent-retarget
relations:
- reviews: story:agent-retarget
revision: 1
---
unit: story:agent-retarget, branch unit/agent-retarget; findings cover 69bce60 (cases at d8363cd)
verdict: NEEDS-CHANGE
cases: executed 146→152, red 4
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 logs under ~/.cache/uilab-wave-w7/agent-retarget/adversary2/
needs-coordinator: no

Cases: `crates/uilab-agent/tests/adversary_retarget_r2.rs` (a second mention of the widget's entity, a field the widget shows, "page" as a measure — red; a page said beside a widget — green) and in `crates/uilab-app/src/app.rs` tests (presence names no selector it no longer lists after the agent expires — red; a later move keeps the agent present — green). Widgets from the operator's document: member_card, loan_card, stat_tile, page_intro.

Held: page names inside widget names, plurals, multi-word titles, repeated moves, a browser named "agent", the `api` → `expires` rename, journal shapes, no journal readers, no model calls.

```findings
- file: crates/uilab-agent/src/lib.rs
  line: 210
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "On the Components tab check_retarget admits component:loan_card -> page:loans for \"make the loan card bigger and show the loan's due date\", loan_card -> page:members for \"make the loan card show the member's name\" and stat_tile -> page:overview for \"make the stat tile as wide as the page\", because any word outside a widget name that matches a page name, title or \"page\" counts as naming a page."
- file: crates/uilab-app/src/app.rs
  line: 448
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "After Tick prunes the expired session agent, presence still carries selected_by \"agent\" while no longer listing it, so the widget shows the agent's move as made by a human named \"agent\"."
```

Coordinator correction: a page is named only as a place (followed by "page", after "page", or after in/on/to/into/onto/at with or without "the"), outside a named widget's words; Tick clears `selected_by` for an operator it removes. Verified: `cargo test -p uilab-agent -p uilab-app --no-fail-fast` every lane green (152), clippy -D warnings clean, fmt applied.
