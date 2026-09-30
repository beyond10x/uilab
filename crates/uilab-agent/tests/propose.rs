//! The proposer against a scripted model port: no network.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

use harness_wire::{
    CallId, Item, ModelPort, StopReason, StreamSink, ToolCall, ToolName, TurnOutcome, TurnRequest,
    Usage, WireError, WireId,
};
use serde_json::{Value, json};
use uilab_agent::{ProposeError, Proposer, ProposerConfig, Step};
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

#[test]
fn a_decline_proposes_nothing_and_says_why() {
    let doc = library();
    let (mut proposer, _) = proposer(vec![
        serde_json::json!({"op": "decline", "target": "page:loans", "reason": "that was thanks, not an instruction"}),
    ]);
    match proposer.propose(&doc, &loans(), "Thank you.") {
        Err(uilab_agent::ProposeError::Declined(reason)) => assert!(reason.contains("thanks")),
        other => panic!("expected a decline, got {other:?}"),
    }
}

const MEMBER_AREA: &str =
    "build out the member area: a list with search, a details card and an edit drawer";

fn members() -> NodePath {
    "page:members".parse().unwrap()
}

fn step(instruction: &str, target: &str) -> Value {
    json!({"instruction": instruction, "target": target, "why": format!("so that {instruction}")})
}

fn plan_of(steps: Vec<Value>) -> Value {
    json!({"op": "plan", "steps": steps})
}

/// Three steps at `page:members`; the second targets the section the first creates.
fn member_area_plan() -> Value {
    plan_of(vec![
        step("add a details card for the selected member", "page:members"),
        step(
            "add the member's loans to the details card",
            "page:members/section:details",
        ),
        step("add a search bar above the member list", "page:members"),
    ])
}

fn schema_of(request: &TurnRequest) -> &Value {
    &request.tools[0].input_schema
}

#[test]
fn a_goal_yields_ordered_steps_whose_targets_resolve_or_are_created_earlier() {
    let doc = library();
    let (mut proposer, seen) = proposer(vec![member_area_plan()]);
    let plan = proposer
        .plan_goal(&doc, &members(), MEMBER_AREA, 8)
        .unwrap();

    assert_eq!(plan.turns, 1);
    assert_eq!(plan.cost_micro_usd, None, "no rate card, no price");
    let targets: Vec<String> = plan.steps.iter().map(|s| s.target.to_string()).collect();
    assert_eq!(
        targets,
        [
            "page:members",
            "page:members/section:details",
            "page:members"
        ],
        "the steps come back in the order the model gave them"
    );
    assert_eq!(
        plan.steps[0].instruction,
        "add a details card for the selected member"
    );
    assert!(plan.steps[1].why.contains("loans"), "{:?}", plan.steps[1]);
    let last: &Step = &plan.steps[2];
    assert_eq!(last.instruction, "add a search bar above the member list");

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    let request = &seen[0];
    assert_eq!(request.tools.len(), 1, "{:?}", request.tools);
    assert_eq!(request.tools[0].name.as_str(), "answer");
    let schema = schema_of(request);
    assert_eq!(schema["properties"]["steps"]["maxItems"], json!(8));
    assert_eq!(
        schema["properties"]["op"]["enum"],
        json!(["plan", "decline"])
    );
    for field in ["instruction", "target", "why"] {
        assert!(
            schema["properties"]["steps"]["items"]["required"]
                .as_array()
                .unwrap()
                .contains(&json!(field)),
            "{schema}"
        );
    }
    assert!(
        request.instructions.contains("propose"),
        "{}",
        request.instructions
    );
    let user = texts(request);
    assert!(user.contains(MEMBER_AREA), "{user}");
    assert!(user.contains("Target node: page:members"), "{user}");
    assert!(
        user.contains("page:members/section:list"),
        "the outline: {user}"
    );
    assert!(
        user.contains("page:loans/overlay:edit"),
        "the outline: {user}"
    );
    assert!(user.contains("members.All"), "{user}");
}

#[test]
fn view_fields_reach_the_planner() {
    let doc = library();
    let (mut proposer, seen) = proposer(vec![member_area_plan()]);
    let fields = vec![(
        "members.All".to_owned(),
        vec![
            "name".to_owned(),
            "joined".to_owned(),
            "standing".to_owned(),
        ],
    )];
    proposer
        .plan_goal_with(&doc, &members(), MEMBER_AREA, 8, &fields)
        .unwrap();
    let user = texts(&seen.lock().unwrap()[0]);
    assert!(
        user.contains("members.All (name, joined, standing)"),
        "{user}"
    );
}

#[test]
fn a_target_nothing_earlier_creates_is_refused_and_retried_with_the_refusal() {
    let doc = library();
    let premature = plan_of(vec![
        step(
            "add the member's loans to the details card",
            "page:members/section:details",
        ),
        step("add a details card for the selected member", "page:members"),
    ]);
    let (mut proposer, seen) = proposer(vec![premature, member_area_plan()]);
    let plan = proposer
        .plan_goal(&doc, &members(), MEMBER_AREA, 8)
        .unwrap();

    assert_eq!(plan.turns, 2);
    assert_eq!(plan.steps.len(), 3);
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 2);
    assert!(!texts(&seen[0]).contains("plan_target"));
    let second = texts(&seen[1]);
    assert!(second.contains("plan_target: "), "{second}");
    assert!(second.contains("page:members/section:details"), "{second}");
}

