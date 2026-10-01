// generated from uilab v1
// model digest 8bbec934f18fca6258713253bcfca181cac2a1249bf16684012986ad1b427455
// contract digest c789fcd30e3ffcc51487d00315741eca272a88a53a2be1e3be26a919b30c4bfb
// do not edit: regenerate with `ess synthesize`

//! What the specification fully determines, generated: the behaviour of every command the plan
//! lists as generated, written against ports the implementor supplies.
//!
//! Storage is a port: one trait per entity, get, put and delete of a snapshot by identity. ess
//! generates the trait and never a store. `Context` is the other port: the caller's attributes,
//! every identity and value the model says the implementation assigns, and the answer to each
//! `external:` branch. [`Generated`] implements every generated `…Behavior` trait over those ports
//! and forwards every behaviour and query the plan still owes to them, so it is a complete bundle
//! for every component port. To replace one generated behaviour, write a bundle of your own that
//! implements that trait and delegates the rest to a `Generated`.
//!
//! An `Err` from a generated behaviour is the typed refusal naming the command: the model declares
//! no outcome for the request (a guard is undecidable over it, or no declared branch answers it),
//! or — as `entity invariant` — the declared outcome would leave an entity breaking an invariant.

use crate::obligation::UnmetObligation;

/// Where `uilab.session.Document` is stored — a port the implementor provides.
///
/// Keyed by the identity `document_id`. ess generates this trait and never an implementation of it.
pub trait DocumentStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::session::DocumentId) -> Option<crate::session::DocumentSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::session::DocumentSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::session::DocumentId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::session::DocumentSnapshot>;
}

/// Where `uilab.session.Proposal` is stored — a port the implementor provides.
///
/// Keyed by the identity `proposal_id`. ess generates this trait and never an implementation of it.
pub trait ProposalStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::session::ProposalId) -> Option<crate::session::ProposalSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::session::ProposalSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::session::ProposalId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::session::ProposalSnapshot>;
}

/// Every generated behaviour of this workspace, over the ports `P` supplies.
///
/// `P` implements the storage trait of each entity a generated behaviour reads or writes,
/// `Context` where one asks it anything, and every `…Behavior` and `…Query` trait the plan still
/// owes; `Generated<P>` forwards those to it.
pub struct Generated<P> {
    /// The storage and context ports, and every behaviour or query still owed.
    pub ports: P,
}

impl<P> Generated<P> {
    /// The generated behaviours, over `ports`.
    pub fn new(ports: P) -> Self {
        Self { ports }
    }
}

impl<P: crate::session::obligations::AcceptProposalBehavior> crate::session::obligations::AcceptProposalBehavior for Generated<P> {
    fn accept_proposal(&mut self, input: crate::session::AcceptProposal) -> Result<crate::session::AcceptProposalOutcome, UnmetObligation> {
        crate::session::obligations::AcceptProposalBehavior::accept_proposal(&mut self.ports, input)
    }
}

impl<P: crate::session::obligations::OpenDocumentBehavior> crate::session::obligations::OpenDocumentBehavior for Generated<P> {
    fn open_document(&mut self, input: crate::session::OpenDocument) -> Result<crate::session::OpenDocumentOutcome, UnmetObligation> {
        crate::session::obligations::OpenDocumentBehavior::open_document(&mut self.ports, input)
    }
}

impl<P: crate::session::obligations::ProposePatchBehavior> crate::session::obligations::ProposePatchBehavior for Generated<P> {
    fn propose_patch(&mut self, input: crate::session::ProposePatch) -> Result<crate::session::ProposePatchOutcome, UnmetObligation> {
        crate::session::obligations::ProposePatchBehavior::propose_patch(&mut self.ports, input)
    }
}

