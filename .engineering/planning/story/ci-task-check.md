---
format: aep.planning-md/3
id: story:ci-task-check
kind: story
status: implemented
title: task check runs on GitHub and is a required check
relations:
- serves: vision:website-harness
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T14:29:52Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T14:29:52Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-01T18:40:34Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

`task check` runs on GitHub for every pull request to `main`, and the ruleset "Required shared and
repository gates" requires it next to `common / Security and privacy`. Today a pull request needs
only the shared security check (AGENTS.md "Delivery to main"); uilab's own gate runs only on the
machine that opened it.

## Acceptance

- A workflow runs `task check` on `pull_request` with pinned action SHAs, the ess toolchain the
  repository pins, and a CPU-only whisper build (`--no-default-features`; runners have no Vulkan).
- The ruleset lists the workflow's check as required; a pull request with a failing unit test
  cannot merge (one red run recorded as evidence).
- AGENTS.md "Delivery to main" names the new required check.

## Scope

- .github/workflows/check.yml (new), Taskfile.yml (inferred: a CI variant without Vulkan), AGENTS.md
