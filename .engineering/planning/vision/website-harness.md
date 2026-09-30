---
format: aep.planning-md/3
id: vision:website-harness
kind: vision
status: draft
title: 'uilab: the in-browser website harness'
revision: 1
---
## uilab: the in-browser website harness

Stated by Timo on 2026-09-30. uilab is where an application's UI is built by talking, in the browser, against a `ui-spec/1` document. It has workspaces over the same document:

| workspace | what it is | status |
|---|---|---|
| App | the application: shells, menu, pages, sections, overlays, rendered live | exists (M1, M2) |
| Components | the component library: named, reusable widgets composed from the built-in kinds and from each other; browse, search, compose, all by voice | not started; needs app-defined composites in ui-spec/1 (R1 to ess) |
| Styles | design tokens and UI options: theme (light/dark), density, typography, and which options the end user gets as preferences | not started; ui-spec/1 has no style tokens (R8 to ess) |
| Docs | generated documentation over the spec: components, data, commands, state, events | first version (GET /api/docs.md) |

Across all of them: view modes (rendered, raw YAML), export, help, and an agent that can work towards a goal over many steps.
