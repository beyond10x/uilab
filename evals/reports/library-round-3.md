# library round 3

20 of 20 cases pass. Model: `claude-sonnet-5-5`. Document: `examples/library/library.ui.yaml`. Median 2422 ms.

| case | pass | ms | got | why |
|---|---|---|---|---|
| overdue-table | yes | 2326 | Insert page:loans -> page:loans/section:overdue |  |
| filler-words | yes | 3406 | Insert page:loans -> page:loans/section:due_soon |  |
| misheard-kind | yes | 2422 | Insert page:overview -> page:overview/section:recent_members |  |
| metric-count | yes | 2951 | Insert page:members -> page:members/section:member_count |  |
| chart | yes | 1896 | Insert page:overview -> page:overview/section:loans_per_week |  |
| filter-bar | yes | 2019 | Insert page:loans -> page:loans/section:filters |  |
| record | yes | 1830 | Insert page:members -> page:members/section:details |  |
| drawer-form | yes | 2587 | Insert page:members -> page:members/overlay:add_member |  |
| confirm-dialog | yes | 5159 | Batch page:loans -> page:loans |  |
| new-page-in-menu | yes | 1965 | Insert / -> page:reservations |  |
| new-hidden-page | yes | 1864 | Insert / -> page:opening_hours |  |
| nav-section | yes | 2693 | Insert nav -> nav/nav_section:reports |  |
| rename-title | yes | 1692 | Replace page:members/section:list -> page:members/section:list |  |
| add-column | yes | 5676 | Replace page:loans/section:list -> page:loans/section:list |  |
| remove-section | yes | 1600 | Remove page:overview/section:recent -> page:overview/section:recent |  |
| board-widget | yes | 2117 | Insert page:overview -> page:overview/section:dashboard |  |
| item-in-collection | yes | 3299 | Insert page:members/section:list -> page:members/section:list/item:open_loans |  |
| shell-overlay | yes | 1492 | Insert shell:app -> shell:app/overlay:help |  |
| german | yes | 2536 | Insert page:members -> page:members/section:members |  |
| row-action | yes | 4674 | Batch page:members/section:list -> page:members |  |
