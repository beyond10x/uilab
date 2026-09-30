# library round 1

18 of 20 cases pass. Model: `claude-sonnet-5-5`. Document: `examples/library/library.ui.yaml`. Median 2618 ms.

| case | pass | ms | got | why |
|---|---|---|---|---|
| overdue-table | yes | 3269 | Insert page:loans -> page:loans/section:overdue |  |
| filler-words | yes | 5163 | Insert page:loans -> page:loans/section:due_soon |  |
| misheard-kind | yes | 1731 | Insert page:overview -> page:overview/section:recent_members |  |
| metric-count | yes | 2926 | Insert page:members -> page:members/section:member_count |  |
| chart | yes | 1654 | Insert page:overview -> page:overview/section:loans_per_week |  |
| filter-bar | yes | 1970 | Insert page:loans -> page:loans/section:filters |  |
| record | yes | 1610 | Insert page:members -> page:members/section:details |  |
| drawer-form | yes | 2457 | Insert page:members -> page:members/overlay:add_member |  |
| confirm-dialog | yes | 3618 | Insert page:loans -> page:loans/overlay:cancel |  |
| new-page-in-menu | yes | 1726 | Insert / -> page:reservations |  |
| new-hidden-page | yes | 2165 | Insert / -> page:opening_hours |  |
| nav-section | yes | 3355 | Insert nav -> nav/nav_section:reports |  |
| rename-title | yes | 2004 | Replace page:members/section:list -> page:members/section:list |  |
| add-column | no | 4260 | Insert page:loans/section:list -> page:loans/section:list/item:member | op Insert, expected Replace; layer item, expected section; component record, expected collection |
| remove-section | yes | 1339 | Remove page:overview/section:recent -> page:overview/section:recent |  |
| board-widget | yes | 2797 | Insert page:overview -> page:overview/section:dashboard |  |
| item-in-collection | yes | 3063 | Insert page:members/section:list -> page:members/section:list/item:open_loans |  |
| shell-overlay | yes | 2042 | Insert shell:app -> shell:app/overlay:help |  |
| german | yes | 2618 | Insert page:members -> page:members/section:member_dates |  |
| row-action | no | 10977 | refused opens_resolves: page:members/section:list: opens `edit_member`, which neither the page nor its shell declares | refused opens_resolves: page:members/section:list: opens `edit_member`, which neither the page nor its shell declares |
