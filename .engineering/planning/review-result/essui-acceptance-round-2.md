---
format: aep.planning-md/3
id: review-result:essui-acceptance-round-2
kind: review-result
status: active
title: Acceptance critic, round 2, epic:ess-ui-adoption
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

story:essui-document — `uilab_checks_survivors` now names the 18 baseline checks (fixed), but not which of the 11 whose ids ESS's `CHECKS` lacks must survive (`format_marker`, `nav_unique`, `shell_refs`, `page_kind_known`, `page_outlet`, `widget_args`, `widget_recursion`, `widget_resolves`, `widget_named_like_builtin`, `draft_read`, `replace_drops`). "every one ESS's `ess_ui_check::CHECKS` covers is deleted" has no rule for "covers", and `## Survivors` is "Filled by the implementor", so the exact list the test asserts is whatever the implementor chooses — .engineering/planning/story/essui-document.md:102
story:essui-document — "no behaviour change there beyond the format" names no test or command, so it reads true whatever the call-site edits did — .engineering/planning/story/essui-document.md:121
story:essui-document — `placeholder_reads` ends "`grep -rn 'draft\.' crates/uilab-doc/src crates/uilab-app/src` finds no view prefix", which is a judgement over grep hits, not an output or exit status; say what the grep must print — .engineering/planning/story/essui-document.md:83
story:essui-library-model — "the coordinator runs both" has one command: `ess specify compile --format json` is named, but "every view field occurs as a key in a fixture row" has none, and the output goes to a wave log rather than a test (the command half is fixed) — .engineering/planning/story/essui-library-model.md:33
story:essui-agent-schema — "every case's expectations are stated in `ess-ui/1` terms (no section `title`, placeholder reads)" names no test or grep; only the `eval_suite.rs` half is checkable — .engineering/planning/story/essui-agent-schema.md:47
story:essui-app-widget — the Outcome says the canvas shows the expanded document with kind-contributed sections and widget bodies marked inherited, but the acceptance tests only the outline mark and that the canvas offers no remove, so nothing checks the canvas shows them — .engineering/planning/story/essui-app-widget.md:53
story:essui-app-widget — `serve_refuses_draft_format` ends "with the converted library example it serves", which names no observable (an HTTP response, an exit state) — .engineering/planning/story/essui-app-widget.md:36
story:essui-docs — the Outcome says the README and AGENTS.md point at ESS's reference, but the only link check is one grep over seven `website/docs` files, and the `ui-spec` grep is an absence check that a deleted mention passes — .engineering/planning/story/essui-docs.md:34
story:essui-docs — the Outcome says screenshots are retaken "with the Preview button", but the acceptance checks only that every file in `website/static/img/screens` changed in the diff — .engineering/planning/story/essui-docs.md:39
story:essui-docs — "One independent visual review of the deployed site returns PASS" gives no criteria, so PASS is a vote, and "deployed" gives no gate that must pass before the story can close — .engineering/planning/story/essui-docs.md:42
story:eval-round-5 — the D7 bullet joins an "until" state with an "afterwards" outcome, names no path for the archived working copy ("archived beside it"), and its second half cannot be observed until the epic's last pull request merges, which carries this story's own report — .engineering/planning/story/eval-round-5.md:40

What I read: 6 of 6 ids, whole bodies, via `aep plan artifact show`. I also ran `aep plan artifact findings` and `show` on round 1. For the tree and ESS, I ran `git show 0.48.0:` on `ess-ui-check` (`CHECKS`, `unbound_placeholder`) and the 0.48.0 `ess` help and `specify validate`. I grepped `check.rs` (18 ids), `examples/library/library.ui.yaml`, `website/docs`, `crates/uilab-agent/tests` and `widget/src`.

Round 1 against this round:

| Round 1 | Now |
|---|---|
| 12 findings | 10 fixed: the 19 findings (now 7 named cases), the agent-schema bullet (now 4 named tests), the app-widget bullets (now named tests), the docs grep and link (now scoped, with a named URL), the 2 eval-round-5 findings, the library-model "hand conversion" |
| survivors baseline | persisted in part: the baseline is named, the survivor rule is not |
| library-model negative | persisted in part: one command named, the other half still has none |

