---
format: aep.planning-md/3
id: review-result:essui-design-round-2
kind: review-result
status: active
title: Design critic, round 2, epic:ess-ui-adoption
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
story:essui-agent-schema — its outcome says it is "the only one in the epic that changes the prompt text", but story:eval-round-5 scopes `crates/uilab-agent/src/lib.rs` prompt lines for fixes, so two bodies claim the same surface. State that eval-round-5 edits the prompt only after this story, for a named failure — .engineering/planning/story/essui-agent-schema.md:30 and .engineering/planning/story/eval-round-5.md:48
story:eval-round-5 — its D7 acceptance is conditioned on "the epic's last pull request" being merged, yet it has no edge to story:essui-docs, which runs in the same wave and may land last, so the item cannot be closed on its own. Add `depends_on story:essui-docs`, or say D7 is a post-merge step of the epic's final pull request — .engineering/planning/story/eval-round-5.md:40 and `aep plan artifact graph`

Round 1 verdict by finding. Four are fixed and one is only partly fixed, which is the first finding above.

| Round 1 finding | Now |
|---|---|
| library-model gate before the conversion | fixed: `depends_on story:essui-document` is now in the graph |
| prompt claimed by document and agent-schema | mostly fixed: document scopes agent `src` "except the prompt text", but the eval-round-5 overlap remains (first finding) |
| node shapes split between `path.rs` and `schema.rs` | fixed: `children_from_ess` sits in document and agent-schema "builds on" it |
| eval-round-5 "against the model" with no edge | fixed: its outcome now says the check runs without `--model` |
| app-widget regenerating `generated/**` before the pin | fixed: the pin and regeneration are in document, wave 1 |

What I read: 7 artifacts (the epic, the 6 stories and review-result:essui-design-round-1), whole bodies, through `aep plan artifact show`. I also ran `aep plan artifact relations`, `aep plan artifact graph` and `aep plan artifact validate` (valid). I walked every edge in the store, outside the set as well. There is no cycle. The `depends_on` edges form a fan-out from document to the wave-2 stories, and join again at docs and eval-round-5. The set is neither a serial chain nor a horizontal slice.

What I could not establish:
- Whether the D2 path bijection covers inherited nodes. document's `paths_round_trip` is stated over "every node of every document", and app-widget needs uilab paths for expanded nodes. The text does not say which. This is an unease, not a finding.
- Whether `ess ui check` without `--model` is clean on a document that declares `model: library`. That is acceptance's lane.
- Whether the "API call sites only" carve-outs in document are workable. That is parallel-safety's lane.

```findings
[
  {
    "file": ".engineering/planning/story/essui-agent-schema.md",
    "line": 30,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "its outcome says it is the only story in the epic that changes the prompt text, but story:eval-round-5 scopes crates/uilab-agent/src/lib.rs prompt lines for fixes (eval-round-5.md:48), so two bodies claim one surface; say eval-round-5 edits the prompt only after this story, for a named failure"
  },
  {
    "file": ".engineering/planning/story/eval-round-5.md",
    "line": 40,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "its D7 acceptance is conditioned on the epic's last pull request being merged, but it has no depends_on edge to story:essui-docs, which runs in the same wave and may land last; add depends_on story:essui-docs or state that D7 is a post-merge step of the epic's final pull request"
  }
]
```
