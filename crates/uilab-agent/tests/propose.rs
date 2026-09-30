//! The proposer against a scripted model port: no network.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

use harness_wire::{
    CallId, Item, ModelPort, StopReason, StreamSink, ToolCall, ToolName, TurnOutcome, TurnRequest,
    Usage, WireError, WireId,
};
use serde_json::{Value, json};
use uilab_agent::{ProposeError, Proposer, ProposerConfig};
use uilab_doc::{Document, NodePath, Patch};

fn library() -> Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    Document::from_yaml(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn loans() -> NodePath {
    "page:loans".parse().unwrap()
}

/// Answers each turn with the next scripted `answer` call and keeps every request it was sent.
struct ScriptedModel {
    wire: WireId,
    answers: VecDeque<Value>,
    seen: Arc<Mutex<Vec<TurnRequest>>>,
}

impl ScriptedModel {
    fn new(answers: Vec<Value>) -> (Self, Arc<Mutex<Vec<TurnRequest>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let model = Self {
            wire: WireId::new("anthropic-messages").unwrap(),
            answers: answers.into(),
            seen: Arc::clone(&seen),
        };
        (model, seen)
    }
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
    let (model, seen) = ScriptedModel::new(answers);
    (
        Proposer::with_port(&ProposerConfig::default(), Box::new(model)),
        seen,
    )
}

fn overdue_section() -> Value {
    json!({
        "op": "insert",
        "target": "page:loans",
        "child": {
            "layer": "section",
            "name": "overdue",
            "node": {
                "component": "collection",
                "title": "Overdue loans",
                "reads": {"view": "loans.All", "params": {"state": "overdue"}},
                "columns": [{"field": "title"}, {"field": "member"}, {"field": "due"}]
            }
        }
    })
}

/// Valid against the patch schema, refused by `opens_resolves`: the page has no `nowhere` overlay.
fn dangling_opens() -> Value {
    json!({
        "op": "insert",
        "target": "page:loans",
        "child": {
            "layer": "section",
            "name": "overdue",
            "node": {
                "component": "collection",
                "reads": {"view": "loans.All"},
                "row_actions": [{"opens": "nowhere", "label": "Remind"}]
            }
        }
    })
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

#[test]
fn a_valid_answer_comes_back_admitted() {
    let doc = library();
    let (mut proposer, seen) = proposer(vec![overdue_section()]);
    let proposal = proposer
        .propose(&doc, &loans(), "um add a table of uh overdue loans")
        .unwrap();

    assert_eq!(proposal.attempts, 1);
    assert_eq!(proposal.turns, 1);
    assert_eq!(proposal.cost_micro_usd, None, "no rate card, no price");
    let Patch::Insert { target, child } = &proposal.patch else {
        panic!("an insert: {:?}", proposal.patch);
    };
    assert_eq!(target, &loans());
    assert_eq!(child.name, "overdue");
    uilab_doc::admit(&doc, &proposal.patch).expect("the returned patch is admitted");

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    let request = &seen[0];
    // The only tool is the loop's `answer`, published with the node's patch schema.
    assert_eq!(request.tools.len(), 1, "{:?}", request.tools);
    assert_eq!(request.tools[0].name.as_str(), "answer");
    assert_eq!(
        request.tools[0].input_schema,
        uilab_doc::patch_schema(&doc, &loans()).unwrap()
    );
    assert!(request.instructions.contains("ui-spec/1"));
    let user = texts(request);
    assert!(
        user.contains("um add a table of uh overdue loans"),
        "{user}"
    );
    assert!(user.contains("Target node: page:loans"), "{user}");
    assert!(user.contains("section:list"), "{user}");
    assert!(user.contains("loans.All"), "{user}");
}

#[test]
fn a_refused_answer_is_retried_once_with_the_refusal_in_the_second_request() {
    let doc = library();
    let (mut proposer, seen) = proposer(vec![dangling_opens(), overdue_section()]);
    let proposal = proposer
        .propose(&doc, &loans(), "add a table of overdue loans")
        .unwrap();

    assert_eq!(proposal.attempts, 2);
    assert_eq!(proposal.turns, 2);
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 2);
    assert!(!texts(&seen[0]).contains("opens_resolves"));
    let second = texts(&seen[1]);
    assert!(second.contains("opens_resolves: "), "{second}");
    assert!(second.contains("nowhere"), "{second}");
    // The retry continues the conversation: the first answer is still in it.
    assert!(
        seen[1]
            .items
            .iter()
            .any(|item| matches!(item, Item::ToolCall(call) if call.arguments == dangling_opens()))
    );
}

#[test]
fn two_refusals_are_refused() {
    let doc = library();
    let (mut proposer, seen) =
        proposer(vec![dangling_opens(), dangling_opens(), overdue_section()]);
    let error = proposer
        .propose(&doc, &loans(), "add a table of overdue loans")
        .unwrap_err();

    let ProposeError::Refused { check, message } = error else {
        panic!("refused: {error}");
    };
    assert_eq!(check, "opens_resolves");
    assert!(message.contains("nowhere"), "{message}");
    assert_eq!(seen.lock().unwrap().len(), 2, "no third attempt");
}

#[test]
fn an_answer_that_is_not_a_patch_is_refused_as_patch_shape_and_retried() {
    let doc = library();
    // The schema does not tie `child` to `insert`, so this passes the loop and fails the patch.
    let no_child = json!({"op": "insert", "target": "page:loans"});
    let (mut proposer, seen) = proposer(vec![no_child, overdue_section()]);
    let proposal = proposer.propose(&doc, &loans(), "add a table").unwrap();
    assert_eq!(proposal.attempts, 2);
    assert!(texts(&seen.lock().unwrap()[1]).contains("patch_shape: "));
}

#[test]
fn a_target_that_names_no_node_is_refused_before_any_turn() {
    let doc = library();
    let (mut proposer, seen) = proposer(vec![overdue_section()]);
    let error = proposer
        .propose(&doc, &"page:nowhere".parse().unwrap(), "add a table")
        .unwrap_err();
    assert!(matches!(error, ProposeError::Target(_)), "{error}");
    assert!(seen.lock().unwrap().is_empty());
}
