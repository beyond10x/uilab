//! Adversarial cases for `check_plan` and `plan_goal`, against a scripted model port: no network.

use std::collections::VecDeque;
use std::path::Path;

use harness_wire::{
    CallId, Item, ModelPort, StopReason, StreamSink, ToolCall, ToolName, TurnOutcome, TurnRequest,
    Usage, WireError, WireId,
};
use serde_json::{Value, json};
use uilab_agent::{ProposeError, Proposer, ProposerConfig, Step, check_plan};
use uilab_doc::{Document, NodePath, Patch, Refusal};

fn library() -> Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    Document::from_yaml(&std::fs::read_to_string(path).unwrap()).unwrap()
}

struct ScriptedModel {
    wire: WireId,
    answers: VecDeque<Value>,
    turns: usize,
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
        self.turns += 1;
        let arguments = self
            .answers
            .pop_front()
            .ok_or_else(|| WireError::protocol("the script ran out of turns"))?;
        Ok(TurnOutcome {
            stop_reason: StopReason::ToolCalls,
            items: vec![Item::ToolCall(ToolCall {
                call_id: CallId::new(format!("call-{}", self.turns)).unwrap(),
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

fn proposer(answers: Vec<Value>) -> Proposer {
    let model = ScriptedModel {
        wire: WireId::new("anthropic-messages").unwrap(),
        answers: answers.into(),
        turns: 0,
    };
    Proposer::with_port(&ProposerConfig::default(), Box::new(model))
}

fn step(instruction: &str, target: &str) -> Step {
    Step {
        instruction: instruction.to_owned(),
        target: target.parse().unwrap(),
        why: "a test".to_owned(),
    }
}

fn step_json(instruction: &str, target: &str) -> Value {
    json!({"instruction": instruction, "target": target, "why": "a test"})
}

/// What `propose` may answer for "add an Edit row action to the member list that opens a new
/// edit drawer": one batch at the step's target, `page:members/section:list`, that also inserts
/// the drawer the action opens under the page, as the planner instructions ask ("a drawer or
/// dialog and the row action that opens it belong in one step").
fn edit_action_and_drawer() -> Patch {
    serde_json::from_value(json!({
        "op": "batch",
        "target": "page:members/section:list",
        "patches": [
            {
                "op": "insert",
                "target": "page:members",
                "child": {
                    "layer": "overlay",
                    "name": "edit",
                    "node": {
                        "kind": "drawer",
                        "component": "record",
                        "title": "Member",
                        "reads": {"view": "members.All"}
                    }
                }
            },
            {
                "op": "replace",
                "target": "page:members/section:list",
                "node": {
                    "component": "collection",
                    "reads": {"view": "members.All"},
                    "columns": [{"field": "name"}, {"field": "joined"}, {"field": "loans"}, {"field": "standing", "as": "tag"}],
                    "row_actions": [{"opens": "edit", "label": "Edit"}]
                }
            }
        ]
    }))
    .unwrap()
}

#[test]
fn a_drawer_an_earlier_steps_batch_creates_is_a_target_the_plan_accepts() {
    let doc = library();
    let drawer: NodePath = "page:members/overlay:edit".parse().unwrap();
    assert!(uilab_doc::resolve(&doc, &drawer).is_err(), "not there yet");
    let (after, _) = uilab_doc::admit(&doc, &edit_action_and_drawer())
        .expect("propose's answer for step 1 is admitted");
    uilab_doc::resolve(&after, &drawer).expect("step 1 creates the drawer");

    let plan = [
        step(
            "add an Edit row action to the member list that opens a new edit drawer `edit`",
            "page:members/section:list",
        ),
        step(
            "show the member's standing in the edit drawer",
            "page:members/overlay:edit",
        ),
    ];
    check_plan(&doc, &plan, 8)
        .expect("step 2 targets a node step 1 creates, which the acceptance says is valid");
}

#[test]
fn a_blank_instruction_is_not_a_step_the_runner_can_carry_out() {
    let doc = library();
    for blank in ["", "   "] {
        let plan = [
            step("add a search bar above the member list", "page:members"),
            step(blank, "page:members/section:list"),
        ];
        let refused: Refusal = check_plan(&doc, &plan, 8)
            .expect_err("a step with no instruction gives propose nothing to carry out");
        assert!(refused.message.contains('2'), "{}", refused.message);
    }
}

/// `check_plan`'s target rule with the `starts_with` conjunct dropped: the mutant the unit's
/// suite does not kill, since every `plan_target` refusal it asserts has no shorter earlier step.
fn mutant_without_prefix(doc: &Document, steps: &[Step]) -> bool {
    steps.iter().enumerate().all(|(index, step)| {
        steps[..index]
            .iter()
            .any(|earlier| earlier.target.0.len() < step.target.0.len())
            || uilab_doc::resolve(doc, &step.target).is_ok()
    })
}

#[test]
fn a_target_under_no_earlier_target_is_refused_even_after_a_shorter_unrelated_step() {
    let doc = library();
    let plan = [
        step("add a search bar above the loan list", "page:loans"),
        step(
            "add the member's loans to the details card",
            "page:members/section:details",
        ),
    ];
    let refused = check_plan(&doc, &plan, 8).unwrap_err();
    assert_eq!(refused.check, "plan_target");
    assert!(refused.message.contains("step 2"), "{}", refused.message);
    assert!(
        mutant_without_prefix(&doc, &plan),
        "this input tells the rule from the mutant"
    );
}

#[test]
fn the_cap_holds_at_exactly_n_and_refuses_n_plus_one() {
    let doc = library();
    let three = [
        step("a", "page:members"),
        step("b", "page:members/section:list"),
        step("c", "page:loans"),
    ];
    check_plan(&doc, &three, 3).expect("exactly the cap");
    assert_eq!(
        check_plan(&doc, &three, 2).unwrap_err().check,
        "plan_too_long"
    );
    assert_eq!(
        check_plan(&doc, &three[..1], 0).unwrap_err().check,
        "plan_too_long"
    );
    assert_eq!(check_plan(&doc, &[], 0).unwrap_err().check, "plan_empty");
}

#[test]
fn root_and_nav_are_plan_and_step_targets() {
    let doc = library();
    let plan = [
        step("add a reports page", "/"),
        step(
            "list the reports page under people",
            "nav/nav_section:people",
        ),
        step("add a menu section for reports", "nav"),
        step("give the reports page a table", "page:reports"),
    ];
    check_plan(&doc, &plan, 8).expect("root, nav and a page under root");

    for target in ["/", "nav"] {
        let mut planner = proposer(vec![json!({"op": "plan", "steps": [
            step_json("add a reports page", "/"),
            step_json("give the reports page a table", "page:reports/section:table"),
        ]})]);
        let planned = planner
            .plan_goal(&doc, &target.parse().unwrap(), "add a reports page", 8)
            .unwrap_or_else(|error| panic!("at {target}: {error}"));
        assert_eq!(planned.steps.len(), 2);
        assert_eq!(planned.steps[0].target, NodePath::root());
    }
}

#[test]
fn a_decline_after_a_refused_plan_is_a_decline() {
    let doc = library();
    let mut planner = proposer(vec![
        json!({"op": "plan", "steps": [step_json("add a card", "page:nowhere/section:card")]}),
        json!({"op": "decline", "reason": "that was a question, not a goal"}),
    ]);
    match planner.plan_goal(&doc, &"page:members".parse().unwrap(), "what can you do", 8) {
        Err(ProposeError::Declined(reason)) => assert!(reason.contains("question"), "{reason}"),
        other => panic!("expected a decline, got {other:?}"),
    }
}
