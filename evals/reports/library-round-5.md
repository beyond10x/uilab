# library round 5

23 of 27 cases pass. Model: `claude-sonnet-5-5`. Document: `examples/library/library.ui.yaml`. Median 3163 ms.

| case | pass | ms | got | why |
|---|---|---|---|---|
| overdue-table | yes | 2388 | Insert page:loans -> page:loans/section:overdue |  |
| filler-words | yes | 3163 | Insert page:loans -> page:loans/section:due_soon |  |
| misheard-kind | yes | 2133 | Insert page:overview -> page:overview/section:recent_members |  |
| metric-count | yes | 2981 | Insert page:members -> page:members/section:member_count |  |
| chart | yes | 2631 | Insert page:overview -> page:overview/section:loans_per_week |  |
| filter-bar | yes | 2705 | Insert page:loans -> page:loans/section:filters |  |
| record | no | 6914 | Batch page:members -> page:members | batch holds no section record |
| drawer-form | yes | 10097 | Batch page:members -> page:members |  |
| confirm-dialog | yes | 5762 | Batch page:loans -> page:loans |  |
| new-page-in-menu | yes | 3106 | Insert / -> page:reservations |  |
| new-hidden-page | yes | 8835 | Insert / -> page:opening_hours |  |
| nav-section | yes | 3139 | Insert nav -> nav/nav_section:reports |  |
| relabel-column | no | 15348 | Batch page:members/section:list -> page:members/section:list | batch holds no section collection |
| add-column | no | 29816 | Batch page:loans/section:list -> page:loans/section:list | batch holds no section collection |
| remove-section | yes | 2034 | Remove page:overview/section:recent -> page:overview/section:recent |  |
| board-widget | yes | 5976 | Insert page:overview -> page:overview/section:overdue_board |  |
| item-in-collection | yes | 3437 | Insert page:members/section:list -> page:members/section:list/item:open_loans |  |
| shell-overlay | yes | 1846 | Insert shell:app -> shell:app/overlay:help |  |
| german | yes | 3340 | Insert page:members -> page:members/section:member_joined |  |
| row-action | yes | 4945 | Batch page:members/section:list -> page:members |  |
| thanks | yes | 1663 | refused declined: "Thank you." is not a request to change the UI, so nothing was changed. |  |
| dot | yes | 1765 | refused declined: The instruction was only punctuation, not a request to change the UI. |  |
| question | yes | 1948 | refused declined: I can edit this Loans page: add sections like filters, metrics or charts, add or change columns and row actions, and add drawers or dialogs. Tell me what you'd like. |  |
| creative | yes | 7482 | Batch page:overview -> page:overview |  |
| retarget-new-page | yes | 4149 | moved to /; Insert / -> page:overdue |  |
| retarget-navigate | yes | 1759 | moved to page:members (navigate only) |  |
| retarget-none | no | 16439 | Batch page:loans/section:list -> page:loans/section:list | batch holds no section collection |

## ESS check of the admitted proposals

23 proposals were admitted (journal: 23 `proposed`, 23 `shown`). Each patch was replayed onto
`examples/library/library.ui.yaml` and the result checked with `ess ui check` (ess 0.48.0, no
`--model`): **0 errors** in 23 documents. 3 documents carry one warning each,
`unbound_placeholder` (metric-count `members.Count`, chart `loans.PerWeek`, new-page-in-menu
`reservations.All`): placeholder reads the prompt asks for when no view holds the data.

## Runs

| run | binary | cases | agent runs in the journal | result |
|---|---|---|---|---|
| 1 | base `b6a98c1` | 27 | 27, all refused before the model ran | 0 of 27: every request answered 400, `input_schema does not support oneOf, allOf, or anyOf at the top level` |
| 2 | with the schema fix | 27 | 28 (retarget-new-page is a move and a proposal), 36 model turns | 23 of 27, the table above |

Run 1: the patch schema carried its per-operation requirements as a top-level `allOf`, which the
Messages API refuses. Fixed in `crates/uilab-doc/src/schema.rs` (one `if`/`then`/`else` chain
instead), pinned by `no_answer_schema_sent_has_a_combinator_at_its_top_level`
(`crates/uilab-agent/tests/propose.rs`), which checks the schema of every run the agent makes at
every node of the library.

## Failures

| case | reason | fix |
|---|---|---|
| add-column, relabel-column, retarget-none | The answer took 4 turns (the most one attempt has) and came back as a `batch` holding the same `replace` of the collection twice. The replaced node is right (the `id` column added, `joined` labelled "Member since", `sort: {by: due}`), and as a plain `replace` the schema accepts it and ESS admits it (`the_replaces_the_model_wrote_at_a_collection_pass_the_schema`). The first three answers were refused inside the loop, which the journal does not record. Most likely cause: the prompt taught a section as `{name: …, component: …}`, while a patch's section node may not carry `name`, and the schema refuses one that does with `{"required":["name"]} is not allowed`. The model does write `name` into a section it replaces: the `record` batch below does. | Prompt (`crates/uilab-agent/src/lib.rs`): a section is taught without `name`, and the prompt now says a node in a patch never carries its own name. Pinned by `the_prompt_teaches_a_section_node_without_its_name`. Not yet tested against the model: both runs are spent. |
| record | Asked for "a details card for the selected member", the agent added a drawer holding a `record` and a row action that opens it, not a `record` section. ESS admits it with no error. The case expects a section. | None: a reasonable reading of "the selected member" on a list page. Recorded as the model's choice. |
