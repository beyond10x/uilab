---
format: aep.planning-md/3
id: story:sidebar-layout
kind: story
status: active
title: Proposal card first; a compact sidebar header
relations:
- decomposes: epic:ux-fidelity
- serves: vision:website-harness
scope:
- confidence: cited
  path: widget/src/components/PresenceStrip.vue
- confidence: cited
  path: widget/src/components/ProposalCard.vue
- confidence: cited
  path: widget/src/components/SidebarPanel.vue
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T12:08:57Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T12:08:57Z", actor: "human:timo", revision: 4}
---
## Outcome

A waiting proposal's card, with Accept, Reject and its findings, is the first thing in the sidebar and visible without scrolling at 1440×900 and 1024×700. Undo sits with the card area. The sidebar header is one compact row: operator chips, connection dot, findings badge, help.

## Found by

Screenshot `proposal-proposal.png` (1440×900): the card starts at y≈790 below name field, presence, status, title, file path, findings, a fixed-height tree, selection line, mic box, phase bar, input, mode toggle, undo and activity; Accept and Reject are off screen. The "you" name field, the full file path and the bare "idle" status take three rows.

## Acceptance

- Order in SidebarPanel: header row, proposal card (when one waits), input with mic and mode, tree, activity.
- Name edit moves into the operator chip; the file path becomes the title's tooltip; "idle" is not shown, other phases are.
- Checked by screenshot at both sizes with a waiting proposal; the images are recorded as evidence.
