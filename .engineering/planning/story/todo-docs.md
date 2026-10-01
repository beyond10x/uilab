---
format: aep.planning-md/3
id: story:todo-docs
kind: story
status: active
title: Docs page and README for the todo-app example
relations:
- decomposes: epic:todo-app-example
- serves: vision:website-harness
- depends_on: story:todo-synthesized-apps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T19:54:05Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T19:54:05Z", actor: "human:timo", revision: 3}
---
## Outcome

The docs site gets a page "From a model to two apps" with the screencast embedded, and `examples/todo-app/README.md` says how to run each part.

## Acceptance

- The page is deployed and plays the screencast; links to the model, document and generated apps resolve.
- `task check` covers `examples/todo-app` (model validate, drift, `ess ui check --model`, Go tests, React build and its Playwright run, TUI tests).
