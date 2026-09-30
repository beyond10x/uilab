// generated from uilab v1
// model digest b49ec22e0519143d410cde9e2fa36346e670414275339df6320419160b60ba3a
// contract digest 382360f755c7a7fe5f6fc4cb540326082dab33d91dbb381976dae34503c8a0e5
// do not edit: regenerate with `ess synthesize`

//! wire — `uilab.wire`.
//!
//! What travels between the browser and the uilab server over one WebSocket. Audio travels as binary frames of 16 kHz mono little-endian f32 samples between `mic` open and `mic` closed; everything else is one JSON message per text frame, tagged by `type`.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// Changed — `uilab.wire.Changed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changed {
    /// `revision` — `Integer`.
    pub revision: i64,
    /// `by` — `String`.
    pub by: String,
    /// `op` — `uilab.session.PatchOp`.
    pub op: crate::session::PatchOp,
    /// `changed` — `uilab.session.NodePath`.
    pub changed: crate::session::NodePath,
    /// `parent` — `uilab.session.NodePath`.
    pub parent: crate::session::NodePath,
    /// `node` — `Optional<uilab.wire.OutlineNode>`.
    pub node: Option<OutlineNode>,
    /// `findings` — `List<uilab.wire.Finding>`.
    pub findings: Vec<Finding>,
}

/// ClientMessage — `uilab.wire.ClientMessage`: one of a fixed set of shapes, tagged on the wire by `type`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientMessage {
    /// Tagged `accept` — `uilab.wire.Decide`.
    Accept(Decide),
    /// Tagged `goal` — `uilab.wire.StartGoal`.
    Goal(StartGoal),
    /// Tagged `hello` — `uilab.wire.Hello`.
    Hello(Hello),
    /// Tagged `mic` — `uilab.wire.Mic`.
    Mic(Mic),
    /// Tagged `reject` — `uilab.wire.Decide`.
    Reject(Decide),
    /// Tagged `resync` — `uilab.wire.Resync`.
    Resync(Resync),
    /// Tagged `rows` — `uilab.wire.ReadRows`.
    Rows(ReadRows),
    /// Tagged `say` — `uilab.wire.Say`.
    Say(Say),
    /// Tagged `select` — `uilab.wire.Select`.
    Select(Select),
    /// Tagged `settings` — `uilab.wire.Settings`.
    Settings(Settings),
    /// Tagged `stop_goal` — `uilab.wire.StopGoal`.
    StopGoal(StopGoal),
    /// Tagged `undo` — `uilab.wire.Decide`.
    Undo(Decide),
}

/// Decide — `uilab.wire.Decide`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decide {
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: crate::session::ProposalId,
}

/// DocumentState — `uilab.wire.DocumentState`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentState {
    /// `document_id` — `uilab.session.DocumentId`.
    pub document_id: crate::session::DocumentId,
    /// `file` — `String`.
    pub file: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `selected` — `uilab.session.NodePath`.
    pub selected: crate::session::NodePath,
    /// `outline` — `uilab.wire.OutlineNode`.
    pub outline: OutlineNode,
    /// `findings` — `List<uilab.wire.Finding>`.
    pub findings: Vec<Finding>,
    /// `undoable` — `Optional<uilab.session.ProposalId>`.
    pub undoable: Option<crate::session::ProposalId>,
    /// `revision` — `Integer`.
    pub revision: i64,
    /// `review` — `Boolean`.
    pub review: bool,
}

/// Failed — `uilab.wire.Failed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failed {
    /// `by` — `Optional<String>`.
    pub by: Option<String>,
    /// `message` — `String`.
    pub message: String,
}

/// Finding — `uilab.wire.Finding`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// `check` — `String`.
    pub check: String,
    /// `severity` — `uilab.wire.Severity`.
    pub severity: Severity,
    /// `path` — `String`.
    pub path: String,
    /// `message` — `String`.
    pub message: String,
}

/// Goal — `uilab.wire.Goal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goal {
    /// `goal_id` — `String`.
    pub goal_id: String,
    /// `by` — `String`.
    pub by: String,
    /// `text` — `String`.
    pub text: String,
    /// `state` — `uilab.wire.GoalState`.
    pub state: GoalState,
    /// `steps` — `List<uilab.wire.GoalStep>`.
    pub steps: Vec<GoalStep>,
    /// `current` — `Optional<Integer>`.
    pub current: Option<i64>,
    /// `message` — `Optional<String>`.
    pub message: Option<String>,
}