What I could not establish:
- **Zero warnings:** whether `ess ui check` at 0.48.0 prints `0 warning(s)` on the converted library example. The installed `ess` is 0.44, and I did not run the 0.48.0 binary on a converted document (none exists yet).
  - `unbound_placeholder` (rules.rs:555) fires on any `reads.placeholder`, with or without a model. The current library uses only `view:` reads, so it holds only if the conversion adds no placeholder read. No story states that.
  - `layout_complete` is another warning I could not rule out.
- **Out of my lane, not counted in the verdict:**
  - Components-gallery behaviour on `ess-ui/1` is no longer in any acceptance. Coverage and design critics should decide.
  - `crates/uilab-agent/tests/**` is in scope of both essui-document and essui-agent-schema, which is a parallel-safety question.
  - `eval-round-5` does not depend on `essui-library-model`; this holds only if "the model" never means the ESS model.

```findings
[
  {
    "file": ".engineering/planning/story/essui-document.md",
    "line": 102,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "pre-existing",
    "message": "`uilab_checks_survivors` now names the 18 baseline checks, but not which of the 11 whose ids ESS's CHECKS lacks must survive, and gives no rule for what \"ESS's CHECKS covers\" means; Survivors is \"Filled by the implementor\", so the exact list the test asserts is whatever the implementor chooses"
  },
  {
    "file": ".engineering/planning/story/essui-document.md",
    "line": 121,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"no behaviour change there beyond the format\" names no test or command, so it reads true whatever the call-site edits did"
  },
  {
    "file": ".engineering/planning/story/essui-document.md",
    "line": 83,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "`grep -rn 'draft\\.' crates/uilab-doc/src crates/uilab-app/src` \"finds no view prefix\" is a judgement over grep hits, not an output or exit status; say what the grep must print"
  },
  {
    "file": ".engineering/planning/story/essui-library-model.md",
    "line": 33,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "pre-existing",
    "message": "\"the coordinator runs both\" has one command named (`ess specify compile --format json`); \"every view field occurs as a key in a fixture row\" has none, and the output goes to a wave log rather than a test"
  },
  {
    "file": ".engineering/planning/story/essui-agent-schema.md",
    "line": 47,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"every case's expectations are stated in `ess-ui/1` terms (no section `title`, placeholder reads)\" names no test or grep; only the eval_suite.rs half is checkable"
  },
  {
    "file": ".engineering/planning/story/essui-app-widget.md",
    "line": 53,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the Outcome says the canvas shows the expanded document with kind-contributed sections and widget bodies marked inherited, but the acceptance tests only the outline mark and that the canvas offers no remove, so nothing checks the canvas shows them"
  },
  {
    "file": ".engineering/planning/story/essui-app-widget.md",
    "line": 36,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "`serve_refuses_draft_format` ends \"with the converted library example it serves\", which names no observable (an HTTP response, an exit state)"
  },
  {
    "file": ".engineering/planning/story/essui-docs.md",
    "line": 34,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the Outcome says the README and AGENTS.md point at ESS's reference, but the only link check is one grep over seven website/docs files, and the `ui-spec` grep is an absence check that a deleted mention passes"
  },
  {
    "file": ".engineering/planning/story/essui-docs.md",
    "line": 39,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the Outcome says screenshots are retaken \"with the Preview button\", but the acceptance checks only that every file in website/static/img/screens changed in the diff"
  },
  {
    "file": ".engineering/planning/story/essui-docs.md",
    "line": 42,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"One independent visual review of the deployed site returns PASS\" gives no criteria, so PASS is a vote, and \"deployed\" gives no gate that must pass before the story can close"
  },
  {
    "file": ".engineering/planning/story/eval-round-5.md",
    "line": 40,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the D7 bullet joins an \"until\" state with an \"afterwards\" outcome, names no path for the archived working copy (\"archived beside it\"), and its second half cannot be observed until the epic's last pull request merges, which carries this story's own report"
  }
]
```
