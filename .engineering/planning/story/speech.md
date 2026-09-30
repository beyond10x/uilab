---
format: aep.planning-md/3
id: story:speech
kind: story
status: draft
title: Speech to text on the local GPU
relations:
- decomposes: epic:voice-editing
scope:
- confidence: cited
  path: crates/uilab-stt
revision: 2
---
## Outcome

`crates/uilab-stt` transcribes one 16 kHz mono utterance on the local NVIDIA GPU with whisper.cpp
(`whisper-rs`, CUDA) and model `ggml-large-v3-turbo`, biased by the vocabulary at the selected node.

## Acceptance

`uilab-stt --model ~/.cache/uilab/models/ggml-large-v3-turbo.bin <wav>` on a synthesized English and
German instruction prints the text with `audio_ms` and `took_ms`, on the GPU; audio under 0.3 s is
refused with a typed error; `cargo test -p uilab-stt` passes without a model.
