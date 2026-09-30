//! The agent builds and uses widgets, against a scripted model port: no network.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

use harness_wire::{
    CallId, Item, ModelPort, StopReason, StreamSink, ToolCall, ToolName, TurnOutcome, TurnRequest,
    Usage, WireError, WireId,
};
use serde_json::{Value, json};
use uilab_agent::{INSTRUCTIONS, PLAN_INSTRUCTIONS, Proposer, ProposerConfig};
use uilab_doc::model::{CompositeKind, PrimitiveKind};
use uilab_doc::{Document, NodePath, Patch};

fn library() -> Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    Document::from_yaml(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Answers each turn with the next scripted `answer` call and keeps every request it was sent.
struct ScriptedModel {
    wire: WireId,
    answers: VecDeque<Value>,
    seen: Arc<Mutex<Vec<TurnRequest>>>,
}

impl ModelPort for ScriptedModel {
    fn wire(&self) -> &WireId {
        &self.wire
    }

    fn turn(
        &mut self,
        request: &TurnRequest,
        _sink: &mut dyn StreamSink,
    ) -> Result<TurnOutcome, WireError> {
        request.validate()?;
        let mut seen = self.seen.lock().unwrap();
        seen.push(request.clone());
        let arguments = self
            .answers
            .pop_front()
            .ok_or_else(|| WireError::protocol("the script ran out of turns"))?;
        Ok(TurnOutcome {
            stop_reason: StopReason::ToolCalls,
            items: vec![Item::ToolCall(ToolCall {
                call_id: CallId::new(format!("call-{}", seen.len())).unwrap(),
                name: ToolName::new("answer").unwrap(),
                arguments,
            })],
            usage: Some(Usage {
                model: request.model.clone(),
                input_tokens: 100,
                output_tokens: 20,
                cached_input_tokens: 0,
                cache_creation_input_tokens: None,
            }),
        })
    }
}

fn proposer(answers: Vec<Value>) -> (Proposer, Arc<Mutex<Vec<TurnRequest>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let model = ScriptedModel {
        wire: WireId::new("anthropic-messages").unwrap(),
        answers: answers.into(),
        seen: Arc::clone(&seen),
    };
    (
        Proposer::with_port(&ProposerConfig::default(), Box::new(model)),
        seen,
    )
}

fn texts(request: &TurnRequest) -> String {
    request
        .items
        .iter()
        .filter_map(|item| match item {
            Item::UserText { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

const CARD_SUMMARY: &str = "A member as a card with their name and standing.";

/// The inner patch that declares `member_card` at the root.
fn declare_member_card() -> Value {
    json!({
        "op": "insert",
        "target": "/",
        "child": {
            "layer": "component",
            "name": "member_card",
            "node": {
                "summary": CARD_SUMMARY,
                "params": {
                    "member": {"type": "Member", "required": true, "note": "the member row"},
                    "compact": {"type": "boolean", "default": false}
                },
                "arrange": "column",
                "body": [
                    {"name": "name", "primitive": "text", "text": "args.member.name", "style": "heading"},
                    {"name": "standing", "primitive": "badge", "text": "args.member.standing",
                     "tone_by": {"value": "args.member.standing", "map": {"good": "success", "overdue": "danger"}}}
                ]
            }
        }
    })
}

/// A batch at the root: declare `member_card`, then put an instance of it in each row of the
/// members list.
fn member_card_batch() -> Value {
    json!({
        "op": "batch",
        "target": "/",
        "patches": [
            declare_member_card(),
            {
                "op": "insert",
                "target": "page:members/section:list",
                "child": {
                    "layer": "item",
                    "name": "card",
                    "node": {"component": "member_card", "args": {"member": "row"}}
                }
            }
        ]
    })
}

/// The library with `member_card` declared and used, as the batch leaves it.
fn library_with_member_card() -> Document {
    let batch: Patch = serde_json::from_value(member_card_batch()).unwrap();
    uilab_doc::admit(&library(), &batch).unwrap().0
}

#[test]
fn an_answer_declaring_a_widget_and_using_it_is_admitted() {
    let doc = library();
    let (mut proposer, seen) = proposer(vec![member_card_batch()]);
    let proposal = proposer
        .propose(
            &doc,
            &NodePath::root(),
            "make a reusable card for a member with name and standing and use it in the members table",
        )
        .unwrap();

    assert_eq!(proposal.attempts, 1, "admitted on the first answer");
    let Patch::Batch { patches, .. } = &proposal.patch else {
        panic!("a batch: {:?}", proposal.patch);
    };
    assert_eq!(patches.len(), 2);
    let (next, _) =
        uilab_doc::admit(&doc, &proposal.patch).expect("the returned patch is admitted");
    let card = &next.widgets["member_card"];
    assert_eq!(card.summary, CARD_SUMMARY);
    assert!(card.params["member"].is_required());
    assert!(
        uilab_doc::resolve(
            &next,
            &"component:member_card/node:standing".parse().unwrap()
        )
        .is_ok()
    );
    let item = uilab_doc::resolve(
        &next,
        &"page:members/section:list/item:card".parse().unwrap(),
    )
    .expect("the instance sits in the members list");
    assert_eq!(
        item.composite().unwrap().component.widget(),
        Some("member_card")
    );

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    let offered = &seen[0].tools[0].input_schema["properties"]["child"]["oneOf"];
    assert!(
        offered
            .as_array()
            .unwrap()
            .iter()
            .any(|variant| variant["properties"]["layer"]["const"] == "component"),
        "the root's patch schema offers a widget: {offered}"
    );
}

#[test]
fn the_request_lists_the_widgets_the_document_declares() {
    let doc = library_with_member_card();
    let members: NodePath = "page:members".parse().unwrap();
    let use_it = json!({
        "op": "insert",
        "target": "page:members",
        "child": {
            "layer": "section",
            "name": "featured",
            "node": {"component": "member_card", "args": {"member": "rows.first"}}
        }
    });
    let (mut proposer, seen) = proposer(vec![use_it]);
    proposer
        .propose(&doc, &members, "show the first member as a card")
        .unwrap();
    let user = texts(&seen.lock().unwrap()[0]);
    assert!(user.contains("Widgets the document declares:"), "{user}");
    assert!(user.contains("member_card"), "{user}");
    assert!(user.contains(CARD_SUMMARY), "{user}");
    assert!(user.contains("member (Member, required)"), "{user}");
    assert!(user.contains("compact (boolean)"), "{user}");
    assert!(
        !user.contains("compact (boolean, required)"),
        "an optional param is not called required: {user}"
    );
}

#[test]
fn the_request_says_when_no_widget_is_declared() {
    let doc = library();
    assert!(doc.widgets.is_empty(), "the library declares no widget");
    let (mut proposer, seen) = proposer(vec![member_card_batch()]);
    proposer
        .propose(&doc, &NodePath::root(), "make a reusable member card")
        .unwrap();
    let user = texts(&seen.lock().unwrap()[0]);
    assert!(
        user.contains("Widgets the document declares: none"),
        "{user}"
    );
}

#[test]
fn the_plan_request_lists_the_widgets_the_document_declares() {
    let doc = library_with_member_card();
    let plan = json!({"op": "plan", "steps": [
        {"instruction": "show the first member as a member_card", "target": "page:members", "why": "a card"}
    ]});
    let (mut proposer, seen) = proposer(vec![plan]);
    proposer
        .plan_goal(
            &doc,
            &"page:members".parse().unwrap(),
            "feature a member",
            4,
        )
        .unwrap();
    let user = texts(&seen.lock().unwrap()[0]);
    assert!(user.contains("Widgets the document declares:"), "{user}");
    assert!(user.contains("member_card"), "{user}");
    assert!(user.contains("member (Member, required)"), "{user}");
}

#[test]
fn the_instructions_explain_widgets_params_args_and_every_primitive() {
    for needle in [
        "widgets:",
        "layer `component`",
        "summary",
        "params",
        "body",
        "args.<param>",
        "{component: <widget>, args:",
        "reusable",
        "card for each",
        "layer `node`",
    ] {
        assert!(
            INSTRUCTIONS.contains(needle),
            "INSTRUCTIONS lack `{needle}`"
        );
    }
    for kind in PrimitiveKind::ALL {
        let named = format!("`{}`", kind.as_str());
        assert!(
            INSTRUCTIONS.contains(&named),
            "INSTRUCTIONS do not name the primitive {named}"
        );
    }
    for kind in CompositeKind::ALL {
        let listed = format!("- {}:", kind.as_str());
        assert!(
            INSTRUCTIONS.contains(&listed),
            "INSTRUCTIONS do not list the composite kind `{}`",
            kind.as_str()
        );
    }
    for needle in ["widget", "layer `component`", "{component: <widget>, args:"] {
        assert!(
            PLAN_INSTRUCTIONS.contains(needle),
            "PLAN_INSTRUCTIONS lack `{needle}`"
        );
    }
}
