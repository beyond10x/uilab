---
format: aep.planning-md/3
id: review-result:widget-model-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:widget-model
relations:
- reviews: story:widget-model
revision: 1
---
unit: story:widget-model pass 2, correction 44880c1; findings cover 072abde (44880c1 plus my test file only)
verdict: NEEDS-CHANGE
cases: executed 36→40 (`uilab-doc`), red 3
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (a commit-message file, since deleted)
needs-coordinator: no

The correction fixes F1, F2 and F4, and fixes F5 for sections, but instances in a page header's `metrics` are still never checked, and the docs still say "Not used yet." for a widget used only in untyped positions.

**1. Diff stat** (`git --no-pager diff --stat 44880c1 HEAD`)
```
 .../uilab-doc/tests/adversary_widget_model_p2.rs   | 130 +++++++++++++++++++++
 1 file changed, 130 insertions(+)
```
The only file touched is a test file. Author and committer are both `b10x-bot[bot]`.

**2. Cases added** in `crates/uilab-doc/tests/adversary_widget_model_p2.rs`. The red output is from the first run of this file on its own.

| test | asserts | now |
|---|---|---|
| `an_unresolved_instance_in_a_page_header_metric_is_reported` :61 | an undeclared widget in ess `header.metrics: {list: Node}` is reported | red |
| `removing_a_widget_a_page_header_metric_still_uses_is_refused` :75 | removing a widget that only a header metric uses is refused | red |
| `docs_list_a_use_site_in_an_untyped_node_position` :97 | the Widgets docs list a use in `children` | red |
| `an_explicit_required_false_is_kept_and_means_optional` :117 | `required: false` is kept and `is_required()` returns false | green |

```
an instance of an undeclared widget in header.metrics goes unreported: []
---
assertion `left == right` failed: a widget still used by a page header metric was removed
  left: Ok(())
 right: Err("widget_resolves")
---
docs miss the use in `children`:
### state_badge ... Body: `badge` (badge)

Not used yet.
test result: FAILED. 1 passed; 3 failed
```

**3. Suite runs, after the cases existed**
- `cargo test -p uilab-doc --no-fail-fast`: `adversary_widget_model` passed 9 of 9, `adversary_widget_model_p2` passed 1 and failed 3, `doc.rs` passed 27 of 27. Exit 101. The "before" count of 36 is those 9 plus 27, with my pass-2 file left out.
- `cargo fmt --check` exited 0 and clippy with `-D warnings` passed.
- `cargo check --workspace --all-targets` passed with no warnings. The `NodeContext<'a>` and `Param.required: Option<bool>` API changes compile in the other crates.
- `cargo test -p uilab-behaviour -p uilab-agent -p uilab-app`: 26 passed, exit 0.

**4. The two things you asked me to check**
- **F3 case:** it was pinned, not relaxed. It asserts the exact error text `item: invalid type: sequence, expected a map`, and it panics with instructions to flip the case once the list form parses. The test is still called `…_parses_and_resolves` while it now asserts a refusal. That name is misleading, but I did not raise it as a finding.
- **Round-trip assertion in `doc.rs:643`:** it checks less than before. `assert!(!compact.required)` became "the serialized `compact` has no `required` key", which no longer goes through `is_required()`. The same check is still covered elsewhere: if `is_required()` treated a missing value as true, `a_document_with_widgets_round_trips` would fail, because it asserts `check()` finds nothing. My pass-2 case `:117` now asserts `is_required()` directly.

**5. Findings** (tree 072abde)

| # | file:line | finding | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| 1 | `crates/uilab-doc/src/check.rs:565` | `composites()` walks only sections and overlays of a page. `Page.extra` holds `header` with `metrics: {list: Node}` (ess header, order 6), and nothing walks it, so an instance there is never resolved or arg-checked, and the widget it uses can be removed. | any page with `header.metrics`, which ess documents. Page-kind headers and `header.actions[].choice` (ess Action `choice: Node`) are in the same position. | NEEDS-CHANGE | introduced |
| 2 | `crates/uilab-doc/src/docs.rs:331` | "Used at" only matches `c.component.widget()`, so uses that `instances_in` sees in untyped props are missing. The docs say "Not used yet." for a widget that the checks refuse to remove. | the acceptance line "docs list widgets with params and use sites" | NEEDS-CHANGE | introduced |

**6. Attacked and not broken**
- Param keeps untyped keys, `default: null` and `required: false`.
- An `args` literal is not treated as a node.
- Recursion through `children` and `parts` is found.
- Removing a widget used in section children, parts or toolbar is refused.
- `composite_kinds` leaves out the widget itself and widgets that would make it recurse.
- The dependent crates compile and their tests pass.

**7. Paths written outside the worktree**
- `/home/timo/.cache/uilab-wave-w1/widget-model/adv-p2-commit-msg.txt`, deleted after the commit.
- Pass-1 scratch is still there: `/home/timo/.cache/uilab-wave-w1/widget-model/adv-base/` (117M) and `adv-ess-ui.schema.yaml`.
- Free space on `/` is down to 11G.

```findings
- file: crates/uilab-doc/src/check.rs
  line: 565
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "widget instances in a page header's metrics (ess header.metrics: list of Node) are never resolved or arg-checked, and removing the widget they use is admitted"
- file: crates/uilab-doc/src/docs.rs
  line: 331
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the Widgets docs omit use sites in untyped node positions and call such a widget unused although the checks refuse its removal
```
