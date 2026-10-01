// generated from uilab v1
// model digest 8bbec934f18fca6258713253bcfca181cac2a1249bf16684012986ad1b427455
// contract digest c789fcd30e3ffcc51487d00315741eca272a88a53a2be1e3be26a919b30c4bfb
// do not edit: regenerate with `ess synthesize`

//! Session — `uilab.session`.
//!
//! One open UI document, the node the operator has selected in it, and the patches an agent proposes there. A patch changes the document only when the operator accepts it.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// The states of `uilab.session.Document`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `Document<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentState {
    /// `Open`.
    Open,
}

/// DocumentId — `uilab.session.DocumentId`: a distinct wrapper around `Uuid`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentId(pub crate::primitives::Uuid);

/// NodePath — `uilab.session.NodePath`: a distinct wrapper around `String`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodePath(pub String);

/// PatchBody — `uilab.session.PatchBody`: a distinct wrapper around `Json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchBody(pub crate::json::Value);

/// PatchOp — `uilab.session.PatchOp`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchOp {
    /// `Insert`.
    Insert,
    /// `Replace`.
    Replace,
    /// `Remove`.
    Remove,
    /// `Batch`.
    Batch,
}

/// The states of `uilab.session.Proposal`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `Proposal<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalState {
    /// `Accepted`.
    Accepted,
    /// `Proposed`.
    Proposed,
    /// `Rejected`.
    Rejected,
    /// `Undone`.
    Undone,
}

/// ProposalId — `uilab.session.ProposalId`: a distinct wrapper around `Uuid`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalId(pub crate::primitives::Uuid);

/// What Document — `uilab.session.Document` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`Document<S>`], and at a boundary by [`DocumentSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentData {
    /// The identity: `document_id` — `uilab.session.DocumentId`.
    pub document_id: DocumentId,
    /// `path` — `String`.
    pub path: String,
    /// `selected` — `uilab.session.NodePath`.
    pub selected: NodePath,
}

/// The states of `uilab.session.Document`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](document_state::Marker), so [`Document<S>`](Document) can only ever rest in a real state.
pub mod document_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Open {}
    }

    /// A declared state of `Document`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::DocumentState;
    }

    /// `Open`. Where a new instance starts.
    pub struct Open;

    impl Marker for Open {
        const STATE: super::DocumentState = super::DocumentState::Open;
    }
}

/// Document — `uilab.session.Document` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Open`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`DocumentSnapshot`]
/// and [`DocumentSnapshot::refine`].
pub struct Document<S: document_state::Marker> {
    data: DocumentData,
    state: core::marker::PhantomData<S>,
}

impl<S: document_state::Marker> Document<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> DocumentState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &DocumentData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> DocumentData {
        self.data
    }
}

impl Document<document_state::Open> {
    /// A new instance, resting in `Open` — the only state the lifecycle starts one in.
    pub fn new(data: DocumentData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `uilab.session.Document` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`DocumentSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: DocumentState,
    /// What it holds.
    pub data: DocumentData,
}

/// An `Document` in whichever declared state it was found.
pub enum AnyDocument {
    /// Resting in `Open`.
    Open(Document<document_state::Open>),
}

impl DocumentSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `DocumentState` cannot spell one.
    pub fn refine(self) -> AnyDocument {
        match self.state {
            DocumentState::Open => AnyDocument::Open(Document {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyDocument {
    /// The state, as the runtime value.
    pub fn state(&self) -> DocumentState {
        match self {
            Self::Open(_) => DocumentState::Open,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> DocumentSnapshot {
        match self {
            Self::Open(instance) => DocumentSnapshot {
                state: DocumentState::Open,
                data: instance.into_data(),
            },
        }
    }
}

/// What Proposal — `uilab.session.Proposal` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`Proposal<S>`], and at a boundary by [`ProposalSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalData {
    /// The identity: `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: ProposalId,
    /// `document_id` — `uilab.session.DocumentId`.
    ///
    /// Carries `proposals`: `uilab.session.Document` owns many `uilab.session.Proposal`.
    pub document_id: DocumentId,
    /// `target` — `uilab.session.NodePath`.
    pub target: NodePath,
    /// `op` — `uilab.session.PatchOp`.
    pub op: PatchOp,
    /// `utterance` — `String`.
    pub utterance: String,
}

/// The states of `uilab.session.Proposal`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](proposal_state::Marker), so [`Proposal<S>`](Proposal) can only ever rest in a real state.
pub mod proposal_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Accepted {}
        impl Sealed for super::Proposed {}
        impl Sealed for super::Rejected {}
        impl Sealed for super::Undone {}
    }

