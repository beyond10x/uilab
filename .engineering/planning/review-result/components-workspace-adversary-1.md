---
format: aep.planning-md/3
id: review-result:components-workspace-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:components-workspace
relations:
- reviews: story:components-workspace
revision: 1
---
unit: story:components-workspace, head 8900b3f plus adversary commit b347173 (tests only)
verdict: NEEDS-CHANGE
cases: executed 153→156 (widget 94→95, uilab-app 59→61), red 3
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w3/components-workspace/adversary/ (5 logs)
needs-coordinator: yes. Both fixes need use sites the outline does not carry (crates/uilab-doc/src/outline.rs, outside the unit's surface), or the server's `widget_uses`.

**Cases**

| file | asserts | red output |
|---|---|---|
| `widget/src/lib/components.adversary.test.ts:19` | a widget `badge` is used once, at `section:latest`; primitive badge nodes are not uses | actual `[component:badge/node:tag, component:loan_card/node:due, page:overview/section:latest]` |
| `crates/uilab-app/tests/adversary_components_workspace.rs:90` | the outline's page props carry the header instance docs list | `page:overview props = {"shell":"app"}` |
| `crates/uilab-app/tests/adversary_components_workspace.rs:110` | a walk of the outline by kind finds the one use docs list | left `["component:loan_card/node:due", "component:badge/node:tag", "page:overview/section:latest"]` |

Suite: `pnpm check` tests 95, fail 1, EXIT=1; `cargo test -p uilab-app` integration file 2 failed, EXIT=101; clippy and fmt EXIT=0.

Attacked and held: no `v-html` anywhere in the preview path; sample args for null/false/0 defaults and list/enum/wrapper/named types; `args.` substitution (nested, missing, `myargs.` untouched, no prototype leak); search; nested use sites; selection paths; key 4 ignored in the search box; the empty-state wording.

```findings
- file: widget/src/lib/components.ts
  line: 188
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "useSites matches outline nodes by kind alone, so a widget named like a primitive (badge, button, link) counts every primitive of that kind, its own body included, as a use site, where /api/docs.md lists one"
- file: widget/src/lib/components.ts
  line: 186
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A widget used only in a page header (header.metrics) or in page_kinds shows as not used in the Components tab because the outline carries no page header, while /api/docs.md lists the use and a removal is refused"
```
