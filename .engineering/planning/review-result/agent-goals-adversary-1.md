---
format: aep.planning-md/3
id: review-result:agent-goals-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:agent-goals
relations:
- reviews: story:agent-goals
revision: 1
---
```
unit: story:agent-goals, branch unit/agent-goals, covers 70f8ac4 plus test commit de2ed2a (b10x-bot)
verdict: NEEDS-CHANGE
cases: executed 103→108 (uilab-app 33→34, widget 70→74), red 5
origin: introduced 7 / pre-existing 1 / undecided 0
wrote-outside-worktree: 4 paths
needs-coordinator: yes (actor tests would have to go in app.rs, which is implementation; see J1)
```

**1. `git --no-pager diff --stat 70f8ac4 HEAD`**
```
 crates/uilab-app/src/api.rs      |  21 +++++
 widget/src/lib/goalstore.test.ts | 184 +++++++++++++++++++++++++++++++
```
All 21 lines in `api.rs` go into its existing `#[cfg(test)] mod tests`. No implementation lines changed.

**2. Cases added.** All are red now. Each red output below was captured by running that case alone, before the suite ran.

| # | Case | Asserts | Red output |
|---|---|---|---|
| C1 | `goalstore.test.ts`: stopping a goal while its step thinks returns the owner browser to idle | after `thinking`(ws-1) then goal `stopped`, `phase` is `idle` and ws-1 is not in `inFlight` | `'thinking' !== 'idle'` |
| C2 | same file: stopping an agent goal while its step thinks drops the agent banner | `agentActions` is empty after goal `stopped` | `['api-1']` vs `[]` |
| C3 | same file: a goal the server does not hold after a reconnect cannot be stopped from the panel | after close, open, and a new document with no goal message, `stopMessage(state.goal)` is null | got `{type:'stop_goal',value:{goal_id:'goal-1'}}` |
| C4 | same file: starting a goal closes the card of the proposal the server rejected for it | the card for proposal p0, then goal `planning`, gives `state.proposal === null` | the card for p0 is still there |
| C5 | `api.rs`: `a_goal_does_not_settle_on_the_operators_earlier_goal` | a `goal` Settle is not settled by this operator's earlier goal, already ended | `panicked at crates/uilab-app/src/api.rs:457:9: settled on the previous goal before the new one was planned` |

In C1 to C4 every precondition assert passed; only the final assert failed. C5 ran in this tree: `--list` shows 34 cases, and the new name is among them.

**3. Suite runs, after the cases existed**
- `cargo test -p uilab-app`: `test result: FAILED. 33 passed; 1 failed`, exit 101.
- `pnpm check` in `widget/`: typecheck clean, then `# tests 74 # pass 70 # fail 4`, exit 1.
- Where the before-counts come from: widget 70 is the same suite run with `goalstore.test.ts` left out. uilab-app 33 is the total minus the one new case.
- `cargo fmt -p uilab-app --check` exits 0. I did not run clippy, to save disk (11G free).

**4. Findings**

| # | file:line | Finding | Verdict | Origin | What reaches it |
|---|---|---|---|---|---|
| F1 | widget/src/store.ts:280 | When a goal is stopped while its step thinks, the server drops the late answer and sends nothing (app.rs, `Cmd::Proposed` discard). The owner's status stays "thinking about …" and the agent banner stays up. | NEEDS-CHANGE | introduced | Stop button during a step. For an API owner the banner clears only when the operator expires. |
| F2 | widget/src/lib/goal.ts:33 | `state.goal` is never cleared on reconnect. If the server restarted it holds no goal, but the panel keeps a running goal with Stop, and Stop is refused `wrong_state`. | NEEDS-CHANGE | introduced | Restarting `uilab` with a tab open; `WsTransport` reconnects on its own. |
| F3 | crates/uilab-app/src/app.rs:635 | `start_goal` rejects the waiting proposal silently, and the widget card stays. Accept on it gives "that proposal is not waiting". If planning fails, the card never closes. | NEEDS-CHANGE | introduced | Review mode: say, then switch to goal mode and send a goal. |
| F4 | crates/uilab-app/src/api.rs:254 | `Settle::Goal` settles on any ended `goal` message from this operator. A `Cmd::Connected` re-broadcast of the earlier goal ends a new `op goal` early. | INFEASIBLE | introduced | Only if a browser connects in the microseconds between `subscribe` and the actor handling the goal; not observed. |
| J1 | crates/uilab-app/src/app.rs:537 | None of the actor's goal wiring (`goal_next`, `run_step`, `stop_goal`, the discard path, keeping the goal's operator alive on Tick) has a test. `App::start` builds its own Proposer and the crate is bin-only, so tests have to live in app.rs, even though `Cmd::Planned`/`Cmd::Proposed` injection needs no model. | CONFIRMED | introduced | Any regression in app.rs goal paths stays green. |
| J2 | crates/uilab-app/src/op.rs:272 | `op goal` without `--auto` prints steps and proposal ids only after it settles. Meanwhile `op accept` with no id uses the cached `last_proposal` from an earlier run. So a goal under review can only be decided from the browser. | CONFIRMED | introduced | `op goal` on a server in review mode (the default). |
| J3 | crates/uilab-app/src/app.rs:311 | `Cmd::Connected` never re-sends a waiting proposal. After a page reload the goal panel shows the step as proposed with no card, and only Stop moves the goal on. | CONFIRMED | pre-existing | The base commit also loses the card on reload; the goal makes it block the whole session. |
| J4 | crates/uilab-app/src/app.rs:572 | If the plan arrives while `busy` is set by a transcription, every step is refused "still working" in one cascade. `start_goal` does not check whether another operator holds the microphone. | INFEASIBLE | introduced | The mic has to be released during planning and the plan has to arrive inside the transcription window; I did not reproduce it. |

