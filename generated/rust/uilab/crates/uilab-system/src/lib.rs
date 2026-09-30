// generated from uilab v1
// model digest c1c9563af89cf1f1f8ef7782b80ff93b32b7086e6e86f27333557738d9508f0c
// contract digest 80afaf984fa26a672bfaf0ddb52dc82a4b8cea5b476335aed27257b03acacaab
// do not edit: regenerate with `ess synthesize`

//! The `uilab` system, v1: its components assembled, its bindings wired, and its one transport.
//!
//! The transport is derived from the specification, not chosen: `at_least_once` is the only
//! delivery guarantee the model declares, so published events land on an append-only log and a
//! pump delivers each to every binding that reacts to it. The log is the system's observable
//! record, and so is the record of what each binding invoked. What no specification determines
//! — how an escalation event is filled, behaviour behind the ports — stays an obligation; see
//! the `PLAN.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event on the system's log: everything any component publishes, and everything a binding
/// escalates into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemEvent {
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

impl From<uilab_session::PublishedEvent> for SystemEvent {
    fn from(event: uilab_session::PublishedEvent) -> Self {
        match event {
            uilab_session::PublishedEvent::DocumentOpened(event) => Self::DocumentOpened(event),
            uilab_session::PublishedEvent::NodeSelected(event) => Self::NodeSelected(event),
            uilab_session::PublishedEvent::PatchProposed(event) => Self::PatchProposed(event),
            uilab_session::PublishedEvent::ProposalAccepted(event) => Self::ProposalAccepted(event),
            uilab_session::PublishedEvent::ProposalRejected(event) => Self::ProposalRejected(event),
            uilab_session::PublishedEvent::ProposalUndone(event) => Self::ProposalUndone(event),
        }
    }
}

/// The `uilab` system: every component behind its port, and the transport between them.
///
/// The component fields are public because commands enter the system through a component's own
/// port; the log and its delivery cursor are not, because publishing happens by pumping, not by
/// writing history directly.
pub struct System<UilabSessionBehaviors> {
    /// The `uilab-session` component.
    pub uilab_session: uilab_session::UilabSession<UilabSessionBehaviors>,
    published: Vec<SystemEvent>,
    cursor: usize,
}

impl<UilabSessionBehaviors> System<UilabSessionBehaviors> {
    /// Assembles the system from its components.
    pub fn new(uilab_session: uilab_session::UilabSession<UilabSessionBehaviors>) -> Self {
        Self {
            uilab_session,
            published: Vec::new(),
            cursor: 0,
        }
    }

    /// Everything published so far, in publication order — the system's observable record.
    pub fn published(&self) -> &[SystemEvent] {
        &self.published
    }
}

impl<UilabSessionBehaviors> System<UilabSessionBehaviors>
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    /// Delivers until quiescent: collects every component's outbox onto the log, then delivers
    /// each logged event to every binding that reacts to it — at least once each, which is the
    /// guarantee the specification declares.
    ///
    /// `Err` carries the first unmet obligation that delivery could not route around; the log
    /// keeps everything already published. A specification whose bindings feed each other
    /// without end will not quiesce, and this pump will not pretend otherwise.
    pub fn pump(&mut self) -> Result<(), uilab_types::obligation::UnmetObligation> {
        loop {
            self.collect();
            if self.cursor == self.published.len() {
                return Ok(());
            }
            self.cursor += 1;
        }
    }

    /// Moves every component's outbox onto the log, in component order.
    fn collect(&mut self) {
        for event in self.uilab_session.drain_outbox() {
            self.published.push(SystemEvent::from(event));
        }
    }
}
