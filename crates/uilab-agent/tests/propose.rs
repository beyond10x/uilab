//! The proposer against a scripted model port: no network.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

use harness_wire::{
    CallId, Item, ModelPort, StopReason, StreamSink, ToolCall, ToolName, TurnOutcome, TurnRequest,
    Usage, WireError, WireId,
};
use serde_json::{Value, json};
use uilab_agent::{
    Answer, INSTRUCTIONS, MOVE_INSTRUCTIONS, PLAN_INSTRUCTIONS, ProposeError, Proposer,
    ProposerConfig, Step, Workspace, check_retarget,
};
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
    assert!(request.instructions.contains("ess-ui/1"));
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
    // A batch's patches name their own targets as strings, so one that is no node path passes the
    // schema and the loop, and fails the patch.
    let no_path = json!({"op": "batch", "target": "page:loans", "patches": [
        {"op": "remove", "target": "page:loans/overlay:edit"},
        {"op": "remove", "target": "not a path"},
    ]});
    let (mut proposer, seen) = proposer(vec![no_path, overdue_section()]);
    let proposal = proposer.propose(&doc, &loans(), "add a table").unwrap();
    assert_eq!(proposal.attempts, 2);
    assert!(texts(&seen.lock().unwrap()[1]).contains("patch_shape: "));
}

