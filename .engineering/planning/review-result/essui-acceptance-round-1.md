---
format: aep.planning-md/3
id: review-result:essui-acceptance-round-1
kind: review-result
status: active
title: Acceptance critic, round 1, epic:ess-ui-adoption
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
story:essui-document — `ess_decides_admission` says "for each of the 19 findings" but the body lists only 7 categories. The 19 live in the operator's working copy, which the epic says is archived, so nobody can enumerate or test the 19. List them in the body, or restate as the 7 named cases — .engineering/planning/story/essui-document.md:68
story:essui-document — `uilab_checks_survivors` ends in "any not listed are deleted" and the Survivors section reads "Filled by the implementor". No baseline set of uilab's own checks is named, so the test cannot tell a survivor from a deleted check. Name the checks that exist today, or the grep that finds them — .engineering/planning/story/essui-document.md:76
story:essui-library-model — the second acceptance line is checked "during the unit, against the hand conversion the epic measured", but that file is not in the tree or the store. The unit's check has no input. Name a committed file, or drop the in-unit clause — .engineering/planning/story/essui-library-model.md:34
story:essui-library-model — "No entity, field or state is invented beyond what the document and fixtures name" is a negative nobody can run. "Listed in the report" names no report. Make it a command (every model name appears in the document or fixtures) and name where the `UNMAPPED:` list is recorded — .engineering/planning/story/essui-library-model.md:37
story:essui-agent-schema — one bullet states four outcomes (names `ess-ui/1`, teaches placeholder reads, no `draft.` views, no section `title`) and no test is named, only "`propose.rs` holds it". One outcome can fail while another passes. Name one test per outcome — .engineering/planning/story/essui-agent-schema.md:36
story:essui-app-widget — "the server test says which" names no test and no assertion, so which node an inherited-node instruction targets is a statement, not a check. Name the test and what it asserts — .engineering/planning/story/essui-app-widget.md:36
story:essui-app-widget — the findings-panel, placeholder, Components-gallery and "inherited mark renders / cannot be removed" bullets say "shows" or "renders" with no named test, command or output. Two of them each join two independent outcomes. Name a test per outcome — .engineering/planning/story/essui-app-widget.md:38
story:essui-docs — the `grep -rn 'ui-spec/1' … crates …` check cannot pass. `crates/uilab-doc/tests` carries `format: ui-spec/1` fixtures, and `loads_only_ess_ui` (story:essui-document) needs a `ui-spec/1` input to assert the refusal. The "sentence that says uilab moved" exception is not greppable either. Scope the grep or name an exact allow-list — .engineering/planning/story/essui-docs.md:30 (grep hits: crates/uilab-doc/tests/adversary_replace_drops.rs:141, adversary_item_list_p2.rs:11, adversary_widget_model_p2.rs:9)
story:essui-docs — "link ESS's reference page for `ess-ui/1`" names no URL or path, and "describe `ess-ui/1` as ESS publishes it" has no checkable test beyond four keywords. Give the exact link target and make the check a grep for it on each page — .engineering/planning/story/essui-docs.md:35
story:essui-docs — "Every screenshot … is retaken" names no before/after distinction. Nothing separates a retaken image from an untouched one that still passes `npm run build`. Name what makes the change observable, such as every file changed in the commit or a capture manifest — .engineering/planning/story/essui-docs.md:36
story:eval-round-5 — "About 30 model calls per round" is not observable. "About" gives it no pass/fail edge, and it is a budget rather than an outcome of the story. State a number the journal can be counted against, or move it out of the acceptance — .engineering/planning/story/eval-round-5.md:38
story:eval-round-5 — "runs all 27 cases against the model" is ambiguous between the LLM and the ESS library model. The acceptance never says which, so a run with or without `--model` both read as done. Say what "the model" is — .engineering/planning/story/eval-round-5.md:21

