---
format: aep.planning-md/3
id: story:ui-polish-leftovers
kind: story
status: draft
title: Leftovers from waves 5-8
relations:
- serves: vision:website-harness
revision: 1
---
## Outcome

- The row substitution in widget/src/lib/instance.ts (substituteRow) and components.ts (substitute) is one shared function.
- The divider primitive renders as a rule, not a placeholder (PrimitiveView.vue).
- The docs site's older screenshots are retaken with the Preview button (reviewer C, round 3, N2).
- A sample-rows view without the draft. prefix and without a fixture is tagged as sample data (needs the rows message to mark samples: ESS wire change).

## Found by

Unit reports of waves 5-8 (components-gallery, canvas-widget-instances, draft-sample-rows) and docs review C round 3.

## Acceptance

- lib tests for the shared substitution; a canvas test for the divider; screenshots recorded; wire change through ESS first.
