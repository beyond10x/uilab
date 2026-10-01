---
format: aep.planning-md/3
id: review-result:essui-parallel-round-1
kind: review-result
status: active
title: Parallel-safety critic, round 1, epic:ess-ui-adoption
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

Within the computed waves, no two items in one wave share a file by their cited scopes. Wave 1 is document and library-model. Wave 2 is agent-schema and app-widget. The two collisions `aep plan artifact waves --status draft` prints are both across waves, so neither is a concurrent collision. The pair that does land on one file is library-model and app-widget, on `generated/**` and `widget/src/generated/**`. The plan does not say so, and the two are in different waves only by accident.

story:essui-app-widget — it and story:essui-library-model both rewrite `generated/**` and `widget/src/generated/**` (cited, both scopes), and neither body mentions the other. No edge links them. Wave 2 is computed only from app-widget's `depends_on essui-document`. library-model's hedge "only if 0.48.0 regenerates different bytes" is already settled: `generated/rust/uilab/.ess-output/state.json` records `"producer":"ess 0.44.0"`, so the pin bump to 0.48.0 changes the generated tree. `task generate` begins with `rm -rf generated widget/src/generated`, so app-widget regenerates the whole tree. Two remedies: add `depends_on story:essui-library-model` to app-widget, giving the shared generated tree as the reason. Or split it so the pin bump and its regeneration are one unit and app-widget regenerates only after it. — .engineering/planning/story/essui-app-widget.md:52 (other side: .engineering/planning/story/essui-library-model.md:51)

story:essui-app-widget — its scope `crates/uilab-app/**` swallows `crates/uilab-app/tests/eval_suite.rs`. story:essui-agent-schema creates that file and claims it as "the only file this story adds in that crate", but app-widget carves it out nowhere. app-widget's own `session_writes_ess_ui` "server test" has no stated file, so the two can land in the same new test file (cited for the claim, inferred for the clash). Name app-widget's test file, or exclude `tests/eval_suite.rs` in its scope. — .engineering/planning/story/essui-app-widget.md:50 (other side: .engineering/planning/story/essui-agent-schema.md:49)

story:essui-agent-schema — it and story:essui-app-widget are both in wave 2 and both need ESS crates in the Rust workspace, yet neither scope lists `Cargo.lock`. Only essui-document lists it. app-widget names `ess_ui_check::check` in a uilab-app test. agent-schema names `ess_ui_check` in `schema_patches_pass_ess`. `crates/uilab-app/Cargo.toml` and `crates/uilab-agent/Cargo.toml` have no `ess-ui*` dependency today. If either takes the dependency directly, each edits adjacent `uilab-agent` and `uilab-app` entries in the one `Cargo.lock` (inferred). Either state which crate carries the dependency, or record `Cargo.lock` as shared with an ordering edge. — .engineering/planning/story/essui-document.md:88 (neither wave-2 body mentions it)

story:essui-library-model — a wave-1 weak overlap with story:essui-document on `Cargo.lock` is possible. `generated/rust/uilab-wire/Cargo.toml` carries exact pins (`serde =1.0.229`, `serde_json =1.0.151`) and is a workspace path dependency. If 0.48.0 regenerates it, it changes `Cargo.lock`, which document also edits. This is inferred, and the body does not list `Cargo.lock`. — .engineering/planning/story/essui-library-model.md:51

story:essui-app-widget — the acceptance "refuses a ui-spec/1 file at startup with ESS's message" is in session text that app-widget does not scope. `ess/domains/session.yaml` lines 26, 107 and 159 say "ui-spec/1", and it feeds the generated server crate. The scope names only `ess/domains/wire.yaml`. `ess/system.yaml:3` also names `ui-spec/1`, and no story owns it. Add `ess/domains/session.yaml` (and decide on `ess/system.yaml`) to a scope. — .engineering/planning/story/essui-app-widget.md:52

What I read: 6 stories, through `aep plan artifact waves --status draft`, `show` on each and `graph`. `aep plan artifact scope &lt;id&gt;` errors with no path, so I read scopes from `show`. I also read `Cargo.toml`, `Taskfile.yml`, `ess/ess-inputs.yaml`, `crates/uilab-app/src/eval.rs`, `evals/library.yaml`, `generated/**/Cargo.toml` and `state.json`, and ran `git ls-files`/grep for `ui-spec` outside the scopes. All six stories are placed. The mix is cited and inferred, as `waves` marks it: essui-document, essui-library-model and eval-round-5 each carry an inferred surface, and I checked those against the tree.

Not established or not mine:
- Whether the `Cargo.lock` hunks of agent-schema and app-widget would actually conflict textually. That depends on which crate takes the ESS dependency, which no body says.
- Out of my lane: the tool's flag of document against eval-round-5 on `crates/uilab-agent/src/lib.rs` (and the same file under agent-schema) is ordered by the `depends_on` chain, so it is not a concurrency finding. The edge records no reason for it.
- Out of my lane: library-model's second acceptance reads document's converted `examples/library/library.ui.yaml` and has no edge. It does say "at wave integration", so the plan states that coupling and I count it as declared.
- No collision found between essui-docs and eval-round-5: grep shows no reference to `evals/` in the README, AGENTS.md or `website/docs`.

```findings
[
  {
    "file": ".engineering/planning/story/essui-app-widget.md",
    "line": 52,
    "category": "parallel-safety",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "both this and story:essui-library-model rewrite generated/** and widget/src/generated/** (state.json records producer ess 0.44.0, so the 0.48.0 bump certainly regenerates them; task generate does rm -rf of both trees), no edge links them, and wave 2 comes only from the essui-document edge; add an ordering edge recording the shared generated tree, or split regeneration into one unit"
  },
  {
    "file": ".engineering/planning/story/essui-app-widget.md",
    "line": 50,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "scope crates/uilab-app/** includes crates/uilab-app/tests/eval_suite.rs that story:essui-agent-schema creates, and the body names no file for its own session_writes_ess_ui server test, so two wave-2 items can land in one new test file (inferred); name the test file or carve out eval_suite.rs"
  },
  {
    "file": ".engineering/planning/story/essui-document.md",
    "line": 88,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "Cargo.lock is listed only by essui-document, but story:essui-agent-schema (ess_ui_check in schema_patches_pass_ess) and story:essui-app-widget (ess_ui_check::check in session_writes_ess_ui) both likely add ESS dependencies to crates that have none today, so both edit Cargo.lock in wave 2 (inferred); state which crate carries the dependency or record Cargo.lock as shared with an ordering edge"
  },
  {
    "file": ".engineering/planning/story/essui-library-model.md",
    "line": 51,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "regenerating generated/rust/uilab-wire under 0.48.0 may change its exact serde pins and so Cargo.lock, which story:essui-document also edits in wave 1, and this body does not list Cargo.lock (inferred)"
  },
  {
    "file": ".engineering/planning/story/essui-app-widget.md",
    "line": 52,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the acceptance that serve refuses a ui-spec/1 file with ESS's message touches session text in ess/domains/session.yaml (lines 26, 107, 159 say ui-spec/1) and ess/system.yaml:3, which no story scopes; the scope names only ess/domains/wire.yaml"
  }
]
```
