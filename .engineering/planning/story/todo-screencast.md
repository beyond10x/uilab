---
format: aep.planning-md/3
id: story:todo-screencast
kind: story
status: active
title: The todo UI built in uilab on a screencast
relations:
- decomposes: epic:todo-app-example
- serves: vision:website-harness
- depends_on: story:todo-model
- depends_on: story:uilab-model-aware
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T19:54:04Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T19:54:04Z", actor: "human:timo", revision: 3}
---
## Outcome

Claude, as the user, builds the todo UI in uilab, starting from an empty `ess-ui/1` document bound to the todo model, by typing instructions into the browser UI and accepting proposals; the browser is recorded. The result is `examples/todo-app/ui/todo.ui.yaml`.

## Acceptance

- `screencast/todo.webm` (1440x900) shows every instruction, its proposal and its acceptance, with a short caption per step; `screencast/script.md` lists the instructions; at most 30 model calls.
- The document holds: a shell with navigation; an Open page listing open tasks with blocked-by shown and row actions complete, block, unblock, rename, delete; an add form; a Done page with reopen; a Blocked view of what is waiting.
- `ess ui check --path ui/todo.ui.yaml --model model` prints 0 errors.
