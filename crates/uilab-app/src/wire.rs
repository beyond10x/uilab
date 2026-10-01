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
    UilabSessionProposalId, UilabWireChanged, UilabWireDecide, UilabWireDocumentState,
    UilabWireFailed, UilabWireFinding, UilabWireGoal, UilabWireGoalStep, UilabWireHello,
    UilabWireMic, UilabWireMicState, UilabWireMoved, UilabWireOperator, UilabWireOperatorKind,
    UilabWireOutlineNode, UilabWirePresence, UilabWireProposalShown, UilabWireReadRows,
    UilabWireRefused, UilabWireResync, UilabWireRows, UilabWireSay, UilabWireSelect,
    UilabWireSettings, UilabWireStartGoal, UilabWireStopGoal, UilabWireThinking,
    UilabWireTranscript, UilabWireWorkspace,
};

use crate::goal::Goal;

/// Browser (or operator API) to server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Client {
    /// Select a node.
    Select(UilabWireSelect),
    /// Open or close the microphone; audio frames travel in between.
    Mic(UilabWireMic),
    /// A typed instruction, at the selection or at `target`.
    Say(UilabWireSay),
    /// Accept a proposal.
    Accept(UilabWireDecide),
    /// Reject a proposal.
    Reject(UilabWireDecide),
    /// Undo an accepted proposal.
    Undo(UilabWireDecide),
    /// Fixture rows of a view.
    Rows(UilabWireReadRows),
    /// Who is on this connection.
    Hello(UilabWireHello),
    /// Send a full snapshot; the sender missed a revision.
    Resync(UilabWireResync),
    /// Change the session settings for everybody.
    Settings(UilabWireSettings),
    /// A goal the agent plans into steps and carries out one proposal at a time.
    Goal(UilabWireStartGoal),
    /// Stop the running goal.
    StopGoal(UilabWireStopGoal),
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
    /// The agent moved the selection for an instruction that names another place.
    Moved(UilabWireMoved),
    /// Fixture rows of a view.
    Rows(UilabWireRows),
    /// Who is operating.
    Presence(UilabWirePresence),
    /// One accepted change, as a delta against the previous revision.
    Changed(UilabWireChanged),
    /// A goal run as it stands, sent whole on every change.
    Goal(UilabWireGoal),
}

impl Server {
    /// The message as one JSON text frame.
    pub fn to_text(&self) -> String {
        serde_json::to_string(self).expect("wire messages serialize")
    }

    /// A failure message.
    pub fn failed(message: impl Into<String>, by: Option<&str>) -> Self {
        Server::Failed(UilabWireFailed {
            message: message.into(),
            by: by_of(by),
        })
    }

    /// A refusal message.
    pub fn refused(check: impl Into<String>, message: impl Into<String>, by: Option<&str>) -> Self {
        Server::Refused(UilabWireRefused {
            check: check.into(),
            message: message.into(),
            by: by_of(by),
        })
    }

    /// The operator the message is attributed to, if any.
    pub fn by(&self) -> Option<&str> {
        let presence = match self {
            Server::Transcript(m) => &m.by,
            Server::Thinking(m) => &m.by,
            Server::Proposal(m) => &m.by,
            Server::Refused(m) => &m.by,
            Server::Failed(m) => &m.by,
            Server::Moved(m) => &m.by,
            Server::Changed(m) => return Some(&m.by),
            Server::Goal(m) => return Some(&m.by),
            Server::Document(_) | Server::Rows(_) | Server::Presence(_) => return None,
        };
        match presence {
            EssPresence::Present(by) => Some(by),
            EssPresence::Absent => None,
        }
    }
}

