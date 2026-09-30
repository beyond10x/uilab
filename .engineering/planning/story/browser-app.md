---
format: aep.planning-md/3
id: story:browser-app
kind: story
status: implemented
title: Canvas and operator sidebar in the browser
relations:
- decomposes: epic:voice-editing
- serves: vision:website-harness
scope:
- confidence: cited
  path: widget
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T03:03:07Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T03:03:07Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T03:03:07Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`widget/` is the browser app: a canvas rendering the document from its outline and fixture rows, and
a sidebar with the node tree, push-to-talk (hold Space) streaming 16 kHz audio, the transcript, the
proposal diff and preview, and accept/reject/undo. Its message types are generated from
`uilab.wire` into `widget/src/generated/`.

## Acceptance

`pnpm check` and `pnpm build` pass; unit tests cover downsampling, audio framing, the line diff and
outline lookups.
