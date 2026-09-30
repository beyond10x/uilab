---
format: aep.planning-md/3
id: story:goal-planner
kind: story
status: draft
title: Plan a goal into ordered steps
relations:
- decomposes: epic:workbench
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-agent
revision: 2
---
## Outcome

uilab-agent plans a goal: given a document, a target and a goal ("build out the member area: list, detail and an edit drawer"), one harness run returns an ordered list of at most N steps, each an instruction with its target, which the runner then executes one proposal at a time.

## Acceptance

`cargo test -p uilab-agent` passes with scripted cases: a goal yields ordered steps with targets that resolve or are created by an earlier step; more steps than the cap are refused; a goal that is not a UI change is declined. One live check plans the member-area goal.
