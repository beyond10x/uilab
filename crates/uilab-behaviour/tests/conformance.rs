//! Replays `generated/suite.json`, the conformance suite ESS synthesized from the session domain,
//! against the generated port over [`Behaviour`].
//!
//! Every scenario runs on a fresh session whose source opens any path as `examples/empty.ui.yaml`
//! and whose [`Externals`] admit the suite's placeholder paths and bodies; a
//! `configure_external_outcome` step forces the named outcome of the next execution of its command.
//! An unknown step kind fails its scenario; nothing is skipped.

use std::collections::HashMap;
use std::path::Path;

use serde_json::{Map, Value, json};
use uilab_behaviour::{Behaviour, Externals, Forced, MemorySource, body_from_json};
use uilab_doc::Document;
use uilab_session::{PublishedEvent, UilabSession};
use uilab_types::primitives::Uuid;
use uilab_types::session::{self as s, PatchOp, ProposalState};

type Row = Map<String, Value>;

/// What the last executed command answered and published.
struct Last {
    command: String,
    outcome: String,
    error: Option<&'static str>,
    events: Vec<(&'static str, Row)>,
}

struct Run {
    session: UilabSession<Behaviour<MemorySource>>,
    externals: Externals,
    instances: HashMap<String, String>,
    observed: HashMap<&'static str, Row>,
    last: Option<Last>,
    views: HashMap<String, Vec<Row>>,
    snapshots: HashMap<String, (String, Row)>,
}

fn empty() -> Document {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/empty.ui.yaml");
    Document::from_yaml(&std::fs::read_to_string(file).unwrap()).unwrap()
}

fn uuid_of(text: &str) -> Uuid {
    Uuid(text.to_owned())
}

fn op_name(op: PatchOp) -> &'static str {
    match op {
        PatchOp::Insert => "Insert",
        PatchOp::Replace => "Replace",
        PatchOp::Remove => "Remove",
    }
}

fn op_named(name: &str) -> Result<PatchOp, String> {
    match name {
        "Insert" => Ok(PatchOp::Insert),
        "Replace" => Ok(PatchOp::Replace),
        "Remove" => Ok(PatchOp::Remove),
        other => Err(format!("`{other}` is not a PatchOp")),
    }
}

fn state_name(state: ProposalState) -> &'static str {
    match state {
        ProposalState::Accepted => "Accepted",
        ProposalState::Proposed => "Proposed",
        ProposalState::Rejected => "Rejected",
        ProposalState::Undone => "Undone",
    }
}

fn row(value: Value) -> Row {
    match value {
        Value::Object(map) => map,
        _ => unreachable!("rows are built as objects"),
    }
}

fn event(published: &PublishedEvent) -> (&'static str, Row) {
    match published {
        PublishedEvent::DocumentOpened(e) => (
            "uilab.session.DocumentOpened",
            row(json!({"document_id": e.document_id.0.0, "path": e.path})),
        ),
        PublishedEvent::NodeSelected(e) => (
            "uilab.session.NodeSelected",
            row(json!({"document_id": e.document_id.0.0, "path": e.path.0})),
        ),
        PublishedEvent::PatchProposed(e) => (
            "uilab.session.PatchProposed",
            row(json!({
                "proposal_id": e.proposal_id.0.0,
                "document_id": e.document_id.0.0,
                "target": e.target.0,
                "op": op_name(e.op),
            })),
        ),
        PublishedEvent::ProposalAccepted(e) => (
            "uilab.session.ProposalAccepted",
            row(json!({"proposal_id": e.proposal_id.0.0})),
        ),
        PublishedEvent::ProposalRejected(e) => (
            "uilab.session.ProposalRejected",
            row(json!({"proposal_id": e.proposal_id.0.0})),
        ),
        PublishedEvent::ProposalUndone(e) => (
            "uilab.session.ProposalUndone",
            row(json!({"proposal_id": e.proposal_id.0.0})),
        ),
    }
}

const WRONG_STATE: (&str, Option<&str>) =
    ("wrong-state", Some("uilab.session.ProposalStateConflict"));
const PATCH_REFUSED: Option<&str> = Some("uilab.session.PatchRefused");

fn str_field<'a>(object: &'a Value, name: &str) -> Result<&'a str, String> {
    object
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("no string `{name}` in {object}"))
}

