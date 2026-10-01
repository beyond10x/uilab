//! Adversarial cases for round 2 of the widget prompt: which pages a request lists, and what the
//! instructions tell the model about primitives. Scripted model port: no network.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

use harness_wire::{
    CallId, Item, ModelPort, StopReason, StreamSink, ToolCall, ToolName, TurnOutcome, TurnRequest,
    Usage, WireError, WireId,
};
use serde_json::{Value, json};
use uilab_agent::{INSTRUCTIONS, Proposer, ProposerConfig};
use uilab_doc::model::PrimitiveKind;
use uilab_doc::{Document, NodePath};

fn library_text() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    std::fs::read_to_string(path).unwrap()
}

/// The library with one more list page appended under `pages:` (the file's last key).
fn library_with_page(name: &str, title: &str) -> Document {
    let mut text = library_text();
    text.push_str(&format!(
        "  {name}:\n    kind: list_page\n    title: {title}\n    sections:\n      - name: queue\n        component: collection\n        reads: {{view: loans.All}}\n        columns: [{{field: title}}]\n"
    ));
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

/// The first user text the proposer sends for `utterance` at the root, answered by a decline.
fn first_request_at_root(doc: &Document, utterance: &str) -> String {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let model = ScriptedModel {
        wire: WireId::new("anthropic-messages").unwrap(),
        answers: vec![json!({"op": "decline", "target": "/", "reason": "scripted"})].into(),
        seen: Arc::clone(&seen),
    };
    let mut proposer = Proposer::with_port(&ProposerConfig::default(), Box::new(model));
    let _ = proposer.propose(doc, &NodePath::root(), utterance);
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

/// `named_pages` drops a trailing `s` and then needs three letters, so a page whose name is
/// two letters plus `s` is never listed, even when the operator says its exact name.
#[test]
fn a_page_with_a_short_plural_name_is_listed_when_the_operator_names_it() {
    let doc = library_with_page("ops", "Ops");
    let user = first_request_at_root(
        &doc,
        "make a reusable card for a loan and use it on the ops page",
    );
    assert!(
        user.contains("page:ops/section:queue"),
        "the ops page was named and is not listed: {user}"
    );
}

/// Stripping one `s` is the only plural rule: a `-ies` page is missed when said in the singular.
#[test]
fn a_page_with_an_ies_plural_name_is_listed_when_named_in_the_singular() {
    let doc = library_with_page("categories", "Categories");
    let user = first_request_at_root(&doc, "make a reusable card and use it on the category page");
    assert!(
        user.contains("page:categories/section:queue"),
        "the categories page was named and is not listed: {user}"
    );
}

/// A substring match on the stem lists `news` for every utterance that says "new".
#[test]
fn a_page_named_like_a_common_word_is_not_listed_when_only_the_word_is_said() {
    let doc = library_with_page("news", "News");
    let user = first_request_at_root(
        &doc,
        "make a new reusable card for a member and use it on the members page",
    );
    assert!(
        user.contains("page:members/section:list"),
        "the members page is listed: {user}"
    );
    assert!(
        !user.contains("page:news/section:queue"),
        "the news page was not named and is listed: {user}"
    );
}

/// The doc crate describes each primitive with the props it takes, in backticks. The prompt's
/// primitive list names them too, except `tone_by`, which `widgets.rs` requires the prompt to omit.
#[test]
fn the_instructions_name_every_prop_the_doc_crate_gives_a_primitive() {
    let mut missing = Vec::new();
    for kind in PrimitiveKind::ALL {
        let summary = kind.summary();
        for (i, part) in summary.split('`').enumerate() {
            if i % 2 == 1 && part != "tone_by" {
                let prop = format!("`{part}`");
                if !INSTRUCTIONS.contains(&prop) {
                    missing.push(format!("{}: {prop}", kind.as_str()));
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "INSTRUCTIONS omit primitive props the doc crate names: {missing:?}"
    );
}

/// A large named page: one listing line per section, none from another page, none below a
/// section.
#[test]
fn the_listing_of_a_large_page_is_one_line_per_section_and_nothing_else() {
    let mut text = library_text();
    for i in 0..120 {
        text.push_str(&format!(
            "      - name: extra_{i}\n        component: collection\n        reads: {{view: members.All}}\n        columns: [{{field: name}}]\n"
        ));
    }
    let doc = Document::from_yaml(&text).unwrap();
    assert_eq!(doc.pages["members"].sections.len(), 121);
    let user = first_request_at_root(&doc, "use a card on the members page");
    let listed: Vec<&str> = user.lines().filter(|l| l.starts_with("- page:")).collect();
    assert_eq!(listed.len(), 121, "{user}");
    assert!(
        listed
            .iter()
            .all(|l| l.starts_with("- page:members/section:"))
    );
    assert!(
        listed.iter().all(|l| l.matches('/').count() == 1),
        "no node below a section is listed"
    );
}
