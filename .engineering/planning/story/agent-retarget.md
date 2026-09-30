---
format: aep.planning-md/3
id: story:agent-retarget
kind: story
status: implemented
title: The agent moves the target when the instruction names another place
relations:
- serves: vision:website-harness
scope:
- confidence: inferred
  path: crates/uilab-agent
- confidence: inferred
  path: crates/uilab-agent/src/lib.rs
- confidence: cited
  path: crates/uilab-app/src/api.rs
- confidence: inferred
  path: crates/uilab-app/src/app.rs
- confidence: cited
  path: crates/uilab-app/src/eval.rs
- confidence: cited
  path: crates/uilab-app/src/op.rs
- confidence: cited
  path: crates/uilab-app/src/wire.rs
- confidence: inferred
  path: ess/domains/wire.yaml
- confidence: inferred
  path: evals/library.yaml
- confidence: inferred
  path: generated
- confidence: inferred
  path: widget/src/components/ActivityFeed.vue
- confidence: inferred
  path: widget/src/generated
- confidence: cited
  path: widget/src/lib/collab.ts
- confidence: inferred
  path: widget/src/store.ts
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:11:15Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T13:11:16Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T14:59:18Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome

When an instruction names a place outside the selection ("a new page", "in the menu", "on the members page", "the header"), the agent moves the target there once and proposes at the new target; the move is a visible selection by the agent with its reason in the activity feed. An instruction that only navigates ("go to the members page", "select the menu") moves the selection and proposes nothing. An instruction that names no other place is proposed at the selection, as today.

## Found by

Operator, 2026-09-30: with `page:loans/section:list` selected, "create a new page in the sidebar and do X, Y, Z" was carried out inside the loans section. Each instruction runs against one target and its patch schema (`uilab_doc::patch_schema(doc, target)`, `crates/uilab-agent/src/lib.rs`), so nothing outside the selected subtree can change.

## Design (approved by the operator, 2026-09-30)

- The agent's answer schema gains `retarget {path, reason}` beside a patch and `decline`.
- The server selects `path` as the agent (presence shows who moved it), writes the reason to the feed, and asks again at `path` with the same instruction. At most one move per instruction; a second `retarget` is refused.
- `path` must resolve, or be the parent of what the instruction creates (`/` for a page, `nav` for a menu entry).
- On the Components tab the move stays inside `component:` paths unless the instruction names a page.
- A navigation-only instruction answers `retarget` with `navigate_only: true`: the selection moves and no proposal follows.
- Goals are unchanged (the planner already targets each step).

## Acceptance

- Agent unit tests: the answer schema admits `retarget`; the prompt states when a move is allowed.
- Server rig tests (no model call): a `retarget` answer selects the path as the agent, sends the reason, re-asks once at the new path; a second `retarget` is refused; `navigate_only` moves and proposes nothing; a path that does not resolve is refused.
- Eval cases in `evals/library.yaml`: the operator's sentence at `page:loans/section:list` (expect a page Insert at `/`); "go to the members page" (expect a move, no proposal); an instruction naming nothing (expect no move).
- ESS first for any wire change (`ess/domains/wire.yaml`, `task generate`, `task drift`).
