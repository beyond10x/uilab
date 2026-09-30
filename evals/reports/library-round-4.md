# library round 4

22 of 24 cases pass. Model: `claude-sonnet-5-5`. Document: `examples/library/library.ui.yaml`. Median 2425 ms.

| case | pass | ms | got | why |
|---|---|---|---|---|
| overdue-table | yes | 2141 | Insert page:loans -> page:loans/section:overdue |  |
| filler-words | yes | 3444 | Insert page:loans -> page:loans/section:due_soon |  |
| misheard-kind | yes | 2160 | Insert page:overview -> page:overview/section:recent_members |  |
| metric-count | yes | 2530 | Insert page:members -> page:members/section:member_count |  |
| chart | yes | 2425 | Insert page:overview -> page:overview/section:loans_per_week |  |
| filter-bar | yes | 1639 | Insert page:loans -> page:loans/section:filters |  |
| record | yes | 1901 | Insert page:members -> page:members/section:details |  |
| drawer-form | yes | 3436 | Insert page:members -> page:members/overlay:add_member |  |
| confirm-dialog | yes | 5800 | Batch page:loans -> page:loans |  |
| new-page-in-menu | yes | 2302 | Insert / -> page:reservations |  |
| new-hidden-page | yes | 2029 | Insert / -> page:opening_hours |  |
| nav-section | yes | 3303 | Insert nav -> nav/nav_section:reports |  |
| rename-title | yes | 1972 | Replace page:members/section:list -> page:members/section:list |  |
| add-column | no | 3278 | refused declined: This table already shows the member column, so nothing needs to change. | refused declined: This table already shows the member column, so nothing needs to change. |
| remove-section | yes | 1614 | Remove page:overview/section:recent -> page:overview/section:recent |  |
| board-widget | yes | 3242 | Insert page:overview -> page:overview/section:dashboard |  |
| item-in-collection | yes | 3218 | Insert page:members/section:list -> page:members/section:list/item:open_loans |  |
| shell-overlay | yes | 1907 | Insert shell:app -> shell:app/overlay:help |  |
| german | yes | 2492 | Insert page:members -> page:members/section:joined |  |
| row-action | yes | 4493 | Batch page:members/section:list -> page:members |  |
| thanks | yes | 1559 | refused declined: "Thank you." is not a request to change the UI, so nothing was changed. |  |
| dot | yes | 1993 | refused declined: The instruction was only punctuation, so it doesn't ask for any change to the UI. |  |
| question | yes | 2395 | refused declined: I can edit this Loans page: add or change sections like tables, filter bars, metrics, charts, forms and record details, plus drawers and dialogs, and rename or remove things. Just say what you want. |  |
| creative | no | 6803 | Batch page:overview -> page:overview | batch holds no node  |
