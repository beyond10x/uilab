// generated from uilab v1
// model digest b49ec22e0519143d410cde9e2fa36346e670414275339df6320419160b60ba3a
// contract digest 382360f755c7a7fe5f6fc4cb540326082dab33d91dbb381976dae34503c8a0e5
// do not edit: regenerate with `ess synthesize`

//! uilab-session — the `uilab-session` component of `uilab` v1.
//!
//! Holds the open document, the selection and every proposal made against it.
//!
//! The component's outer surface exactly as the specification declares it: accepted commands as
//! handlers, declared views as queries, published events as a typed outbox. The behaviour behind
//! every handler is an implementation obligation — see the `PLAN.md` beside this workspace — and
//! until one is satisfied, its stub answers with a typed refusal naming what is owed.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event this component declares it publishes, on its way to the system's transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishedEvent {
    /// `uilab.session.DocumentOpened`.
    DocumentOpened(uilab_types::session::DocumentOpened),
    /// `uilab.session.NodeSelected`.
    NodeSelected(uilab_types::session::NodeSelected),
    /// `uilab.session.PatchProposed`.
    PatchProposed(uilab_types::session::PatchProposed),
    /// `uilab.session.ProposalAccepted`.
    ProposalAccepted(uilab_types::session::ProposalAccepted),
    /// `uilab.session.ProposalRejected`.
    ProposalRejected(uilab_types::session::ProposalRejected),
    /// `uilab.session.ProposalUndone`.
    ProposalUndone(uilab_types::session::ProposalUndone),
}

/// uilab-session — the port over the component's obligations.
///
/// `B` bundles every behaviour and query this component owes; constructing it over the domain's
/// `obligations::Unimplemented` yields a component that compiles and refuses, in the type system,
/// everything not yet implemented.
pub struct UilabSession<B> {
    behaviors: B,
    outbox: Vec<PublishedEvent>,
}

impl<B> UilabSession<B> {
    /// A new port over the given obligation implementations.
    pub fn new(behaviors: B) -> Self {
        Self {
            behaviors,
            outbox: Vec::new(),
        }
    }

    /// Hands over everything published since the last drain, in publication order.
    ///
    /// The system's transport calls this; anything else reading it is taking events the transport
    /// will then never deliver.
    pub fn drain_outbox(&mut self) -> Vec<PublishedEvent> {
        core::mem::take(&mut self.outbox)
    }
}

impl<B> UilabSession<B>
where
    B: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    /// Accepts `uilab.session.AcceptProposal`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn accept_proposal(&mut self, input: uilab_types::session::AcceptProposal) -> Result<uilab_types::session::AcceptProposalOutcome, uilab_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.accept_proposal(input)?;
        match &outcome {
            uilab_types::session::AcceptProposalOutcome::Accepted { proposal_accepted, .. } => {
                self.outbox.push(PublishedEvent::ProposalAccepted(proposal_accepted.clone()));
            }
            uilab_types::session::AcceptProposalOutcome::Stale { .. } => {}
            uilab_types::session::AcceptProposalOutcome::WrongState { .. } => {}
            uilab_types::session::AcceptProposalOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `uilab.session.OpenDocument`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn open_document(&mut self, input: uilab_types::session::OpenDocument) -> Result<uilab_types::session::OpenDocumentOutcome, uilab_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.open_document(input)?;
        match &outcome {
            uilab_types::session::OpenDocumentOutcome::Opened { document_opened, .. } => {
                self.outbox.push(PublishedEvent::DocumentOpened(document_opened.clone()));
            }
            uilab_types::session::OpenDocumentOutcome::Unreadable { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `uilab.session.ProposePatch`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn propose_patch(&mut self, input: uilab_types::session::ProposePatch) -> Result<uilab_types::session::ProposePatchOutcome, uilab_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.propose_patch(input)?;
        match &outcome {
            uilab_types::session::ProposePatchOutcome::Proposed { patch_proposed, .. } => {
                self.outbox.push(PublishedEvent::PatchProposed(patch_proposed.clone()));
            }
            uilab_types::session::ProposePatchOutcome::Refused { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `uilab.session.RejectProposal`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn reject_proposal(&mut self, input: uilab_types::session::RejectProposal) -> Result<uilab_types::session::RejectProposalOutcome, uilab_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.reject_proposal(input)?;
        match &outcome {
            uilab_types::session::RejectProposalOutcome::Rejected { proposal_rejected, .. } => {
                self.outbox.push(PublishedEvent::ProposalRejected(proposal_rejected.clone()));
            }
            uilab_types::session::RejectProposalOutcome::WrongState { .. } => {}
            uilab_types::session::RejectProposalOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Accepts `uilab.session.SelectNode`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn select_node(&mut self, input: uilab_types::session::SelectNode) -> Result<uilab_types::session::SelectNodeOutcome, uilab_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.select_node(input)?;
        match &outcome {
            uilab_types::session::SelectNodeOutcome::Selected { node_selected, .. } => {
                self.outbox.push(PublishedEvent::NodeSelected(node_selected.clone()));
            }
            uilab_types::session::SelectNodeOutcome::UnknownDocument { .. } => {}
            uilab_types::session::SelectNodeOutcome::NotFound { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `uilab.session.UndoProposal`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn undo_proposal(&mut self, input: uilab_types::session::UndoProposal) -> Result<uilab_types::session::UndoProposalOutcome, uilab_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.undo_proposal(input)?;
        match &outcome {
            uilab_types::session::UndoProposalOutcome::Undone { proposal_undone, .. } => {
                self.outbox.push(PublishedEvent::ProposalUndone(proposal_undone.clone()));
            }
            uilab_types::session::UndoProposalOutcome::Stale { .. } => {}
            uilab_types::session::UndoProposalOutcome::WrongState { .. } => {}
            uilab_types::session::UndoProposalOutcome::WrongStateUnknownInstance => {}
        }
        Ok(outcome)
    }

    /// Serves `uilab.session.Documents` at `read_your_writes` consistency, from the owed projection.
    pub fn documents(&self) -> Result<Vec<uilab_types::session::Documents>, uilab_types::obligation::UnmetObligation> {
        self.behaviors.documents()
    }

    /// Serves `uilab.session.Pending` at `read_your_writes` consistency, from the owed projection.
    pub fn pending(&self) -> Result<Vec<uilab_types::session::Pending>, uilab_types::obligation::UnmetObligation> {
        self.behaviors.pending()
    }

    /// Serves `uilab.session.Proposals` at `read_your_writes` consistency, from the owed projection.
    pub fn proposals(&self) -> Result<Vec<uilab_types::session::Proposals>, uilab_types::obligation::UnmetObligation> {
        self.behaviors.proposals()
    }
}
