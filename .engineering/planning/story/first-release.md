---
format: aep.planning-md/3
id: story:first-release
kind: story
status: draft
title: First uilab release
relations:
- serves: vision:website-harness
- depends_on: epic:ess-ui-adoption
- depends_on: story:ci-task-check
revision: 2
---
## Outcome

uilab has a release unit and a first tagged release (0.1.0): a GitHub Release with the Linux binary and the built widget, cut through the bot route after a green full gate. It ships after epic:ess-ui-adoption, so 0.1.0 reads and writes `ess-ui/1`.

## Found by

Close-out 2026-10-01: Atlas could not resolve uilab's objectives because it has no release unit (registration report, atlas PR #55).

## Acceptance

- task check green on the release commit; tag, GitHub Release and assets verified; Atlas release unit recorded.
- The release binary serves the widget it ships with (`uilab serve --assets` pointed at the release archive's widget directory loads the library example).

Depends on epic:ess-ui-adoption and story:ci-task-check.
