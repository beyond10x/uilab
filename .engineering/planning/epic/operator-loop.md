---
format: aep.planning-md/3
id: epic:operator-loop
kind: epic
status: implemented
title: Claude as a visible operator, incremental events, a self-improvement loop
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T07:35:47Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T07:35:47Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-01T07:35:47Z", actor: "human:timo", revision: 4}
---
## Outcome

An agent operator (Claude) joins a running uilab session beside the human operator, visibly: presence chips, per-operator colours, an activity feed. It acts through a request/response operator API (`uilab op`), every accepted change is broadcast as an incremental `changed` event with a revision, and an eval suite run by the agent drives rounds of improvement.

## Budget

At most 3 eval rounds x 20 cases = 60 agent calls unless the operator raises it.
