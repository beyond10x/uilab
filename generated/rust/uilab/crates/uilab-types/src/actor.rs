// generated from uilab v1
// model digest 8bbec934f18fca6258713253bcfca181cac2a1249bf16684012986ad1b427455
// contract digest c789fcd30e3ffcc51487d00315741eca272a88a53a2be1e3be26a919b30c4bfb
// do not edit: regenerate with `ess synthesize`

//! Every actor the specification declares, and the commands each may invoke — as data.
//!
//! Generated, not enforced: a grant is checked against a caller identity, which types do not
//! carry, so the caller that knows who is calling enforces it, against [`may`]. The `PLAN.md`
//! beside this workspace records the same refusal for every actor.

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
