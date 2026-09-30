# library round 2

18 of 20 cases pass. Model: `claude-sonnet-5-5`. Document: `examples/library/library.ui.yaml`. Median 2978 ms.

| case | pass | ms | got | why |
|---|---|---|---|---|
| overdue-table | yes | 3514 | Insert page:loans -> page:loans/section:overdue |  |
| filler-words | yes | 3741 | Insert page:loans -> page:loans/section:due_soon |  |
| misheard-kind | yes | 2236 | Insert page:overview -> page:overview/section:recent_members |  |
| metric-count | yes | 2545 | Insert page:members -> page:members/section:member_count |  |
| chart | yes | 1989 | Insert page:overview -> page:overview/section:loans_per_week |  |
| filter-bar | yes | 2038 | Insert page:loans -> page:loans/section:filters |  |
| record | yes | 1850 | Insert page:members -> page:members/section:details |  |
| drawer-form | yes | 3159 | Insert page:members -> page:members/overlay:add_member |  |
| confirm-dialog | no | 5650 | Batch page:loans -> page:loans | op Batch, expected Insert; layer page, expected overlay; component none, expected confirm |
| new-page-in-menu | yes | 2053 | Insert / -> page:reservations |  |
| new-hidden-page | yes | 4795 | Insert / -> page:opening_hours |  |
| nav-section | yes | 2995 | Insert nav -> nav/nav_section:reports |  |
| rename-title | yes | 1645 | Replace page:members/section:list -> page:members/section:list |  |
| add-column | yes | 4881 | Replace page:loans/section:list -> page:loans/section:list |  |
| remove-section | yes | 1611 | Remove page:overview/section:recent -> page:overview/section:recent |  |
| board-widget | yes | 2978 | Insert page:overview -> page:overview/section:dashboard |  |
| item-in-collection | yes | 4312 | Insert page:members/section:list -> page:members/section:list/item:open_loans |  |
| shell-overlay | yes | 2023 | Insert shell:app -> shell:app/overlay:help |  |
| german | yes | 2432 | Insert page:members -> page:members/section:member_table |  |
| row-action | no | 4314 | Batch page:members/section:list -> page:members | op Batch, expected Replace; layer page, expected section; component none, expected collection |