impl Run {
    fn new() -> Self {
        let externals = Externals::new();
        externals.admit_everything(true);
        let source = MemorySource::new().with_fallback(empty());
        Run {
            session: UilabSession::new(Behaviour::with_externals(source, externals.clone())),
            externals,
            instances: HashMap::new(),
            observed: HashMap::new(),
            last: None,
            views: HashMap::new(),
            snapshots: HashMap::new(),
        }
    }

    /// The value an input or expectation names: a literal, a captured instance, or a field of the
    /// last event of a kind seen in this scenario.
    fn resolve(&self, reference: &Value) -> Result<Value, String> {
        match str_field(reference, "kind")? {
            "literal" => reference
                .get("value")
                .cloned()
                .ok_or_else(|| format!("a literal without a value: {reference}")),
            "instance" => {
                let name = str_field(reference, "instance")?;
                self.instances
                    .get(name)
                    .map(|id| Value::String(id.clone()))
                    .ok_or_else(|| format!("instance `{name}` was never captured"))
            }
            "observed" => {
                let name = str_field(reference, "event")?;
                let field = str_field(reference, "field")?;
                self.observed
                    .get(name)
                    .and_then(|e| e.get(field))
                    .cloned()
                    .ok_or_else(|| format!("no `{name}` observed with a `{field}`"))
            }
            other => Err(format!("unhandled input kind `{other}`")),
        }
    }

    fn inputs(&self, step: &Value) -> Result<Row, String> {
        let input = step
            .get("input")
            .and_then(Value::as_object)
            .ok_or("a command without input")?;
        input
            .iter()
            .map(|(name, reference)| Ok((name.clone(), self.resolve(reference)?)))
            .collect()
    }

