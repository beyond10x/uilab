# library round 5

23 of 27 cases pass. Model: `claude-sonnet-5-5`. Document: `examples/library/library.ui.yaml`. Median 3329 ms.

| case | pass | ms | got | why |
|---|---|---|---|---|
| overdue-table | yes | 3242 | Insert page:loans -> page:loans/section:overdue |  |
| filler-words | yes | 2988 | Insert page:loans -> page:loans/section:due_soon |  |
| misheard-kind | yes | 2277 | Insert page:overview -> page:overview/section:recent_members |  |
| metric-count | yes | 3329 | Insert page:members -> page:members/section:member_count |  |
| chart | yes | 2894 | Insert page:overview -> page:overview/section:loans_per_week |  |
| filter-bar | yes | 2275 | Insert page:loans -> page:loans/section:filters |  |
| record | no | 8131 | Batch page:members -> page:members | batch holds no section record |
| drawer-form | yes | 8296 | Batch page:members -> page:members |  |
| confirm-dialog | yes | 4079 | Batch page:loans -> page:loans |  |
| new-page-in-menu | yes | 2865 | Insert / -> page:reservations |  |
| new-hidden-page | yes | 9262 | Insert / -> page:opening_hours |  |
| nav-section | yes | 1622 | Insert nav -> nav/nav_section:reports |  |
| relabel-column | no | 16670 | Batch page:members/section:list -> page:members/section:list | batch holds no section collection |
| add-column | no | 14283 | refused agent: the agent run ended without an answer: {"kind":"max-turns","limit":4} | refused agent: the agent run ended without an answer: {"kind":"max-turns","limit":4} |
| remove-section | yes | 2090 | Remove page:overview/section:recent -> page:overview/section:recent |  |
| board-widget | yes | 9030 | Insert page:overview -> page:overview/section:dashboard |  |
| item-in-collection | yes | 3396 | Insert page:members/section:list -> page:members/section:list/item:open_loans |  |
| shell-overlay | yes | 2224 | Insert shell:app -> shell:app/overlay:help |  |
| german | yes | 4246 | Insert page:members -> page:members/section:member_names |  |
| row-action | yes | 6060 | Batch page:members/section:list -> page:members |  |
| thanks | yes | 1772 | refused declined: "Thank you." is not a request to change the UI, so nothing was changed. |  |
| dot | yes | 2210 | refused declined: That was only punctuation, not a request, so nothing changed. |  |
| question | yes | 2640 | refused declined: I can edit this Loans page. For example, I can add a filter bar, metrics from loans.Summary, a chart, or a new table. I can also add columns or row actions, add or change drawers, and create reusable widgets. |  |
| creative | yes | 5899 | Batch page:overview -> page:overview |  |
| retarget-new-page | yes | 4064 | moved to /; Insert / -> page:overdue |  |
| retarget-navigate | yes | 1715 | moved to page:members (navigate only) |  |
| retarget-none | no | 15907 | Batch page:loans/section:list -> page:loans/section:list | batch holds no section collection |

## ESS check of the admitted proposals

22 proposals were admitted (journal: 22 `proposed`, 22 `shown`). Each patch was replayed onto
`examples/library/library.ui.yaml` and the result checked with `ess ui check` (ess 0.48.0, no
`--model`): **0 errors** in 22 documents. 4 documents carry one warning each,
`unbound_placeholder` (metric-count `members.Count`, chart `loans.PerWeek`, new-page-in-menu
`loans.Reservations`, board-widget `overview.Dashboard`): placeholder reads the prompt asks for
when no view holds the data.

## Runs

| run | binary | agent runs in the journal | result |
|---|---|---|---|
| 1 | base `b6a98c1` | 27, all refused before the model ran | 0 of 27: every request answered 400, `input_schema does not support oneOf, allOf, or anyOf at the top level` |
| 2 | `1b721cf` (no top-level combinator; prompt: a patch's node carries no `name`) | 28, 36 model turns | 23 of 27; ESS: 23 admitted, 0 errors |
| 3 | `99fd23a` (run 2 plus main v0.1.2), through a forwarder that logged every request body | 28, 40 model requests | 23 of 27, the table above |

Run 1: the patch schema carried its per-operation requirements as a top-level `allOf`, which the
Messages API refuses. Fixed in `crates/uilab-doc/src/schema.rs` (one `if`/`then`/`else` chain),
pinned by `no_answer_schema_sent_has_a_combinator_at_its_top_level`.

## Failures

| case | reason | fix |
|---|---|---|
| add-column, relabel-column, retarget-none | The request log shows every `replace` at the collection sent with `node` as a JSON **string** holding the node, not an object, in every turn. The schema refused each one as `{"required":["name"]} is not allowed for "<the string>"`, a message that does not say what is wrong. add-column ran out of turns (`max-turns`, 4). relabel-column and retarget-none gave up on turn 4 and answered with a `batch` of the same replace twice, whose entry `node` is typed `object`. The node inside the string is correct each time, and as an object it passes the schema and ESS. Cause: the top-level `node` parameter was a bare `$ref` with no `type`, and a top-level parameter with no type reaches the model as a string. `child`, typed `object`, never had this problem. | Schema (after run 3): `node` declares `"type": "object"`, and a stringified node is now refused as the wrong type. Pinned by `every_top_level_property_sent_names_its_type`, which covers every top-level property of every schema sent at every node. Not run against the model (no fourth run). The run-2 prompt change (`name`) did not address this cause. |
| record | Asked for "a details card for the selected member", the agent added a drawer holding a `record` and a row action that opens it, not a `record` section. ESS admits it with no error. The case expects a section. | None: a reasonable reading of "the selected member" on a list page. Recorded as the model's choice. |
