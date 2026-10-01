---
format: aep.planning-md/3
id: story:eval-round-5
kind: story
status: draft
title: Eval round 5 against the model, then fix what fails
relations:
- serves: vision:website-harness
revision: 1
---
## Outcome

`uilab op eval --suite evals/library.yaml` runs all cases (27, including the three agent-retarget cases) against the model on a scratch copy of examples/library; failures are fixed in the prompt, schema or checks and the round is re-run; the reports are committed under evals/reports/.

## Found by

Close-out 2026-10-01: the last live round is round 4 (22/24, both failures eval bugs); waves 3-8 changed the agent (Components workspace, retarget, widgets). About 30 model calls per round.

## Acceptance

- evals/reports/library-round-5.{md,json} committed; every failure has a fix or a recorded reason.
