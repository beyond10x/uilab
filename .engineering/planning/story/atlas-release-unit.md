---
format: aep.planning-md/3
id: story:atlas-release-unit
kind: story
status: draft
title: Atlas records uilab's release unit
relations:
- serves: vision:website-harness
- depends_on: story:first-release
revision: 1
---
## Outcome

Atlas knows uilab's releases: a release unit for beyond10x/uilab is recorded in the Atlas repository, so Atlas can resolve uilab's objectives (O1, O5) and observe v0.1.0 and later tags.

## Found by

Registration report (atlas PR #55): Atlas could not resolve uilab's objectives without a release unit. Split out of story:first-release on 2026-10-01, because it is a change in the Atlas repository.

## Acceptance

- An Atlas pull request, opened and merged by the bot, records the release unit; Atlas's own checks pass on it.
- Atlas reports uilab v0.1.0 (or the latest tag) as observed.
