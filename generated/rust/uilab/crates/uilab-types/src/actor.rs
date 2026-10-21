// generated from uilab v1
// model digest d3dac30e4a3114e008b9d96d1e4eba874d61954f6a5319044185b6c608753f13
// contract digest 9e9941e5243af0824123a784b98dadddce493ba5c713cb7b55ce33bfaf77efa0
// do not edit: regenerate with `ess synthesize`

//! Every actor the specification declares, and the commands each may invoke — as data.
//!
//! A grant is checked against a caller identity, which these types do not read from anywhere:
//! whatever authenticates a request builds a [`Caller`], and a served surface checks it against
//! [`may`] before the command runs. The `PLAN.md` beside this workspace says, per actor,
//! whether a generated surface enforces the grant or the caller does.

/// An actor the specification declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Actor {
    /// `uilab.session.Agent`.
    Agent,
    /// `uilab.session.Operator`.
    Operator,
}

impl Actor {
    /// Every declared actor, ordered by qualified name.
    pub const ALL: &'static [Actor] = &[
        Actor::Agent,
        Actor::Operator,
    ];

    /// The actor's qualified name, as the specification spells it.
    pub const fn name(self) -> &'static str {
        match self {
            Actor::Agent => "uilab.session.Agent",
            Actor::Operator => "uilab.session.Operator",
        }
    }
}

/// The qualified names of the commands `actor` may invoke, ordered by name; empty for an
/// actor that only observes.
pub fn may(actor: Actor) -> &'static [&'static str] {
    match actor {
        Actor::Agent => &[
            "uilab.session.ProposePatch",
        ],
        Actor::Operator => &[
            "uilab.session.AcceptProposal",
            "uilab.session.OpenDocument",
            "uilab.session.RejectProposal",
            "uilab.session.SelectNode",
            "uilab.session.UndoProposal",
        ],
    }
}

/// Who a request was authenticated as.
///
/// Built by whatever authenticates the request — a session, a token, a certificate — and
/// handed to the served surface's `dispatch` and `handle`, which check its grant
/// before the command runs. Never derived from the request itself: a client can write
/// anything into a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Caller {
    /// The declared actor.
    pub actor: Actor,
}

impl Caller {
    /// `true` when this caller may invoke `command`, named by its qualified name.
    pub fn may(&self, command: &str) -> bool {
        may(self.actor).contains(&command)
    }
}
