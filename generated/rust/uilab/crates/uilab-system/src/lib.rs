// generated from uilab v1
// model digest d3dac30e4a3114e008b9d96d1e4eba874d61954f6a5319044185b6c608753f13
// contract digest 9e9941e5243af0824123a784b98dadddce493ba5c713cb7b55ce33bfaf77efa0
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

impl SystemEvent {
    /// The qualified name the specification declares this event under.
    pub fn name(&self) -> &'static str {
        match self {
            Self::DocumentOpened(_) => "uilab.session.DocumentOpened",
            Self::NodeSelected(_) => "uilab.session.NodeSelected",
            Self::PatchProposed(_) => "uilab.session.PatchProposed",
            Self::ProposalAccepted(_) => "uilab.session.ProposalAccepted",
            Self::ProposalRejected(_) => "uilab.session.ProposalRejected",
            Self::ProposalUndone(_) => "uilab.session.ProposalUndone",
        }
    }
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

    /// Takes every event the pump has already delivered off the log, in publication order.
    ///
    /// A long-running shell calls this after each `pump`, or the log holds every event the
    /// process ever published. A `pump` returns with every logged event delivered: each
    /// reacting binding has had its attempt, and a binding whose attempt stopped holds the event in
    /// its own held-back list, not on the log. Events published since the last `pump` stay on the
    /// log, so the next `pump` still delivers them; taking never skips a binding.
    pub fn take_published(&mut self) -> Vec<SystemEvent> {
        let delivered: Vec<SystemEvent> = self.published.drain(..self.cursor).collect();
        self.cursor = 0;
        delivered
    }
}

impl<UilabSessionBehaviors> System<UilabSessionBehaviors>
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    /// Delivers until quiescent: collects every component's outbox onto the log. No binding
    /// reacts to anything this specification publishes, so collecting is the whole delivery.
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
