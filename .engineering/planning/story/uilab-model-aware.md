---
format: aep.planning-md/3
id: story:uilab-model-aware
kind: story
status: active
title: uilab knows the ESS model a document is bound to
relations:
- decomposes: epic:todo-app-example
- serves: vision:website-harness
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T19:54:04Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-01T19:54:04Z", actor: "human:timo", revision: 4}
---
## Outcome

uilab knows the ESS model a document is bound to: `uilab serve --doc <file> --model <spec>` loads the specification, the agent is given the views (with their fields and parameters), commands (with their inputs and errors) and actors it may bind, and admission runs `ess-ui-check` with the model, so a proposal naming a view, command or field the model does not have is refused before the operator sees it. The canvas shows bound views and commands by name; sample rows come from fixtures or are generated from the view's fields.

## Why

The todo-app screencast shows a UI being bound to its API by meaning; today uilab's agent only sees the views a document already reads and its fixtures (`crates/uilab-agent/src/lib.rs`), and admission checks without `--model`.

## Acceptance

- `serve_loads_the_model` (uilab-app): `--model examples/todo-app/model` loads; a missing or invalid model refuses to start with ESS's message.
- `the_agent_sees_the_models_views_and_commands` (uilab-agent): the context sent for a page lists `todo.tasks.OpenTasks` with its fields and `todo.tasks.CompleteTask` with its inputs.
- `admission_refuses_an_unknown_view_or_command_with_the_model` (uilab-doc): a patch reading `todo.tasks.Nope` is refused naming ESS's check; `ess ui check --model` agrees.
- `sample_rows_follow_the_view_fields` (uilab-doc): a view without a fixture gets generated rows with the view's field names and types.
- `task check` exits 0.

## Scope

- `Cargo.toml`, `Cargo.lock`: ess git dependencies from 0.48.0 to 0.50.0 (the todo model is `ess/20`, which 0.48 cannot read); `ess/ess-inputs.yaml` pin to 0.50.0; regenerate `generated/`, `task drift` clean.
- `crates/uilab-app/src/{main.rs,app.rs}` (`--model`, model in the session), `crates/uilab-doc/src/{ess.rs,fixtures.rs}` (admission with the model, rows from view fields), `crates/uilab-agent/src/lib.rs` (model context in the prompt).
