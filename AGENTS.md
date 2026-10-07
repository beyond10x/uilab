# AGENTS.md — uilab

uilab edits an `ess-ui/1` UI document by voice in the browser. What it is and how to run it is in
[README.md](README.md); this file is what an agent changing it must know. The format's reference
is https://beyond10x.github.io/ess/docs/reference/ess-ui.

## Serves

- **O1 — governed reach.** The agent changes the document only through a proposal the person
  decides: one patch at the selected node, shown as a diff and a preview, written back only when
  accepted.
- **O5 — the generic agent platform.** A person shapes an application's UI by talking to it and
  sees what the agent proposes before anything lands.

## Rules

- Anything that runs is Rust, with clap derive for command lines. The browser app in `widget/` is
  TypeScript and is served as built files by the Rust binary.
- The UI document format is ESS's, never uilab's. uilab reads, edits and writes only the format ESS
  publishes (`ess-ui/1` from ess 0.47.0), and ESS's own loader and checker decide what a document
  is. uilab may keep an editing layer on top (node paths, patches, outline, sample rows), but
  everything it writes is an ESS document. A construct uilab needs and the format lacks is filed
  in `../ess` and waits for an ESS release (operator, 2026-10-01; epic:ess-ui-adoption).
- ESS is the contract. `ess/` holds the session domain (`uilab.session`) and the browser↔server
  messages (`uilab.wire`). Change the specification first, then `task generate`; never edit
  `generated/` or `widget/src/generated/` by hand. `task drift` fails when they differ from what
  the specification determines.
- `crates/uilab-doc` is uilab's editing layer over `ess-ui/1`: node paths, patches, the outline,
  sample rows and the agent's patch schema. ESS's `ess-ui` crate loads and expands the document
  and `ess-ui-check` checks it; both are git dependencies of `crates/uilab-doc` only, on ess tag
  `0.48.0`, re-exported from `uilab_doc` so no other crate adds them. A patch is refused when its
  result has an ESS error the document did not already have. uilab writes the authored document
  back, never what a page kind or widget contributes.
- Work is planned in the AEP store under `.engineering/`, written only through `aep plan artifact`.
  Body drafts go in `.engineering/drafts/` (ignored).
- No company or customer names in this repository. Examples use the lending-library app in
  `examples/library/`.
- Models (whisper ggml files) live in `~/.cache/uilab/models/`, never in the tree.
- Build into the tree's own `target/` and never set `CARGO_TARGET_DIR` (operator, 2026-10-06).
  Never delete a `target/` by hand. The whisper.cpp build is large; check `df -h /` first.
- Every worktree's work lands on `main` or on an integration branch (`wave/<date>-<n>`) that is
  merged into `main`; no branch is left unmerged at the end of a wave. When a wave or unit closes,
  end each of its worktrees with `worktree finish --discard-cache --archive <tree>`, which deletes
  the tree's recognised build cache and archives the rest, then remove it with
  `worktree gc --apply --id <id>` on the ids `worktree gc --dry-run` lists. Delete their branches
  and their scratch under `~/.cache/`. Check `worktree list` and `df -h /` at each wave boundary
  (operator, 2026-09-30).

## Layout

| path | what |
|---|---|
| `ess/` | the specification |
| `generated/rust/uilab/` | synthesized component: types, port, obligations (`PLAN.md`) |
| `generated/rust/uilab-wire/`, `widget/src/generated/` | wire types for Rust and TypeScript |
| `generated/suite.json` | the synthesized conformance suite |
| `crates/uilab-doc` | `ess-ui/1` editing layer: node paths, patches, outline, patch schema; ESS loads and checks |
| `crates/uilab-behaviour` | the session obligations, held to `generated/suite.json` |
| `crates/uilab-stt` | speech to text on the GPU (whisper.cpp) |
| `crates/uilab-agent` | one patch per instruction through the harness agent loop |
| `crates/uilab-app` | `uilab serve`: static files, one WebSocket, the session |
| `widget/` | the browser app |

## Gate

```console
task check     # validate, drift, ui, widget build and tests, cargo test/clippy/fmt
task check CARGO_FEATURES=--no-default-features   # the same with CPU speech, as CI runs it
```

No paid model call is part of the gate. `.github/workflows/check.yml` runs `task check` with CPU
speech on every pull request and every push to `main` (GitHub runners have no Vulkan), with ess
0.48.0 from its GitHub Release, checked against the release's `SHA256SUMS`.

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

## Delivery to main

`main` takes changes only through pull requests: the ruleset "Required shared and repository gates"
requires `common / Security and privacy` and `uilab / task check` (`.github/workflows/check.yml`)
on an up-to-date branch, with no bypass (operator, 2026-10-01). Every step is the bot's; `gh` stays
read-only.

1. Commit on a branch with `b10x-gates bot --repo . -- commit -F -`, run `task check`.
2. `b10x-gates --repository beyond10x/uilab check --head <sha> --receipt <file>`, then
   `publish --head <sha> --receipt <file> --remote-ref refs/heads/<branch>`.
3. Open the pull request: `b10x-gates api --method POST --path /repos/beyond10x/uilab/pulls`
   with `{title, head, base: "main", body}`; `--output` in a `mktemp -d` under `$HOME`.
4. When both required checks are green, merge with `PUT /repos/beyond10x/uilab/pulls/<n>/merge`
   (`merge_method: merge`) through the same `b10x-gates api` route, then pull `main` and delete the
   branch.

## Releases

Cut a release whenever a wave lands on `main` (operator, 2026-10-01: "cut releases often"). A
release is an annotated tag `v<x.y.z>` made by the bot on a `main` commit whose required checks are
green. `[workspace.package] version` in `Cargo.toml` must already read `<x.y.z>`; change it through
a pull request first.

1. On an up-to-date `main`: `b10x-gates bot --repo . -- tag -a v<x.y.z> -m "uilab v<x.y.z>"`.
2. Push the tag: `b10x-gates --repository beyond10x/uilab bot -- push origin refs/tags/v<x.y.z>`.
3. `.github/workflows/release.yml` refuses a lightweight tag, a tag whose commit is not on `main`
   and a tag that differs from the `Cargo.toml` version. It runs `task check`, builds `uilab` with
   CPU speech and `widget/dist` for x86_64 Linux, smoke-runs the archive, and publishes
   `uilab-<x.y.z>-x86_64-unknown-linux-gnu.tar.gz` (binary, `widget/dist`, `examples/library`)
   and `SHA256SUMS`. Follow it with `b10x-gh-run-summary beyond10x/uilab <run-id>`.
4. Verify: `gh release view v<x.y.z> -R beyond10x/uilab` lists both assets. Download them with
   `gh release download v<x.y.z> -R beyond10x/uilab` into a `mktemp -d` under `$HOME`, run
   `sha256sum --check SHA256SUMS`, unpack, and in the unpacked directory run `./uilab serve
   --assets widget/dist --doc examples/library/library.ui.yaml --no-stt`: `GET /api/state`
   answers 200 and `GET /api/document.yaml` starts with `format: ess-ui/1`. Then send one
   instruction through the agent, with a scratch `HOME` for `uilab op` and
   `OP="./uilab op --as release --server http://127.0.0.1:8740"`: `$OP join`, then
   `$OP say --review --target page:loans "add a table of overdue loans"`; it prints a proposal
   (not a refusal or an error), and `$OP reject` leaves the document unchanged. The tests drive a scripted model, so only this step shows the model accepts what
   the server sends (v0.1.1 and v0.1.2 shipped a patch schema the Messages API refused).

A pushed tag whose release run has not finished is queued, not released.
