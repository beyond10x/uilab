<!--
  generated from uilab v1
  model digest 8bbec934f18fca6258713253bcfca181cac2a1249bf16684012986ad1b427455
  contract digest c789fcd30e3ffcc51487d00315741eca272a88a53a2be1e3be26a919b30c4bfb
  do not edit: regenerate with `ess synthesize`
-->
# Synthesis plan — uilab v1

Scope: `component-skeletons`, planned by `ess-synth`. Regenerate with `ess synthesize`.

75 capabilities: **68 generated**, **5 obligations**, **2 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

## Generated

| capability | source |
| --- | --- |
| domain type | `uilab.session.Document.State` |
| domain type | `uilab.session.DocumentId` |
| domain type | `uilab.session.NodePath` |
| domain type | `uilab.session.PatchBody` |
| domain type | `uilab.session.PatchOp` |
| domain type | `uilab.session.Proposal.State` |
| domain type | `uilab.session.ProposalId` |
| domain type | `uilab.wire.Changed` |
| domain type | `uilab.wire.ClientMessage` |
| domain type | `uilab.wire.Decide` |
| domain type | `uilab.wire.DocumentState` |
| domain type | `uilab.wire.Failed` |
| domain type | `uilab.wire.Finding` |
| domain type | `uilab.wire.Goal` |
| domain type | `uilab.wire.GoalState` |
| domain type | `uilab.wire.GoalStep` |
| domain type | `uilab.wire.Hello` |
| domain type | `uilab.wire.Mic` |
| domain type | `uilab.wire.MicState` |
| domain type | `uilab.wire.Moved` |
| domain type | `uilab.wire.Operator` |
| domain type | `uilab.wire.OperatorKind` |
| domain type | `uilab.wire.OutlineNode` |
| domain type | `uilab.wire.Presence` |
| domain type | `uilab.wire.ProposalShown` |
| domain type | `uilab.wire.ReadRows` |
| domain type | `uilab.wire.Refused` |
| domain type | `uilab.wire.Resync` |
| domain type | `uilab.wire.Rows` |
| domain type | `uilab.wire.Say` |
| domain type | `uilab.wire.Select` |
| domain type | `uilab.wire.ServerMessage` |
| domain type | `uilab.wire.Settings` |
| domain type | `uilab.wire.Severity` |
| domain type | `uilab.wire.StartGoal` |
| domain type | `uilab.wire.StepStatus` |
| domain type | `uilab.wire.StopGoal` |
| domain type | `uilab.wire.Thinking` |
| domain type | `uilab.wire.Transcript` |
| domain type | `uilab.wire.Workspace` |
| entity lifecycle | `uilab.session.Document` |
| entity lifecycle | `uilab.session.Proposal` |
| command contract | `uilab.session.AcceptProposal` |
| command contract | `uilab.session.OpenDocument` |
| command contract | `uilab.session.ProposePatch` |
| command contract | `uilab.session.RejectProposal` |
| command behaviour | `uilab.session.RejectProposal` |
| command contract | `uilab.session.SelectNode` |
| command contract | `uilab.session.UndoProposal` |
| event type | `uilab.session.DocumentOpened` |
| event type | `uilab.session.NodeSelected` |
| event type | `uilab.session.PatchProposed` |
| event type | `uilab.session.ProposalAccepted` |
| event type | `uilab.session.ProposalRejected` |
| event type | `uilab.session.ProposalUndone` |
| error type | `uilab.session.DocumentNotOpen` |
| error type | `uilab.session.DocumentUnreadable` |
| error type | `uilab.session.NodeNotFound` |
| error type | `uilab.session.PatchRefused` |
| error type | `uilab.session.ProposalStateConflict` |
| view type | `uilab.session.Documents` |
| view query | `uilab.session.Documents` |
| view type | `uilab.session.Pending` |
| view query | `uilab.session.Pending` |
| view type | `uilab.session.Proposals` |
| view query | `uilab.session.Proposals` |
| component port | `uilab-session` |
| component transport | `uilab-session` |