/// story:essui-agent-schema: the schema ties `child` to `insert` (and `node` to `replace`), so an
/// insert without its child no longer reaches uilab as a `patch_shape` refusal: the loop holds it
/// to the published schema and the model answers again in the same attempt.
#[test]
fn an_insert_without_its_child_is_refused_by_the_schema_in_the_loop() {
    let doc = library();
    let no_child = json!({"op": "insert", "target": "page:loans"});
    let (mut proposer, seen) = proposer(vec![no_child, overdue_section()]);
    let proposal = proposer.propose(&doc, &loans(), "add a table").unwrap();
    assert_eq!(proposal.attempts, 1, "no uilab retry was spent");
    assert_eq!(proposal.turns, 2, "the loop asked again");
    assert_eq!(seen.lock().unwrap().len(), 2);
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

fn loans_list() -> NodePath {
    "page:loans/section:list".parse().unwrap()
}

fn retarget(path: &str, navigate_only: bool) -> Value {
    json!({
        "op": "retarget",
        "target": "page:loans/section:list",
        "path": path,
        "reason": format!("the instruction is about {path}"),
        "navigate_only": navigate_only,
    })
}

const NEW_PAGE: &str = "create a new page in the sidebar for overdue loans";

/// Found by the operator on 2026-09-30: with the loans list selected, "create a new page in the
/// sidebar" was carried out inside the list, because the list was all a patch there could change.
#[test]
fn an_instruction_that_names_another_place_answers_a_move_there() {
    let doc = library();
    let (mut proposer, seen) = proposer(vec![retarget("/", false)]);
    let answer = proposer
        .answer_in(&doc, &loans_list(), NEW_PAGE, &[], Workspace::App)
        .unwrap();
    let Answer::Retarget(moved) = answer else {
        panic!("a move: {answer:?}");
    };
    assert_eq!(moved.path, NodePath::root());
    assert_eq!(moved.reason, "the instruction is about /");
    assert!(!moved.navigate_only);
    assert_eq!(moved.attempts, 1);

    let seen = seen.lock().unwrap();
    let schema = schema_of(&seen[0]);
    let ops = schema["properties"]["op"]["enum"].as_array().unwrap();
    assert!(ops.contains(&json!("retarget")), "{ops:?}");
    assert_eq!(schema["properties"]["path"]["type"], "string");
    assert_eq!(schema["properties"]["navigate_only"]["type"], "boolean");
    let user = texts(&seen[0]);
    assert!(
        user.contains("page:members") && user.contains("nav"),
        "the places a move can go are listed: {user}"
    );
}

#[test]
fn a_patch_answer_where_a_move_is_allowed_is_a_patch() {
    let doc = library();
    let (mut proposer, _) = proposer(vec![overdue_section()]);
    let answer = proposer
        .answer_in(
            &doc,
            &loans(),
            "add a table of overdue loans",
            &[],
            Workspace::App,
        )
        .unwrap();
    assert!(matches!(answer, Answer::Patch(_)), "{answer:?}");
}

#[test]
fn the_prompt_states_when_a_move_is_allowed_and_only_where_one_is() {
    let doc = library();
    let (mut moving, seen) = proposer(vec![retarget("page:members", true)]);
    moving
        .answer_in(
            &doc,
            &loans_list(),
            "go to the members page",
            &[],
            Workspace::App,
        )
        .unwrap();
    let instructions = seen.lock().unwrap()[0].instructions.clone();
    for said in [
        "`op: retarget`",
        "once",
        "`/`",
        "`nav`",
        "`navigate_only: true`",
        "names no other place",
        "Components",
    ] {
        assert!(instructions.contains(said), "{said}: {instructions}");
    }

    let (mut proposer, seen) = proposer(vec![overdue_section()]);
    proposer
        .propose(&doc, &loans(), "add a table of overdue loans")
        .unwrap();
    let seen = seen.lock().unwrap();
    assert!(!seen[0].instructions.contains("retarget"));
    let ops = schema_of(&seen[0])["properties"]["op"]["enum"].clone();
    assert!(
        !ops.as_array().unwrap().contains(&json!("retarget")),
        "a plain proposal cannot move: {ops}"
    );
}

#[test]
fn a_navigation_only_move_comes_back_as_one() {
    let doc = library();
    let (mut proposer, _) = proposer(vec![retarget("page:members", true)]);
    let answer = proposer
        .answer_in(
            &doc,
            &loans_list(),
            "go to the members page",
            &[],
            Workspace::App,
        )
        .unwrap();
    let Answer::Retarget(moved) = answer else {
        panic!("a move: {answer:?}");
    };
    assert_eq!(moved.path.to_string(), "page:members");
    assert!(moved.navigate_only);
}

#[test]
fn a_move_to_a_path_that_does_not_resolve_is_refused_and_retried() {
    let doc = library();
    let (mut proposer, seen) =
        proposer(vec![retarget("page:overdue", false), retarget("/", false)]);
    let answer = proposer
        .answer_in(&doc, &loans_list(), NEW_PAGE, &[], Workspace::App)
        .unwrap();
    let Answer::Retarget(moved) = answer else {
        panic!("a move: {answer:?}");
    };
    assert_eq!(moved.path, NodePath::root());
    assert_eq!(moved.attempts, 2);
    let second = texts(&seen.lock().unwrap()[1]);
    assert!(second.contains("path_resolves: "), "{second}");
    assert!(second.contains("page:overdue"), "{second}");
}

#[test]
fn a_move_is_checked_against_the_document_the_target_and_the_workspace() {
    let doc = library();
    let root = NodePath::root();
    let members = members();
    check_retarget(&doc, &loans_list(), &root, NEW_PAGE, false, Workspace::App)
        .expect("a new page moves to the root");
    check_retarget(
        &doc,
        &loans_list(),
        &"nav".parse().unwrap(),
        "add a menu entry",
        false,
        Workspace::App,
    )
    .expect("a menu entry moves to the navigation");
    let refused = check_retarget(
        &doc,
        &loans_list(),
        &"page:overdue".parse().unwrap(),
        NEW_PAGE,
        false,
        Workspace::App,
    )
    .unwrap_err();
    assert_eq!(refused.check, "path_resolves");
    let refused = check_retarget(
        &doc,
        &loans_list(),
        &loans_list(),
        "add a column",
        false,
        Workspace::App,
    )
    .unwrap_err();
    assert_eq!(refused.check, "retarget_same");

    let refused = check_retarget(
        &doc,
        &root,
        &members,
        "make the card bigger",
        false,
        Workspace::Components,
    )
    .unwrap_err();
    assert_eq!(refused.check, "retarget_workspace");
    check_retarget(
        &doc,
        &root,
        &members,
        "put the loan card on the members page",
        false,
        Workspace::Components,
    )
    .expect("the Components tab leaves the widgets when a page is named");
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
            "tag overdue loans in the details card's rows",
            "page:members/section:details/item:loan",
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
    assert!(
        second.contains("page:members/section:details/item:loan"),
        "{second}"
    );
}

#[test]
fn a_blank_target_is_refused_rather_than_read_as_the_root() {
    let doc = library();
    for blank in ["", "  "] {
        let (mut planner, seen) = proposer(vec![
            plan_of(vec![step("add a search bar above the member list", blank)]),
            member_area_plan(),
        ]);
        let plan = planner.plan_goal(&doc, &members(), MEMBER_AREA, 8).unwrap();
        assert_eq!(plan.steps.len(), 3, "the blank-target plan is not the plan");
        let seen = seen.lock().unwrap();
        if blank.is_empty() {
            let refused_in_loop = seen[1].items.iter().any(|item| {
                matches!(item, Item::ToolResult { output, failed: true, .. }
                    if output.to_string().contains("published schema"))
            });
            assert!(refused_in_loop, "minLength refuses it: {:?}", seen[1].items);
        } else {
            let second = texts(&seen[1]);
            assert!(second.contains("plan_step_blank: "), "{second}");
            assert!(second.contains("step 1"), "{second}");
        }
    }
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
fn a_new_child_of_an_earlier_steps_target_is_accepted() {
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

/// One sent prompt: which run it opened, its instructions, its user text and the schema of the
/// tool it answers with.
struct Sent {
    run: &'static str,
    instructions: String,
    user: String,
    schema: Value,
}

/// The first request of every kind of run the agent makes at `page:loans`: a plain proposal, a
/// proposal that may move (app canvas and Components tab) and a plan. Each is answered with a
/// decline, so only what was sent matters.
fn sent_prompts() -> Vec<Sent> {
    let doc = library();
    let decline = || json!({"op": "decline", "target": "page:loans", "reason": "a test"});
    let first = |run, seen: Arc<Mutex<Vec<TurnRequest>>>| {
        let seen = seen.lock().unwrap();
        let request = seen
            .first()
            .unwrap_or_else(|| panic!("{run}: nothing was sent"));
        Sent {
            run,
            instructions: request.instructions.clone(),
            user: texts(request),
            schema: schema_of(request).clone(),
        }
    };
    let mut sent = Vec::new();
    let (mut agent, seen) = proposer(vec![decline()]);
    let _ = agent.propose(&doc, &loans(), "add a table of overdue loans");
    sent.push(first("propose", seen));
    for (run, workspace) in [
        ("answer_in app", Workspace::App),
        ("answer_in components", Workspace::Components),
    ] {
        let (mut agent, seen) = proposer(vec![decline()]);
        let _ = agent.answer_in(
            &doc,
            &loans(),
            "add a table of overdue loans",
            &[],
            workspace,
        );
        sent.push(first(run, seen));
    }
    let (mut agent, seen) = proposer(vec![json!({"op": "decline", "reason": "a test"})]);
    let _ = agent.plan_goal(&doc, &loans(), MEMBER_AREA, 4);
    sent.push(first("plan_goal", seen));
    sent
}

/// The instruction texts the agent is built from, by name.
fn instruction_constants() -> [(&'static str, &'static str); 3] {
    [
        ("INSTRUCTIONS", INSTRUCTIONS),
        ("MOVE_INSTRUCTIONS", MOVE_INSTRUCTIONS),
        ("PLAN_INSTRUCTIONS", PLAN_INSTRUCTIONS),
    ]
}

/// story:essui-agent-schema: the prompt says `ess-ui/1` and never `ui-spec/1`.
#[test]
fn prompt_names_ess_ui() {
    for (name, text) in [
        ("INSTRUCTIONS", INSTRUCTIONS),
        ("PLAN_INSTRUCTIONS", PLAN_INSTRUCTIONS),
    ] {
        assert!(text.contains("`ess-ui/1`"), "{name} does not name ess-ui/1");
    }
    for (name, text) in instruction_constants() {
        assert!(!text.contains("ui-spec"), "{name} names ui-spec");
    }
    for sent in sent_prompts() {
        assert!(
            sent.instructions.contains("`ess-ui/1`"),
            "{}: {}",
            sent.run,
            sent.instructions
        );
        assert!(
            !sent.instructions.contains("ui-spec") && !sent.user.contains("ui-spec"),
            "{} names ui-spec:\n{}\n{}",
            sent.run,
            sent.instructions,
            sent.user
        );
    }
}

/// story:essui-agent-schema: data the model does not provide yet is a placeholder read with a
/// fixture, `reads: {placeholder, fixture}` (ESS `Reads`: exactly one of `view` and
/// `placeholder`, and a placeholder requires a `fixture`).
#[test]
fn prompt_teaches_placeholder_reads() {
    const SHAPE: &str = "`reads: {placeholder: <Name>, fixture: <file>}`";
    for (name, text) in [
        ("INSTRUCTIONS", INSTRUCTIONS),
        ("PLAN_INSTRUCTIONS", PLAN_INSTRUCTIONS),
    ] {
        assert!(text.contains(SHAPE), "{name} does not show {SHAPE}");
    }
    for sent in sent_prompts() {
        assert!(
            sent.instructions.contains(SHAPE),
            "{}: {}",
            sent.run,
            sent.instructions
        );
    }
}

/// story:essui-agent-schema: no `draft.` view prefix anywhere in the prompt, instructions or
/// user text.
#[test]
fn prompt_has_no_draft_views() {
    for (name, text) in instruction_constants() {
        assert!(!text.contains("draft."), "{name} teaches a `draft.` view");
    }
    for sent in sent_prompts() {
        assert!(
            !sent.instructions.contains("draft.") && !sent.user.contains("draft."),
            "{} sends a `draft.` view:\n{}\n{}",
            sent.run,
            sent.instructions,
            sent.user
        );
    }
}

/// The kinds a section holds: the members of ESS's composite union, read from the schema this
/// build of ESS embeds (`Composite.union.members`).
fn section_kinds() -> Vec<String> {
    let schema: serde_yaml::Value =
        serde_yaml::from_str(uilab_doc::ess_ui::SCHEMA).expect("the ESS schema is YAML");
    let members = schema["constructs"]["Composite"]["union"]["members"]
        .as_sequence()
        .expect("ESS declares the composite union's members");
    members
        .iter()
        .map(|m| m.as_str().expect("a member is a name").to_owned())
        .collect()
}

/// Every composite or widget variant in `schema` that is not an overlay: an object schema whose
/// `properties` hold `component` and no overlay `kind`. Found by walking the whole schema, so no
/// definition name is assumed.
fn composite_variants<'a>(schema: &'a Value, out: &mut Vec<&'a Value>) {
    match schema {
        Value::Object(object) => {
            if let Some(properties) = object.get("properties").and_then(Value::as_object)
                && properties.contains_key("component")
                && !properties.contains_key("kind")
            {
                out.push(schema);
            }
            for value in object.values() {
                composite_variants(value, out);
            }
        }
        Value::Array(values) => values.iter().for_each(|v| composite_variants(v, out)),
        _ => {}
    }
}

/// story:essui-agent-schema: an `ess-ui/1` section has no `title`. No section kind's entry in the
/// instructions offers one; every sentence of the instructions that says "title" is about a page,
/// an overlay or a page header, or says there is none; and no section or nested composite in the
/// schema sent with the prompt has a `title` field (an overlay does: ESS's overlay frame has one).
#[test]
fn prompt_offers_no_section_title() {
    let kinds = section_kinds();
    assert_eq!(kinds.len(), 12, "ESS 0.48.0 has 12 composite kinds");
    for sent in sent_prompts() {
        let mut variants = Vec::new();
        composite_variants(&sent.schema, &mut variants);
        if sent.run != "plan_goal" {
            // Every kind as a section and nested, at least.
            assert!(
                variants.len() >= 2 * kinds.len(),
                "{}: {} composite variants in the schema",
                sent.run,
                variants.len()
            );
        }
        for variant in variants {
            assert!(
                variant["properties"].get("title").is_none(),
                "{}: the schema offers a section or nested composite a `title`: {}",
                sent.run,
                variant["properties"]["component"]
            );
        }
    }
    for kind in &kinds {
        let entry = format!("- {kind}:");
        let line = INSTRUCTIONS
            .lines()
            .find(|line| line.starts_with(&entry))
            .unwrap_or_else(|| panic!("INSTRUCTIONS do not list `{kind}`"));
        assert!(
            !line.contains("title"),
            "INSTRUCTIONS offer `{kind}` a title: {line}"
        );
    }
    let mut offending = Vec::new();
    for (name, text) in instruction_constants() {
        for sentence in text
            .split(['\n', ';'])
            .flat_map(|line| line.split(". "))
            .filter(|sentence| sentence.to_lowercase().contains("title"))
        {
            let about = ["page", "overlay", "header", "no `title`"]
                .iter()
                .any(|allowed| sentence.contains(allowed));
            if !about {
                offending.push(format!("{name}: {sentence}"));
            }
        }
    }
    assert!(
        offending.is_empty(),
        "sentences that give a section or composite a title:\n{}",
        offending.join("\n")
    );
}

/// Every node path of the outline under `node`, `node`'s own first.
fn outline_paths(node: &uilab_doc::OutlineNode, out: &mut Vec<String>) {
    out.push(node.path.clone());
    for child in &node.children {
        outline_paths(child, out);
    }
}

/// story:eval-round-5: the Messages API refuses a tool whose `input_schema` has `oneOf`, `allOf`
/// or `anyOf` at its top level (round 5, run 1: every case answered 400 "input_schema does not
/// support oneOf, allOf, or anyOf at the top level"). So no schema the agent sends has one: the
/// patch schema at every node of the library, as a plain proposal and as one that may move, on
/// both workspaces, and the plan schema. Each run is answered with a decline; only what was sent
/// matters.
#[test]
fn no_answer_schema_sent_has_a_combinator_at_its_top_level() {
    let doc = library();
    let mut all = Vec::new();
    outline_paths(&uilab_doc::outline(&doc), &mut all);
    let mut sent: Vec<(String, Value)> = Vec::new();
    let first = |seen: Arc<Mutex<Vec<TurnRequest>>>| {
        seen.lock()
            .unwrap()
            .first()
            .map(|request| schema_of(request).clone())
    };
    for at in &all {
        let target: NodePath = at.parse().unwrap();
        let decline = json!({"op": "decline", "target": at, "reason": "a test"});
        for workspace in [Workspace::App, Workspace::Components] {
            let (mut agent, seen) = proposer(vec![decline.clone()]);
            let _ = agent.propose_in(&doc, &target, "add a table", &[], workspace);
            sent.extend(first(seen).map(|s| (format!("propose_in {at} {workspace:?}"), s)));
            let (mut agent, seen) = proposer(vec![decline.clone()]);
            let _ = agent.answer_in(&doc, &target, "add a table", &[], workspace);
            sent.extend(first(seen).map(|s| (format!("answer_in {at} {workspace:?}"), s)));
        }
        let (mut agent, seen) = proposer(vec![json!({"op": "decline", "reason": "a test"})]);
        let _ = agent.plan_goal(&doc, &target, MEMBER_AREA, 4);
        sent.extend(first(seen).map(|s| (format!("plan_goal {at}"), s)));
    }
    assert!(
        sent.len() >= 5 * all.len(),
        "{} schemas sent for {} nodes",
        sent.len(),
        all.len()
    );
    let offending: Vec<String> = sent
        .iter()
        .filter_map(|(run, schema)| {
            let combinators: Vec<&str> = ["oneOf", "allOf", "anyOf"]
                .into_iter()
                .filter(|key| schema.get(*key).is_some())
                .collect();
            let object = schema["type"] == "object";
            (!combinators.is_empty() || !object)
                .then(|| format!("{run}: type {}, top-level {combinators:?}", schema["type"]))
        })
        .collect();
    assert!(
        offending.is_empty(),
        "{} of {} schemas sent would be refused by the Messages API:\n{}",
        offending.len(),
        sent.len(),
        offending.join("\n")
    );
}

/// story:eval-round-5, run 2: the prompt taught a section as `{name: …, component: …}`, while a
/// patch's section node may not carry `name` (the patch names it: `child.name`, or the target),
/// and the schema refuses one that does with `{"required":["name"]} is not allowed`. At a
/// collection, the model's replaces were refused in the loop three times and it fell back to a
/// batch of the same replace twice (add-column, relabel-column, retarget-none). The prompt now
/// teaches the shape the schema takes, and says where the name goes.
#[test]
fn the_prompt_teaches_a_section_node_without_its_name() {
    let doc = library();
    let named = json!({"op": "replace", "target": "page:loans/section:list", "node": {
        "name": "list", "component": "collection", "reads": {"view": "loans.All"},
        "columns": [{"field": "title"}],
    }});
    let schema =
        uilab_doc::patch_schema(&doc, &"page:loans/section:list".parse().unwrap()).unwrap();
    assert!(
        harness_loop::OutputSchema::new(schema)
            .unwrap()
            .validate(&named)
            .is_err(),
        "the schema takes a section node with its name; this test's premise is gone"
    );
    assert!(
        !INSTRUCTIONS.contains("`{name: …, component:"),
        "INSTRUCTIONS teach a section as `{{name: …, component: …}}`"
    );
    const RULE: &str = "a node in a patch never carries its own `name`";
    // Every run that answers a patch; the planner answers steps, not nodes.
    let patch_runs: Vec<Sent> = sent_prompts()
        .into_iter()
        .filter(|sent| sent.run != "plan_goal")
        .collect();
    assert_eq!(patch_runs.len(), 3);
    for sent in patch_runs {
        assert!(
            sent.instructions.contains(RULE),
            "{}: the instructions do not say `{RULE}`",
            sent.run
        );
    }
}
