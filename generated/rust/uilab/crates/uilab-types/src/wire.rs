// generated from uilab v1
// model digest 1a75f0421d837e2ee884066ba29e42b368e91a0d2fb46630399bf48ac98fa437
// contract digest 1419f72b14d91b0b3ee89f6e567db8160920e8728d07d1772d1e9b0cc5848690
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

/// Severity — `uilab.wire.Severity`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// `error`.
    Error,
    /// `warning`.
    Warning,
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
