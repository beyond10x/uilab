---
format: aep.planning-md/3
id: story:eval-round-5
kind: story
status: active
title: Eval round 5 on ess-ui/1 through the LLM agent; demo server switches after merge
relations:
- serves: vision:website-harness
- decomposes: epic:ess-ui-adoption
- depends_on: story:essui-agent-schema
- depends_on: story:essui-app-widget
scope:
- confidence: inferred
  path: crates/uilab-agent/src/lib.rs
- confidence: cited
  path: evals/reports
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T11:11:13Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-10-01T11:11:13Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":5}}}
---
## Outcome

`uilab op eval --suite evals/library.yaml` runs all 27 cases through the LLM agent
(`claude-sonnet-5-5`, as in round 4) on a scratch copy of the `ess-ui/1` library example, as the
acceptance run of epic:ess-ui-adoption. Every document a case proposes is checked by ESS without
`--model`; failures are fixed in the prompt or schema, or recorded with a reason, and the round is
re-run at most once. Afterwards the demo server runs the merged binary (epic decision D7).

## Found by

Close-out 2026-10-01: the last live round is round 4 (22/24, both failures eval bugs,
`evals/reports/library-round-4.md`). The adoption changes the document format, the agent's schema
and its prompt.

## Acceptance

- `evals/reports/library-round-5.{md,json}` committed.
- Every admitted proposal in the round passes `ess-ui-check` with 0 errors; the report states the
  count of admitted proposals and of ESS errors (expected 0).
- At least 25 of 27 cases pass; every failure has a fix or a recorded reason.
- At most 2 runs of the suite (54 agent calls), counted from the session journal.
- D7, before: while the epic's pull requests are open, the process on port 8740 is PID 488702
  (`ls -l /proc/488702/exe` names `~/.cache/b10x-target/uilab/release/uilab`); the coordinator
  records that line at each wave close.
- D7, after (a step after the epic's final pull request merges, story:essui-docs included; the
  record goes into the epic's closing store commit): the operator's working copy
  `~/.cache/uilab/eval/library.ui.yaml` is copied to `~/.cache/uilab/eval/library.ui-spec-1.yaml`
  and kept; the server on 8740 runs the merged `main` binary on a copy of the converted library
  example, and `curl -s http://127.0.0.1:8740/api/state` returns a document whose format is
  `ess-ui/1`.

## Scope

- evals/reports/** (cited)
- crates/uilab-agent/src/lib.rs prompt lines, only after story:essui-agent-schema and only for a fix a
  failure in this round names (inferred)

Depends on story:essui-agent-schema and story:essui-app-widget.
