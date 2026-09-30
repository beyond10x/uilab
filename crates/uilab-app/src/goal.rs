//! One goal run: the goal an operator gave, the steps it was planned into, and where the run stands.
//!
//! Pure. The session actor tells it what happened — the plan came back, a step was proposed, a
//! proposal was decided, the agent refused, somebody pressed stop — and does what the returned
//! [`Next`] says: propose a step, broadcast the change, or nothing at all.

use uilab_doc::NodePath;

/// Steps planned when the goal names no cap.
pub const DEFAULT_MAX_STEPS: usize = 8;

/// Where a goal run stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalState {
    /// The plan has been asked for and has not come back.
    Planning,
    /// Steps are being carried out.
    Running,
    /// Every step was carried out, accepted or not.
    Done,
    /// An operator stopped it.
    Stopped,
    /// There is no plan: planning failed or the agent declined the goal.
    Failed,
}

impl GoalState {
    /// The spec name, `uilab.wire.GoalState`.
    pub fn name(self) -> &'static str {
        match self {
            GoalState::Planning => "planning",
            GoalState::Running => "running",
            GoalState::Done => "done",
            GoalState::Stopped => "stopped",
            GoalState::Failed => "failed",
        }
    }

    /// Planning or running: the run still acts, and nothing else may.
    pub fn is_active(self) -> bool {
        matches!(self, GoalState::Planning | GoalState::Running)
    }
}

/// Where one step stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    /// Not reached yet, or stopped before its proposal came back.
    Pending,
    /// The agent is working on it.
    Thinking,
    /// Its proposal waits for accept or reject.
    Proposed,
    Accepted,
    Rejected,
    /// No proposal: a check refused the agent's answer, or the target does not resolve.
    Refused,
    /// The agent judged the instruction not a UI change.
    Declined,
}