#[test]
fn two_refused_plans_are_refused() {
    let doc = library();
    let dangling = plan_of(vec![step("add a card", "page:nowhere/section:card")]);
    let (mut proposer, seen) = proposer(vec![dangling.clone(), dangling, member_area_plan()]);
    let error = proposer
        .plan_goal(&doc, &members(), MEMBER_AREA, 8)
        .unwrap_err();
    let ProposeError::Refused { check, message } = error else {
        panic!("refused: {error}");
    };
    assert_eq!(check, "plan_target");
    assert!(message.contains("page:nowhere/section:card"), "{message}");
    assert_eq!(seen.lock().unwrap().len(), 2, "no third attempt");
}

#[test]
fn a_step_under_an_earlier_steps_target_is_accepted_at_any_depth() {
    let doc = library();
    let deep = plan_of(vec![
        step("add a details card with a loans list", "page:members"),
        step("show each loan's due date", "page:members/section:details"),
        step(
            "tag overdue loans in the details list",
            "page:members/section:details/item:loan",
        ),
    ]);
    let (mut proposer, _) = proposer(vec![deep]);
    let plan = proposer
        .plan_goal(&doc, &members(), MEMBER_AREA, 8)
        .unwrap();
    assert_eq!(plan.steps.len(), 3);
}

#[test]
fn more_steps_than_the_cap_are_refused() {
    let doc = library();
    let three = member_area_plan();
    let two = plan_of(vec![
        step("add a details card for the selected member", "page:members"),
        step("add a search bar above the member list", "page:members"),
    ]);

    let (mut planner, seen) = proposer(vec![three.clone(), two]);
    let plan = planner.plan_goal(&doc, &members(), MEMBER_AREA, 2).unwrap();
    assert_eq!(plan.steps.len(), 2, "the over-cap answer is not the plan");
    assert_eq!(plan.turns, 2, "the over-cap answer cost a turn");
    let seen = seen.lock().unwrap();
    assert_eq!(
        schema_of(&seen[0])["properties"]["steps"]["maxItems"],
        json!(2)
    );
    let refused_in_loop = seen[1].items.iter().any(|item| {
        matches!(item, Item::ToolResult { output, failed: true, .. }
            if output.to_string().contains("published schema"))
    });
    assert!(
        refused_in_loop,
        "the over-cap answer was refused against the published cap: {:?}",
        seen[1].items
    );
    drop(seen);

    let (mut proposer, _) = proposer(vec![three; 8]);
    let error = proposer
        .plan_goal(&doc, &members(), MEMBER_AREA, 2)
        .unwrap_err();
    assert!(
        matches!(
            error,
            ProposeError::Stopped(_) | ProposeError::Refused { .. }
        ),
        "an over-cap plan never comes back: {error}"
    );
}

#[test]
fn an_empty_plan_is_refused_and_retried() {
    let doc = library();
    let (mut planner, seen) = proposer(vec![plan_of(vec![]), member_area_plan()]);
    let plan = planner.plan_goal(&doc, &members(), MEMBER_AREA, 8).unwrap();
    assert_eq!(plan.steps.len(), 3);
    assert!(texts(&seen.lock().unwrap()[1]).contains("plan_empty: "));

    let (mut proposer, _) = proposer(vec![
        json!({"op": "plan"}),
        json!({"op": "plan", "steps": []}),
    ]);
    let error = proposer
        .plan_goal(&doc, &members(), MEMBER_AREA, 8)
        .unwrap_err();
    let ProposeError::Refused { check, .. } = error else {
        panic!("refused: {error}");
    };
    assert_eq!(check, "plan_empty");
}

#[test]
fn a_goal_that_is_not_a_ui_change_is_declined() {
    let doc = library();
    let (mut proposer, seen) = proposer(vec![
        json!({"op": "decline", "reason": "that asks what time it is, not for a UI change"}),
    ]);
    match proposer.plan_goal(&doc, &members(), "what time is it", 8) {
        Err(ProposeError::Declined(reason)) => assert!(reason.contains("time"), "{reason}"),
        other => panic!("expected a decline, got {other:?}"),
    }
    assert_eq!(seen.lock().unwrap().len(), 1);
}

#[test]
fn a_planner_target_that_names_no_node_or_a_zero_cap_is_refused_before_any_turn() {
    let doc = library();
    let (mut proposer, seen) = proposer(vec![member_area_plan()]);
    let error = proposer
        .plan_goal(&doc, &"page:nowhere".parse().unwrap(), MEMBER_AREA, 8)
        .unwrap_err();
    assert!(matches!(error, ProposeError::Target(_)), "{error}");
    let error = proposer
        .plan_goal(&doc, &members(), MEMBER_AREA, 0)
        .unwrap_err();
    assert!(matches!(error, ProposeError::Config(_)), "{error}");
    assert!(seen.lock().unwrap().is_empty());
}
