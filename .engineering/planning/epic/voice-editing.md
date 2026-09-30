---
format: aep.planning-md/3
id: epic:voice-editing
kind: epic
status: draft
title: Voice-edit a ui-spec/1 document in the browser
revision: 1
---
## Outcome

An operator opens uilab in a browser over a `ui-spec/1` document, selects a node, holds a key and
speaks one instruction. Local speech-to-text on the GPU turns it into text; one agent turn through
the harness library proposes one patch at the selected node; the patch is checked, shown as a diff
and a preview, and accepted, rejected or undone. Accepted patches are written back to the file.

## Specification

- `ess/domains/session.yaml`: the session (`uilab.session`: Document, Proposal, six commands, three views).
- `ess/domains/wire.yaml`: the browser↔server messages (`uilab.wire`).
- `ess validate --path ess` → `uilab v1 — 4 file(s), valid`; `ess verify conform synthesize` → 27 scenarios, 0 refusals.

## Not in this epic

- A TUI renderer of the same document (M2).
- A Parakeet comparison, streaming transcripts, WebRTC audio (M3).
- App-defined composites and specified primitives, pending `ui-spec/1` decisions in `ess` (M4).