impl StepStatus {
    /// The spec name, `uilab.wire.StepStatus`.
    pub fn name(self) -> &'static str {
        match self {
            StepStatus::Pending => "pending",
            StepStatus::Thinking => "thinking",
            StepStatus::Proposed => "proposed",
            StepStatus::Accepted => "accepted",
            StepStatus::Rejected => "rejected",
            StepStatus::Refused => "refused",
            StepStatus::Declined => "declined",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GoalStep {
    pub instruction: String,
    pub target: NodePath,
    pub why: String,
    pub status: StepStatus,
    /// The proposal the step produced, once it produced one.
    pub proposal_id: Option<String>,
}

/// What the actor does after a transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Next {
    /// Propose step `index`: its instruction at its target.
    Propose {
        index: usize,
        instruction: String,
        target: NodePath,
    },
    /// The run changed and waits: for a decision on the step's proposal.
    Wait,
    /// The run changed and is over: done, stopped or failed.
    Ended,
    /// The event was not this run's (a late answer, another proposal); nothing changed.
    Ignored,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Goal {
    pub id: String,
    /// The operator the goal and every step are attributed to.
    pub by: String,
    pub text: String,
    /// The node the goal is planned at.
    pub target: NodePath,
    pub max_steps: usize,
    /// Each step's proposal waits for accept or reject; otherwise it is applied at once.
    pub review: bool,
    pub state: GoalState,
    pub steps: Vec<GoalStep>,
    /// The step being carried out; `None` while planning and once the run is over.
    pub current: Option<usize>,
    /// Why the run failed.
    pub message: Option<String>,
}

impl Goal {
    /// A goal whose plan has been asked for.
    pub fn new(
        id: impl Into<String>,
        by: impl Into<String>,
        text: impl Into<String>,
        target: NodePath,
        max_steps: usize,
        review: bool,
    ) -> Self {
        Goal {
            id: id.into(),
            by: by.into(),
            text: text.into(),
            target,
            max_steps,
            review,
            state: GoalState::Planning,
            steps: Vec::new(),
            current: None,
            message: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Whether the run still waits for step `index`'s proposal; an answer it does not wait for
    /// is thrown away, not recorded.
    pub fn awaits(&self, index: usize) -> bool {
        self.at(index, StepStatus::Thinking)
    }

    /// Running, at step `index`, in `status`.
    fn at(&self, index: usize, status: StepStatus) -> bool {
        self.state == GoalState::Running
            && self.current == Some(index)
            && self.steps.get(index).is_some_and(|s| s.status == status)
    }

    /// The plan came back: run its first step, or end when it has none.
    pub fn planned(&mut self, steps: Vec<uilab_agent::Step>) -> Next {
        if self.state != GoalState::Planning {
            return Next::Ignored;
        }
        self.steps = steps
            .into_iter()
            .map(|s| GoalStep {
                instruction: s.instruction,
                target: s.target,
                why: s.why,
                status: StepStatus::Pending,
                proposal_id: None,
            })
            .collect();
        self.state = GoalState::Running;
        self.start(0)
    }

    /// Planning failed or the agent declined the goal.
    pub fn plan_failed(&mut self, message: impl Into<String>) -> Next {
        if self.state != GoalState::Planning {
            return Next::Ignored;
        }
        self.state = GoalState::Failed;
        self.message = Some(message.into());
        Next::Ended
    }

    /// Step `index` produced a proposal; it waits for a decision.
    pub fn proposed(&mut self, index: usize, proposal_id: impl Into<String>) -> Next {
        if !self.at(index, StepStatus::Thinking) {
            return Next::Ignored;
        }
        let step = &mut self.steps[index];
        step.status = StepStatus::Proposed;
        step.proposal_id = Some(proposal_id.into());
        Next::Wait
    }

    /// Step `index` produced no proposal: refused, or declined when `declined`. The run goes on.
    pub fn not_proposed(&mut self, index: usize, declined: bool) -> Next {
        if !self.at(index, StepStatus::Thinking) {
            return Next::Ignored;
        }
        self.steps[index].status = if declined {
            StepStatus::Declined
        } else {
            StepStatus::Refused
        };
        self.start(index + 1)
    }

    /// A proposal was accepted or rejected. The run goes on when it was the current step's.
    pub fn decided(&mut self, proposal_id: &str, accepted: bool) -> Next {
        let Some(index) = self.current else {
            return Next::Ignored;
        };
        if !self.at(index, StepStatus::Proposed)
            || self.steps[index].proposal_id.as_deref() != Some(proposal_id)
        {
            return Next::Ignored;
        }
        self.steps[index].status = if accepted {
            StepStatus::Accepted
        } else {
            StepStatus::Rejected
        };
        self.start(index + 1)
    }

    /// An operator stopped the run. Returns what follows and the proposal left waiting, which the
    /// actor rejects.
    pub fn stop(&mut self) -> (Next, Option<String>) {
        if !self.is_active() {
            return (Next::Ignored, None);
        }
        let mut waiting = None;
        if let Some(step) = self.current.and_then(|i| self.steps.get_mut(i)) {
            match step.status {
                StepStatus::Thinking => step.status = StepStatus::Pending,
                StepStatus::Proposed => {
                    step.status = StepStatus::Rejected;
                    waiting = step.proposal_id.clone();
                }
                _ => {}
            }
        }
        self.state = GoalState::Stopped;
        self.current = None;
        (Next::Ended, waiting)
    }

    /// Step `index` starts thinking, or the run is done when there is none.
    fn start(&mut self, index: usize) -> Next {
        match self.steps.get_mut(index) {
            Some(step) => {
                step.status = StepStatus::Thinking;
                self.current = Some(index);
                Next::Propose {
                    index,
                    instruction: step.instruction.clone(),
                    target: step.target.clone(),
                }
            }
            None => {
                self.state = GoalState::Done;
                self.current = None;
                Next::Ended
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(p: &str) -> NodePath {
        p.parse().unwrap()
    }

    fn step(instruction: &str, target: &str) -> uilab_agent::Step {
        uilab_agent::Step {
            instruction: instruction.into(),
            target: path(target),
            why: format!("why {instruction}"),
        }
    }

    fn goal() -> Goal {
        Goal::new(
            "goal-1",
            "api-1",
            "build out the member area",
            path("page:members"),
            DEFAULT_MAX_STEPS,
            true,
        )
    }

    fn three() -> Vec<uilab_agent::Step> {
        vec![
            step("add a list", "page:members"),
            step("add a details card", "page:members"),
            step("add an edit drawer", "page:members/section:list"),
        ]
    }

    fn statuses(g: &Goal) -> Vec<StepStatus> {
        g.steps.iter().map(|s| s.status).collect()
    }

    fn running() -> Goal {
        let mut g = goal();
        g.planned(three());
        g
    }

    #[test]
    fn a_new_goal_is_planning_with_nothing_current() {
        let g = goal();
        assert_eq!(g.state, GoalState::Planning);
        assert!(g.is_active());
        assert!(g.steps.is_empty());
        assert_eq!(g.current, None);
        assert_eq!(g.message, None);
        assert_eq!(g.by, "api-1");
        assert_eq!(g.target, path("page:members"));
        assert!(g.review);
    }

    #[test]
    fn a_plan_starts_the_first_step() {
        let mut g = goal();
        let next = g.planned(three());
        assert_eq!(
            next,
            Next::Propose {
                index: 0,
                instruction: "add a list".into(),
                target: path("page:members"),
            }
        );
        assert_eq!(g.state, GoalState::Running);
        assert_eq!(g.current, Some(0));
        assert_eq!(
            statuses(&g),
            [
                StepStatus::Thinking,
                StepStatus::Pending,
                StepStatus::Pending
            ]
        );
        assert_eq!(g.steps[2].why, "why add an edit drawer");
        assert!(g.awaits(0));
        assert!(!g.awaits(1));
    }

    #[test]
    fn an_empty_plan_is_done_at_once() {
        let mut g = goal();
        assert_eq!(g.planned(vec![]), Next::Ended);
        assert_eq!(g.state, GoalState::Done);
        assert_eq!(g.current, None);
        assert!(!g.is_active());
    }

    #[test]
    fn a_planning_failure_fails_the_goal_with_its_message() {
        let mut g = goal();
        assert_eq!(g.plan_failed("declined: that is a greeting"), Next::Ended);
        assert_eq!(g.state, GoalState::Failed);
        assert_eq!(g.message.as_deref(), Some("declined: that is a greeting"));
        assert!(!g.is_active());
    }

    #[test]
    fn a_plan_or_failure_after_planning_is_ignored() {
        let mut g = running();
        let before = g.clone();
        assert_eq!(g.planned(three()), Next::Ignored);
        assert_eq!(g.plan_failed("late"), Next::Ignored);
        assert_eq!(g, before);

        let mut stopped = goal();
        stopped.stop();
        assert_eq!(stopped.planned(three()), Next::Ignored);
        assert_eq!(stopped.state, GoalState::Stopped);
        assert!(stopped.steps.is_empty());
    }

    #[test]
    fn a_proposal_waits_for_a_decision() {
        let mut g = running();
        assert_eq!(g.proposed(0, "p1"), Next::Wait);
        assert_eq!(g.steps[0].status, StepStatus::Proposed);
        assert_eq!(g.steps[0].proposal_id.as_deref(), Some("p1"));
        assert_eq!(g.current, Some(0));
        assert!(!g.awaits(0), "the answer came; a second one is not wanted");
    }

    #[test]
    fn a_proposal_for_another_step_is_ignored() {
        let mut g = running();
        let before = g.clone();
        assert_eq!(g.proposed(1, "p1"), Next::Ignored);
        assert_eq!(g.not_proposed(2, false), Next::Ignored);
        assert_eq!(g, before);
    }

    #[test]
    fn accepting_the_step_proposal_starts_the_next_step() {
        let mut g = running();
        g.proposed(0, "p1");
        assert_eq!(
            g.decided("p1", true),
            Next::Propose {
                index: 1,
                instruction: "add a details card".into(),
                target: path("page:members"),
            }
        );
        assert_eq!(
            statuses(&g),
            [
                StepStatus::Accepted,
                StepStatus::Thinking,
                StepStatus::Pending
            ]
        );
        assert_eq!(g.current, Some(1));
    }

    #[test]
    fn rejecting_the_step_proposal_starts_the_next_step() {
        let mut g = running();
        g.proposed(0, "p1");
        assert!(matches!(
            g.decided("p1", false),
            Next::Propose { index: 1, .. }
        ));
        assert_eq!(g.steps[0].status, StepStatus::Rejected);
        assert_eq!(g.steps[0].proposal_id.as_deref(), Some("p1"));
    }

    #[test]
    fn a_decision_on_another_proposal_is_ignored() {
        let mut g = running();
        g.proposed(0, "p1");
        let before = g.clone();
        assert_eq!(g.decided("p0", true), Next::Ignored);
        assert_eq!(g, before);

        let mut thinking = running();
        let before = thinking.clone();
        assert_eq!(
            thinking.decided("p1", true),
            Next::Ignored,
            "no proposal yet"
        );
        assert_eq!(thinking, before);
    }

    #[test]
    fn a_refused_step_is_marked_and_the_run_goes_on() {
        let mut g = running();
        assert!(matches!(
            g.not_proposed(0, false),
            Next::Propose { index: 1, .. }
        ));
        assert_eq!(g.steps[0].status, StepStatus::Refused);
        assert_eq!(g.steps[0].proposal_id, None);
    }

    #[test]
    fn a_declined_step_is_marked_and_the_run_goes_on() {
        let mut g = running();
        assert!(matches!(
            g.not_proposed(0, true),
            Next::Propose { index: 1, .. }
        ));
        assert_eq!(g.steps[0].status, StepStatus::Declined);
    }

    #[test]
    fn after_the_last_step_the_goal_is_done() {
        let mut g = running();
        g.proposed(0, "p1");
        g.decided("p1", true);
        g.not_proposed(1, false);
        g.proposed(2, "p3");
        assert_eq!(g.decided("p3", false), Next::Ended);
        assert_eq!(g.state, GoalState::Done);
        assert_eq!(g.current, None);
        assert!(!g.is_active());
        assert_eq!(
            statuses(&g),
            [
                StepStatus::Accepted,
                StepStatus::Refused,
                StepStatus::Rejected
            ]
        );
        assert_eq!(g.message, None);
    }

    #[test]
    fn a_refused_last_step_ends_the_run() {
        let mut g = goal();
        g.planned(vec![step("add a list", "page:members")]);
        assert_eq!(g.not_proposed(0, false), Next::Ended);
        assert_eq!(g.state, GoalState::Done);
    }

    #[test]
    fn stopping_while_planning_stops_with_nothing_to_reject() {
        let mut g = goal();
        assert_eq!(g.stop(), (Next::Ended, None));
        assert_eq!(g.state, GoalState::Stopped);
        assert!(!g.is_active());
    }

    #[test]
    fn stopping_while_a_step_thinks_leaves_it_pending_and_drops_its_answer() {
        let mut g = running();
        assert_eq!(g.stop(), (Next::Ended, None));
        assert_eq!(g.state, GoalState::Stopped);
        assert_eq!(g.current, None);
        assert_eq!(
            statuses(&g),
            [
                StepStatus::Pending,
                StepStatus::Pending,
                StepStatus::Pending
            ]
        );
        assert!(!g.awaits(0));
        let before = g.clone();
        assert_eq!(g.proposed(0, "late"), Next::Ignored);
        assert_eq!(g.not_proposed(0, false), Next::Ignored);
        assert_eq!(g, before);
    }

    #[test]
    fn stopping_while_a_proposal_waits_rejects_it() {
        let mut g = running();
        g.proposed(0, "p1");
        assert_eq!(g.stop(), (Next::Ended, Some("p1".into())));
        assert_eq!(g.steps[0].status, StepStatus::Rejected);
        assert_eq!(g.state, GoalState::Stopped);
        let before = g.clone();
        assert_eq!(g.decided("p1", true), Next::Ignored);
        assert_eq!(g, before);
    }

    #[test]
    fn a_finished_run_cannot_be_stopped_or_changed() {
        let mut done = goal();
        done.planned(vec![]);
        let mut failed = goal();
        failed.plan_failed("no");
        for mut g in [done, failed] {
            let before = g.clone();
            assert_eq!(g.stop(), (Next::Ignored, None));
            assert_eq!(g.decided("p1", true), Next::Ignored);
            assert_eq!(g, before);
        }
    }

    #[test]
    fn names_are_the_spec_names() {
        let states = [
            GoalState::Planning,
            GoalState::Running,
            GoalState::Done,
            GoalState::Stopped,
            GoalState::Failed,
        ];
        assert_eq!(
            states.map(GoalState::name),
            ["planning", "running", "done", "stopped", "failed"]
        );
        let statuses = [
            StepStatus::Pending,
            StepStatus::Thinking,
            StepStatus::Proposed,
            StepStatus::Accepted,
            StepStatus::Rejected,
            StepStatus::Refused,
            StepStatus::Declined,
        ];
        assert_eq!(
            statuses.map(StepStatus::name),
            [
                "pending", "thinking", "proposed", "accepted", "rejected", "refused", "declined"
            ]
        );
    }
}