impl Client {
    /// Parses one JSON text frame.
    pub fn from_text(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

fn by_of(by: Option<&str>) -> EssPresence<String> {
    present(by.map(str::to_owned))
}

/// A generated enum value by its spec name. The generated variants are numbered in sorted
/// order, so picking one by index silently changes meaning when the spec gains a variant.
fn named<T: serde::de::DeserializeOwned>(name: &str) -> T {
    serde_json::from_value(Value::String(name.to_owned())).expect("a name the spec declares")
}

/// The spec name of a generated enum value.
fn name_of<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// Whether a mic message opens the microphone.
pub fn mic_open(mic: &UilabWireMic) -> bool {
    name_of::<UilabWireMicState>(&mic.state) == "open"
}

/// The workspace an instruction came from; the app canvas when the message names none.
pub fn workspace(sent: &EssPresence<Box<UilabWireWorkspace>>) -> uilab_agent::Workspace {
    match sent {
        EssPresence::Present(w) if name_of::<UilabWireWorkspace>(w) == "components" => {
            uilab_agent::Workspace::Components
        }
        _ => uilab_agent::Workspace::App,
    }
}

/// Whether a hello or operator is an agent.
pub fn is_agent(kind: &UilabWireOperatorKind) -> bool {
    name_of(kind) == "agent"
}

pub fn operator_kind(agent: bool) -> Box<UilabWireOperatorKind> {
    Box::new(named(if agent { "agent" } else { "human" }))
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

/// The generated enum by its spec name; its variants are numbered, so never pick one by index.
pub fn op(name: &str) -> Box<UilabSessionPatchOp> {
    Box::new(named(name))
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
                severity: Box::new(named(match f.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                })),
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
    pub revision: u64,
    pub review: bool,
}

pub fn document(parts: DocumentParts<'_>) -> Server {
    Server::Document(UilabWireDocumentState {
        document_id: document_id(parts.document_id),
        file: parts.file.to_owned(),
        findings: findings(parts.findings),
        outline: outline(parts.outline),
        revision: Number::from(parts.revision),
        review: parts.review,
        selected: node_path(parts.selected),
        title: present(parts.title),
        undoable: present(parts.undoable.map(|id| proposal_id(&id))),
    })
}

pub struct ChangedParts<'a> {
    pub revision: u64,
    pub by: &'a str,
    pub op: &'a str,
    pub changed: &'a str,
    pub parent: &'a str,
    pub node: Option<&'a OutlineNode>,
    pub findings: &'a [Finding],
}

pub fn changed(parts: ChangedParts<'_>) -> Server {
    Server::Changed(UilabWireChanged {
        by: parts.by.to_owned(),
        changed: node_path(parts.changed),
        findings: findings(parts.findings),
        node: present(parts.node.map(outline)),
        op: op(parts.op),
        parent: node_path(parts.parent),
        revision: Number::from(parts.revision),
    })
}

pub struct OperatorParts {
    pub id: String,
    pub name: String,
    pub agent: bool,
    pub last_seen_ms: u64,
}

pub fn presence(operators: Vec<OperatorParts>, selected_by: Option<String>) -> Server {
    Server::Presence(UilabWirePresence {
        operators: operators
            .into_iter()
            .map(|o| {
                Box::new(UilabWireOperator {
                    id: o.id,
                    kind: operator_kind(o.agent),
                    last_seen_ms: Number::from(o.last_seen_ms),
                    name: o.name,
                })
            })
            .collect(),
        selected_by: present(selected_by),
    })
}

pub fn transcript(text: &str, audio_ms: u64, took_ms: u64, by: Option<&str>) -> Server {
    Server::Transcript(UilabWireTranscript {
        text: text.to_owned(),
        audio_ms: Number::from(audio_ms),
        took_ms: Number::from(took_ms),
        by: by_of(by),
    })
}

pub struct MovedParts<'a> {
    /// The operator whose instruction it was.
    pub by: &'a str,
    /// Who moved the selection: the agent.
    pub selected_by: &'a str,
    pub from: &'a str,
    pub to: &'a str,
    pub reason: &'a str,
    pub navigate_only: bool,
    pub utterance: &'a str,
}

