//! The test seam over externally decided outcomes.
//!
//! The specification marks several outcomes as decided outside the system: whether a file parses,
//! whether a node exists, whether a patched document passes its checks. A conformance suite has to
//! reach each of them on demand, with literal inputs that a real document would refuse. [`Externals`]
//! lets it: a forced outcome answers the next execution of its command before the real decision
//! runs, and "admit everything" lets placeholder paths and bodies through the document checks.
//!
//! [`Behaviour::new`](crate::Behaviour::new) holds no `Externals`, so the production path cannot
//! be steered by one; only [`Behaviour::with_externals`](crate::Behaviour::with_externals) takes it.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// An externally decided outcome, forced for the next execution of its command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forced {
    /// `OpenDocument` answers `unreadable`.
    OpenDocumentUnreadable,
    /// `SelectNode` answers `unknown-document`.
    SelectNodeUnknownDocument,
    /// `SelectNode` answers `not-found`.
    SelectNodeNotFound,
    /// `ProposePatch` answers `refused`.
    ProposePatchRefused,
    /// `AcceptProposal` answers `stale`.
    AcceptProposalStale,
    /// `UndoProposal` answers `stale`.
    UndoProposalStale,
}

/// The command a forced outcome belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Command {
    OpenDocument,
    SelectNode,
    ProposePatch,
    AcceptProposal,
    UndoProposal,
}

impl Forced {
    /// The forced outcome for a command and outcome as the specification names them, e.g.
    /// `("uilab.session.AcceptProposal", "stale")`. `None` for an outcome that is not externally
    /// decided.
    pub fn named(command: &str, outcome: &str) -> Option<Self> {
        let command = command.strip_prefix("uilab.session.").unwrap_or(command);
        Some(match (command, outcome) {
            ("OpenDocument", "unreadable") => Forced::OpenDocumentUnreadable,
            ("SelectNode", "unknown-document") => Forced::SelectNodeUnknownDocument,
            ("SelectNode", "not-found") => Forced::SelectNodeNotFound,
            ("ProposePatch", "refused") => Forced::ProposePatchRefused,
            ("AcceptProposal", "stale") => Forced::AcceptProposalStale,
            ("UndoProposal", "stale") => Forced::UndoProposalStale,
            _ => return None,
        })
    }

    pub(crate) fn command(self) -> Command {
        match self {
            Forced::OpenDocumentUnreadable => Command::OpenDocument,
            Forced::SelectNodeUnknownDocument | Forced::SelectNodeNotFound => Command::SelectNode,
            Forced::ProposePatchRefused => Command::ProposePatch,
            Forced::AcceptProposalStale => Command::AcceptProposal,
            Forced::UndoProposalStale => Command::UndoProposal,
        }
    }
}

#[derive(Debug, Default)]
struct State {
    forced: Vec<Forced>,
    admit_everything: bool,
}

/// Overrides for externally decided outcomes. A clone shares the same overrides.
#[derive(Debug, Clone, Default)]
pub struct Externals(Arc<Mutex<State>>);

impl Externals {
    /// No overrides: every decision is the real one.
    pub fn new() -> Self {
        Self::default()
    }

    /// Forces `outcome` for the next execution of its command, replacing any earlier forcing of
    /// that command.
    pub fn force(&self, outcome: Forced) {
        let mut state = self.lock();
        state.forced.retain(|f| f.command() != outcome.command());
        state.forced.push(outcome);
    }

    /// When on, a node path that does not resolve can be selected, and a patch whose body or
    /// target does not make a patch the document admits is still proposed and accepted — without
    /// changing the document.
    pub fn admit_everything(&self, on: bool) {
        self.lock().admit_everything = on;
    }

    pub(crate) fn take(&self, command: Command) -> Option<Forced> {
        let mut state = self.lock();
        let at = state.forced.iter().position(|f| f.command() == command)?;
        Some(state.forced.remove(at))
    }

    pub(crate) fn admits_everything(&self) -> bool {
        self.lock().admit_everything
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
