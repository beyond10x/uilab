# library round 5

26 of 27 cases pass. Model: `claude-sonnet-5-5`. Document: `examples/library/library.ui.yaml`. Median 2827 ms.

| case | pass | ms | got | why |
|---|---|---|---|---|
| overdue-table | yes | 2827 | Insert page:loans -> page:loans/section:overdue |  |
| filler-words | yes | 3747 | Insert page:loans -> page:loans/section:due_soon |  |
| misheard-kind | yes | 2363 | Insert page:overview -> page:overview/section:recent_members |  |
| metric-count | yes | 4445 | Insert page:members -> page:members/section:member_count |  |
| chart | yes | 4300 | Insert page:overview -> page:overview/section:loans_per_week |  |
| filter-bar | yes | 2974 | Insert page:loans -> page:loans/section:filters |  |
| record | no | 5614 | Batch page:members -> page:members | batch holds no section record |
| drawer-form | yes | 7546 | Batch page:members -> page:members |  |
| confirm-dialog | yes | 5785 | Batch page:loans -> page:loans |  |
| new-page-in-menu | yes | 3135 | Insert / -> page:reservations |  |
| new-hidden-page | yes | 7431 | Insert / -> page:opening_hours |  |
| nav-section | yes | 1645 | Insert nav -> nav/nav_section:reports |  |
| relabel-column | yes | 1898 | Replace page:members/section:list -> page:members/section:list |  |
| add-column | yes | 1881 | Replace page:loans/section:list -> page:loans/section:list |  |
| remove-section | yes | 1366 | Remove page:overview/section:recent -> page:overview/section:recent |  |
| board-widget | yes | 8330 | Insert page:overview -> page:overview/section:overdue_board |  |
| item-in-collection | yes | 2425 | Insert page:members/section:list -> page:members/section:list/item:open_loans |  |
| shell-overlay | yes | 1834 | Insert shell:app -> shell:app/overlay:help |  |
| german | yes | 2653 | Insert page:members -> page:members/section:member_joined |  |
| row-action | yes | 5617 | Batch page:members/section:list -> page:members |  |
| thanks | yes | 1626 | refused declined: "Thank you." is not a request to change the UI, so nothing was changed. |  |
| dot | yes | 1724 | refused declined: The instruction was just a period, which isn't a request to change the UI. |  |
| question | yes | 2092 | refused declined: I can add or change sections (tables, filters, metrics, charts, forms), overlays, widgets and pages on this loans page. Tell me what you want, for example "add a filter bar" or "show overdue loans as a chart". |  |
| creative | yes | 5754 | Batch page:overview -> page:overview |  |
| retarget-new-page | yes | 3738 | moved to /; Insert / -> page:overdue |  |
| retarget-navigate | yes | 1768 | moved to page:members (navigate only) |  |
| retarget-none | yes | 2079 | Replace page:loans/section:list -> page:loans/section:list |  |

All five demo-gap cases pass: add-column, creative, retarget-new-page, retarget-navigate,
retarget-none.

## ESS check of the admitted proposals

23 proposals were admitted (journal: 23 `proposed`, 23 `shown`). Each patch was replayed onto
`examples/library/library.ui.yaml` and the result checked with `ess ui check` (ess 0.48.0, no
`--model`): **0 errors** in 23 documents. 3 documents carry one warning each,
`unbound_placeholder` (metric-count `members.Count`, chart `loans.PerWeek`, new-page-in-menu
`loans.Reservations`): placeholder reads the prompt asks for when no view holds the data.

## Runs

| run | binary | agent runs in the journal | result |
|---|---|---|---|
| 1 | base `b6a98c1` | 27, all refused before the model ran | 0 of 27: every request answered 400, `input_schema does not support oneOf, allOf, or anyOf at the top level` |
| 2 | `1b721cf` (no top-level combinator; prompt: a patch's node carries no `name`) | 28, 36 model turns | 23 of 27; ESS: 23 admitted, 0 errors |
| 3 | `99fd23a` (run 2 plus main v0.1.2), through a forwarder that logged every request body | 28, 40 model requests | 23 of 27; ESS: 22 admitted, 0 errors |
| 4 | `cbaaee6` (`node` typed `object`), same forwarder | 28, 31 model requests | 26 of 27, the table above |

- Run 1: the patch schema carried its per-operation requirements as a top-level `allOf`, which
  the Messages API refuses. Fixed in `crates/uilab-doc/src/schema.rs` (one `if`/`then`/`else`
  chain), pinned by `no_answer_schema_sent_has_a_combinator_at_its_top_level`.
- Runs 2 and 3: add-column, relabel-column and retarget-none failed. The request log of run 3
  shows the model sent every `replace` at the collection with `node` as a JSON string, because
  the top-level `node` parameter named no `type` (a bare `$ref`). Each was refused with
  `{"required":["name"]} is not allowed for "<the string>"`, until the run ran out of turns or
  fell back to a `batch` of the same replace twice. Fixed by typing `node` as `object`, pinned by
  `every_top_level_property_sent_names_its_type`. In run 4 all three answer a `replace` in one
  turn (1881–2079 ms, down from 14–17 s).
- Run 4: three cases took a second attempt after a document refusal they then corrected
  (drawer-form `document_loads`: `sections` written inside a collection; new-hidden-page
  `document_loads`: a `form` without `does`; board-widget `name_unique`: the inherited `board`
  section).

## Failures

| case | reason | fix |
|---|---|---|
| record | Asked for "a details card for the selected member", the agent inserted a drawer overlay holding a `record` and replaced the members list to add the row action that opens it, not a `record` section. ESS admits it with no error. The case expects a section. The same answer came back in runs 2, 3 and 4. | None: a reasonable reading of "the selected member" on a list page. Recorded as the model's choice. |
