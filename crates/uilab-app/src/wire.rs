//! The WebSocket messages, over the payload types generated from `uilab.wire`.
//!
//! The payloads are the generated structs. The two tagged enums are written here because the
//! generated unions decode by trying each shape in turn and name their variants `V0…`; the test at
//! the bottom holds these enums to the generated unions, so the wire cannot drift from the spec.

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use uilab_doc::{Finding, OutlineNode, Severity};
use uilab_wire::{
    EssPresence, UilabSessionDocumentId, UilabSessionNodePath, UilabSessionPatchOp,
    UilabSessionProposalId, UilabWireDecide, UilabWireDocumentState, UilabWireFailed,
    UilabWireFinding, UilabWireMic, UilabWireMicState, UilabWireOutlineNode,
    UilabWireProposalShown, UilabWireReadRows, UilabWireRefused, UilabWireRows, UilabWireSay,
    UilabWireSelect, UilabWireSeverity, UilabWireThinking, UilabWireTranscript,
};

/// Browser to server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Client {
    /// Select a node.
    Select(UilabWireSelect),
    /// Open or close the microphone; audio frames travel in between.
    Mic(UilabWireMic),
    /// A typed instruction.
    Say(UilabWireSay),
    /// Accept a proposal.
    Accept(UilabWireDecide),
    /// Reject a proposal.
    Reject(UilabWireDecide),
    /// Undo an accepted proposal.
    Undo(UilabWireDecide),
    /// Fixture rows of a view.
    Rows(UilabWireReadRows),
}

/// Server to browser.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Server {
    /// The document as it stands.
    Document(UilabWireDocumentState),
    /// What the speech model heard.
    Transcript(UilabWireTranscript),
    /// The agent is working at a node.
    Thinking(UilabWireThinking),
    /// A patch waiting for the operator.
    Proposal(UilabWireProposalShown),
    /// A patch or decision was refused by a check.
    Refused(UilabWireRefused),
    /// Something failed that is not a check.
    Failed(UilabWireFailed),
    /// Fixture rows of a view.
    Rows(UilabWireRows),
}

impl Server {
    /// The message as one JSON text frame.
    pub fn to_text(&self) -> String {
        serde_json::to_string(self).expect("wire messages serialize")
    }

    /// A failure message.
    pub fn failed(message: impl Into<String>) -> Self {
        Server::Failed(UilabWireFailed {
            message: message.into(),
        })
    }

    /// A refusal message.
    pub fn refused(check: impl Into<String>, message: impl Into<String>) -> Self {
        Server::Refused(UilabWireRefused {
            check: check.into(),
            message: message.into(),
        })
    }
}

