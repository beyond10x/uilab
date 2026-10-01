---
format: aep.planning-md/3
id: epic:styles
kind: epic
status: draft
title: 'Styles workspace: tokens, themes, user UI options'
relations:
- serves: vision:website-harness
- depends_on: epic:ess-ui-adoption
revision: 2
---
## Outcome

A Styles workspace: design tokens (colour, spacing, type, radius) and themes (light, dark), and which UI options the end user gets as preferences. The canvas renders with them; the agent changes them by voice.

## Depends on

- Design tokens, themes and preferences in `ess-ui/1`: ess `story:ui-spec-style-tokens`, draft at ess main `6c3a811` (2026-10-01). Recorded as dependency-blocker:ess-style-tokens.
- epic:ess-ui-adoption: tokens arrive in `ess-ui/1`, which uilab reads only after that epic.