## Ports — yours to provide

What the specification fully determines is generated; what it cannot determine is an obligation. A generated command behaviour or view query reads and writes through the ports below, and they are yours to provide: synthesis generates each port's contract and never an implementation of one, so where instances live stays your decision.

| port | what it answers |
| --- | --- |
| storage | one per entity a generated behaviour or query reads or writes: the instance stored under an identity; storing, replacing and removing one; and every stored instance, in the order the store keeps them |
| context | where a generated behaviour asks it: the caller's attributes, every identity and value the specification says the implementation assigns, and whether each `external:` branch is taken |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |
| command behaviour | `uilab.session.AcceptProposal` | kept an obligation by the fields of error `uilab.session.PatchRefused`, which the specification gives no source, in `stale` | given `uilab.session.AcceptProposal` input, decide and enact exactly one outcome — `accepted` otherwise, takes `accept` of `uilab.session.Proposal`, emits `uilab.session.ProposalAccepted`; `stale` externally decided (the document changed since the proposal and the patch no longer applies), error `uilab.session.PatchRefused`; `wrong-state` from a state no declared move starts in, error `uilab.session.ProposalStateConflict`, and for an instance no record carries, without the error's fields |
| command behaviour | `uilab.session.OpenDocument` | kept an obligation by `creates:` leaving the required field `selected` of `uilab.session.Document` undetermined, in `opened` | given `uilab.session.OpenDocument` input, decide and enact exactly one outcome — `opened` otherwise, creates `uilab.session.Document`, emits `uilab.session.DocumentOpened`; `unreadable` externally decided (the file does not parse as an ess-ui/1 document), error `uilab.session.DocumentUnreadable` |
| command behaviour | `uilab.session.ProposePatch` | kept an obligation by the fields of error `uilab.session.PatchRefused`, which the specification gives no source, in `refused` | given `uilab.session.ProposePatch` input, decide and enact exactly one outcome — `proposed` otherwise, creates `uilab.session.Proposal`, emits `uilab.session.PatchProposed`; `refused` externally decided (the patched document fails a document check), error `uilab.session.PatchRefused` |
| command behaviour | `uilab.session.SelectNode` | kept an obligation by an unknown identity, which reaches no declared outcome (neither `unknown_instance:` nor `wrong_state:`) | given `uilab.session.SelectNode` input, decide and enact exactly one outcome — `selected` otherwise, updates `uilab.session.Document`, emits `uilab.session.NodeSelected`; `unknown-document` externally decided (no document with this id is open), error `uilab.session.DocumentNotOpen`; `not-found` externally decided (the document has no node at this path), error `uilab.session.NodeNotFound` |
| command behaviour | `uilab.session.UndoProposal` | kept an obligation by the fields of error `uilab.session.PatchRefused`, which the specification gives no source, in `stale` | given `uilab.session.UndoProposal` input, decide and enact exactly one outcome — `undone` otherwise, takes `undo` of `uilab.session.Proposal`, emits `uilab.session.ProposalUndone`; `stale` externally decided (a later change to the document would be lost by undoing this one), error `uilab.session.PatchRefused`; `wrong-state` from a state no declared move starts in, error `uilab.session.ProposalStateConflict`, and for an instance no record carries, without the error's fields |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
| actor grants | `uilab.session.Agent` | planning | may invoke `uilab.session.ProposePatch`; generated as data, not enforced: the grant is available as the declared actors and the qualified commands each may invoke, and enforcement stays with the caller, because a grant is checked against a caller identity, which types do not carry |
| actor grants | `uilab.session.Operator` | planning | may invoke `uilab.session.AcceptProposal`, `uilab.session.OpenDocument`, `uilab.session.RejectProposal`, `uilab.session.SelectNode`, `uilab.session.UndoProposal`; generated as data, not enforced: the grant is available as the declared actors and the qualified commands each may invoke, and enforcement stays with the caller, because a grant is checked against a caller identity, which types do not carry |