/// GoalState — `uilab.wire.GoalState`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalState {
    /// `planning`.
    Planning,
    /// `running`.
    Running,
    /// `done`.
    Done,
    /// `stopped`.
    Stopped,
    /// `failed`.
    Failed,
}

/// GoalStep — `uilab.wire.GoalStep`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalStep {
    /// `instruction` — `String`.
    pub instruction: String,
    /// `target` — `uilab.session.NodePath`.
    pub target: crate::session::NodePath,
    /// `why` — `String`.
    pub why: String,
    /// `status` — `uilab.wire.StepStatus`.
    pub status: StepStatus,
    /// `proposal_id` — `Optional<uilab.session.ProposalId>`.
    pub proposal_id: Option<crate::session::ProposalId>,
}

/// Hello — `uilab.wire.Hello`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    /// `name` — `String`.
    pub name: String,
    /// `kind` — `uilab.wire.OperatorKind`.
    pub kind: OperatorKind,
}

/// Mic — `uilab.wire.Mic`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mic {
    /// `state` — `uilab.wire.MicState`.
    pub state: MicState,
    /// `workspace` — `Optional<uilab.wire.Workspace>`.
    pub workspace: Option<Workspace>,
}

/// MicState — `uilab.wire.MicState`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicState {
    /// `open`.
    Open,
    /// `closed`.
    Closed,
}

/// Operator — `uilab.wire.Operator`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator {
    /// `id` — `String`.
    pub id: String,
    /// `name` — `String`.
    pub name: String,
    /// `kind` — `uilab.wire.OperatorKind`.
    pub kind: OperatorKind,
    /// `last_seen_ms` — `Integer`.
    pub last_seen_ms: i64,
}

/// OperatorKind — `uilab.wire.OperatorKind`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorKind {
    /// `human`.
    Human,
    /// `agent`.
    Agent,
}

/// OutlineNode — `uilab.wire.OutlineNode`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineNode {
    /// `path` — `uilab.session.NodePath`.
    pub path: crate::session::NodePath,
    /// `layer` — `String`.
    pub layer: String,
    /// `name` — `String`.
    pub name: String,
    /// `kind` — `String`.
    pub kind: String,
    /// `title` — `Optional<String>`.
    pub title: Option<String>,
    /// `view` — `Optional<String>`.
    pub view: Option<String>,
    /// `props` — `Optional<Json>`.
    pub props: Option<crate::json::Value>,
    /// `children` — `List<uilab.wire.OutlineNode>`.
    pub children: Vec<OutlineNode>,
}

/// Presence — `uilab.wire.Presence`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presence {
    /// `operators` — `List<uilab.wire.Operator>`.
    pub operators: Vec<Operator>,
    /// `selected_by` — `Optional<String>`.
    pub selected_by: Option<String>,
}

/// ProposalShown — `uilab.wire.ProposalShown`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalShown {
    /// `by` — `Optional<String>`.
    pub by: Option<String>,
    /// `proposal_id` — `uilab.session.ProposalId`.
    pub proposal_id: crate::session::ProposalId,
    /// `target` — `uilab.session.NodePath`.
    pub target: crate::session::NodePath,
    /// `changed` — `uilab.session.NodePath`.
    pub changed: crate::session::NodePath,
    /// `op` — `uilab.session.PatchOp`.
    pub op: crate::session::PatchOp,
    /// `utterance` — `String`.
    pub utterance: String,
    /// `before` — `String`.
    pub before: String,
    /// `after` — `String`.
    pub after: String,
    /// `findings` — `List<uilab.wire.Finding>`.
    pub findings: Vec<Finding>,
    /// `outline` — `uilab.wire.OutlineNode`.
    pub outline: OutlineNode,
}

/// ReadRows — `uilab.wire.ReadRows`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadRows {
    /// `view` — `String`.
    pub view: String,
}

