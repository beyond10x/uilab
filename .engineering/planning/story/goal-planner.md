---
format: aep.planning-md/3
id: story:goal-planner
kind: story
status: implemented
title: Plan a goal into ordered steps
relations:
- decomposes: epic:workbench
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-agent
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T03:04:47Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T03:04:47Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T03:58:06Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":4,"verification":1}}}
---
## Outcome

uilab-agent plans a goal: given a document, a target and a goal ("build out the member area: list, detail and an edit drawer"), one harness run returns an ordered list of at most N steps, each an instruction with its target, which the runner then executes one proposal at a time.

## Acceptance

`cargo test -p uilab-agent` passes with scripted cases: a goal yields ordered steps with targets that resolve or are created by an earlier step; more steps than the cap are refused; a goal that is not a UI change is declined. One live check plans the member-area goal.