    /// A declared state of `Proposal`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::ProposalState;
    }

    /// `Accepted`.
    pub struct Accepted;

    impl Marker for Accepted {
        const STATE: super::ProposalState = super::ProposalState::Accepted;
    }

    /// `Proposed`. Where a new instance starts.
    pub struct Proposed;

    impl Marker for Proposed {
        const STATE: super::ProposalState = super::ProposalState::Proposed;
    }

    /// `Rejected`. Terminal: an instance may rest here forever.
    pub struct Rejected;

    impl Marker for Rejected {
        const STATE: super::ProposalState = super::ProposalState::Rejected;
    }

    /// `Undone`. Terminal: an instance may rest here forever.
    pub struct Undone;

    impl Marker for Undone {
        const STATE: super::ProposalState = super::ProposalState::Undone;
    }
}

/// Proposal — `uilab.session.Proposal` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Proposed`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`ProposalSnapshot`]
/// and [`ProposalSnapshot::refine`].
pub struct Proposal<S: proposal_state::Marker> {
    data: ProposalData,
    state: core::marker::PhantomData<S>,
}

impl<S: proposal_state::Marker> Proposal<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> ProposalState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &ProposalData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> ProposalData {
        self.data
    }
}

impl Proposal<proposal_state::Proposed> {
    /// A new instance, resting in `Proposed` — the only state the lifecycle starts one in.
    pub fn new(data: ProposalData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

impl Proposal<proposal_state::Accepted> {
    /// `undo` — `Accepted` → `Undone`. Taken by the `undone` outcome of `uilab.session.UndoProposal`.
    pub fn undo(self) -> Proposal<proposal_state::Undone> {
        Proposal {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl Proposal<proposal_state::Proposed> {
    /// `accept` — `Proposed` → `Accepted`. Taken by the `accepted` outcome of `uilab.session.AcceptProposal`.
    pub fn accept(self) -> Proposal<proposal_state::Accepted> {
        Proposal {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `reject` — `Proposed` → `Rejected`. Taken by the `rejected` outcome of `uilab.session.RejectProposal`.
    pub fn reject(self) -> Proposal<proposal_state::Rejected> {
        Proposal {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

/// `uilab.session.Proposal` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`ProposalSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: ProposalState,
    /// What it holds.
    pub data: ProposalData,
}

/// An `Proposal` in whichever declared state it was found.
pub enum AnyProposal {
    /// Resting in `Accepted`.
    Accepted(Proposal<proposal_state::Accepted>),
    /// Resting in `Proposed`.
    Proposed(Proposal<proposal_state::Proposed>),
    /// Resting in `Rejected`.
    Rejected(Proposal<proposal_state::Rejected>),
    /// Resting in `Undone`.
    Undone(Proposal<proposal_state::Undone>),
}

impl ProposalSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `ProposalState` cannot spell one.
    pub fn refine(self) -> AnyProposal {
        match self.state {
            ProposalState::Accepted => AnyProposal::Accepted(Proposal {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            ProposalState::Proposed => AnyProposal::Proposed(Proposal {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            ProposalState::Rejected => AnyProposal::Rejected(Proposal {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            ProposalState::Undone => AnyProposal::Undone(Proposal {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyProposal {
    /// The state, as the runtime value.
    pub fn state(&self) -> ProposalState {
        match self {
            Self::Accepted(_) => ProposalState::Accepted,
            Self::Proposed(_) => ProposalState::Proposed,
            Self::Rejected(_) => ProposalState::Rejected,
            Self::Undone(_) => ProposalState::Undone,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> ProposalSnapshot {
        match self {
            Self::Accepted(instance) => ProposalSnapshot {
                state: ProposalState::Accepted,
                data: instance.into_data(),
            },
            Self::Proposed(instance) => ProposalSnapshot {
                state: ProposalState::Proposed,
                data: instance.into_data(),
            },
            Self::Rejected(instance) => ProposalSnapshot {
                state: ProposalState::Rejected,
                data: instance.into_data(),
            },
            Self::Undone(instance) => ProposalSnapshot {
                state: ProposalState::Undone,
                data: instance.into_data(),
            },
        }
    }
}

/// Accept a proposal — the input of `uilab.session.AcceptProposal`.
///
/// Everything it can result in is [`AcceptProposalOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptProposal {
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: ProposalId,
}

/// Everything `uilab.session.AcceptProposal` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptProposalOutcome {
    /// `accepted` — otherwise.
    ///
    /// The patch is applied and the document is saved.
    Accepted {
        /// The `uilab.session.ProposalAccepted` this outcome publishes.
        proposal_accepted: ProposalAccepted,
    },
    /// `stale` — externally decided (the document changed since the proposal and the patch no longer applies).
    ///
    /// The proposal stays waiting and the document is unchanged.
    Stale {
        /// Why it was refused: `uilab.session.PatchRefused`.
        error: PatchRefused,
    },
    /// `wrong-state` — from a state no declared move starts in.
    ///
    /// The proposal is not waiting, so the document is unchanged.
    WrongState {
        /// Why it was refused: `uilab.session.ProposalStateConflict`.
        error: ProposalStateConflict,
    },
    /// `wrong-state` — for an instance no record carries.
    ///
    /// The same declared branch and error as [`Self::WrongState`], without the error's fields: an instance
    /// that does not exist has nothing for them to describe (`docs/design/unknown-instance-seams.md`).
    WrongStateUnknownInstance,
}

/// Open a document — the input of `uilab.session.OpenDocument`.
///
/// Everything it can result in is [`OpenDocumentOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenDocument {
    /// `path` — `String`.
    pub path: String,
}

/// Everything `uilab.session.OpenDocument` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenDocumentOutcome {
    /// `opened` — otherwise.
    ///
    /// The document is open with its root selected.
    Opened {
        /// The `uilab.session.DocumentOpened` this outcome publishes.
        document_opened: DocumentOpened,
    },
    /// `unreadable` — externally decided (the file does not parse as an ess-ui/1 document).
    ///
    /// Nothing was opened.
    Unreadable {
        /// Why it was refused: `uilab.session.DocumentUnreadable`.
        error: DocumentUnreadable,
    },
}

/// Propose a patch — the input of `uilab.session.ProposePatch`.
///
/// Everything it can result in is [`ProposePatchOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposePatch {
    /// `document_id` — `uilab.session.DocumentId`.
    pub document_id: DocumentId,
    /// `target` — `uilab.session.NodePath`.
    pub target: NodePath,
    /// `op` — `uilab.session.PatchOp`.
    pub op: PatchOp,
    /// `utterance` — `String`.
    pub utterance: String,
    /// `body` — `Optional<uilab.session.PatchBody>`.
    pub body: Option<PatchBody>,
}

/// Everything `uilab.session.ProposePatch` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposePatchOutcome {
    /// `proposed` — otherwise.
    ///
    /// The patch passes every document check and waits for the operator.
    Proposed {
        /// The `uilab.session.PatchProposed` this outcome publishes.
        patch_proposed: PatchProposed,
    },
    /// `refused` — externally decided (the patched document fails a document check).
    ///
    /// No proposal was made and the document is unchanged.
    Refused {
        /// Why it was refused: `uilab.session.PatchRefused`.
        error: PatchRefused,
    },
}

/// Reject a proposal — the input of `uilab.session.RejectProposal`.
///
/// Everything it can result in is [`RejectProposalOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectProposal {
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: ProposalId,
}

/// Everything `uilab.session.RejectProposal` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectProposalOutcome {
    /// `rejected` — otherwise.
    ///
    /// The patch is dropped and the document is unchanged.
    Rejected {
        /// The `uilab.session.ProposalRejected` this outcome publishes.
        proposal_rejected: ProposalRejected,
    },
    /// `wrong-state` — from a state no declared move starts in.
    ///
    /// The proposal is not waiting, so nothing was rejected.
    WrongState {
        /// Why it was refused: `uilab.session.ProposalStateConflict`.
        error: ProposalStateConflict,
    },
    /// `wrong-state` — for an instance no record carries.
    ///
    /// The same declared branch and error as [`Self::WrongState`], without the error's fields: an instance
    /// that does not exist has nothing for them to describe (`docs/design/unknown-instance-seams.md`).
    WrongStateUnknownInstance,
}

/// Select a node — the input of `uilab.session.SelectNode`.
///
/// Everything it can result in is [`SelectNodeOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectNode {
    /// `document_id` — `uilab.session.DocumentId`.
    pub document_id: DocumentId,
    /// `path` — `uilab.session.NodePath`.
    pub path: NodePath,
}

/// Everything `uilab.session.SelectNode` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectNodeOutcome {
    /// `selected` — otherwise.
    ///
    /// The node at the path is selected; proposals are made there.
    Selected {
        /// The `uilab.session.NodeSelected` this outcome publishes.
        node_selected: NodeSelected,
    },
    /// `unknown-document` — externally decided (no document with this id is open).
    ///
    /// The selection did not change.
    UnknownDocument {
        /// Why it was refused: `uilab.session.DocumentNotOpen`.
        error: DocumentNotOpen,
    },
    /// `not-found` — externally decided (the document has no node at this path).
    ///
    /// The selection did not change.
    NotFound {
        /// Why it was refused: `uilab.session.NodeNotFound`.
        error: NodeNotFound,
    },
}

/// Undo a proposal — the input of `uilab.session.UndoProposal`.
///
/// Everything it can result in is [`UndoProposalOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoProposal {
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: ProposalId,
}

/// Everything `uilab.session.UndoProposal` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoProposalOutcome {
    /// `undone` — otherwise.
    ///
    /// The document is back to what it was before the patch, and saved.
    Undone {
        /// The `uilab.session.ProposalUndone` this outcome publishes.
        proposal_undone: ProposalUndone,
    },
    /// `stale` — externally decided (a later change to the document would be lost by undoing this one).
    ///
    /// The document is unchanged.
    Stale {
        /// Why it was refused: `uilab.session.PatchRefused`.
        error: PatchRefused,
    },
    /// `wrong-state` — from a state no declared move starts in.
    ///
    /// The proposal was never applied, so there is nothing to undo.
    WrongState {
        /// Why it was refused: `uilab.session.ProposalStateConflict`.
        error: ProposalStateConflict,
    },
    /// `wrong-state` — for an instance no record carries.
    ///
    /// The same declared branch and error as [`Self::WrongState`], without the error's fields: an instance
    /// that does not exist has nothing for them to describe (`docs/design/unknown-instance-seams.md`).
    WrongStateUnknownInstance,
}

/// DocumentOpened — the event `uilab.session.DocumentOpened`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentOpened {
    /// `document_id` — `uilab.session.DocumentId`.
    pub document_id: DocumentId,
    /// `path` — `String`.
    pub path: String,
}

/// NodeSelected — the event `uilab.session.NodeSelected`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSelected {
    /// `document_id` — `uilab.session.DocumentId`.
    pub document_id: DocumentId,
    /// `path` — `uilab.session.NodePath`.
    pub path: NodePath,
}

/// PatchProposed — the event `uilab.session.PatchProposed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchProposed {
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: ProposalId,
    /// `document_id` — `uilab.session.DocumentId`.
    pub document_id: DocumentId,
    /// `target` — `uilab.session.NodePath`.
    pub target: NodePath,
    /// `op` — `uilab.session.PatchOp`.
    pub op: PatchOp,
}

/// ProposalAccepted — the event `uilab.session.ProposalAccepted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalAccepted {
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: ProposalId,
}

/// ProposalRejected — the event `uilab.session.ProposalRejected`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalRejected {
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: ProposalId,
}

/// ProposalUndone — the event `uilab.session.ProposalUndone`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalUndone {
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: ProposalId,
}

/// The declared error `uilab.session.DocumentNotOpen`.
///
/// No open document has this id, so nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentNotOpen {
    /// `document_id` — `uilab.session.DocumentId`.
    pub document_id: DocumentId,
}

/// The declared error `uilab.session.DocumentUnreadable`.
///
/// The file is not an ess-ui/1 document, so nothing was opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentUnreadable {
    /// `path` — `String`.
    pub path: String,
}

/// The declared error `uilab.session.NodeNotFound`.
///
/// No node of the document has this path, so the selection did not change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeNotFound {
    /// `path` — `uilab.session.NodePath`.
    pub path: NodePath,
}

