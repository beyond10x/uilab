---
format: aep.planning-md/3
id: review-result:help-accuracy-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:help-accuracy
relations:
- reviews: story:help-accuracy
revision: 1
---
unit: story:help-accuracy, branch unit/help-accuracy, findings cover 4f2f0a0 (tree at 5edcfa7, which adds only the adversary's test)
verdict: NEEDS-CHANGE
cases: executed 103→105, red 1
origin: introduced 4 / pre-existing 1 / undecided 0
wrote-outside-worktree: 6 paths (build dir, rendered help and docs under ~/.cache/uilab-wave-w5/help-accuracy)
needs-coordinator: none

Cases (`crates/uilab-doc/tests/adversary_help_accuracy.rs`): `the_components_section_names_the_layer_it_keys_on` (red), `the_components_claims_are_in_the_components_section` (green).

Attacked and held: enum spelling (placement, severity, overlay kind via serde snake_case), overlay names keep case, goal claims (refusals at app.rs:1017, :631, :805), card findings (app.rs:1138-1155), keys 1–4, markdown renders (7 tables).

```findings
- file: crates/uilab-doc/src/docs.rs
  line: 398
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the Components section says the agent works under a selected widget without naming the `component` layer the target rule checks, while the same help calls a board's `widget` nodes widgets"
- file: crates/uilab-doc/tests/doc.rs
  line: 578
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the \"at the root\" check runs against the whole help and passes on the Widgets section's sentence, so it stays green if the Components section loses the claim"
- file: crates/uilab-doc/src/docs.rs
  line: 388
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "\"you accept or reject each proposal\" is false under `serve --auto-apply`"
- file: crates/uilab-doc/src/docs.rs
  line: 400
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "\"A goal given on the tab keeps every step to it\" overstates: run_step never applies `placed` to a step target"
- file: crates/uilab-doc/src/docs.rs
  line: 380
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "\"Each instruction is one agent turn\" is kept although a proposal runs up to DEFAULT_MAX_TURNS=4 turns over MAX_ATTEMPTS=2 attempts"
```

Coordinator correction 36c5e03 (text only): findings 1, 3, 4, 5 rewritten; finding 2 is held by the adversary's section-scoped case. Verified: `cargo test -p uilab-doc --no-fail-fast` every binary ok (adversary file 2/2, doc 43); clippy and fmt exit 0. No second adversary pass: the correction changes help prose only, and the adversary's cases hold each claim.
