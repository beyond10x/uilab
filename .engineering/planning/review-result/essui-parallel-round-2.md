---
format: aep.planning-md/3
id: review-result:essui-parallel-round-2
kind: review-result
status: active
title: Parallel-safety critic, round 2, epic:ess-ui-adoption
relations:
- reviews: story:essui-document
- reviews: story:essui-library-model
- reviews: story:essui-agent-schema
- reviews: story:essui-app-widget
- reviews: story:essui-docs
- reviews: story:eval-round-5
revision: 1
---
needs-revision

story:essui-agent-schema — it rewrites `evals/library.yaml`, but `crates/uilab-app/src/eval.rs` (a file in story:essui-app-widget's `crates/uilab-app/src/**` scope) holds a unit test, `the_library_suite_loads_with_its_move_cases`, that pins cases in that suite (`retarget-new-page`, `retarget-navigate`, `retarget-none` with their targets and expect fields). The body says nothing about keeping them; any fix to that test lands in app-widget's file in the same wave. Say that the pinned case ids, targets and expect fields stay, or put that test in this story's scope (inferred coupling, not a shared edit). — .engineering/planning/story/essui-agent-schema.md:47 (pin at crates/uilab-app/src/eval.rs:486-506; `retarget-none` says "give this table the title current loans" at evals/library.yaml:136, and ESS check class 6 refuses `collection` with `title`, so this case is the likely one to be rewritten)

Answer to the one question: in the computed waves, no two items land on one file. The plan does name the cross-wave overlaps, and each is ordered by a `depends_on` edge. The one thing it leaves unsaid is the `eval.rs` coupling above.

Round 1 comparison: none of the five round-1 findings survived, so none is marked `persisted`.

| Round-1 finding | Now | Where fixed |
|---|---|---|
| Both stories rewrite `generated/**` | gone | document owns `generated`, `widget/src/generated`, `ess/ess-inputs.yaml`, `ess/domains/session.yaml` and `ess/system.yaml`. app-widget owns only `wire.yaml` and regenerates after document. library-model is down to `examples/library/model/**` and `Taskfile.yml`. |
| `tests/eval_suite.rs` clash | gone | app-widget scope says "not `tests/eval_suite.rs`". agent-schema says it is the only file it adds in that crate. app-widget's own tests are `tests/session_ess_ui.rs`. |
| `Cargo.lock` in wave 2 | gone | document: "no other story in the epic edits them". agent-schema and app-widget both say "No Cargo manifest changes" and use the `uilab_doc` re-export. |
| `Cargo.lock` from `uilab-wire` pins | gone | library-model no longer regenerates anything. |
| `ui-spec/1` text in `session.yaml` | gone | document scopes `session.yaml` and `system.yaml` and lists the lines. |

Wave-2 pairs checked against the tree:
- **library-model with agent-schema and app-widget:** only library-model writes `Taskfile.yml`. It reads `examples/library/fixtures` and `library.ui.yaml`, which document alone edits. `crates/uilab-app/src/app.rs:1446-1457` copies `examples/library` recursively, so the new `model/` dir is harmless there.
- **agent-schema with app-widget, `uilab-doc`:** `schema.rs` and `outline.rs` are different files. `doc.rs` is a document-scope file, and no tests there pin shapes the two stories would both change.
- **agent-schema with app-widget, tests:** `tests/eval_suite.rs` and `tests/session_ess_ui.rs` are different new files.
- **widget build:** `widget/dist` and `.build/` are gitignored per-tree output. `widget/package.json` already runs `src/lib/*.test.ts`, so the new widget tests need no `package.json` or lockfile edit.

What I read: 6 stories (all placed: 5 cited, 1 inferred through `eval-round-5`'s `crates/uilab-agent/src/lib.rs`), plus `review-result:essui-parallel-round-1`. Commands: `aep plan artifact waves --status draft`, `show`, `graph`. In the tree I read `Taskfile.yml`, the `Cargo.toml` files, `crates/uilab-app/src/eval.rs`, `evals/library.yaml`, `crates/uilab-doc/tests/doc.rs`, `outline.rs` and `app.rs`.

Could not establish, or not mine:
- **`uilab-app` is bin-only** (`mod eval;` and `mod app;` in `main.rs`). `session_writes_ess_ui` and `eval_suite.rs` therefore cannot call `App` or `eval::load` from `tests/` without a new `src/lib.rs` or a spawned binary. Whether either story needs a lib target is not decided; if both do, they would each create `crates/uilab-app/src/lib.rs`. Out of my lane (acceptance feasibility).
- **`crates/uilab-doc/tests/doc.rs`:** whether agent-schema's schema rewrite breaks its `patch_schema` asserts (lines 101 and 1139) I cannot tell from the code. If it does, the edit lands in a file document scopes, in a different wave.
- **Shared Cargo target:** `Taskfile.yml` sets one `CARGO_TARGET_DIR` for every worktree, so three parallel wave-2 gates share one build directory. That is an operational hazard, not a file collision.
- **Wave 3:** docs and eval-round-5 share no file. Whether the screenshot app and D7's port-8740 server collide at runtime is not stated and I did not assess it.

```findings
[
  {
    "file": ".engineering/planning/story/essui-agent-schema.md",
    "line": 47,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "it rewrites evals/library.yaml, but the unit test the_library_suite_loads_with_its_move_cases in crates/uilab-app/src/eval.rs (story:essui-app-widget's crates/uilab-app/src/** scope) pins cases retarget-new-page, retarget-navigate and retarget-none with their targets and expect fields, and retarget-none ('title current loans') is the kind of case ess-ui/1 forces it to rewrite; the body says neither that those pins stay nor that the test is in its scope, so a fix lands in app-widget's file in the same wave (inferred coupling, not a shared edit); state the pinned ids and fields stay, or take the test into scope"
  }
]
```
