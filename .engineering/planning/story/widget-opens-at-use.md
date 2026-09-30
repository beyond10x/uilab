---
format: aep.planning-md/3
id: story:widget-opens-at-use
kind: story
status: active
title: Check a widget body's opens at each use site
relations:
- decomposes: epic:component-library
- serves: vision:website-harness
scope:
- confidence: cited
  path: crates/uilab-doc
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T05:45:22Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T05:45:22Z", actor: "human:timo", revision: 4}
---
## Outcome

A widget body is checked at each use site as ess WidgetInstance expansion says: an `opens` in a body node (composite or primitive) must resolve against the overlays of the page (and shell) where the instance sits, with findings at `<instance path>/body/<node>`; `admit` refuses an instance that brings an unresolved `opens` onto a page.

## Found by

review-result:item-list-adversary-2 finding 1 (pre-existing at 8b337a1; `crates/uilab-doc/src/check.rs:687`); case `an_opens_in_a_widget_body_is_checked_at_each_use_site` in `crates/uilab-doc/tests/adversary_item_list_p2.rs`, pinned to today.

## Acceptance

Flip that pinned case: the instance on `members` (no `extend` overlay) is reported or refused; on `overview` (has it) it is clean.