    fn execute(&mut self, step: &Value) -> Result<(), String> {
        let command = str_field(step, "command")?.to_owned();
        let input = self.inputs(step)?;
        let text = |name: &str| -> Result<String, String> {
            input
                .get(name)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| format!("input `{name}` is not a string"))
        };
        let session = &mut self.session;
        let unmet =
            |e: uilab_types::obligation::UnmetObligation| format!("unmet obligation: {e:?}");
        let (outcome, error): (&str, Option<&'static str>) = match command.as_str() {
            "uilab.session.OpenDocument" => {
                match session
                    .open_document(s::OpenDocument {
                        path: text("path")?,
                    })
                    .map_err(unmet)?
                {
                    s::OpenDocumentOutcome::Opened { .. } => ("opened", None),
                    s::OpenDocumentOutcome::Unreadable { .. } => {
                        ("unreadable", Some("uilab.session.DocumentUnreadable"))
                    }
                }
            }
            "uilab.session.SelectNode" => {
                let input = s::SelectNode {
                    document_id: s::DocumentId(uuid_of(&text("document_id")?)),
                    path: s::NodePath(text("path")?),
                };
                match session.select_node(input).map_err(unmet)? {
                    s::SelectNodeOutcome::Selected { .. } => ("selected", None),
                    s::SelectNodeOutcome::UnknownDocument { .. } => {
                        ("unknown-document", Some("uilab.session.DocumentNotOpen"))
                    }
                    s::SelectNodeOutcome::NotFound { .. } => {
                        ("not-found", Some("uilab.session.NodeNotFound"))
                    }
                }
            }
            "uilab.session.ProposePatch" => {
                let input = s::ProposePatch {
                    document_id: s::DocumentId(uuid_of(&text("document_id")?)),
                    target: s::NodePath(text("target")?),
                    op: op_named(&text("op")?)?,
                    utterance: text("utterance")?,
                    body: input.get("body").map(body_from_json),
                };
                match session.propose_patch(input).map_err(unmet)? {
                    s::ProposePatchOutcome::Proposed { .. } => ("proposed", None),
                    s::ProposePatchOutcome::Refused { .. } => ("refused", PATCH_REFUSED),
                }
            }
            "uilab.session.AcceptProposal" => {
                let input = s::AcceptProposal {
                    proposal_id: s::ProposalId(uuid_of(&text("proposal_id")?)),
                };
                match session.accept_proposal(input).map_err(unmet)? {
                    s::AcceptProposalOutcome::Accepted { .. } => ("accepted", None),
                    s::AcceptProposalOutcome::Stale { .. } => ("stale", PATCH_REFUSED),
                    s::AcceptProposalOutcome::WrongState { .. }
                    | s::AcceptProposalOutcome::WrongStateUnknownInstance => WRONG_STATE,
                }
            }
            "uilab.session.RejectProposal" => {
                let input = s::RejectProposal {
                    proposal_id: s::ProposalId(uuid_of(&text("proposal_id")?)),
                };
                match session.reject_proposal(input).map_err(unmet)? {
                    s::RejectProposalOutcome::Rejected { .. } => ("rejected", None),
                    s::RejectProposalOutcome::WrongState { .. }
                    | s::RejectProposalOutcome::WrongStateUnknownInstance => WRONG_STATE,
                }
            }
            "uilab.session.UndoProposal" => {
                let input = s::UndoProposal {
                    proposal_id: s::ProposalId(uuid_of(&text("proposal_id")?)),
                };
                match session.undo_proposal(input).map_err(unmet)? {
                    s::UndoProposalOutcome::Undone { .. } => ("undone", None),
                    s::UndoProposalOutcome::Stale { .. } => ("stale", PATCH_REFUSED),
                    s::UndoProposalOutcome::WrongState { .. }
                    | s::UndoProposalOutcome::WrongStateUnknownInstance => WRONG_STATE,
                }
            }
            other => return Err(format!("unhandled command `{other}`")),
        };
        let events: Vec<_> = self.session.drain_outbox().iter().map(event).collect();
        for (name, payload) in &events {
            self.observed.insert(name, payload.clone());
        }
        self.last = Some(Last {
            command,
            outcome: outcome.to_owned(),
            error,
            events,
        });
        Ok(())
    }

    fn last(&self) -> Result<&Last, String> {
        self.last
            .as_ref()
            .ok_or_else(|| "no command has run".to_owned())
    }

    fn query(&mut self, view: &str) -> Result<(), String> {
        let unmet =
            |e: uilab_types::obligation::UnmetObligation| format!("unmet obligation: {e:?}");
        let rows: Vec<Row> = match view {
            "uilab.session.Documents" => self
                .session
                .documents()
                .map_err(unmet)?
                .into_iter()
                .map(|r| row(json!({"document_id": r.document_id.0.0, "path": r.path, "selected": r.selected.0})))
                .collect(),
            "uilab.session.Pending" => self
                .session
                .pending()
                .map_err(unmet)?
                .into_iter()
                .map(|r| row(json!({"proposal_id": r.proposal_id.0.0, "target": r.target.0, "op": op_name(r.op)})))
                .collect(),
            "uilab.session.Proposals" => self
                .session
                .proposals()
                .map_err(unmet)?
                .into_iter()
                .map(|r| {
                    row(json!({
                        "proposal_id": r.proposal_id.0.0,
                        "document_id": r.document_id.0.0,
                        "target": r.target.0,
                        "op": op_name(r.op),
                        "utterance": r.utterance,
                        "state": state_name(r.state),
                    }))
                })
                .collect(),
            other => return Err(format!("unhandled view `{other}`")),
        };
        self.views.insert(view.to_owned(), rows);
        Ok(())
    }

    fn rows(&self, view: &str) -> Result<&Vec<Row>, String> {
        self.views
            .get(view)
            .ok_or_else(|| format!("`{view}` was never queried"))
    }

    /// Whether `row` carries every field of `fields`, each with the value it names.
    fn matches(&self, row: &Row, fields: &Map<String, Value>) -> Result<bool, String> {
        for (name, reference) in fields {
            if row.get(name) != Some(&self.resolve(reference)?) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn step(&mut self, step: &Value) -> Result<(), String> {
        let kind = str_field(step, "step")?;
        match kind {
            "execute_command" => self.execute(step),
            "expect_outcome" => {
                let want = step.get("outcome").ok_or("no outcome")?;
                let (command, outcome) = (str_field(want, "command")?, str_field(want, "outcome")?);
                let last = self.last()?;
                if last.command != command || last.outcome != outcome {
                    return Err(format!(
                        "expected {command} `{outcome}`, got {} `{}`",
                        last.command, last.outcome
                    ));
                }
                Ok(())
            }
            "expect_error" => {
                let want = str_field(step, "error")?;
                match self.last()?.error {
                    Some(got) if got == want => Ok(()),
                    got => Err(format!("expected error {want}, got {got:?}")),
                }
            }
            "capture_instance" => {
                let (name, event, field) = (
                    str_field(step, "instance")?,
                    str_field(step, "event")?,
                    str_field(step, "field")?,
                );
                let value = self
                    .last()?
                    .events
                    .iter()
                    .find(|(e, _)| *e == event)
                    .and_then(|(_, payload)| payload.get(field))
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("no {event} with `{field}` to capture"))?
                    .to_owned();
                self.instances.insert(name.to_owned(), value);
                Ok(())
            }
            "expect_event" => {
                let name = str_field(step, "event")?;
                let shape = step
                    .get("shape")
                    .and_then(Value::as_object)
                    .ok_or("an event without a shape")?;
                let payload = step.get("payload").and_then(Value::as_object);
                let candidates: Vec<_> = self
                    .last()?
                    .events
                    .iter()
                    .filter(|(e, _)| *e == name)
                    .collect();
                if candidates.is_empty() {
                    return Err(format!("expected {name}, none published"));
                }
                let mut why = Vec::new();
                for (_, event) in candidates {
                    match conforms(event, shape, payload) {
                        Ok(()) => return Ok(()),
                        Err(e) => why.push(e),
                    }
                }
                Err(format!("no {name} conforms: {}", why.join("; ")))
            }
            "expect_no_event" => {
                let name = str_field(step, "event")?;
                if self.last()?.events.iter().any(|(e, _)| *e == name) {
                    return Err(format!("{name} was published"));
                }
                Ok(())
            }
            "expect_no_events" => {
                let events = &self.last()?.events;
                if !events.is_empty() {
                    return Err(format!(
                        "published {:?}",
                        events.iter().map(|(e, _)| *e).collect::<Vec<_>>()
                    ));
                }
                Ok(())
            }
            "query_view" => self.query(str_field(step, "view")?),
            "expect_view" => {
                let view = str_field(step, "view")?;
                let expectation = step.get("expectation").ok_or("no expectation")?;
                let fields = expectation
                    .get("fields")
                    .and_then(Value::as_object)
                    .ok_or("no fields")?;
                let mut found = false;
                for row in self.rows(view)? {
                    found |= self.matches(row, fields)?;
                }
                match (str_field(expectation, "expect")?, found) {
                    ("contains", true) | ("excludes", false) => Ok(()),
                    ("contains", false) => Err(format!(
                        "{view} has no row with {fields:?}: {:?}",
                        self.rows(view)?
                    )),
                    ("excludes", true) => Err(format!("{view} has a row with {fields:?}")),
                    (other, _) => Err(format!("unhandled view expectation `{other}`")),
                }
            }
            "snapshot_complete_subject" => {
                let view = str_field(step, "view")?;
                let subject = step
                    .get("subject")
                    .and_then(Value::as_object)
                    .ok_or("no subject")?;
                let shape = step.get("shape").ok_or("no shape")?;
                let mut subjects = Vec::new();
                for row in self.rows(view)? {
                    if self.matches(row, subject)? {
                        subjects.push(row.clone());
                    }
                }
                let [subject_row] = subjects.as_slice() else {
                    return Err(format!(
                        "{view}: {} rows for the subject, expected one",
                        subjects.len()
                    ));
                };
                complete(subject_row, shape)?;
                let identity = str_field(shape, "identity_field")?.to_owned();
                self.snapshots
                    .insert(view.to_owned(), (identity, subject_row.clone()));
                Ok(())
            }
            "expect_complete_subject_unchanged" => {
                let view = str_field(step, "view")?;
                let (identity, before) = self
                    .snapshots
                    .get(view)
                    .ok_or_else(|| format!("no snapshot of {view}"))?;
                let now: Vec<_> = self
                    .rows(view)?
                    .iter()
                    .filter(|r| r.get(identity) == before.get(identity))
                    .collect();
                match now.as_slice() {
                    [row] if *row == before => Ok(()),
                    [row] => Err(format!("{view} subject changed: {before:?} -> {row:?}")),
                    rows => Err(format!(
                        "{view}: {} rows for the subject, expected one",
                        rows.len()
                    )),
                }
            }
            "configure_external_outcome" => {
                let force = step.get("force").ok_or("no force")?;
                let (command, outcome) =
                    (str_field(force, "command")?, str_field(force, "outcome")?);
                let forced = Forced::named(command, outcome).ok_or_else(|| {
                    format!("{command} `{outcome}` is not an externally decided outcome")
                })?;
                self.externals.force(forced);
                Ok(())
            }
            other => Err(format!("unhandled step kind `{other}`")),
        }
    }
}

