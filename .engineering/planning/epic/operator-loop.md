---
format: aep.planning-md/3
id: epic:operator-loop
kind: epic
status: draft
title: Claude as a visible operator, incremental events, a self-improvement loop
revision: 1
---
## Outcome

An agent operator (Claude) joins a running uilab session beside the human operator, visibly: presence chips, per-operator colours, an activity feed. It acts through a request/response operator API (`uilab op`), every accepted change is broadcast as an incremental `changed` event with a revision, and an eval suite run by the agent drives rounds of improvement.

## Budget

At most 3 eval rounds x 20 cases = 60 agent calls unless the operator raises it.
