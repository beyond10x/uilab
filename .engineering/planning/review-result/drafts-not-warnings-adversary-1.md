---
format: aep.planning-md/3
id: review-result:drafts-not-warnings-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:drafts-not-warnings
relations:
- reviews: story:drafts-not-warnings
revision: 1
---
unit: story:drafts-not-warnings, branch unit/drafts-not-warnings; findings cover fa4f916 (adversary case at 20782a2)
verdict: CONFIRMED (warning); no blocker; the suite has no red case
cases: executed 206→209, red 0 (1 todo that fails as expected; 2 cases red against a mutated check.rs)
origin: introduced 2 / pre-existing 0 / undecided 1
wrote-outside-worktree: ~/.cache/uilab-wave-w6/drafts-not-warnings/adversary/
needs-coordinator: no

Cases (`widget/src/lib/sidebar.drafts.adversary.test.ts`): `draftView` against the format string read from `crates/uilab-doc/src/check.rs:466` with five view shapes (green); the operator document as the check formats it, plus a widget-body read (green); a backtick in a view name (todo). On a mutated check.rs message the unit's tests stay 20/20 green and these go red.

Held in the browser (port 8786): "4 draft views", `aria-expanded`, Enter on the toggle, selecting a widget-body node and a view's first path, `✕ 1` with one error plus drafts; `changed` carries full findings so the list updates; the proposal card still shows a draft read a proposal brings.

```findings
- file: widget/src/lib/sidebar.test.ts
  line: 13
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the draft_read message format is hand-copied into the unit's test and pinned nowhere in Rust, so a check.rs wording change leaves the suite green while every draft view falls back to its raw message"
- file: widget/src/lib/sidebar.ts
  line: 99
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "a view name containing a backtick is truncated at it (draft.a`b parses as draft.a); no document was found that uses one"
- file: widget/src/lib/sidebar.ts
  line: 106
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: undecided
  message: "\"document order\" is the check's finding order (shells, pages, then widget bodies), so a widget-body draft read lists last while the tree and YAML show components before pages"
```

Coordinator: no correction. Finding 1 is closed by the adversary's case reading the format string from check.rs; findings 2 and 3 are not reached by any document and stay as recorded.
