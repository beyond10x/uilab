---
format: aep.planning-md/3
id: review-result:agent-retarget-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:agent-retarget
relations:
- reviews: story:agent-retarget
revision: 1
---
unit: story:agent-retarget, branch unit/agent-retarget; findings cover f84ec8b (cases at c8ea605)
verdict: NEEDS-CHANGE
cases: executed 137→141, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 logs under ~/.cache/uilab-wave-w7/agent-retarget/adversary/
needs-coordinator: no

Cases: `crates/uilab-agent/tests/adversary_retarget.rs` (a Components move to a page refused when the instruction names only a widget — red; a patch outside the target is not admitted, before and after a move — green) and one case inside `crates/uilab-app/src/eval.rs` tests (an unexpected navigation is reported as the move — red).

Held: patches outside the target never admitted; `nav` and `/` as destinations; a second move impossible (the re-ask schema has no `retarget`); goal steps never move; the mic path moves like a typed instruction; no interleaving with other operators; eval "stays" never passes on a refusal; API settling; the `op` output; no model calls made.

```findings
- file: crates/uilab-agent/src/lib.rs
  line: 209
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "On the Components tab, check_retarget admits a move from component:loan_card to page:loans for \"make the loan card bigger\", because names_a_page reads \"loan\" as the page \"loans\"; the design keeps such moves among the widgets."
- file: crates/uilab-app/src/eval.rs
  line: 115
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "judge reports an unexpected navigation-only move as \"no answer before the wait ran out\" although the act settled on that move."
- file: crates/uilab-app/src/app.rs
  line: 675
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The implicit \"agent\" operator is inserted with api false, so Tick never prunes it and it stays in presence for the rest of the session after the first move for a human."
- file: crates/uilab-app/src/app.rs
  line: 576
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The journal records a \"retarget\" entry before the server checks the move, so a refused move reads as a retarget followed by a refusal."
```

Coordinator decisions for round 2: fix all four. (1) a page counts as named only by a word that is not part of a widget name the instruction also names; (2) judge reports a navigation-only move as the move; (3) the implicit agent operator leaves presence when its move's instruction settles (or expires like an API operator); (4) the journal writes the retarget with its outcome after the check.
