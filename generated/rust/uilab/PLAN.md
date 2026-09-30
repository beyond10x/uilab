<!--
  generated from uilab v1
  model digest 1a75f0421d837e2ee884066ba29e42b368e91a0d2fb46630399bf48ac98fa437
  contract digest 1419f72b14d91b0b3ee89f6e567db8160920e8728d07d1772d1e9b0cc5848690
  do not edit: regenerate with `ess synthesize`
-->
# Synthesis plan — uilab v1

Scope: `component-skeletons`, planned by `ess-synth`. Regenerate with `ess synthesize`.

66 capabilities: **55 generated**, **9 obligations**, **2 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

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
| domain type | `uilab.wire.Hello` |
| domain type | `uilab.wire.Mic` |
| domain type | `uilab.wire.MicState` |
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
| domain type | `uilab.wire.Severity` |
| domain type | `uilab.wire.Thinking` |
| domain type | `uilab.wire.Transcript` |
| entity lifecycle | `uilab.session.Document` |
| entity lifecycle | `uilab.session.Proposal` |
| command contract | `uilab.session.AcceptProposal` |
| command contract | `uilab.session.OpenDocument` |
| command contract | `uilab.session.ProposePatch` |
| command contract | `uilab.session.RejectProposal` |
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
| view type | `uilab.session.Pending` |
| view type | `uilab.session.Proposals` |
| component port | `uilab-session` |
| component transport | `uilab-session` |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |
| command behaviour | `uilab.session.AcceptProposal` | decided outside the system: the document changed since the proposal and the patch no longer applies | given `uilab.session.AcceptProposal` input, decide and enact exactly one outcome — `accepted` otherwise, takes `accept` of `uilab.session.Proposal`, emits `uilab.session.ProposalAccepted`; `stale` externally decided (the document changed since the proposal and the patch no longer applies), error `uilab.session.PatchRefused`; `wrong-state` from a state no declared move starts in, error `uilab.session.ProposalStateConflict`, and for an instance no record carries, without the error's fields |
| command behaviour | `uilab.session.OpenDocument` | decided outside the system: the file does not parse as a ui-spec/1 document | given `uilab.session.OpenDocument` input, decide and enact exactly one outcome — `opened` otherwise, creates `uilab.session.Document`, emits `uilab.session.DocumentOpened`; `unreadable` externally decided (the file does not parse as a ui-spec/1 document), error `uilab.session.DocumentUnreadable` |
| command behaviour | `uilab.session.ProposePatch` | decided outside the system: the patched document fails a document check | given `uilab.session.ProposePatch` input, decide and enact exactly one outcome — `proposed` otherwise, creates `uilab.session.Proposal`, emits `uilab.session.PatchProposed`; `refused` externally decided (the patched document fails a document check), error `uilab.session.PatchRefused` |
| command behaviour | `uilab.session.RejectProposal` | the contract is declared; the algorithm is not | given `uilab.session.RejectProposal` input, decide and enact exactly one outcome — `rejected` otherwise, takes `reject` of `uilab.session.Proposal`, emits `uilab.session.ProposalRejected`; `wrong-state` from a state no declared move starts in, error `uilab.session.ProposalStateConflict`, and for an instance no record carries, without the error's fields |
| command behaviour | `uilab.session.SelectNode` | decided outside the system: no document with this id is open | given `uilab.session.SelectNode` input, decide and enact exactly one outcome — `selected` otherwise, updates `uilab.session.Document`, emits `uilab.session.NodeSelected`; `unknown-document` externally decided (no document with this id is open), error `uilab.session.DocumentNotOpen`; `not-found` externally decided (the document has no node at this path), error `uilab.session.NodeNotFound` |
| command behaviour | `uilab.session.UndoProposal` | decided outside the system: a later change to the document would be lost by undoing this one | given `uilab.session.UndoProposal` input, decide and enact exactly one outcome — `undone` otherwise, takes `undo` of `uilab.session.Proposal`, emits `uilab.session.ProposalUndone`; `stale` externally decided (a later change to the document would be lost by undoing this one), error `uilab.session.PatchRefused`; `wrong-state` from a state no declared move starts in, error `uilab.session.ProposalStateConflict`, and for an instance no record carries, without the error's fields |
| view query | `uilab.session.Documents` | how the projection is kept current is a storage decision | a query answering `uilab.session.Documents` with rows projected from `uilab.session.Document` at `read_your_writes` consistency |
| view query | `uilab.session.Pending` | how the projection is kept current is a storage decision | a query answering `uilab.session.Pending` with rows projected from `uilab.session.Proposal` at `read_your_writes` consistency, containing instances where `state == Proposed` |
| view query | `uilab.session.Proposals` | how the projection is kept current is a storage decision | a query answering `uilab.session.Proposals` with rows projected from `uilab.session.Proposal` at `read_your_writes` consistency |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
| actor grants | `uilab.session.Agent` | planning | may invoke `uilab.session.ProposePatch`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `uilab.session.Operator` | planning | may invoke `uilab.session.AcceptProposal`, `uilab.session.OpenDocument`, `uilab.session.RejectProposal`, `uilab.session.SelectNode`, `uilab.session.UndoProposal`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