What I read: 6 of 6 ids, whole bodies. I ran `aep plan artifact show` on each, `aep plan artifact kinds` and `aep plan artifact lifecycle story`. I also read the epic and the story files for line numbers. I checked `evals/library.yaml` (27 case ids, so the "27" holds) and `crates/uilab-doc/tests`. I checked ess `0.48.0` for `ess_ui::SCHEMA` (crates/ui/ess-ui/src/lib.rs:40), `ess_ui_check::check` (crates/ui/ess-ui-check/src/lib.rs:305) and the `unbound_placeholder` warning.

What I could not establish:
- I could not run `ess ui check` at 0.48.0: the installed `ess` is 0.44.0.
- I could not tell whether `examples/library/library.ui.yaml` has an unbound placeholder, which would make "0 warning(s)" unreachable (essui-document, essui-library-model).
- The stories do not cover the epic's D7 (the demo server keeps running the current binary until the gate is green) or ESS-side findings filed from survivors. Whether anything should cover them is out of my lane, for the scope and design critics. It did not set my verdict.
- The missing `essui-library-model` dependency from eval-round-5 (if "the model" means the ESS model) is a design-lane question. It did not set my verdict.

```findings
[
  {
    "file": ".engineering/planning/story/essui-document.md",
    "line": 68,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "`ess_decides_admission` says \"for each of the 19 findings\" but the body lists only 7 categories; the 19 live in the archived working copy, so nobody can enumerate or test the 19"
  },
  {
    "file": ".engineering/planning/story/essui-document.md",
    "line": 76,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "`uilab_checks_survivors` ends in \"any not listed are deleted\" and Survivors reads \"Filled by the implementor\"; no baseline set of uilab's own checks is named, so a survivor cannot be told from a deleted check"
  },
  {
    "file": ".engineering/planning/story/essui-library-model.md",
    "line": 34,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the in-unit check runs against \"the hand conversion the epic measured\", which is not in the tree or the store, so the unit's check has no input"
  },
  {
    "file": ".engineering/planning/story/essui-library-model.md",
    "line": 37,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"No entity, field or state is invented\" is an unrunnable negative and \"listed in the report\" names no report; make it a command and name where the UNMAPPED list is recorded"
  },
  {
    "file": ".engineering/planning/story/essui-agent-schema.md",
    "line": 36,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "one bullet states four independent outcomes (names ess-ui/1, teaches placeholder reads, no draft. views, no section title) and names no test, only \"propose.rs holds it\""
  },
  {
    "file": ".engineering/planning/story/essui-app-widget.md",
    "line": 36,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"the server test says which\" names no test and no assertion, so which node an inherited-node instruction targets is a statement, not a check"
  },
  {
    "file": ".engineering/planning/story/essui-app-widget.md",
    "line": 38,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the findings-panel, placeholder, gallery and inherited-mark bullets say \"shows\"/\"renders\" with no named test, command or output, and two of them join two independent outcomes"
  },
  {
    "file": ".engineering/planning/story/essui-docs.md",
    "line": 30,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the `grep ... crates ...` check cannot pass, because crates/uilab-doc/tests carries `format: ui-spec/1` fixtures that `loads_only_ess_ui` needs, and the \"sentence that says uilab moved\" exception is not greppable"
  },
  {
    "file": ".engineering/planning/story/essui-docs.md",
    "line": 35,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"link ESS's reference page\" names no URL or path, and \"as ESS publishes it\" has no checkable test beyond four keywords"
  },
  {
    "file": ".engineering/planning/story/essui-docs.md",
    "line": 36,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"Every screenshot ... is retaken\" names no before/after distinction, so a retaken image and an untouched one both pass `npm run build`"
  },
  {
    "file": ".engineering/planning/story/eval-round-5.md",
    "line": 38,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"About 30 model calls per round\" has no pass/fail edge and is a budget rather than an outcome"
  },
  {
    "file": ".engineering/planning/story/eval-round-5.md",
    "line": 21,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"runs all 27 cases against the model\" is ambiguous between the LLM and the ESS library model, so a run with or without `--model` both read as done"
  }
]
```
