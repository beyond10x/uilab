---
format: aep.planning-md/3
id: epic:todo-app-example
kind: epic
status: active
title: 'Todo-app example: one ESS model, a uilab-made UI, a Go server, plain React and a Rust TUI'
relations:
- serves: vision:website-harness
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T19:54:03Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T19:54:03Z", actor: "human:timo", revision: 3}
---
## Outcome

A showcase in `examples/todo-app/`: a todo-list model specified in ESS and synthesized as a Go server; its UI made in uilab on a screencast, with Claude as the user; and from that one `ess-ui/1` document, two fully synthesized apps bound to the server by meaning: a plain React web app (react and react-dom only) and a Rust TUI. The unique feature is **blocked by**: a task may name the task blocking it, and completing it is refused by the server until the blocker is done; both apps show the refusal.

## Rule

Nothing in the example is hand-written to cover an ESS gap. ESS gains what is missing first (beyond10x/ess #304, #311, #314, #315), under ESS's process; the example uses the released ESS (operator, 2026-10-01).

## Stories

| story | what |
|---|---|
| story:todo-model | the ESS model in `examples/todo-app/model/`, with the blocked-by guard once ESS has it; conformance against the synthesized Go server |
| story:todo-screencast | the UI built in uilab by Claude as the user, recorded; `ui/todo.ui.yaml` checked against the model |
| story:todo-synthesized-apps | Go server, plain React app and Rust TUI generated; end-to-end flows over HTTP including the blocked-by refusal |
| story:todo-docs | the docs page "From a model to two apps" with the screencast, and the example README |

## Depends on

The ESS release that carries #304 (or its fit-review answer), #311, #314 and #315.