/// Whether a primitive or enum value holds what an event shape declares.
fn holds(value: &Value, shape: &Value) -> Result<(), String> {
    match str_field(shape, "holds")? {
        "primitive" => match str_field(shape, "kind")? {
            "string" => value
                .as_str()
                .map(|_| ())
                .ok_or_else(|| format!("{value} is not a string")),
            "uuid" => is_uuid(value),
            other => Err(format!("unhandled primitive kind `{other}`")),
        },
        "enum" => is_variant(value, shape.get("variants")),
        other => Err(format!("unhandled shape `{other}`")),
    }
}

fn is_uuid(value: &Value) -> Result<(), String> {
    value
        .as_str()
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .map(|_| ())
        .ok_or_else(|| format!("{value} is not a uuid"))
}

fn is_variant(value: &Value, variants: Option<&Value>) -> Result<(), String> {
    let variants = variants
        .and_then(Value::as_array)
        .ok_or("an enum without variants")?;
    if variants.contains(value) {
        Ok(())
    } else {
        Err(format!("{value} is not one of {variants:?}"))
    }
}

/// An event conforms when it carries exactly the shape's fields, each holding its declared kind,
/// and every payload field has the declared value.
fn conforms(
    event: &Row,
    shape: &Map<String, Value>,
    payload: Option<&Map<String, Value>>,
) -> Result<(), String> {
    if event.len() != shape.len() || shape.keys().any(|k| !event.contains_key(k)) {
        return Err(format!(
            "fields {:?}, shape {:?}",
            event.keys().collect::<Vec<_>>(),
            shape.keys().collect::<Vec<_>>()
        ));
    }
    for (name, field_shape) in shape {
        holds(&event[name], field_shape).map_err(|e| format!("`{name}`: {e}"))?;
    }
    for (name, want) in payload.into_iter().flatten() {
        if event.get(name) != Some(want) {
            return Err(format!(
                "`{name}` is {:?}, expected {want}",
                event.get(name)
            ));
        }
    }
    Ok(())
}

