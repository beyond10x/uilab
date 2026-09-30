---
format: aep.planning-md/3
id: story:yaml-follows-selection
kind: story
status: active
title: YAML tab follows the selection, with colours
relations:
- decomposes: epic:ux-fidelity
- serves: vision:website-harness
scope:
- confidence: cited
  path: widget/src/components/YamlView.vue
- confidence: cited
  path: widget/src/lib/yamlblock.ts
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T12:08:57Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T12:08:57Z", actor: "human:timo", revision: 4}
---
## Outcome

The YAML tab highlights the selected node's block and scrolls to it, and colours keys, strings, numbers and comments.

## Found by

Screenshot `views-2-yaml.png`: plain monochrome text from line 1 whatever is selected; `widget/src/lib/yamlblock.ts` already finds a node's line range.

## Acceptance

- lib tests: the block range for a section, an item in a list, a widget body node.
- The selected block is highlighted and in view on switching to the tab and on selection change.
