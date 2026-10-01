---
format: aep.planning-md/3
id: review-result:essui-scope-round-1
kind: review-result
status: active
title: Scope critic, round 1, epic:ess-ui-adoption
relations:
- reviews: epic:ess-ui-adoption
- reviews: story:essui-document
- reviews: story:essui-library-model
- reviews: story:essui-agent-schema
- reviews: story:essui-app-widget
- reviews: story:essui-docs
- reviews: story:eval-round-5
revision: 1
---
needs-revision

epic:ess-ui-adoption — D7 "The demo server keeps running the current binary until the integration gate is green on the merged pull request" is promised and no story claims it; essui-app-widget or eval-round-5 would most naturally carry it — .engineering/planning/epic/ess-ui-adoption.md:68
epic:ess-ui-adoption — "uilab reads, edits and writes only ESS's published UI format… it owns no format of its own" (line 12) and D5 "for its own specification" (line 64) leave uilab's own ESS spec naming `ui-spec/1` (the `OpenDocument` unreadable outcome, `ess/system.yaml:3`, `ess/domains/session.yaml:26,107,159` and its generated copies), and no story edits those files; essui-app-widget already owns `ess/domains/wire.yaml` and `generated/**` and would take it — .engineering/planning/epic/ess-ui-adoption.md:12
story:essui-app-widget — the acceptance "the Components gallery shows widget params with their `note`" traces to no epic sentence; D6 covers the canvas, and the `note` appears only as an ESS refusal in the gap list — .engineering/planning/story/essui-app-widget.md:41
story:essui-document — its scope and its `grep '"draft\.' crates` acceptance claim the `draft.&lt;Name&gt;` prompt lines in `crates/uilab-agent/src/lib.rs`, and story:essui-agent-schema's acceptance claims the same outcome ("teaches placeholder reads… and no `draft.` views"), so both would be marked done for one change — .engineering/planning/story/essui-document.md:92

**What I read:** the epic (revision 2) and all six stories, whole, using `aep plan artifact show`. I also ran `aep plan artifact graph`, `aep plan artifact show` on story:first-release and story:ci-task-check, and `git ls-files` plus `grep -rln 'ui-spec/1'` over the tree. I extracted 14 promises from the epic: 4 in the outcome, 1 in the rule, D1 to D7, the filed ESS requirements, and the format conversions in the gap table. Twelve trace to a story. The two that do not are the findings above.

**What I could not establish:**
- Whether the 19 ESS findings in the epic are all named in `ess_decides_admission`. The acceptance lists 7 categories. That is the acceptance critic's lane.
- eval-round-5 has two bars the epic does not state: "at least 25 of 27 cases pass" and "about 30 model calls, at most 2 rounds". I treated them as acceptance detail, not reach, and did not count them as findings.
- story:essui-docs has a grep that omits `website/src/pages/index.tsx` (lines 143 and 293 say `ui-spec/1`). Its outcome and its `website/**` scope do cover that file, so this is an acceptance gap and out of my lane.
- The epic's filing of beyond10x/ess#281 is stated as done. I did not verify the issue on GitHub.

```findings
[
  {
    "file": ".engineering/planning/epic/ess-ui-adoption.md",
    "line": 68,
    "category": "scope",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "D7 \"The demo server keeps running the current binary until the integration gate is green on the merged pull request\" is promised and no story claims it; essui-app-widget or eval-round-5 would most naturally carry it"
  },
  {
    "file": ".engineering/planning/epic/ess-ui-adoption.md",
    "line": 12,
    "category": "scope",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"owns no format of its own\" and D5 \"for its own specification\" leave uilab's own ESS spec (ess/system.yaml:3, ess/domains/session.yaml:26,107,159, generated copies) naming ui-spec/1, and no story edits it; essui-app-widget, which already owns ess/domains/wire.yaml and generated/**, would take it"
  },
  {
    "file": ".engineering/planning/story/essui-app-widget.md",
    "line": 41,
    "category": "scope",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the acceptance \"the Components gallery shows widget params with their note\" traces to no sentence in the epic; D6 covers the canvas only"
  },
  {
    "file": ".engineering/planning/story/essui-document.md",
    "line": 92,
    "category": "scope",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the story claims the draft.&lt;Name&gt; prompt lines in crates/uilab-agent/src/lib.rs, and story:essui-agent-schema (essui-agent-schema.md:37) claims the same outcome (prompt teaches placeholder reads and no draft. views), so two stories claim one outcome"
  }
]
```
