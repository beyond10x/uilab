---
format: aep.planning-md/3
id: story:speech
kind: story
status: implemented
title: Speech to text on the local GPU
relations:
- decomposes: epic:voice-editing
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-stt
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T03:03:06Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T03:03:06Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T03:03:06Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`crates/uilab-stt` transcribes one 16 kHz mono utterance on the local NVIDIA GPU with whisper.cpp
(`whisper-rs`, CUDA) and model `ggml-large-v3-turbo`, biased by the vocabulary at the selected node.

## Acceptance

`uilab-stt --model ~/.cache/uilab/models/ggml-large-v3-turbo.bin <wav>` on a synthesized English and
German instruction prints the text with `audio_ms` and `took_ms`, on the GPU; audio under 0.3 s is
refused with a typed error; `cargo test -p uilab-stt` passes without a model.
