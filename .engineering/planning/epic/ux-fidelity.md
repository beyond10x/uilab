---
format: aep.planning-md/3
id: epic:ux-fidelity
kind: epic
status: implemented
title: 'Operator experience: see what a proposal does, decide without scrolling, preview the app'
relations:
- serves: vision:website-harness
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T07:35:48Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T07:35:48Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-01T07:35:48Z", actor: "human:timo", revision: 4}
---
## Outcome

The operator sees what a proposal will do before accepting it, reaches every decision without scrolling, and can switch the canvas between a structure view (paths, kinds, views) and a preview that looks like the app it describes, with sample data wherever the model has none yet.

## Found by

Review of 2026-09-30: headless screenshots of every view at 1440×900 and 1024×700 on a copy of the operator's document (probe outside the repository, `~/.cache/uilab-review/shots`), plus one live proposal. Findings with their source lines are in each story.

## Not in scope

Design tokens, themes and dark mode: epic:styles, waiting on ess R8.
