---
format: aep.planning-md/3
id: story:widget-model
kind: story
status: draft
title: Widgets and primitives in the ui-spec/1 reader
relations:
- decomposes: epic:component-library
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-doc
- confidence: cited
  path: examples
revision: 2
---
## Outcome

uilab-doc reads, checks, addresses and patches app-defined widgets and primitives as ess ui-spec/1 declares them on branch integrate/ess-ui-1 (schemas/ui/ess-ui.schema.yaml Widget :850-875, WidgetInstance :877-891, Node :893-901).

## Acceptance

`cargo test -p uilab-doc` passes with new cases: a document with `widgets:` round-trips; an instance `{component: <widget>, args}` inside a section, board widget or collection item parses and resolves; primitives (`primitive: text|image|link|button|...`) parse as nodes of a widget body; checks refuse a widget named like a built-in kind, an instance with an unknown or a missing required arg, and a recursive widget, each with its own check id and the instance path; node paths `component:<name>` and `component:<name>/node:<name>` resolve; the patch schema offers inserting a widget at the root and a node into a widget body, and `component` accepts declared widget names; the outline lists widgets; docs list widgets with params and use sites.
