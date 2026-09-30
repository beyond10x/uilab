---
format: aep.planning-md/3
id: story:browser-app
kind: story
status: draft
title: Canvas and operator sidebar in the browser
relations:
- decomposes: epic:voice-editing
scope:
- confidence: cited
  path: widget
revision: 2
---
## Outcome

`widget/` is the browser app: a canvas rendering the document from its outline and fixture rows, and
a sidebar with the node tree, push-to-talk (hold Space) streaming 16 kHz audio, the transcript, the
proposal diff and preview, and accept/reject/undo. Its message types are generated from
`uilab.wire` into `widget/src/generated/`.

## Acceptance

`pnpm check` and `pnpm build` pass; unit tests cover downsampling, audio framing, the line diff and
outline lookups.