/// Refused — `uilab.wire.Refused`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    /// `by` — `Optional<String>`.
    pub by: Option<String>,
    /// `check` — `String`.
    pub check: String,
    /// `message` — `String`.
    pub message: String,
}

/// Resync — `uilab.wire.Resync`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resync {
    /// `revision` — `Integer`.
    pub revision: i64,
}

/// Rows — `uilab.wire.Rows`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rows {
    /// `view` — `String`.
    pub view: String,
    /// `total` — `Optional<Integer>`.
    pub total: Option<i64>,
    /// `rows` — `List<Json>`.
    pub rows: Vec<crate::json::Value>,
}

/// Say — `uilab.wire.Say`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Say {
    /// `text` — `String`.
    pub text: String,
    /// `target` — `Optional<uilab.session.NodePath>`.
    pub target: Option<crate::session::NodePath>,
    /// `review` — `Optional<Boolean>`.
    pub review: Option<bool>,
    /// `workspace` — `Optional<uilab.wire.Workspace>`.
    pub workspace: Option<Workspace>,
}

/// Select — `uilab.wire.Select`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Select {
    /// `path` — `uilab.session.NodePath`.
    pub path: crate::session::NodePath,
}

/// ServerMessage — `uilab.wire.ServerMessage`: one of a fixed set of shapes, tagged on the wire by `type`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerMessage {
    /// Tagged `changed` — `uilab.wire.Changed`.
    Changed(Changed),
    /// Tagged `document` — `uilab.wire.DocumentState`.
    Document(DocumentState),
    /// Tagged `failed` — `uilab.wire.Failed`.
    Failed(Failed),
    /// Tagged `goal` — `uilab.wire.Goal`.
    Goal(Goal),
    /// Tagged `presence` — `uilab.wire.Presence`.
    Presence(Presence),
    /// Tagged `proposal` — `uilab.wire.ProposalShown`.
    Proposal(ProposalShown),
    /// Tagged `refused` — `uilab.wire.Refused`.
    Refused(Refused),
    /// Tagged `rows` — `uilab.wire.Rows`.
    Rows(Rows),
    /// Tagged `thinking` — `uilab.wire.Thinking`.
    Thinking(Thinking),
    /// Tagged `transcript` — `uilab.wire.Transcript`.
    Transcript(Transcript),
}

/// Settings — `uilab.wire.Settings`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// `review` — `Boolean`.
    pub review: bool,
}

/// Severity — `uilab.wire.Severity`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// `error`.
    Error,
    /// `warning`.
    Warning,
}

/// StartGoal — `uilab.wire.StartGoal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartGoal {
    /// `text` — `String`.
    pub text: String,
    /// `target` — `Optional<uilab.session.NodePath>`.
    pub target: Option<crate::session::NodePath>,
    /// `max_steps` — `Optional<Integer>`.
    pub max_steps: Option<i64>,
    /// `review` — `Optional<Boolean>`.
    pub review: Option<bool>,
    /// `workspace` — `Optional<uilab.wire.Workspace>`.
    pub workspace: Option<Workspace>,
}

/// StepStatus — `uilab.wire.StepStatus`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    /// `pending`.
    Pending,
    /// `thinking`.
    Thinking,
    /// `proposed`.
    Proposed,
    /// `accepted`.
    Accepted,
    /// `rejected`.
    Rejected,
    /// `refused`.
    Refused,
    /// `declined`.
    Declined,
}

/// StopGoal — `uilab.wire.StopGoal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopGoal {
    /// `goal_id` — `String`.
    pub goal_id: String,
}

/// Thinking — `uilab.wire.Thinking`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thinking {
    /// `by` — `Optional<String>`.
    pub by: Option<String>,
    /// `target` — `uilab.session.NodePath`.
    pub target: crate::session::NodePath,
}

/// Transcript — `uilab.wire.Transcript`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transcript {
    /// `by` — `Optional<String>`.
    pub by: Option<String>,
    /// `text` — `String`.
    pub text: String,
    /// `audio_ms` — `Integer`.
    pub audio_ms: i64,
    /// `took_ms` — `Integer`.
    pub took_ms: i64,
}

/// Workspace — `uilab.wire.Workspace`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Workspace {
    /// `app`.
    App,
    /// `components`.
    Components,
}
