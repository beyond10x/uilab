//! Adversary pass 2 on story:essui-agent-schema, after correction 1 (the schema guides, ESS
//! admission judges). Each case holds the patch schema the agent is sent, and the refusal it
//! gets back, to what the agent can act on.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

use harness_loop::OutputSchema;
use harness_wire::{
    CallId, Item, ModelPort, StopReason, StreamSink, ToolCall, ToolName, TurnOutcome, TurnRequest,
    Usage, WireError, WireId,
};
use serde_json::{Value, json};
use uilab_agent::{Proposer, ProposerConfig};
use uilab_doc::{Document, NodePath, Patch, admit, patch_schema};

fn library() -> Document {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library");
    let text = std::fs::read_to_string(dir.join("library.ui.yaml")).unwrap();
    Document::from_yaml_in(&text, &dir).unwrap()
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

fn valid(doc: &Document, patch: &Value) -> Result<(), String> {
    let target = path(patch["target"].as_str().unwrap());
    let schema = patch_schema(doc, &target).map_err(|e| e.to_string())?;
    OutputSchema::new(schema)
        .map_err(|e| e.to_string())?
        .validate(patch)
}

/// What uilab's admission says about a patch: its refusal, or `admitted`.
fn judged(doc: &Document, patch: &Value) -> String {
    let parsed: Patch = match serde_json::from_value(patch.clone()) {
        Ok(parsed) => parsed,
        Err(e) => return format!("not a patch: {e}"),
    };
    match admit(doc, &parsed) {
        Ok(_) => "admitted".to_owned(),
        Err(r) => format!("{}: {}", r.check, r.message),
    }
}

fn insert(target: &str, layer: &str, name: &str, node: Value) -> Value {
    json!({"op": "insert", "target": target, "child": {"layer": layer, "name": name, "node": node}})
}

/// The schema's refusal of an inserted node names the value at fault, as its refusal of the same
/// node in a replace does (`5 is not of type "string"`). Every field shape this story added sits
/// under `child`'s `oneOf` (one branch per layer), so a mistake in any field of an inserted
/// section or item comes back as the whole child and "is not valid under any of the schemas
/// listed in the 'oneOf' keyword", which names no field.
#[test]
fn the_schema_refusal_of_an_inserted_node_names_the_value_at_fault() {
    let doc = library();
    let metric = |label: Value| json!({"component": "metric", "label": label, "from": "on_loan", "reads": {"view": "loans.Summary"}});
    // The control: the same mistake in a replace is named.
    let replace = json!({"op": "replace", "target": "page:overview/section:on_loan", "node": metric(json!(5))});
    let named = valid(&doc, &replace).unwrap_err();
    assert!(named.starts_with("5 "), "control: {named}");

    let mut opaque = Vec::new();
    for (at_fault, patch) in [
        (
            "5",
            insert("page:overview", "section", "copies", metric(json!(5))),
        ),
        (
            "\"donut\"",
            insert(
                "page:overview",
                "section",
                "by_state",
                json!({"component": "chart", "chart": "donut", "x": "state", "series": ["on_loan"], "reads": {"view": "loans.Summary"}}),
            ),
        ),
        (
            "\"purple\"",
            insert(
                "page:loans/section:list",
                "item",
                "flag",
                json!({"primitive": "badge", "text": "late", "tone": "purple"}),
            ),
        ),
    ] {
        let refusal = valid(&doc, &patch).expect_err("the schema refuses it");
        if !refusal.starts_with(&format!("{at_fault} ")) {
            opaque.push(format!(
                "{at_fault} at {}:\n  schema: {refusal}\n  admission would say: {}",
                patch["target"],
                judged(&doc, &patch)
            ));
        }
    }
    assert!(
        opaque.is_empty(),
        "the schema's refusal does not name the value at fault:\n{}",
        opaque.join("\n")
    );
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

/// The same refusal, as the agent receives it inside the loop: a metric section whose `label` is
/// a number, then the corrected answer. The tool result the model reads after its first answer
/// must say which value is wrong.
#[test]
fn the_agent_is_told_which_field_its_inserted_section_got_wrong() {
    let doc = library();
    let section = |label: Value| {
        insert(
            "page:overview",
            "section",
            "copies",
            json!({"component": "metric", "label": label, "from": "on_loan", "reads": {"view": "loans.Summary"}}),
        )
    };
    let seen = Arc::new(Mutex::new(Vec::new()));
    let model = ScriptedModel {
        wire: WireId::new("anthropic-messages").unwrap(),
        answers: vec![section(json!(5)), section(json!("Copies out"))].into(),
        seen: Arc::clone(&seen),
    };
    let mut proposer = Proposer::with_port(&ProposerConfig::default(), Box::new(model));
    proposer
        .propose(&doc, &path("page:overview"), "add the number of copies out")
        .expect("the corrected answer is admitted");

    let seen = seen.lock().unwrap();
    let told: Vec<String> = seen[1]
        .items
        .iter()
        .filter_map(|item| match item {
            Item::ToolResult {
                output,
                failed: true,
                ..
            } => Some(output.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(told.len(), 1, "one refusal: {told:?}");
    assert!(
        told[0].contains("5 is not of type"),
        "the agent is not told what is wrong with its answer: {}",
        told[0]
    );
}

/// A node has exactly one of `component` and `primitive` (`Node.exactly_one_of`). Correction 1
/// keeps that for a refinement (no tag); a tagged variant is open now and takes the other tag as
/// any other key, so a node carrying both is valid against the schema at a section, an overlay
/// and an item, and only admission refuses it.
#[test]
fn a_node_carrying_both_tags_is_refused_by_the_schema() {
    let doc = library();
    let mut accepted = Vec::new();
    for patch in [
        insert(
            "page:overview",
            "section",
            "both",
            json!({"component": "metric", "primitive": "text", "label": "x", "from": "on_loan", "reads": {"view": "loans.Summary"}}),
        ),
        insert(
            "page:loans",
            "overlay",
            "both",
            json!({"kind": "drawer", "component": "record", "primitive": "text", "fields": ["due"]}),
        ),
        insert(
            "page:loans/section:list",
            "item",
            "both",
            json!({"component": "metric", "primitive": "text", "from": "due", "label": "Due"}),
        ),
    ] {
        if valid(&doc, &patch).is_ok() {
            accepted.push(format!(
                "{} {}: admission says {}",
                patch["target"],
                patch["child"]["layer"],
                judged(&doc, &patch)
            ));
        }
    }
    assert!(
        accepted.is_empty(),
        "the schema accepts a node with both tags:\n{}",
        accepted.join("\n")
    );
}

/// The schema offers a node without its tag as "refines the node of the same name its page kind
/// contributes, writing only what differs". Where nothing is contributed it is never admitted:
/// a replace of a section its page kind does not contribute, an item (no page kind contributes
/// items), and an overlay of the shell (no page kind contributes those). The schema knows each
/// of these targets when it is built.
#[test]
fn a_node_without_its_tag_is_refused_where_nothing_is_inherited() {
    let doc = library();
    let mut accepted = Vec::new();
    for patch in [
        json!({"op": "replace", "target": "page:overview/section:on_loan", "node": {"label": "Copies out"}}),
        insert(
            "page:loans/section:list",
            "item",
            "due",
            json!({"from": "due", "label": "Due"}),
        ),
        insert(
            "shell:app",
            "overlay",
            "help",
            json!({"kind": "drawer", "title": "Help"}),
        ),
    ] {
        if valid(&doc, &patch).is_ok() {
            accepted.push(format!(
                "{} {}: admission says {}",
                patch["op"],
                patch["target"],
                judged(&doc, &patch)
            ));
        }
    }
    assert!(
        accepted.is_empty(),
        "the schema accepts an untagged node where nothing is inherited:\n{}",
        accepted.join("\n")
    );
}

/// An insert names its child (`child.name`); the section position also offers ESS's `name` inside
/// the node ("node name among the page's sections"). When the two differ, the schema accepts the
/// patch and admission keys the section by `child.name` while the node keeps its own `name`, so
/// the document as written (and as reloaded) names the section `other`: the section the patch
/// named, and that a later plan step targets, is gone after a save.
#[test]
fn an_inserted_section_keeps_the_name_the_patch_gives_it_when_written() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library");
    let doc = library();
    let patch = insert(
        "page:overview",
        "section",
        "copies",
        json!({"name": "other", "component": "metric", "label": "Copies out", "from": "on_loan", "reads": {"view": "loans.Summary"}}),
    );
    if let Err(why) = valid(&doc, &patch) {
        eprintln!("the schema refuses it: {why}");
        return;
    }
    let parsed: Patch = serde_json::from_value(patch).unwrap();
    let (next, _) = admit(&doc, &parsed).expect("admitted");
    let written = next.to_yaml().unwrap();
    let reloaded = Document::from_yaml_in(&written, &dir).unwrap();
    assert!(
        uilab_doc::resolve(&reloaded, &path("page:overview/section:copies")).is_ok(),
        "admitted as `section:copies` (in memory: {}), written and reloaded as `section:other` ({})",
        uilab_doc::resolve(&next, &path("page:overview/section:copies")).is_ok(),
        uilab_doc::resolve(&reloaded, &path("page:overview/section:other")).is_ok()
    );
}
