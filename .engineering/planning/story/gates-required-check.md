---
format: aep.planning-md/3
id: story:gates-required-check
kind: story
status: implemented
title: Require the shared Gates check on uilab main
relations:
- serves: vision:website-harness
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T08:08:17Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T08:08:17Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-01T08:08:17Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

uilab's main carries the required-check ruleset with common / Security and privacy, as eventlog's "Required shared and repository gates".

## Blocked

The shared check fails on GitHub until the organization secret B10X_GATES_POLICY is refreshed from gates-policy/policy.json (last set 2026-09-27); the bot cannot write secrets, so the operator runs: jq -c . ~/beyond10x/gates-policy/policy.json | gh secret set B10X_GATES_POLICY --org beyond10x --visibility all

## Acceptance

- A green shared check on main; the ruleset applied through b10x-gates api and verified.
