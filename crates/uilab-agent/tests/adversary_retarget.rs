//! Adversarial cases for story:agent-retarget: the Components-tab guard on a move, and whether a
//! patch can land outside the target it was asked at (the premise the story rests on).

use std::collections::VecDeque;
use std::path::Path;

use harness_wire::{
    CallId, Item, ModelPort, StopReason, StreamSink, ToolCall, ToolName, TurnOutcome, TurnRequest,
    Usage, WireError, WireId,
};
use serde_json::{Value, json};
use uilab_agent::{Answer, Proposer, ProposerConfig, Workspace, check_retarget};
use uilab_doc::{Document, NodePath};

/// The library example with one widget, `loan_card`, declared (named the way a widget that
/// shows a loan is named).
fn library_with_loan_card() -> Document {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    let text = std::fs::read_to_string(file).unwrap();
    assert!(text.contains("\npages:\n"), "fixture anchor is missing");
    let text = text.replacen(
        "\npages:\n",
        "\nwidgets:\n  loan_card:\n    summary: A loan as a card.\n    params:\n      loan: {type: Loan, required: true}\n    body:\n      - {name: title, primitive: text, text: args.loan.title, style: heading}\npages:\n",
        1,
    );
    Document::from_yaml(&text).unwrap()
}

/// Design (story:agent-retarget): "On the Components tab the move stays inside `component:`
/// paths unless the instruction names a page." An instruction about the widget `loan_card` names
/// no page, yet `names_a_page` reads the word `loan` as the page `loans` (plural rule of
/// `same_word`), so the guard lets the move leave the widgets.
#[test]
fn a_components_move_to_a_page_is_refused_when_the_instruction_names_only_a_widget() {
    let doc = library_with_loan_card();
    let from: NodePath = "component:loan_card".parse().unwrap();
    let to: NodePath = "page:loans".parse().unwrap();
    let refused = check_retarget(
        &doc,
        &from,
        &to,
        "make the loan card bigger",
        false,
        Workspace::Components,
    );
    assert_eq!(
        refused.map_err(|r| r.check),
        Err("retarget_workspace".to_owned()),
        "\"make the loan card bigger\" names the widget, not the loans page"
    );
}

/// Answers each turn with the next scripted `answer` call.
struct ScriptedModel {
    wire: WireId,
    answers: VecDeque<Value>,
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
        let arguments = self
            .answers
            .pop_front()
            .ok_or_else(|| WireError::protocol("the script ran out of turns"))?;
        Ok(TurnOutcome {
            stop_reason: StopReason::ToolCalls,
            items: vec![Item::ToolCall(ToolCall {
                call_id: CallId::new(format!("call-{}", self.answers.len())).unwrap(),
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
    Proposer::with_port(
        &ProposerConfig::default(),
        Box::new(ScriptedModel {
            wire: WireId::new("anthropic-messages").unwrap(),
            answers: answers.into(),
        }),
    )
}

/// A new page at the root: what "create a new page in the sidebar" should become.
fn new_page_at_root() -> Value {
    json!({
        "op": "insert",
        "target": "/",
        "child": {
            "layer": "page",
            "name": "overdue",
            "node": {
                "kind": "list_page",
                "title": "Overdue",
                "sections": {"list": {"component": "collection", "reads": {"view": "loans.All"}}}
            }
        }
    })
}

/// Found by (story:agent-retarget): "nothing outside the selected subtree can change". Held by the
/// answer schema (`target` is a const) that the harness validates each answer against: a patch
/// at `/` answered at `page:loans/section:list` is not admitted, and the answer the model gives
/// next, at the target, is the one that comes back.
#[test]
fn a_patch_outside_the_target_is_not_admitted_where_a_move_is_allowed() {
    let doc = library_with_loan_card();
    let list: NodePath = "page:loans/section:list".parse().unwrap();
    let retitle = json!({
        "op": "replace",
        "target": "page:loans/section:list",
        "node": {"component": "collection", "title": "Current loans", "reads": {"view": "loans.All"}}
    });
    let mut proposer = proposer(vec![new_page_at_root(), retitle]);
    let answer = proposer
        .answer_in(
            &doc,
            &list,
            "create a new page in the sidebar for overdue loans",
            &[],
            Workspace::App,
        )
        .unwrap();
    let Answer::Patch(proposal) = answer else {
        panic!("a patch: {answer:?}");
    };
    assert_eq!(proposal.patch.target(), &list);
}

/// The re-ask after a move runs `propose_in` at the new target (`page:members`); a patch that
/// goes back to the old target is outside it, is not admitted, and the answer at the new target
/// is the one that comes back.
#[test]
fn the_re_ask_after_a_move_does_not_admit_a_patch_back_at_the_old_target() {
    let doc = library_with_loan_card();
    let back_at_the_list = json!({
        "op": "replace",
        "target": "page:loans/section:list",
        "node": {"component": "collection", "title": "Current loans", "reads": {"view": "loans.All"}}
    });
    let members: NodePath = "page:members".parse().unwrap();
    let at_members = json!({
        "op": "insert",
        "target": "page:members",
        "child": {
            "layer": "section",
            "name": "loans",
            "node": {"component": "collection", "title": "Loans", "reads": {"view": "loans.All"}}
        }
    });
    let mut proposer = proposer(vec![back_at_the_list, at_members]);
    let proposal = proposer
        .propose_in(
            &doc,
            &members,
            "add a table of members' loans on the members page",
            &[],
            Workspace::App,
        )
        .unwrap();
    assert_eq!(proposal.patch.target(), &members);
}
