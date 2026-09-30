# AGENTS.md — uilab

uilab edits a `ui-spec/1` UI document by voice in the browser. What it is and how to run it is in
[README.md](README.md); this file is what an agent changing it must know.

## Serves

- **O1 — governed reach.** The agent changes the document only through a proposal the person
  decides: one patch at the selected node, shown as a diff and a preview, written back only when
  accepted.
- **O5 — the generic agent platform.** A person shapes an application's UI by talking to it and
  sees what the agent proposes before anything lands.

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

## Common Gates

Public Gates owns the common security and privacy checks and bot delivery (Atlas ADR 0048).
`.github/workflows/shared-gates.yml` runs the shared check; require it before integration alongside
`task check`. Install coordinated hooks with `b10x-gates --repository beyond10x/uilab install`. They
scan the index, messages, filenames, metadata, annotated tags and every outgoing commit since the
adoption baseline. Private policy and signing keys stay outside this repository. From the baseline
on, an absolute home path is refused in any file, planning records and evidence included: write
`$HOME` or `~`.

Direct commits, tags and pushes use `b10x-gates bot` as `b10x-bot[bot]`; `check`, `verify` and
`publish` carry signed common evidence. A receipt reuses only the common checks and does not replace
`task check`. Commit and publish paths need no Atlas checkout.
