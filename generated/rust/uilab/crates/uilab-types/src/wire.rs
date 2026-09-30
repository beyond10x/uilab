// generated from uilab v1
// model digest 55369e6e2fd022d062872af44b9c252bf6b80b0b82da849d828e32c4bf780a9f
// contract digest c7e76b5222a53b2b047350c46d9086d71ebcc13314a0ee6ce79ea111afb83928
// do not edit: regenerate with `ess synthesize`

//! wire — `uilab.wire`.
//!
//! What travels between the browser and the uilab server over one WebSocket. Audio travels as binary frames of 16 kHz mono little-endian f32 samples between `mic` open and `mic` closed; everything else is one JSON message per text frame, tagged by `type`.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// ClientMessage — `uilab.wire.ClientMessage`: one of a fixed set of shapes, tagged on the wire by `type`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientMessage {
    /// Tagged `accept` — `uilab.wire.Decide`.
    Accept(Decide),
    /// Tagged `mic` — `uilab.wire.Mic`.
    Mic(Mic),
    /// Tagged `reject` — `uilab.wire.Decide`.
    Reject(Decide),
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
}

/// Failed — `uilab.wire.Failed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failed {
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

/// ProposalShown — `uilab.wire.ProposalShown`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalShown {
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
    /// `check` — `String`.
    pub check: String,
    /// `message` — `String`.
    pub message: String,
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
    /// Tagged `document` — `uilab.wire.DocumentState`.
    Document(DocumentState),
    /// Tagged `failed` — `uilab.wire.Failed`.
    Failed(Failed),
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
    /// `target` — `uilab.session.NodePath`.
    pub target: crate::session::NodePath,
}

/// Transcript — `uilab.wire.Transcript`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transcript {
    /// `text` — `String`.
    pub text: String,
    /// `audio_ms` — `Integer`.
    pub audio_ms: i64,
    /// `took_ms` — `Integer`.
    pub took_ms: i64,
}
