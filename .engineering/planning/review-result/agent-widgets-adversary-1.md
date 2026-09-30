---
format: aep.planning-md/3
id: review-result:agent-widgets-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:agent-widgets
relations:
- reviews: story:agent-widgets
revision: 1
---
unit: story:agent-widgets round 2, unit/agent-widgets cd1e73c (my cases committed on top as 2f272fb)
verdict: NEEDS-CHANGE
cases: executed 42→47, red 4
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none

The unit is held: 4 of the 5 cases I added fail against the page-matching rule and the badge line in the prompt.

**1. Diff stat (cd1e73c..HEAD)**
```
 crates/uilab-agent/tests/adversary_widgets_r2.rs | 184 +++++++++++++++++++++++
 1 file changed, 184 insertions(+)
```
It touches one test file and no other path. The commit is authored and committed by `b10x-bot[bot]`.

**2. Cases added** (`/home/timo/.local/state/worktree/trees/b10x/uilab/uilab-w2-agent-widgets/crates/uilab-agent/tests/adversary_widgets_r2.rs`)

| case | asserts | now |
|---|---|---|
| `a_page_with_a_short_plural_name_is_listed_when_the_operator_names_it` | page `ops` is listed when the utterance says "ops page" | red |
| `a_page_with_an_ies_plural_name_is_listed_when_named_in_the_singular` | page `categories` is listed when the utterance says "category page" | red |
| `a_page_named_like_a_common_word_is_not_listed_when_only_the_word_is_said` | page `news` is not listed for "make a new … on the members page" | red |
| `the_instructions_name_every_prop_the_doc_crate_gives_a_primitive` | every prop in backticks in `PrimitiveKind::summary()` appears in INSTRUCTIONS (`tone_by` excluded, because `widgets.rs` requires the prompt to leave it out) | red |
| `the_listing_of_a_large_page_is_one_line_per_section_and_nothing_else` | a page with 121 sections gives 121 listing lines, all from that page, none below a section | green (probe) |

Red output from running this file alone, before the suite:
```
the ops page was named and is not listed: Instruction (speech-to-text): "make a reusable card for a loan and use it on the ops page"
  (listing shows only page:loans/section:list and page:loans/overlay:edit, matched by "loan")
the categories page was named and is not listed: Instruction (speech-to-text): "make a reusable card and use it on the category page"
the news page was not named and is listed: Instruction (speech-to-text): "make a new reusable card for a member and use it on the members page"
INSTRUCTIONS omit primitive props the doc crate names: ["badge: `tone`"]
test result: FAILED. 0 passed; 4 failed   (the probe was added afterwards and ran green alone)
```

**3. Suite:** `cargo test -p uilab-agent --no-fail-fast` exited 101.

| test binary | passed | failed |
|---|---|---|
| lib | 6 | 0 |
| uilab-plan | 1 | 0 |
| adversary_widgets_r2 | 1 | 4 |
| plan_adversary | 10 | 0 |
| propose | 16 | 0 |
| widgets | 9 | 0 |

- The "before" count of 42 is the same run with my file left out; 47 is the count from `-- --list`.
- `cargo clippy -p uilab-agent --all-targets -- -D warnings` passes, and `cargo fmt -p uilab-agent --check` exits 0.

**4. Findings** (tree cd1e73c; none of the matching code exists at base 8b337a1)

| # | file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| 1 | crates/uilab-agent/src/lib.rs:913 | `strip_suffix('s')` then `len() >= 3`: a page named `ops`, `ads` or `bus` is never listed, even when its name is said exactly | NEEDS-CHANGE | introduced | any document with such a page, targeted at `/` or at a page |
| 2 | crates/uilab-agent/src/lib.rs:912 | dropping one `s` is the only plural rule: `categories` or `branches` is missed when said in the singular | NEEDS-CHANGE | introduced | same path; speech-to-text often gives the singular |
| 3 | crates/uilab-agent/src/lib.rs:914 | `said.contains(stem)` matches inside words: `news` is listed whenever "new" is said, `art` whenever "chart" is | NEEDS-CHANGE | introduced | a website document with a news page (vision:website-harness); "add a new …" is a common utterance |
| 4 | crates/uilab-agent/src/lib.rs:701 | the badge is described as (`text`) only, while the doc crate describes it as "(`tone` or `tone_by`)"; the prompt never says how to colour a badge, although the unit's own fixture uses `tone_by` | NEEDS-CHANGE | introduced | every widget with a badge |

The fix for findings 1 to 3 would be to match whole words (a word, or the word plus `s`/`es`/`ies`) and to drop the three-letter minimum when the whole name is said. For finding 4, name `tone` on the badge.

**5. Attacked and could not break**
- **Leaks below a page:** a section, shell or component target gets no listing, and sections list their children one level deep only.
- **Size:** there is one line per section with no duplicates; for a page target the page YAML is already in the request.
- **Declared widgets:** name, summary, and params with type and required all appear. A constructor-map type is shown as compact JSON.
- **Request text:** at a section target it is unchanged apart from the new `Widgets…` line. The plan request gets the widgets line and no listing.
- **Plan outline:** it does contain `component:<name>`, as PLAN_INSTRUCTIONS claims.
- **Doc crate agreement:** the prompt names all 9 primitive kinds and the 14 composite kinds. The `component` and `node` layers, the ban on naming a widget like a composite kind, and the args rules all match the doc crate.
- **Item insert:** a `/` batch whose inner patch inserts an `item` is allowed by the schema, because the inner patch's `child.layer` includes `item`.
- **Titles vs names:** single-word titles match; underscores in a name match as spaces.

**6. Written outside the worktree:** none. Only the assigned build directory `$HOME/.cache/b10x-target/uilab-w2-agent-widgets` was used as build output. Before building, `/` had 21G free.

```findings
- file: crates/uilab-agent/src/lib.rs
  line: 913
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a page whose name is two letters plus s (ops, ads, bus) is never listed even when the operator says its exact name"
- file: crates/uilab-agent/src/lib.rs
  line: 912
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "stripping one trailing s is the only plural rule, so a categories page is missed when the utterance says category"
- file: crates/uilab-agent/src/lib.rs
  line: 914
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the stem is matched as a raw substring, so a news page is listed for every utterance containing new"
- file: crates/uilab-agent/src/lib.rs
  line: 701
  category: contract-drift
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the prompt describes badge as taking text only while the doc crate names tone as its prop, so the model is never told how to tone a badge"
```
