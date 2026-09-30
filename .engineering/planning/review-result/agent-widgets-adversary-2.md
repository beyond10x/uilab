---
format: aep.planning-md/3
id: review-result:agent-widgets-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:agent-widgets
relations:
- reviews: story:agent-widgets
revision: 1
---
unit: story:agent-widgets pass 2, unit/agent-widgets 9339b57 (my cases committed on top as f2d69fb)
verdict: CONFIRMED
cases: executed 48→51, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: finding 1 is the 4-letter floor you accepted, seen from the other side: decide whether its red case stays

The correction holds everywhere I pushed on it, but 2 of my 3 new cases are red. One is the other half of the 4-letter floor you accepted; the other is a comment that says the target page comes first when it does not.

**1. Diff stat (9339b57..HEAD)**
```
 crates/uilab-agent/tests/adversary_widgets_p2.rs | 157 +++++++++++++++++++++++
 1 file changed, 157 insertions(+)
```
Only one test file changed. The commit's author and committer are both `b10x-bot[bot]`.

**2. Cases added** (`/home/timo/.local/state/worktree/trees/b10x/uilab/uilab-w2-agent-widgets/crates/uilab-agent/tests/adversary_widgets_p2.rs`)

| case | asserts | now |
|---|---|---|
| `a_three_letter_singular_names_its_plural_page` | page `logs` is listed when the utterance says "log page" | red |
| `the_target_page_is_listed_first` | at target `page:members`, the first listing line is from `page:members` | red |
| `whole_word_matching_holds_at_the_edges` | 7 utterances (list below) each list exactly the expected pages | green |

The green case's utterances: "Overdue Loan, as a card" lists `loans` and `late` (title "Overdue loans"); a bare "overdue" lists nothing; "Q3 REPORT" lists `q3_report`; "member's" lists `members`; "members" alone at the edge lists `members`; "city" lists `cities`; "news." lists `news`, and "new" does not.

Red output from running this file alone, before the suite:
```
the logs page was named as `log` and is not listed: Instruction (speech-to-text): "use the loan card on the log page"
the target page is not first: [
    "- page:loans/section:list: collection reads loans.All; columns title, member, due, state (as tag); children none",
    "- page:loans/overlay:edit: drawer form",
    "- page:members/section:list: collection reads members.All; columns name, joined, loans, standing (as tag); children none",
]
test result: FAILED. 1 passed; 2 failed
```

**3. Suite:** `cargo test -p uilab-agent --no-fail-fast` exits 101.

| binary | passed | failed |
|---|---|---|
| lib | 6 | 0 |
| uilab-plan | 1 | 0 |
| adversary_widgets_p2 | 1 | 2 |
| adversary_widgets_r2 | 5 | 0 |
| plan_adversary | 10 | 0 |
| propose | 16 | 0 |
| widgets | 10 | 0 |

- The before count of 48 is the same run with my new file left out.
- All 4 cases that were red last round now pass. That includes `ops`, which passes because an exact word has no minimum length.
- `cargo clippy -p uilab-agent --all-targets -- -D warnings` passes and `cargo fmt -p uilab-agent --check` exits 0.

**4. Findings** (covering 9339b57)

| # | file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| 1 | crates/uilab-agent/src/lib.rs:917 | the 4-letter floor applies both ways, so a 3-letter singular no longer names its plural page (`log`→`logs`, `job`→`jobs`, `tag`→`tags`, `doc`→`docs`); cd1e73c listed these | CONFIRMED | introduced | any page with a 3-letter plural name, targeted at `/` or at a page |
| 2 | crates/uilab-agent/src/lib.rs:927 | the doc comment says "the target page, then every other page", but the code filters `doc.pages` in document order | NEEDS-CHANGE | introduced | any page target that comes after another page the utterance names; members after loans in the library example |

Possible fixes, which I have not applied:
- **Finding 1:** drop the floor on the singular-to-plural direction and keep it only for `news`-style words. That needs a rule you choose, because no simple letter count separates `new`/`news` from `log`/`logs`. If you keep the floor as it is, finding 1 is the accepted cost and its case should be removed.
- **Finding 2:** put `own` first when building the list, or change the comment.

Whether the whole-word split changed any earlier request text: all 36 cases that predate this round, and the 5 from last round, pass unchanged.

**5. Attacked and could not break:**
- Case, digits, trailing punctuation, possessive apostrophes and utterance edges.
- Multi-word titles, both as a consecutive run of words and with a plural in them.
- `ies`↔`y`, and the floor keeping "new" from naming `news`.
- The `es` rule does cause false matches (a page named `plan` is listed for "planes", one named `stat` for "states"). I wrote no case for this because I could not show a realistic document that reaches it.
- A title with an apostrophe spoken without it (title "Today's loans", said "todays loans") is missed. I wrote no case because whisper normally keeps the apostrophe.

**6. Written outside the worktree:** none. Build output went only to the assigned `$HOME/.cache/b10x-target/uilab-w2-agent-widgets`.

```findings
- file: crates/uilab-agent/src/lib.rs
  line: 917
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the four-letter plural floor also stops a three-letter singular from naming its plural page, so log no longer names logs as it did at cd1e73c"
- file: crates/uilab-agent/src/lib.rs
  line: 927
  category: contract-drift
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "named_pages documents the target page first but returns pages in document order, so a page target is listed after earlier named pages"
```
