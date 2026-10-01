---
format: aep.planning-md/3
id: story:todo-synthesized-apps
kind: story
status: active
title: Go server, plain React app and Rust TUI generated and running together
relations:
- decomposes: epic:todo-app-example
- serves: vision:website-harness
- depends_on: story:todo-screencast
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T19:54:05Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T19:54:05Z", actor: "human:timo", revision: 3}
---
## Outcome

From the model and the document, generated and committed: `server/` (Go), `web/` (plain React), `tui/` (Rust). All three bound by ESS's binding contract; nothing hand-written.

## Acceptance

- Drift: regenerating each of the three trees gives the committed bytes.
- A Playwright run against `web/` and the running Go server: add two tasks, block one on the other, completing it is refused with the server's reason on screen, complete the blocker, complete the task.
- The same flow in the TUI against the server (`ess ui test` or the generated app's own test).
- `web/package.json` has `react` and `react-dom` as its only runtime dependencies.