pub fn moved(parts: MovedParts<'_>) -> Server {
    Server::Moved(UilabWireMoved {
        by: by_of(Some(parts.by)),
        from: node_path(parts.from),
        navigate_only: parts.navigate_only,
        reason: parts.reason.to_owned(),
        selected_by: parts.selected_by.to_owned(),
        to: node_path(parts.to),
        utterance: parts.utterance.to_owned(),
    })
}

pub fn thinking(target: &str, by: Option<&str>) -> Server {
    Server::Thinking(UilabWireThinking {
        target: node_path(target),
        by: by_of(by),
    })
}

/// A goal run as the `goal` message carries it.
pub fn goal(goal: &Goal) -> Server {
    Server::Goal(UilabWireGoal {
        by: goal.by.clone(),
        current: present(goal.current.map(|i| Number::from(i as u64))),
        goal_id: goal.id.clone(),
        message: present(goal.message.clone()),
        state: Box::new(named(goal.state.name())),
        steps: goal
            .steps
            .iter()
            .map(|s| {
                Box::new(UilabWireGoalStep {
                    instruction: s.instruction.clone(),
                    proposal_id: present(s.proposal_id.as_deref().map(proposal_id)),
                    status: Box::new(named(s.status.name())),
                    target: node_path(&s.target.to_string()),
                    why: s.why.clone(),
                })
            })
            .collect(),
        text: goal.text.clone(),
    })
}

/// Whether a `goal` message says the run is over: done, stopped or failed.
pub fn goal_ended(goal: &UilabWireGoal) -> bool {
    matches!(goal_state(goal).as_str(), "done" | "stopped" | "failed")
}

/// The spec name of a `goal` message's state.
pub fn goal_state(goal: &UilabWireGoal) -> String {
    name_of(&goal.state)
}

