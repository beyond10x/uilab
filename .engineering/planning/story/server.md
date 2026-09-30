---
format: aep.planning-md/3
id: story:server
kind: story
status: draft
title: 'uilab serve: browser app, WebSocket, session'
relations:
- decomposes: epic:voice-editing
- depends_on: story:session-behaviour
- depends_on: story:speech
- depends_on: story:agent-proposer
- depends_on: story:browser-app
scope:
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/uilab-app
revision: 2
---
## Outcome

`crates/uilab-app` (binary `uilab`, clap) serves the browser app and one WebSocket, holds the session
through the generated `uilab-session` port, and runs speech and the agent off the async runtime.

## Acceptance

`cargo test -p uilab-app` passes, including every server and client message decoding as the
generated `uilab.wire` unions. `uilab serve --doc examples/library/library.ui.yaml` serves
`http://127.0.0.1:8740`; in the browser the operator selects `page:loans`, holds Space, says "add a
table of overdue loans with title, member and due date", sees the proposal, presses Enter, and the
file on disk gains the section.