/// `uilab.session.RejectProposal`, generated: every outcome is one the specification fully determines.
impl<P> crate::session::obligations::RejectProposalBehavior for Generated<P>
where
    P: ProposalStorage,
{
    fn reject_proposal(&mut self, input: crate::session::RejectProposal) -> Result<crate::session::RejectProposalOutcome, UnmetObligation> {
        let _ = &input;
        // `rejected`: the default.
        let Some(held) = ProposalStorage::get(&self.ports, &input.proposal_id) else {
            return Ok(crate::session::RejectProposalOutcome::WrongStateUnknownInstance);
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::session::AnyProposal::Proposed(instance) => crate::session::AnyProposal::Rejected(instance.reject()),
            _ => return Ok(crate::session::RejectProposalOutcome::WrongState { error: crate::session::ProposalStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        ProposalStorage::put(&mut self.ports, next);
        return Ok(crate::session::RejectProposalOutcome::Rejected { proposal_rejected: crate::session::ProposalRejected { proposal_id: input.proposal_id.clone() } });
    }
}

impl<P: crate::session::obligations::SelectNodeBehavior> crate::session::obligations::SelectNodeBehavior for Generated<P> {
    fn select_node(&mut self, input: crate::session::SelectNode) -> Result<crate::session::SelectNodeOutcome, UnmetObligation> {
        crate::session::obligations::SelectNodeBehavior::select_node(&mut self.ports, input)
    }
}

impl<P: crate::session::obligations::UndoProposalBehavior> crate::session::obligations::UndoProposalBehavior for Generated<P> {
    fn undo_proposal(&mut self, input: crate::session::UndoProposal) -> Result<crate::session::UndoProposalOutcome, UnmetObligation> {
        crate::session::obligations::UndoProposalBehavior::undo_proposal(&mut self.ports, input)
    }
}

/// `uilab.session.Documents`, generated: every row is one the specification fully determines from the stored `uilab.session.Document`s.
impl<P> crate::session::obligations::DocumentsQuery for Generated<P>
where
    P: DocumentStorage,
{
    fn documents(&self) -> Result<Vec<crate::session::Documents>, UnmetObligation> {
        let admitted = DocumentStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::session::Documents {
                document_id: held.data.document_id,
                path: held.data.path,
                selected: held.data.selected,
            })
            .collect())
    }
}

/// `uilab.session.Pending`, generated: every row is one the specification fully determines from the stored `uilab.session.Proposal`s.
impl<P> crate::session::obligations::PendingQuery for Generated<P>
where
    P: ProposalStorage,
{
    fn pending(&self) -> Result<Vec<crate::session::Pending>, UnmetObligation> {
        let mut admitted = ProposalStorage::list(&self.ports);
        // `filter:` shows a row where it holds; false or unknown hides it.
        admitted.retain(|held| equal(Some(&held.state).map(|value| match value { crate::session::ProposalState::Accepted => "Accepted", crate::session::ProposalState::Proposed => "Proposed", crate::session::ProposalState::Rejected => "Rejected", crate::session::ProposalState::Undone => "Undone" }.to_owned()), Some("Proposed".to_owned())) == Some(true));
        Ok(admitted
            .into_iter()
            .map(|held| crate::session::Pending {
                proposal_id: held.data.proposal_id,
                target: held.data.target,
                op: held.data.op,
            })
            .collect())
    }
}

/// `uilab.session.Proposals`, generated: every row is one the specification fully determines from the stored `uilab.session.Proposal`s.
impl<P> crate::session::obligations::ProposalsQuery for Generated<P>
where
    P: ProposalStorage,
{
    fn proposals(&self) -> Result<Vec<crate::session::Proposals>, UnmetObligation> {
        let admitted = ProposalStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::session::Proposals {
                proposal_id: held.data.proposal_id,
                document_id: held.data.document_id,
                target: held.data.target,
                op: held.data.op,
                utterance: held.data.utterance,
                state: held.state,
            })
            .collect())
    }
}

/// Equality of two read values; an unread one is Unknown.
fn equal<T: PartialEq>(left: Option<T>, right: Option<T>) -> Option<bool> {
    Some(left? == right?)
}