/// The declared error `uilab.session.PatchRefused`.
///
/// The patch fails a document check, so no proposal was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchRefused {
    /// `check` — `String`.
    pub check: String,
}

/// The declared error `uilab.session.ProposalStateConflict`.
///
/// The proposal is not in a state this command acts from, so nothing moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalStateConflict {
    /// `state` — `uilab.session.Proposal.State`.
    pub state: ProposalState,
}

/// Documents — one row of the view `uilab.session.Documents`.
///
/// Projects `uilab.session.Document` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Documents {
    /// `document_id` — `uilab.session.DocumentId`.
    pub document_id: DocumentId,
    /// `path` — `String`.
    pub path: String,
    /// `selected` — `uilab.session.NodePath`.
    pub selected: NodePath,
}

/// Pending proposals — one row of the view `uilab.session.Pending`.
///
/// Projects `uilab.session.Proposal` at `read_your_writes` consistency, containing instances where `state == Proposed`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: ProposalId,
    /// `target` — `uilab.session.NodePath`.
    pub target: NodePath,
    /// `op` — `uilab.session.PatchOp`.
    pub op: PatchOp,
}

/// Proposals — one row of the view `uilab.session.Proposals`.
///
/// Projects `uilab.session.Proposal` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposals {
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: ProposalId,
    /// `document_id` — `uilab.session.DocumentId`.
    pub document_id: DocumentId,
    /// `target` — `uilab.session.NodePath`.
    pub target: NodePath,
    /// `op` — `uilab.session.PatchOp`.
    pub op: PatchOp,
    /// `utterance` — `String`.
    pub utterance: String,
    /// `state` — `uilab.session.Proposal.State`.
    pub state: ProposalState,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every owed trait by refusing in the type system.
