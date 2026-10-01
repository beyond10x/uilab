---
format: aep.planning-md/3
id: story:first-release
kind: story
status: active
title: Release workflow and v0.1.0; a release per landed wave
relations:
- serves: vision:website-harness
- depends_on: story:ci-task-check
- depends_on: story:essui-document
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T14:29:52Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-01T14:29:52Z", actor: "human:timo", revision: 6}
---
## Outcome

uilab has a release workflow and cuts releases often (operator, 2026-10-01: "cut releases often"): an annotated tag `v<semver>` made by the bot on a `main` commit whose gate is green starts `.github/workflows/release.yml`, which builds the Linux binary and the widget and publishes a GitHub Release with the archive and its checksum. The first release is v0.1.0, cut as soon as wave 1 of epic:ess-ui-adoption is on GitHub `main` with the required checks green; each later wave that lands is another release.

## Found by

Close-out 2026-10-01: Atlas could not resolve uilab's objectives because it has no release unit (registration report, atlas PR #55).

## Acceptance

- `release.yml`, adapted from ESS's (`beyond10x/ess` `.github/workflows/release.yml`): runs on `v*` tags, refuses a lightweight tag and a tag whose commit is not on `main`, pins every action by SHA, builds `uilab` in release mode for x86_64 Linux with CPU speech (`--no-default-features`; runners have no Vulkan), builds `widget/dist`, and uploads `uilab-<version>-x86_64-unknown-linux-gnu.tar.gz` (binary, `widget/dist`, `examples/library`) and `SHA256SUMS` to the GitHub Release.
- v0.1.0: tag, GitHub Release and both assets verified with `gh release view v0.1.0 -R beyond10x/uilab`; the downloaded archive's `uilab serve --assets widget/dist --doc examples/library/library.ui.yaml --no-stt` answers `GET /api/state` with HTTP 200 and a document whose format is `ess-ui/1`.
- AGENTS.md documents the release steps (bot tag, publish, verify).
- Not here: the Atlas release unit (a change in the Atlas repository), drafted as its own story.

## Scope

- .github/workflows/release.yml (new), AGENTS.md (release section), Cargo.toml version fields

Depends on story:ci-task-check and on wave 1 of epic:ess-ui-adoption (story:essui-document) being on `main`.
