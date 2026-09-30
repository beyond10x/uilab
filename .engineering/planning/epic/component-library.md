---
format: aep.planning-md/3
id: epic:component-library
kind: epic
status: draft
title: 'Components workspace: a voice-built component library'
relations:
- serves: vision:website-harness
revision: 2
---
## Outcome

A Components workspace: the app's own named, reusable widgets, each composed from built-in composite kinds, primitives and other widgets, with typed parameters. Operators browse, search and preview them, create and change them by voice, and use them in pages like built-in kinds.

## Shape (from ess, 2026-09-30)

ui-spec/1 app-defined composites are merged on ess branch `integrate/ess-ui-1`, `schemas/ui/ess-ui.schema.yaml` (not on ess main yet):

- `Widget` (:850-875) under the document key `widgets:` (name -> Widget): `summary` (required), `doc`, `params` (typed, `required`, `default`), `arrange: row|column|grid`, `body` (list of named nodes that may read `args.<param>`).
- `WidgetInstance` (:877-891): `{component: <widget>, args: {...}}`, never a bare name; expanded at the use site, then checked like a built-in; findings at `<instance>/body/<node>`.
- `Node` (:893-901): exactly one of `component` (composite or widget) and `primitive` (text, image, link, button, ...). This also settles R2.
- Refused: a widget named like a built-in kind, unknown or missing required args, recursion.
- Loader: crate `crates/ui/ess-ui` on that branch.

## Plan

1. uilab-doc: `widgets:` and primitives in the subset reader; `component` becomes built-in kind or widget name; expansion and the four refusals as checks; node paths `widget:<name>/node:<name>`.
2. When `integrate/ess-ui-1` reaches ess main, replace the hand-written reader with `ess-ui` rather than keep two.
3. Components workspace in the browser: list, search, preview with sample args, used-by; the agent creates and edits widgets by voice (new layers in the patch schema).

## Not started

Starts after story:spec-views lands.