pub mod obligations {
    /// The behaviour `uilab.session.AcceptProposal` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by the fields of error `uilab.session.PatchRefused`, which the specification gives no source, in `stale`.
    ///
    /// Contract: given `uilab.session.AcceptProposal` input, decide and enact exactly one outcome — `accepted` otherwise, takes `accept` of `uilab.session.Proposal`, emits `uilab.session.ProposalAccepted`; `stale` externally decided (the document changed since the proposal and the patch no longer applies), error `uilab.session.PatchRefused`; `wrong-state` from a state no declared move starts in, error `uilab.session.ProposalStateConflict`, and for an instance no record carries, without the error's fields.
    pub trait AcceptProposalBehavior {
        /// Decides and enacts exactly one declared outcome of `uilab.session.AcceptProposal`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn accept_proposal(&mut self, input: super::AcceptProposal) -> Result<super::AcceptProposalOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `uilab.session.OpenDocument` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by `creates:` leaving the required field `selected` of `uilab.session.Document` undetermined, in `opened`.
    ///
    /// Contract: given `uilab.session.OpenDocument` input, decide and enact exactly one outcome — `opened` otherwise, creates `uilab.session.Document`, emits `uilab.session.DocumentOpened`; `unreadable` externally decided (the file does not parse as an ess-ui/1 document), error `uilab.session.DocumentUnreadable`.
    pub trait OpenDocumentBehavior {
        /// Decides and enacts exactly one declared outcome of `uilab.session.OpenDocument`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn open_document(&mut self, input: super::OpenDocument) -> Result<super::OpenDocumentOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `uilab.session.ProposePatch` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by the fields of error `uilab.session.PatchRefused`, which the specification gives no source, in `refused`.
    ///
    /// Contract: given `uilab.session.ProposePatch` input, decide and enact exactly one outcome — `proposed` otherwise, creates `uilab.session.Proposal`, emits `uilab.session.PatchProposed`; `refused` externally decided (the patched document fails a document check), error `uilab.session.PatchRefused`.
    pub trait ProposePatchBehavior {
        /// Decides and enacts exactly one declared outcome of `uilab.session.ProposePatch`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn propose_patch(&mut self, input: super::ProposePatch) -> Result<super::ProposePatchOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `uilab.session.RejectProposal` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RejectProposalBehavior {
        /// Decides and enacts exactly one declared outcome of `uilab.session.RejectProposal`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn reject_proposal(&mut self, input: super::RejectProposal) -> Result<super::RejectProposalOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `uilab.session.SelectNode` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by an unknown identity, which reaches no declared outcome (neither `unknown_instance:` nor `wrong_state:`).
    ///
    /// Contract: given `uilab.session.SelectNode` input, decide and enact exactly one outcome — `selected` otherwise, updates `uilab.session.Document`, emits `uilab.session.NodeSelected`; `unknown-document` externally decided (no document with this id is open), error `uilab.session.DocumentNotOpen`; `not-found` externally decided (the document has no node at this path), error `uilab.session.NodeNotFound`.
    pub trait SelectNodeBehavior {
        /// Decides and enacts exactly one declared outcome of `uilab.session.SelectNode`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn select_node(&mut self, input: super::SelectNode) -> Result<super::SelectNodeOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `uilab.session.UndoProposal` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by the fields of error `uilab.session.PatchRefused`, which the specification gives no source, in `stale`.
    ///
    /// Contract: given `uilab.session.UndoProposal` input, decide and enact exactly one outcome — `undone` otherwise, takes `undo` of `uilab.session.Proposal`, emits `uilab.session.ProposalUndone`; `stale` externally decided (a later change to the document would be lost by undoing this one), error `uilab.session.PatchRefused`; `wrong-state` from a state no declared move starts in, error `uilab.session.ProposalStateConflict`, and for an instance no record carries, without the error's fields.
    pub trait UndoProposalBehavior {
        /// Decides and enacts exactly one declared outcome of `uilab.session.UndoProposal`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn undo_proposal(&mut self, input: super::UndoProposal) -> Result<super::UndoProposalOutcome, crate::obligation::UnmetObligation>;
    }