/// The rows of a view; `sample` when they are made up because no fixture answers it.
pub fn rows(view: &str, total: Option<u64>, rows: Vec<Value>, sample: bool) -> Server {
    Server::Rows(UilabWireRows {
        view: view.to_owned(),
        total: present(total.map(Number::from)),
        rows,
        sample: EssPresence::Present(sample),
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
        uilab_doc::outline::rendered(&uilab_doc::Document::from_yaml(&text).unwrap())
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
                revision: 3,
                review: true,
            }),
            document(DocumentParts {
                document_id: "d",
                file: "f",
                title: None,
                selected: "/",
                outline: &tree,
                findings: &[],
                undoable: None,
                revision: 0,
                review: false,
            }),
            transcript("add a table", 2100, 340, Some("op-1")),
            transcript("add a table", 2100, 340, None),
            thinking("page:loans", Some("op-2")),
            Server::Proposal(UilabWireProposalShown {
                after: "a".into(),
                before: "".into(),
                by: by_of(Some("op-2")),
                changed: node_path("page:loans/section:overdue"),
                findings: findings(std::slice::from_ref(&finding)),
                op: op("Insert"),
                outline: outline(&tree),
                proposal_id: proposal_id("p"),
                target: node_path("page:loans"),
                utterance: "u".into(),
            }),
            Server::refused("name_unique", "m", Some("op-1")),
            Server::failed("m", None),
            moved(MovedParts {
                by: "op-1",
                selected_by: "agent",
                from: "page:loans/section:list",
                to: "/",
                reason: "a new page goes under the root",
                navigate_only: false,
                utterance: "create a new page",
            }),
            moved(MovedParts {
                by: "op-2",
                selected_by: "op-2",
                from: "page:loans",
                to: "page:members",
                reason: "r",
                navigate_only: true,
                utterance: "go to the members page",
            }),
            rows(
                "loans.All",
                Some(4),
                vec![serde_json::json!({"title": "x"})],
                false,
            ),
            rows("loans.X", None, vec![], true),
            presence(
                vec![
                    OperatorParts {
                        id: "op-1".into(),
                        name: "Timo".into(),
                        agent: false,
                        last_seen_ms: 0,
                    },
                    OperatorParts {
                        id: "op-2".into(),
                        name: "Claude".into(),
                        agent: true,
                        last_seen_ms: 1200,
                    },
                ],
                Some("op-2".into()),
            ),
            presence(vec![], None),
            changed(ChangedParts {
                revision: 4,
                by: "op-2",
                op: "Insert",
                changed: "page:loans/section:overdue",
                parent: "page:loans",
                node: Some(&tree.children[0]),
                findings: std::slice::from_ref(&finding),
            }),
            changed(ChangedParts {
                revision: 5,
                by: "op-2",
                op: "Remove",
                changed: "page:loans/section:overdue",
                parent: "page:loans",
                node: None,
                findings: &[],
            }),
        ];
        let mut messages = messages;
        messages.extend(goal_fixtures().iter().map(goal));
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
            r#"{"type":"say","value":{"text":"add a table","target":"page:loans"}}"#,
            r#"{"type":"accept","value":{"proposal_id":"p"}}"#,
            r#"{"type":"reject","value":{"proposal_id":"p"}}"#,
            r#"{"type":"undo","value":{"proposal_id":"p"}}"#,
            r#"{"type":"rows","value":{"view":"loans.All"}}"#,
            r#"{"type":"hello","value":{"name":"Claude","kind":"agent"}}"#,
            r#"{"type":"resync","value":{"revision":2}}"#,
            r#"{"type":"settings","value":{"review":true}}"#,
            r#"{"type":"say","value":{"text":"t","target":"page:loans","review":true}}"#,
            r#"{"type":"goal","value":{"text":"build out the member area"}}"#,
            r#"{"type":"goal","value":{"text":"t","target":"page:members","max_steps":4,"review":false}}"#,
            r#"{"type":"stop_goal","value":{"goal_id":"goal-1"}}"#,
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
        assert!(is_agent(match &Client::from_text(texts[9]).unwrap() {
            Client::Hello(h) => &h.kind,
            _ => unreachable!(),
        }));
        assert!(Client::from_text(r#"{"type":"shout","value":{}}"#).is_err());
    }

    #[test]
    fn messages_name_their_operator() {
        assert_eq!(Server::refused("c", "m", Some("op-1")).by(), Some("op-1"));
        assert_eq!(Server::failed("m", None).by(), None);
        assert_eq!(thinking("/", Some("op-2")).by(), Some("op-2"));
        assert_eq!(goal(&goal_fixtures()[0]).by(), Some("api-1"));
        let by_agent = moved(MovedParts {
            by: "op-1",
            selected_by: "agent",
            from: "/",
            to: "nav",
            reason: "r",
            navigate_only: false,
            utterance: "u",
        });
        assert_eq!(
            by_agent.by(),
            Some("op-1"),
            "a move is the instruction's operator's"
        );
    }

    /// A goal in each state the runner reaches: planning, running mid-way, done, stopped, failed.
    fn goal_fixtures() -> Vec<Goal> {
        let step = |instruction: &str, target: &str| uilab_agent::Step {
            instruction: instruction.into(),
            target: target.parse().unwrap(),
            why: "w".into(),
        };
        let fresh = || {
            Goal::new(
                "goal-1",
                "api-1",
                "build out the member area",
                "page:members".parse().unwrap(),
                8,
                true,
            )
        };
        let planning = fresh();
        let mut running = fresh();
        running.planned(vec![
            step("add a list", "page:members"),
            step("add a card", "page:members/section:list"),
        ]);
        running.proposed(0, "p1");
        let mut done = running.clone();
        done.decided("p1", true);
        done.not_proposed(1, true);
        let mut stopped = running.clone();
        stopped.stop();
        let mut failed = fresh();
        failed.plan_failed("declined: a greeting");
        vec![planning, running, done, stopped, failed]
    }

    #[test]
    fn a_goal_message_carries_the_run() {
        let [planning, running, done, stopped, failed] = goal_fixtures().try_into().unwrap();
        let value = |g: &Goal| serde_json::to_value(goal(g)).unwrap()["value"].clone();

        let v = value(&running);
        assert_eq!(v["goal_id"], "goal-1");
        assert_eq!(v["by"], "api-1");
        assert_eq!(v["text"], "build out the member area");
        assert_eq!(v["state"], "running");
        assert_eq!(v["current"], 0);
        assert_eq!(v["steps"][0]["status"], "proposed");
        assert_eq!(v["steps"][0]["proposal_id"], "p1");
        assert_eq!(v["steps"][0]["target"], "page:members");
        assert_eq!(v["steps"][1]["status"], "pending");
        assert_eq!(v["steps"][1]["target"], "page:members/section:list");
        assert!(v["steps"][1].get("proposal_id").is_none());
        assert!(v.get("message").is_none());

        let v = value(&planning);
        assert_eq!(v["state"], "planning");
        assert!(v.get("current").is_none());
        assert_eq!(v["steps"], serde_json::json!([]));

        assert_eq!(value(&done)["state"], "done");
        assert_eq!(value(&done)["steps"][1]["status"], "declined");
        assert_eq!(value(&stopped)["state"], "stopped");
        assert_eq!(value(&stopped)["steps"][0]["status"], "rejected");
        assert_eq!(value(&failed)["state"], "failed");
        assert_eq!(value(&failed)["message"], "declined: a greeting");
    }

    #[test]
    fn only_done_stopped_and_failed_end_a_goal() {
        let ended: Vec<bool> = goal_fixtures()
            .iter()
            .map(|g| match goal(g) {
                Server::Goal(m) => goal_ended(&m),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(ended, [false, false, true, true, true]);
    }

    /// story:essui-app-widget `finding_carries_ess_check_id`: a section naming a widget the
    /// document does not declare is ESS's `widget_expands`, and the wire finding carries that id on
    /// the uilab path of the section. The document is read past ESS's loader, which refuses it,
    /// the way a patch's result is checked before it is admitted.
    #[test]
    fn finding_carries_ess_check_id() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/library/library.ui.yaml"
        ))
        .unwrap();
        let anchor = "    sections:\n      - name: list\n        component: collection\n        reads: {view: loans.All, paging: server}\n";
        assert!(text.contains(anchor), "fixture anchor is missing");
        let text = text.replacen(
            anchor,
            &format!(
                "    sections:\n      - {{name: ghost, component: no_such_widget}}\n{}",
                &anchor["    sections:\n".len()..]
            ),
            1,
        );
        assert!(
            uilab_doc::Document::from_yaml(&text).is_err(),
            "ESS's loader refuses the document"
        );
        let doc: uilab_doc::Document = serde_yaml::from_str(&text).unwrap();
        let sent = findings(&uilab_doc::check(&doc));
        let found: Vec<(&str, &str)> = sent
            .iter()
            .filter(|f| f.check == "widget_expands")
            .map(|f| (f.check.as_str(), f.path.as_str()))
            .collect();
        assert_eq!(
            found,
            [("widget_expands", "page:loans/section:ghost")],
            "{sent:?}"
        );
        let value = serde_json::to_value(&sent[0]).unwrap();
        assert!(
            serde_json::from_value::<UilabWireFinding>(value).is_ok(),
            "the finding is the generated wire type"
        );
    }

    /// The outline a browser is shown carries the inherited mark through the generated type: the
    /// library's Loans page holds its kind's `filters` section, marked.
    #[test]
    fn the_wire_outline_carries_the_inherited_mark() {
        let wire = serde_json::to_value(outline(&outline_fixture())).unwrap();
        let loans = wire["children"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["path"] == "page:loans")
            .unwrap();
        let filters = loans["children"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["path"] == "page:loans/section:filters")
            .expect("the kind's filters section is shown");
        assert_eq!(filters["inherited"], true);
        assert!(loans.get("inherited").is_none());
    }
}
