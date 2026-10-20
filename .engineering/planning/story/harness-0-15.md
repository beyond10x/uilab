---
format: aep.planning-md/3
id: story:harness-0-15
kind: story
status: implemented
title: Move to harness 0.15.0 (Substrate 0.7.17)
summary: uilab-agent on harness 0.15.0 and llm 0.8.0 through harness's public llm adapter (Atlas ADR 0097)
relations:
- serves: vision:website-harness
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-20T10:57:49Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-20T10:57:49Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-20T14:30:33Z", actor: "human:timo", revision: 5, decided_on: {"asserted":{"test_result":1}}}
---
## Outcome

uilab-agent builds on harness 0.15.0 (Substrate 0.7.17) and reaches the model through llm 0.8.0 by
way of harness's public llm adapter (`LlmPort`, a `ModelPort` over llm), as Atlas ADR 0097 asks of
each harness consumer. The four pins at `crates/uilab-agent/Cargo.toml:18-21` (rev `798325f0`) go:
`b10x-harness-messages` and `b10x-harness-credential` no longer exist (harness 0.14.0, `8825e9c3`),
and `b10x-harness-loop` and `b10x-harness-wire` move to the 0.15.0 tag. Behaviour is unchanged: the
same Messages endpoint, the same subscription token file and pointer, the same API-key option, the
same loop, schema and retry.

## Scope

| file:line | today | after |
|---|---|---|
| `crates/uilab-agent/Cargo.toml:18-21` | four harness crates at rev `798325f0` | `harness-loop`, `harness-wire` and the adapter crate at tag 0.15.0 |
| `crates/uilab-agent/src/lib.rs:25-27` | imports `Bearer`, `BearerSource`, `WireError` | dropped |
| `crates/uilab-agent/src/lib.rs:516-526` | `harness_messages::Endpoint` and `MessagesClient` | the adapter's client on the Messages protocol, same base URL, model and context window |
| `crates/uilab-agent/src/lib.rs:899-938` | `harness_credential::SubscriptionToken` at a pointer; `EnvApiKey` | the adapter's credential source: token file at the pointer, API key from the named variable |
| `Cargo.lock` | `b10x-harness-credential`, `-http`, `-messages` | gone; llm 0.8.0 crates in |

Unchanged: `harness_loop` (`AgentLoop`, `LoopConfig`, `OutputSchema`, `RateCard`, `DenyAll`, ...) and
`harness_wire` (`ModelPort`, `ToolPort`, scripted test ports in `crates/uilab-agent/tests/`): their
sources differ between `798325f0` and 0.14.1 only in comments and tests.

Toolchain: no change. The repository has no `rust-toolchain` file and no `rust-version`; CI uses
`stable` (`.github/workflows/check.yml:40`); local rustc is 1.99.0, above the 1.98 llm requires.
Substrate: uilab depends on no `harness-substrate` crate, so no Substrate crate enters its lock.

## Needs from harness 0.15.0

Both answered by b10x-chef-orchestrator on 2026-10-20 for harness 0.15.0:

- `Send`: `Proposer` holds `Box<dyn ModelPort + Send>` (`crates/uilab-agent/src/lib.rs:506`) and is
  shared as `Arc<Mutex<Proposer>>` into `spawn_blocking` (`crates/uilab-app/src/app.rs:219`,
  `:974`). harness 0.14.1's `harness-cli/src/llm_port.rs` `client()` returns `Box<dyn ModelPort>`;
  0.15.0's returns `Box<dyn ModelPort + Send>`.
- API key from a named environment variable (`crates/uilab-agent/src/lib.rs:908`): 0.15.0 has
  `Source::resolver(name, Arc<dyn SecretResolver>, Presentation)` and re-exports `llm_credentials`.
  uilab uses `llm_credentials::environment::EnvironmentResolver` with `Presentation::ApiKey`, so the
  variable is still read on every request and an unset one still fails at the turn, as
  `EnvApiKey` does today.

## Acceptance

- `grep -rn 'harness-messages\|harness-credential\|harness_messages\|harness_credential' crates Cargo.lock` prints nothing.
- `task check` and `task check CARGO_FEATURES=--no-default-features` exit 0 with the same test count as on `8a10e9c`, no test removed.
- `uilab-propose` with the default subscription route returns an admitted patch for one instruction on `examples/library/library.ui.yaml` (one live model call).