/// A subject row is complete when it carries exactly the shape's fields, each of its declared type.
fn complete(row: &Row, shape: &Value) -> Result<(), String> {
    let fields = shape
        .get("fields")
        .and_then(Value::as_array)
        .ok_or("a shape without fields")?;
    let declarations = shape.get("declarations").and_then(Value::as_object);
    if row.len() != fields.len() {
        return Err(format!(
            "row has {} fields, shape {}",
            row.len(),
            fields.len()
        ));
    }
    for field in fields {
        let name = str_field(field, "name")?;
        let value = row
            .get(name)
            .ok_or_else(|| format!("row has no `{name}`"))?;
        of_type(value, str_field(field, "type")?, declarations)
            .map_err(|e| format!("`{name}`: {e}"))?;
    }
    Ok(())
}

fn of_type(
    value: &Value,
    ty: &str,
    declarations: Option<&Map<String, Value>>,
) -> Result<(), String> {
    match ty {
        "String" => value
            .as_str()
            .map(|_| ())
            .ok_or_else(|| format!("{value} is not a string")),
        "Uuid" => is_uuid(value),
        _ => {
            let declaration = declarations
                .and_then(|d| d.get(ty))
                .ok_or_else(|| format!("unhandled type `{ty}`"))?;
            match str_field(declaration, "kind")? {
                "newtype" => of_type(value, str_field(declaration, "of")?, declarations),
                "enum" => is_variant(value, declaration.get("variants")),
                other => Err(format!("unhandled declaration kind `{other}`")),
            }
        }
    }
}

#[test]
fn every_scenario_of_the_synthesized_suite_passes() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../generated/suite.json");
    let suite: Value = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let scenarios = suite["scenarios"].as_object().expect("scenarios by id");
    assert!(!scenarios.is_empty(), "the suite has no scenarios");

    let mut failed = Vec::new();
    let mut steps_run = 0;
    for (id, scenario) in scenarios {
        let steps = scenario["steps"].as_array().expect("steps");
        let mut run = Run::new();
        let result = steps.iter().enumerate().try_for_each(|(at, step)| {
            steps_run += 1;
            run.step(step)
                .map_err(|e| format!("step {at} ({}): {e}", step["step"]))
        });
        match result {
            Ok(()) => println!("PASS {id} ({} steps)", steps.len()),
            Err(why) => {
                println!("FAIL {id}: {why}");
                failed.push(id.clone());
            }
        }
    }
    println!(
        "{} of {} scenarios pass, {steps_run} steps run",
        scenarios.len() - failed.len(),
        scenarios.len()
    );
    assert!(failed.is_empty(), "failing scenarios: {failed:#?}");
}