    /// The query `uilab.session.Documents` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait DocumentsQuery {
        /// Serves `uilab.session.Documents` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn documents(&self) -> Result<Vec<super::Documents>, crate::obligation::UnmetObligation>;
    }

    /// The query `uilab.session.Pending` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait PendingQuery {
        /// Serves `uilab.session.Pending` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn pending(&self) -> Result<Vec<super::Pending>, crate::obligation::UnmetObligation>;
    }

    /// The query `uilab.session.Proposals` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait ProposalsQuery {
        /// Serves `uilab.session.Proposals` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn proposals(&self) -> Result<Vec<super::Proposals>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl AcceptProposalBehavior for Unimplemented {
        fn accept_proposal(&mut self, _input: super::AcceptProposal) -> Result<super::AcceptProposalOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "uilab.session.AcceptProposal" })
        }
    }

    impl OpenDocumentBehavior for Unimplemented {
        fn open_document(&mut self, _input: super::OpenDocument) -> Result<super::OpenDocumentOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "uilab.session.OpenDocument" })
        }
    }

    impl ProposePatchBehavior for Unimplemented {
        fn propose_patch(&mut self, _input: super::ProposePatch) -> Result<super::ProposePatchOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "uilab.session.ProposePatch" })
        }
    }

    impl SelectNodeBehavior for Unimplemented {
        fn select_node(&mut self, _input: super::SelectNode) -> Result<super::SelectNodeOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "uilab.session.SelectNode" })
        }
    }

    impl UndoProposalBehavior for Unimplemented {
        fn undo_proposal(&mut self, _input: super::UndoProposal) -> Result<super::UndoProposalOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "uilab.session.UndoProposal" })
        }
    }
}
