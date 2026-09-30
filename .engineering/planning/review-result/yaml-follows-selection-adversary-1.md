---
format: aep.planning-md/3
id: review-result:yaml-follows-selection-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:yaml-follows-selection
relations:
- reviews: story:yaml-follows-selection
revision: 1
---
unit: story:yaml-follows-selection, branch unit/yaml-follows-selection; findings cover 807e5bf (adversary commit c6d1b2f)
verdict: NEEDS-CHANGE
cases: executed 110→122, red 3
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w5/yaml-follows-selection/adversary
needs-coordinator: none

Cases (`widget/src/lib/yamltokens.adversary.test.ts`, 12): list-entry literal and folded block scalars (red), `1_000` and `12__` as strings (red); numbers, keys with colons and quotes, URLs, `#` in strings, nested flow, empty values, unicode, CRLF and lone CR, every example file spells its lines, linear time on 400k-char lines (green).

Attacked and held: tokens concatenate to the line on every example and the eval document (386 lines, 0 mismatches); selection from the UI tab then switching; 61 of 62 tree nodes placed in view (the root has no block); a node without a block; a waiting proposal shows the document.

```findings
- file: widget/src/lib/yamltokens.ts
  line: 170
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a block scalar that is a list entry (`- |-`, which serde_yaml writes for every multi-line string in a sequence) is never recognised, so its content lines are coloured as keys and comments"
- file: widget/src/lib/yamltokens.ts
  line: 17
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "NUMBER accepts underscores, so plain `1_000` and `12__` are coloured as numbers although serde_yaml reads them as strings"
```

Coordinator correction (tokenizer only): a list-entry block scalar is recognised before the key scan; NUMBER drops `_`. Verified: `pnpm check` 122/122 (the 3 red cases green), `pnpm build` ok. No second pass: a two-branch tokenizer fix held by the adversary's property cases.
