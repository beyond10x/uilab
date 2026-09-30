---
format: aep.planning-md/3
id: story:agent-goals
kind: story
status: draft
title: 'Long-running goals: the agent works in steps until done'
relations:
- decomposes: epic:workbench
revision: 1
---
## Outcome

An operator gives the agent a goal ("build out the member area: list, detail, edit") instead of one instruction. The agent plans steps and proposes them one at a time through the same operator API, each visible in the browser and each accepted or rejected, until the goal is met, it is stopped, or a budget runs out.

## Open design points

- The harness loop gets tools (read state, propose, wait for decision) instead of a single structured answer.
- Where the goal and its progress live: an ESS entity (Goal: Running, Done, Stopped) in uilab.session.
- Budget per goal (turns, proposals, spend).
