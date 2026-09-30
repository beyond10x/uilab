# AGENTS.md — uilab

uilab edits a `ui-spec/1` UI document by voice in the browser. What it is and how to run it is in
[README.md](README.md); this file is what an agent changing it must know.

## Rules

- Anything that runs is Rust, with clap derive for command lines. The browser app in `widget/` is
  TypeScript and is served as built files by the Rust binary.
- ESS is the contract. `ess/` holds the session domain (`uilab.session`) and the browser↔server
  messages (`uilab.wire`). Change the specification first, then `task generate`; never edit
  `generated/` or `widget/src/generated/` by hand. `task drift` fails when they differ from what
  the specification determines.
- `crates/uilab-doc` is a hand-written reader of the `ui-spec/1` subset, because `ess generate
  types` cannot read `ui-spec/1` as written yet (ordered maps, inline composite props). It follows
  `ui-spec/1` as the `ess` repository publishes it.
- Work is planned in the AEP store under `.engineering/`, written only through `aep plan artifact`.
  Body drafts go in `.engineering/drafts/` (ignored).
- No company or customer names in this repository. Examples use the lending-library app in
  `examples/library/`.
- Models (whisper ggml files) live in `~/.cache/uilab/models/`, never in the tree.
- Build with `CARGO_TARGET_DIR=$HOME/.cache/b10x-target/uilab` (the Taskfile sets it). The
  whisper.cpp build is large; check `df -h /` first.
- Every worktree's work lands on `main` or on an integration branch (`wave/<date>-<n>`) that is
  merged into `main`; no branch is left unmerged at the end of a wave. When a wave or unit closes,
  archive and remove its worktrees (`worktree archive`, `finish`, `gc --apply --id`), delete their
  branches, and remove their build directories (`~/.cache/b10x-target/uilab-<unit>`), their
  `node_modules`/`dist`, and their scratch under `~/.cache/`. Check `worktree list` and
  `du -sh ~/.cache/b10x-target/uilab*` at each wave boundary (operator, 2026-09-30).

## Layout

| path | what |
|---|---|
| `ess/` | the specification |
| `generated/rust/uilab/` | synthesized component: types, port, obligations (`PLAN.md`) |
| `generated/rust/uilab-wire/`, `widget/src/generated/` | wire types for Rust and TypeScript |
| `generated/suite.json` | the synthesized conformance suite |
| `crates/uilab-doc` | `ui-spec/1` subset: model, node paths, patches, checks, patch schema |
| `crates/uilab-behaviour` | the session obligations, held to `generated/suite.json` |
| `crates/uilab-stt` | speech to text on the GPU (whisper.cpp) |
| `crates/uilab-agent` | one patch per instruction through the harness agent loop |
| `crates/uilab-app` | `uilab serve`: static files, one WebSocket, the session |
| `widget/` | the browser app |

## Gate

```console
task check     # validate, drift, widget build, cargo test/clippy/fmt
```

No paid model call is part of the gate.
