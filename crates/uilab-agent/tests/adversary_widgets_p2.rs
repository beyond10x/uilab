//! Adversarial cases for pass 2 of the page listing: whole-word page matching. Scripted model
//! port: no network.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

use harness_wire::{
    CallId, Item, ModelPort, StopReason, StreamSink, ToolCall, ToolName, TurnOutcome, TurnRequest,
    Usage, WireError, WireId,
};
use serde_json::{Value, json};
use uilab_agent::{Proposer, ProposerConfig};
use uilab_doc::{Document, NodePath};

/// The library with list pages appended under `pages:` (the file's last key), each `(name, title)`.
fn library_with_pages(pages: &[(&str, &str)]) -> Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    let mut text = std::fs::read_to_string(path).unwrap();
    for (name, title) in pages {
        text.push_str(&format!(
            "  {name}:\n    kind: list_page\n    title: \"{title}\"\n    sections:\n      - name: queue\n        component: collection\n        reads: {{view: loans.All}}\n        columns: [{{field: title}}]\n"
        ));
    }
    Document::from_yaml(&text).unwrap()
}

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

/// The first user text the proposer sends for `utterance` at `target`, answered by a decline.
fn first_request(doc: &Document, target: &str, utterance: &str) -> String {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let model = ScriptedModel {
        wire: WireId::new("anthropic-messages").unwrap(),
        answers: vec![json!({"op": "decline", "target": target, "reason": "scripted"})].into(),
        seen: Arc::clone(&seen),
    };
    let mut proposer = Proposer::with_port(&ProposerConfig::default(), Box::new(model));
    let path: NodePath = target.parse().unwrap();
    let _ = proposer.propose(doc, &path, utterance);
    let seen = seen.lock().unwrap();
    seen[0]
        .items
        .iter()
        .filter_map(|item| match item {
            Item::UserText { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The listing lines of a request, in order.
fn listed(user: &str) -> Vec<String> {
    user.lines()
        .filter(|line| line.starts_with("- page:"))
        .map(str::to_owned)
        .collect()
}

/// The four-letter floor on plurals, the other way from `news`: a three-letter singular said for
/// a plural page name (`log` for `logs`) does not name it. cd1e73c listed it. This is the accepted
/// cost of the floor.
#[test]
fn a_three_letter_singular_does_not_name_its_plural_page() {
    let doc = library_with_pages(&[("logs", "Logs")]);
    let user = first_request(&doc, "/", "use the loan card on the log page");
    assert!(
        !user.contains("page:logs/section:queue"),
        "`log` lists the `logs` page. The four-letter floor on plurals is the accepted cost: no \
         letter count separates new/news from log/logs, so a plural of a three-letter singular is \
         not matched. Changing this needs a rule that still keeps \"new\" from naming `news`: \
         {user}"
    );
}

/// `named_pages` promises "the target page, then every other page"; the listing is in document
/// order instead, so the target page comes after any earlier page the utterance names.
#[test]
fn the_target_page_is_listed_first() {
    let doc = library_with_pages(&[]);
    let user = first_request(&doc, "page:members", "put the loan card here");
    let lines = listed(&user);
    assert!(
        lines
            .first()
            .is_some_and(|line| line.starts_with("- page:members/")),
        "the target page is not first: {lines:#?}"
    );
}

/// What the correction claims, held at the edges: multi-word titles, case, digits, apostrophes,
/// words at the utterance's start and end, and the floor keeping `new` from naming `news`.
#[test]
fn whole_word_matching_holds_at_the_edges() {
    let doc = library_with_pages(&[
        ("late", "Overdue loans"),
        ("q3_report", "Q3 report"),
        ("news", "News"),
        ("cities", "Cities"),
    ]);
    let pages = |utterance: &str| {
        let lines = listed(&first_request(&doc, "/", utterance));
        let mut names: Vec<String> = lines
            .iter()
            .filter_map(|line| {
                line.strip_prefix("- page:")
                    .and_then(|rest| rest.split('/').next())
                    .map(str::to_owned)
            })
            .collect();
        names.dedup();
        names
    };
    assert_eq!(pages("Overdue Loan, as a card"), ["loans", "late"]);
    assert_eq!(pages("overdue"), Vec::<String>::new());
    assert_eq!(pages("card for the Q3 REPORT"), ["q3_report"]);
    assert_eq!(pages("the member's page"), ["members"]);
    assert_eq!(pages("members"), ["members"]);
    assert_eq!(pages("add a new card, city by city"), ["cities"]);
    assert_eq!(pages("news."), ["news"]);
}
