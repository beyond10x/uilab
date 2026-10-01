---
format: aep.planning-md/3
id: story:todo-model
kind: story
status: active
title: The todo model in ESS, with the blocked-by refusal
relations:
- decomposes: epic:todo-app-example
- serves: vision:website-harness
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T19:54:03Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T19:54:04Z", actor: "human:timo", revision: 3}
---
## Outcome

The ESS model of the todo app in `examples/todo-app/model/` (system `todo`, domain `todo.tasks`): Task with title, an optional `blocked_by` task and the states Open and Done; commands AddTask, RenameTask, BlockTask (refused for a blocker that does not exist), UnblockTask, CompleteTask (refused while the blocker is not Done), ReopenTask, DeleteTask; views Tasks, OpenTasks, DoneTasks; a component reached by network.

## Acceptance

- `ess specify validate --path examples/todo-app/model` prints `valid` with the ESS release the example pins.
- The blocked-by refusal is in the model, in the form the fit review of beyond10x/ess#304 settles.
- `ess verify conform synthesize` writes a suite with 0 refusals; it passes against the generated Go server, in process and over HTTP.
- `task check` validates the model.

## Draft

Written against ess 0.49.0: valid, 14 scenarios, 0 refusals, without the blocked guard. Two ESS idioms used: `{cleared: true}` clears `blocked_by`; a self-block check would need both identities in one struct, so it is left out (a task blocked by itself stays blocked until unblocked).
