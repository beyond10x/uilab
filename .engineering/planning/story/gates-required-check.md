---
format: aep.planning-md/3
id: story:gates-required-check
kind: story
status: draft
title: Require the shared Gates check on uilab main
relations:
- serves: vision:website-harness
revision: 1
---
## Outcome

uilab's main carries the required-check ruleset with common / Security and privacy, as eventlog's "Required shared and repository gates".

## Blocked

The shared check fails on GitHub until the organization secret B10X_GATES_POLICY is refreshed from gates-policy/policy.json (last set 2026-09-27); the bot cannot write secrets, so the operator runs: jq -c . ~/beyond10x/gates-policy/policy.json | gh secret set B10X_GATES_POLICY --org beyond10x --visibility all

## Acceptance

- A green shared check on main; the ruleset applied through b10x-gates api and verified.
