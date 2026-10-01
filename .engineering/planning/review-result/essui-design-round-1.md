---
format: aep.planning-md/3
id: review-result:essui-design-round-1
kind: review-result
status: active
title: Design critic, round 1, epic:ess-ui-adoption
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
story:essui-library-model — its `task check` gate runs `ess ui check` on every `examples/**/*.ui.yaml` and fails on any error, but those examples stay `ui-spec/1` until story:essui-document converts them, and no edge records that order; add `depends_on story:essui-document`, or move the Taskfile gate into story:essui-document and keep only the model here — .engineering/planning/story/essui-library-model.md:43
story:essui-document — it claims the three `draft.&lt;Name&gt;` prompt lines in `crates/uilab-agent/src/lib.rs`, and story:essui-agent-schema also claims that prompt ("teaches placeholder reads and no `draft.` views"), so two items own one surface and both will mark it done; drop the prompt lines and the `crates` grep from this scope, or from agent-schema, but not both — .engineering/planning/story/essui-document.md:92
story:essui-agent-schema — D3 (shapes come from `ess_ui::SCHEMA`) lands in `schema.rs` only, while `path.rs` ("what each node may hold") is rewritten by story:essui-document, so the two halves of the node-shape knowledge sit in different stories and `patch_schema` consumes both; the body must say who derives the per-kind child layers from ESS once, and which story's acceptance proves it — crates/uilab-doc/src/lib.rs:5 and crates/uilab-doc/src/schema.rs:1-3
story:eval-round-5 — its outline and the epic's story table say "against the model", and the epic means the ESS library model (`ess ui check` against the model), but no edge to story:essui-library-model records that the round needs that model to exist; add `depends_on story:essui-library-model`, or state in the outcome that "the model" is the LLM and the check runs without `--model` — .engineering/planning/epic/ess-ui-adoption.md:79 and .engineering/planning/story/eval-round-5.md:21
story:essui-app-widget — it regenerates `generated/**` and `widget/src/generated/**` after editing `wire.yaml`, and the bytes depend on the ess pin that story:essui-library-model sets (that story's own scope lists the same generated dirs "if 0.48.0 regenerates different bytes"), with no edge between them; add `depends_on story:essui-library-model`, or move the pin and its regeneration into story:essui-document — .engineering/planning/story/essui-app-widget.md:52 and .engineering/planning/story/essui-library-model.md:51

What I read: 7 artifacts (the epic and 6 stories, whole bodies) with `aep plan artifact show`. I also ran `aep plan artifact relations`, `aep plan artifact graph` and `aep plan artifact validate` (valid), and read `crates/uilab-doc/src/lib.rs`, `crates/uilab-doc/src/schema.rs`, `Taskfile.yml` and the ESS 0.48.0 schema for `model`. I walked all the edges in the store, outside the set as well, which is about 23 that touch the set.

No cycle. The only `depends_on` edges are essui-agent-schema and essui-app-widget to essui-document, essui-docs to both of those, and eval-round-5 to both of those. epic:styles, story:first-release and story:ui-polish-leftovers depend on the epic or on essui-app-widget and add no back edge. The set is not a serial chain and not a horizontal slice, because each story leaves `task check` green.

What I could not establish:
- Whether the "compile fixes only" carve-outs are workable (essui-document in `app.rs`, `schema.rs` and `outline.rs`, which other stories own). That is parallel-safety's lane.
- Whether `ess ui check` without `--model` is clean on a document that declares `model: library`. That is acceptance's lane.
- Whether eval-round-5 means the ESS model or the LLM by "the model". The epic's table reads as the ESS model, but "about 30 model calls" in the same story uses the word for the LLM.

```findings
[
  {
    "file": ".engineering/planning/story/essui-library-model.md",
    "line": 43,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "its task check gate runs ess ui check on every examples/**/*.ui.yaml and fails on any error, but those examples stay ui-spec/1 until story:essui-document converts them, and no edge records that order; add depends_on story:essui-document or move the Taskfile gate into story:essui-document"
  },
  {
    "file": ".engineering/planning/story/essui-document.md",
    "line": 92,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "it claims the three draft.&lt;Name&gt; prompt lines in crates/uilab-agent/src/lib.rs that story:essui-agent-schema also claims as its prompt acceptance, so two items own one surface; drop the prompt lines and the crates grep from one of the two scopes"
  },
  {
    "file": "crates/uilab-doc/src/lib.rs",
    "line": 5,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "node-shape knowledge is split, with path.rs (\"what each node may hold\") rewritten by story:essui-document and schema.rs (D3, from ess_ui::SCHEMA) by story:essui-agent-schema, while patch_schema consumes both; the agent-schema body must say who derives per-kind child layers from ESS once and which acceptance proves it"
  },
  {
    "file": ".engineering/planning/story/eval-round-5.md",
    "line": 21,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "it runs \"against the model\" (the ESS library model per the epic story table at epic/ess-ui-adoption.md:79) with no depends_on story:essui-library-model; add the edge or state in the outcome that the model is the LLM and the check runs without --model"
  },
  {
    "file": ".engineering/planning/story/essui-app-widget.md",
    "line": 52,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "it regenerates generated/** and widget/src/generated/** whose bytes depend on the ess pin that story:essui-library-model sets (library-model.md:51 lists the same dirs), with no edge between them; add depends_on story:essui-library-model or move the pin and regeneration into story:essui-document"
  }
]
```
