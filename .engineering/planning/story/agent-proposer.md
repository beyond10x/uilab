---
format: aep.planning-md/3
id: story:agent-proposer
kind: story
status: draft
title: One patch per instruction through the harness loop
relations:
- decomposes: epic:voice-editing
- depends_on: story:doc-model
scope:
- confidence: cited
  path: crates/uilab-agent
revision: 2
---
## Outcome

`crates/uilab-agent` turns one instruction at one node into one patch the document admits, through
one agent run of the harness library (`b10x-harness-loop` at harness `0.13.3`), with the patch
schema at that node as the run's structured output (`answer` tool) and one retry on a refusal.

## Acceptance

`cargo test -p uilab-agent` passes with a scripted model port and no network: a valid answer is
admitted; a refused first answer is retried with the refusal visible to the model; two refusals end
in `ProposeError::Refused`. One live run of `uilab-propose` against `examples/library` returns an
admitted patch.