impl Client {
    /// Parses one JSON text frame.
    pub fn from_text(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

/// Whether a mic message opens the microphone.
pub fn mic_open(mic: &UilabWireMic) -> bool {
    matches!(*mic.state, UilabWireMicState::V1)
}

pub fn node_path(path: &str) -> Box<UilabSessionNodePath> {
    Box::new(UilabSessionNodePath(path.to_owned()))
}

pub fn proposal_id(id: &str) -> Box<UilabSessionProposalId> {
    Box::new(UilabSessionProposalId(id.to_owned()))
}

pub fn document_id(id: &str) -> Box<UilabSessionDocumentId> {
    Box::new(UilabSessionDocumentId(id.to_owned()))
}

pub fn op(name: &str) -> Box<UilabSessionPatchOp> {
    Box::new(match name {
        "Insert" => UilabSessionPatchOp::V0,
        "Remove" => UilabSessionPatchOp::V1,
        _ => UilabSessionPatchOp::V2,
    })
}

fn present<T>(value: Option<T>) -> EssPresence<T> {
    value.map_or(EssPresence::Absent, EssPresence::Present)
}

/// The outline, through the generated type: a shape the spec does not allow fails here.
pub fn outline(node: &OutlineNode) -> Box<UilabWireOutlineNode> {
    let value = serde_json::to_value(node).expect("an outline serializes");
    Box::new(serde_json::from_value(value).expect("the outline matches uilab.wire.OutlineNode"))
}

/// The generated payloads hold `Vec<Box<Finding>>`, so the box is theirs.
#[allow(clippy::vec_box)]
pub fn findings(findings: &[Finding]) -> Vec<Box<UilabWireFinding>> {
    findings
        .iter()
        .map(|f| {
            Box::new(UilabWireFinding {
                check: f.check.to_owned(),
                message: f.message.clone(),
                path: f.path.clone(),
                severity: Box::new(match f.severity {
                    Severity::Error => UilabWireSeverity::V0,
                    Severity::Warning => UilabWireSeverity::V1,
                }),
            })
        })
        .collect()
}

pub struct DocumentParts<'a> {
    pub document_id: &'a str,
    pub file: &'a str,
    pub title: Option<String>,
    pub selected: &'a str,
    pub outline: &'a OutlineNode,
    pub findings: &'a [Finding],
    pub undoable: Option<String>,
}

pub fn document(parts: DocumentParts<'_>) -> Server {
    Server::Document(UilabWireDocumentState {
        document_id: document_id(parts.document_id),
        file: parts.file.to_owned(),
        findings: findings(parts.findings),
        outline: outline(parts.outline),
        selected: node_path(parts.selected),
        title: present(parts.title),
        undoable: present(parts.undoable.map(|id| proposal_id(&id))),
    })
}

pub fn transcript(text: &str, audio_ms: u64, took_ms: u64) -> Server {
    Server::Transcript(UilabWireTranscript {
        text: text.to_owned(),
        audio_ms: Number::from(audio_ms),
        took_ms: Number::from(took_ms),
    })
}

pub fn thinking(target: &str) -> Server {
    Server::Thinking(UilabWireThinking {
        target: node_path(target),
    })
}

pub fn rows(view: &str, total: Option<u64>, rows: Vec<Value>) -> Server {
    Server::Rows(UilabWireRows {
        view: view.to_owned(),
        total: present(total.map(Number::from)),
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uilab_wire::{UilabWireClientMessage, UilabWireServerMessage};

    fn outline_fixture() -> OutlineNode {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/library/library.ui.yaml"
        ))
        .unwrap();
        uilab_doc::outline(&uilab_doc::Document::from_yaml(&text).unwrap())
    }

    /// Every server message this crate builds decodes as the generated union, and back.
    #[test]
    fn server_messages_are_the_generated_union() {
        let tree = outline_fixture();
        let finding = Finding {
            check: "draft_read",
            severity: Severity::Warning,
            path: "page:loans".into(),
            message: "m".into(),
        };
        let messages = vec![
            document(DocumentParts {
                document_id: "d",
                file: "f.ui.yaml",
                title: Some("t".into()),
                selected: "/",
                outline: &tree,
                findings: std::slice::from_ref(&finding),
                undoable: Some("p".into()),
            }),
            document(DocumentParts {
                document_id: "d",
                file: "f",
                title: None,
                selected: "/",
                outline: &tree,
                findings: &[],
                undoable: None,
            }),
            transcript("add a table", 2100, 340),
            thinking("page:loans"),
            Server::Proposal(UilabWireProposalShown {
                after: "a".into(),
                before: "".into(),
                changed: node_path("page:loans/section:overdue"),
                findings: findings(&[finding]),
                op: op("Insert"),
                outline: outline(&tree),
                proposal_id: proposal_id("p"),
                target: node_path("page:loans"),
                utterance: "u".into(),
            }),
            Server::refused("name_unique", "m"),
            Server::failed("m"),
            rows(
                "loans.All",
                Some(4),
                vec![serde_json::json!({"title": "x"})],
            ),
            rows("draft.X", None, vec![]),
        ];
        for message in messages {
            let text = message.to_text();
            let generated: UilabWireServerMessage =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
            let again = serde_json::to_string(&generated).unwrap();
            assert_eq!(
                serde_json::from_str::<Server>(&again).unwrap(),
                message,
                "{text}"
            );
        }
    }

    /// Every client message the generated union accepts parses here, and the other way round.
    #[test]
    fn client_messages_are_the_generated_union() {
        let texts = [
            r#"{"type":"select","value":{"path":"page:loans"}}"#,
            r#"{"type":"mic","value":{"state":"open"}}"#,
            r#"{"type":"mic","value":{"state":"closed"}}"#,
            r#"{"type":"say","value":{"text":"add a page"}}"#,
            r#"{"type":"accept","value":{"proposal_id":"p"}}"#,
            r#"{"type":"reject","value":{"proposal_id":"p"}}"#,
            r#"{"type":"undo","value":{"proposal_id":"p"}}"#,
            r#"{"type":"rows","value":{"view":"loans.All"}}"#,
        ];
        for text in texts {
            let ours = Client::from_text(text).unwrap_or_else(|e| panic!("{text}: {e}"));
            let generated: UilabWireClientMessage =
                serde_json::from_str(text).unwrap_or_else(|e| panic!("{text}: {e}"));
            assert_eq!(
                serde_json::to_value(&ours).unwrap(),
                serde_json::to_value(&generated).unwrap(),
                "{text}"
            );
        }
        assert!(mic_open(match &Client::from_text(texts[1]).unwrap() {
            Client::Mic(m) => m,
            _ => unreachable!(),
        }));
        assert!(Client::from_text(r#"{"type":"shout","value":{}}"#).is_err());
    }
}
