---
format: aep.planning-md/3
id: review-result:agent-goals-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:agent-goals
relations:
- reviews: story:agent-goals
revision: 1
---
```
unit: story:agent-goals pass 2, branch unit/agent-goals, covers fb5dcc1 plus my test commit f1119a9 (b10x-bot)
verdict: NEEDS-CHANGE
cases: executed 126→130 (uilab-app 51→52, widget 75→78), red 4
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths
needs-coordinator: no
```

Pass 2 found 4 new red cases, all regressions from the correction. The correction itself holds: my 5 pass-1 cases (C1–C5) are green on fb5dcc1.

**1. `git --no-pager diff --stat fb5dcc1 HEAD`**
```
 crates/uilab-app/src/app.rs   |  28 ++++++++
 widget/src/lib/resend.test.ts | 145 +++++++++++++++++++++
```
The 28 lines in `app.rs` are all added inside the existing `#[cfg(test)] mod tests`, and none are removed. No implementation lines changed.

**2. Cases added.** All are red now. Each red output below was captured by running its file or case alone, before the suite ran.

| # | Case | Red output |
|---|---|---|
| D1 | `resend.test.ts`: the waiting proposal re-sent for a browser that connects changes nothing in one already connected | "the activity feed lists the same proposal twice" `2 !== 1` |
| D2 | same file: an ended goal re-sent on connect does not end an instruction its operator is now working on | "the status line went idle while the instruction is still being worked on" `'idle' !== 'thinking'` |
| D3 | same file: the agent banner survives that re-sent ended goal | "the agent banner went away while the agent is still working on its instruction" |
| D4 | `app.rs` tests: `a_stale_decision_on_another_proposal_keeps_the_step_proposal_for_a_browser_that_connects` | `panicked at crates/uilab-app/src/app.rs:1596:9` "the step waits on a proposal no browser that connects is shown" `left: [] right: ["3c56f098-…"]` |

In every case the precondition asserts passed and only the final assert failed. D4 runs on the new `App::open` rig and makes no model call.

**3. Suite runs, after the cases existed**
- `cargo test -p uilab-app`: `FAILED. 51 passed; 1 failed`, exit 101.
- `pnpm check` in `widget/`: typecheck clean, then `# tests 78 # pass 75 # fail 3`, exit 1.
- The before-counts are 51 and 75. They come from the correction's suite with my new files left out: the filtered-out count for Rust, and a run at fb5dcc1 for the widget.
- `cargo fmt -p uilab-app --check` exits 0.

**4. Findings**

| # | file:line | Finding | Verdict | Origin | What reaches it |
|---|---|---|---|---|---|
| G1 | crates/uilab-app/src/app.rs:345 | The J3 re-send goes out on the shared broadcast. Every browser already connected gets the proposal again: the feed shows it twice, `deciding` is reset, and the owner's view jumps to the node (`reveal`). This also happens for plain `say` proposals, not only goal steps. | NEEDS-CHANGE | introduced | Any browser connecting or reconnecting while a proposal waits. |
| G2 | widget/src/store.ts:345 | The F1 fix ends "thinking" on *any* ended goal from this operator, including the one `Cmd::Connected` re-broadcasts. When a new browser connects, the operator's next instruction flips to idle. | NEEDS-CHANGE | introduced | Goal ends, the same operator gives a `say`, another browser connects. |
| G3 | widget/src/lib/collab.ts:130 | Same cause in `trackInFlight`: the re-broadcast ended goal clears the agent's in-flight entry, so its banner disappears while it works on an instruction. | NEEDS-CHANGE | introduced | `op goal` ends, then `op say`, then a browser connects. |
| G4 | crates/uilab-app/src/app.rs:949 | `accept` (and `Reject` at :839) sets `pending = None` for any id. One stale decision while a step waits hides that step's proposal from every browser that connects afterwards, which undoes J3. | NEEDS-CHANGE | introduced | A leftover card, or `op accept <old id>`, while a goal step waits. The unconditional clear existed at base; J3 now depends on it. |
| G5 | crates/uilab-app/src/op.rs:286 | `last_goal` starts at the last polled state, then the loop replays every collected goal message from `planning` on. After the act returns, every progress line the poll already printed is printed again. | NEEDS-CHANGE | introduced | `op goal` running longer than 2 s. This comes from reading the code only; I did not run it, because running it needs a model. |

**5. Attacked, could not break**
- F1: the owner receives `goal_stopped`, and the late answer is discarded.
- F2: `state.goal` is cleared on open and the server re-sends it.
- F3: the document is sent and the card closes on `planning`.
- F4: `prior`/`id` ignores the earlier goal and any other goal that ends.
- J4: `goal_busy` covers `busy` and an open mic.
- `op accept`/`reject` with no id resolve to the step's waiting proposal.
- The 14 actor tests from the correction pass.

**6. Written outside the worktree**
- `/home/timo/.cache/uilab-wave-w2/agent-goals/adversary-p2-widget-red.log`
- `/home/timo/.cache/uilab-wave-w2/agent-goals/adversary-p2-widget-suite.log`
- `/home/timo/.cache/uilab-wave-w2/agent-goals/adversary-p2-rust-red.log`
- `/home/timo/.cache/uilab-wave-w2/agent-goals/adversary-p2-rust-suite.log`
- Builds went into the assigned `/home/timo/.cache/b10x-target/uilab-w2-agent-goals`. The commit-message file was deleted.

**7.**
```findings
- file: crates/uilab-app/src/app.rs
  line: 345
  category: concurrency
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the waiting proposal re-sent on Connected goes to every subscriber, duplicating the feed entry, resetting deciding and moving the owner's view in browsers already connected"
- file: widget/src/store.ts
  line: 345
  category: concurrency
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the ended goal re-broadcast on Connected sets the phase of its operator's later instruction back to idle"
- file: widget/src/lib/collab.ts
  line: 130
  category: concurrency
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the ended goal re-broadcast on Connected clears the in-flight entry of an agent that is working on a later instruction, so its banner disappears"
- file: crates/uilab-app/src/app.rs
  line: 949
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a stale accept or reject of another id clears pending, so the J3 re-send no longer shows the step proposal the goal waits on"
- file: crates/uilab-app/src/op.rs
  line: 286
  category: judgement
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "op goal replays every collected goal message against the last polled state, so progress lines the poll already printed are printed again (found by reading, not run)"
```
