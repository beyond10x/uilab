---
format: aep.planning-md/3
id: review-result:essui-scope-round-2
kind: review-result
status: active
title: Scope critic, round 2, epic:ess-ui-adoption
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
story:essui-document — D1 promises "uilab edits the authored document (unexpanded, key order kept)", and only key order is claimed. No acceptance says that a saved file holds only what the author wrote, with no page-kind sections, widget bodies or other expansion output written back. `session_writes_ess_ui` checks only "0 errors", which an expanded file would also pass. The body needs a named acceptance on the saved file, such as an expanded contribution is never present in it after any patch. — .engineering/planning/epic/ess-ui-adoption.md:54 (story body: .engineering/planning/story/essui-document.md:69-94, `grep -n -i 'unexpanded' story/essui-*.md` finds nothing)

Round 1's four findings did not survive, so none is marked persisted. Each was fixed at the cited place:
- **D7:** now in `story:eval-round-5` (Outcome, last acceptance bullet).
- **uilab's own spec naming `ui-spec/1`:** now in `story:essui-document` (the Toolchain acceptance, naming `ess/system.yaml:3` and `ess/domains/session.yaml:26,107,159`).
- **Components gallery `note`:** the acceptance is gone from `story:essui-app-widget`.
- **Duplicate prompt claim:** now split. `story:essui-document` says "The agent's prompt text is not changed here", and `story:essui-agent-schema` owns the prompt.

**What I read:** 8 artifacts, whole. I ran `aep plan artifact show` on the epic (revision 4), the six stories and `review-result:essui-scope-round-1`, and `aep plan artifact graph`. I also ran `git ls-files` and `git grep 'ui-spec'` over the worktree to check which `ui-spec/1` mentions a story owns. From the epic I extracted 17 promises: the outcome (4), the rule, D1 to D7, the gap-table conversions, the filed #281 workaround, and the three "Not covered" exclusions. 16 trace to a story. The one that does not is D1's "unexpanded". The exclusions (ESS paths on the wire, style tokens, the 452-line working copy) are not drafted anywhere.

**What I could not establish:**
- Out of my lane, and not setting the verdict: `story:essui-app-widget` makes the canvas offer no remove on an inherited node, and retargets an instruction to the nearest authored node. Neither is in the epic's words. I read both as consequences of D1 and D6, so I did not count them as reach.
- Out of my lane, and not setting the verdict: `story:eval-round-5` is still titled "against the model", while its outcome runs ESS "without `--model`".
- Out of my lane, and not setting the verdict: `crates/uilab-agent/Cargo.toml:3` carries the text "ui-spec/1". `story:essui-agent-schema` says "No Cargo manifest changes", so no story is clearly assigned that one-line description. The epic promises no description text, so this is not a gap.
- I did not verify beyond10x/ess#281 on GitHub.

```findings
[
  {
    "file": ".engineering/planning/epic/ess-ui-adoption.md",
    "line": 54,
    "category": "scope",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "D1 promises \"uilab edits the authored document (unexpanded, key order kept)\" and no story claims the unexpanded half: story:essui-document names only authored key order, and session_writes_ess_ui checks only 0 errors, which an expanded file would also pass; essui-document should add an acceptance that a saved file never holds expansion output"
  }
]
```
