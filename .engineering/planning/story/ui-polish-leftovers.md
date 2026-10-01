---
format: aep.planning-md/3
id: story:ui-polish-leftovers
kind: story
status: draft
title: Leftovers from waves 5-8
relations:
- serves: vision:website-harness
- depends_on: story:essui-app-widget
revision: 3
---
## Outcome

- The row substitution in widget/src/lib/instance.ts (substituteRow) and components.ts (substitute) is one shared function.
- A sample-rows read that is neither a placeholder nor backed by a fixture is tagged as sample data (needs the rows message to mark samples: ESS wire change).

The docs screenshots moved to story:essui-docs (2026-10-01).

## Found by

Unit reports of waves 5-8 (components-gallery, canvas-widget-instances, draft-sample-rows).

## Acceptance

- lib tests for the shared substitution; wire change through ESS first.

Depends on story:essui-app-widget: all three land in files that story rewrites.

The divider moved to story:essui-app-widget (2026-10-01, demo gap).