**5. Attacked, could not break (by code reading; the actor has no harness)**
- State machine: the planning, running, done, stopped and failed transitions, planning decline/failure, stop during planning, and a late plan are all handled.
- Stop with a proposal waiting rejects it in the port, clears `pending` and sends the document.
- A late answer after stop is discarded and `busy` is cleared.
- `say`, a second `goal`, and opening the mic are refused while a goal runs. Say and goal are refused with `goal_running`; mic opening gets `failed`, which is deliberate.
- Review vs auto: `goal.review` is fixed at start. The auto path accepts and moves to the next step.
- Refused or declined steps continue the run, including a target that does not resolve.
- A stale accept, a reject of another id, and an accept of a proposal that is not the current step's are all ignored by `decided`.
- The goal's API operator is kept alive while the goal is active, and expires after it ends.
- When the goal's browser operator disconnects, the goal goes on and anyone can stop it.
- The wire enums and the generated Rust/TS unions agree on all 5 states and 7 statuses.
- Settle for `stop_goal` works across owners.

**6. Written outside the worktree**
- `/home/timo/.cache/uilab-wave-w2/agent-goals/adversary-widget-red.log`
- `/home/timo/.cache/uilab-wave-w2/agent-goals/adversary-widget-suite.log`
- `/home/timo/.cache/uilab-wave-w2/agent-goals/adversary-rust-red.log`
- `/home/timo/.cache/uilab-wave-w2/agent-goals/adversary-rust-suite.log`
- Builds went into the assigned `/home/timo/.cache/b10x-target/uilab-w2-agent-goals`. The commit-message file in the same scratch dir was deleted.

**7.**
```findings
- file: widget/src/store.ts
  line: 280
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "stopping a goal while a step thinks leaves the owner's phase at thinking and the agent banner up, because the discarded answer sends nothing and no goal message clears either"
- file: widget/src/lib/goal.ts
  line: 33
  category: concurrency
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "after reconnecting to a restarted server the panel keeps the old running goal with a Stop button that the server refuses"
- file: crates/uilab-app/src/app.rs
  line: 635
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "start_goal silently rejects the waiting proposal and the browser card stays open, so Accept on it is refused"
- file: crates/uilab-app/src/api.rs
  line: 254
  category: concurrency
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "a new op goal settles on the operator's earlier ended goal if a Connected re-broadcast of it reaches act before the planning message"
- file: crates/uilab-app/src/app.rs
  line: 537
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the session actor's goal wiring has no test, and the only seam (injecting Cmd::Planned and Cmd::Proposed) is reachable only from a test module inside app.rs"
- file: crates/uilab-app/src/op.rs
  line: 272
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "op goal under review prints proposal ids only after settling, so a waiting step cannot be decided from the CLI and op accept without an id uses a stale cached proposal"
- file: crates/uilab-app/src/app.rs
  line: 311
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "Connected never re-sends the waiting proposal, so after a reload a goal waiting on review can only be stopped"
- file: crates/uilab-app/src/app.rs
  line: 572
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "a plan arriving while a transcription holds busy refuses every step in one cascade, and start_goal does not check for an open microphone"
```
