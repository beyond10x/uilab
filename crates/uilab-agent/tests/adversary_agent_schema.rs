//! Adversary pass 1 on story:essui-agent-schema. Each case holds the patch schema the agent is
//! sent to the story's contract: a patch valid against it is admitted with no ESS error, or the
//! schema refuses it; and the prompt teaches shapes ESS admits.

use std::path::Path;

use harness_loop::OutputSchema;
use serde_json::{Value, json};
use uilab_agent::INSTRUCTIONS;
use uilab_doc::{Document, NodePath, Patch, Severity, admit, check, patch_schema};

fn library() -> Document {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library");
    let text = std::fs::read_to_string(dir.join("library.ui.yaml")).unwrap();
    Document::from_yaml_in(&text, &dir).unwrap()
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

fn errors(doc: &Document) -> Vec<String> {
    check(doc)
        .into_iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| format!("{} at {}: {}", f.check, f.path, f.message))
        .collect()
}

fn valid(doc: &Document, patch: &Value) -> Result<(), String> {
    let target = path(patch["target"].as_str().unwrap());
    let schema = patch_schema(doc, &target).map_err(|e| e.to_string())?;
    OutputSchema::new(schema)
        .map_err(|e| e.to_string())?
        .validate(patch)
}

/// What uilab does with a patch: admitted with its error findings, or refused.
fn admitted(doc: &Document, patch: &Value) -> Result<Vec<String>, String> {
    let parsed: Patch =
        serde_json::from_value(patch.clone()).map_err(|e| format!("not a patch: {e}"))?;
    admit(doc, &parsed)
        .map(|(next, _)| errors(&next))
        .map_err(|r| format!("{}: {}", r.check, r.message))
}

fn insert(target: &str, layer: &str, name: &str, node: Value) -> Value {
    json!({"op": "insert", "target": target, "child": {"layer": layer, "name": name, "node": node}})
}

/// The prompt's own placeholder example, written exactly as `INSTRUCTIONS` shows it, as a new
/// section of the loans page: the shape the prompt teaches for data the model lacks.
#[test]
fn the_prompts_placeholder_example_is_admitted() {
    assert!(
        INSTRUCTIONS.contains(
            "`reads: {placeholder: \\\nloans.Overdue, fixture: fixtures/loans_overdue.yaml}`"
        ) || INSTRUCTIONS.contains("loans.Overdue, fixture: fixtures/loans_overdue.yaml}"),
        "the prompt's example changed"
    );
    let doc = library();
    let patch = insert(
        "page:loans",
        "section",
        "overdue",
        json!({
            "component": "collection",
            "reads": {"placeholder": "loans.Overdue", "fixture": "fixtures/loans_overdue.yaml"},
            "columns": [{"field": "title"}, {"field": "member"}, {"field": "due"}],
        }),
    );
    valid(&doc, &patch).unwrap_or_else(|why| panic!("the schema refuses it: {why}"));
    let result = admitted(&doc, &patch);
    assert_eq!(
        result,
        Ok(Vec::new()),
        "the placeholder read the prompt teaches is not admitted; the agent cannot create the fixture file it names"
    );
}

/// A widget declared and used in one batch, as the prompt tells the agent to: params written the
/// way `INSTRUCTIONS` teaches them (`{type: …, required: true}`), the batch starting at `/`.
#[test]
fn a_widget_declared_as_the_prompt_teaches_is_admitted_in_a_batch() {
    assert!(
        INSTRUCTIONS.contains("`params` (each `{type: …, required: true}` or with a \\\n`default`")
            || INSTRUCTIONS
                .contains("`params` (each `{type: …, required: true}` or with a `default`"),
        "the prompt's param shape changed"
    );
    let doc = library();
    let batch = json!({
        "op": "batch",
        "target": "/",
        "patches": [
            insert("/", "component", "member_card", json!({
                "summary": "A member as a card.",
                "params": {"member": {"type": "string", "required": true, "note": "the member shown"}},
                "body": [{"name": "title", "primitive": "text", "text": "args.member"}],
            })),
            insert("page:members/section:list", "item", "card",
                json!({"component": "member_card", "args": {"member": "row.name"}})),
        ],
    });
    valid(&doc, &batch).unwrap_or_else(|why| panic!("the schema refuses it: {why}"));
    assert_eq!(
        admitted(&doc, &batch),
        Ok(Vec::new()),
        "the widget params the prompt teaches are refused"
    );
}

/// The same declaration with each param's `note`, as ESS's `Widget.params` record requires: a
/// batch that declares a widget and uses it is admitted.
#[test]
fn a_batch_declaring_a_widget_and_using_it_is_admitted() {
    let doc = library();
    let batch = json!({
        "op": "batch",
        "target": "/",
        "patches": [
            insert("/", "component", "member_card", json!({
                "summary": "A member as a card.",
                "params": {"member": {"type": "string", "required": true, "note": "the name"}},
                "body": [{"name": "title", "primitive": "text", "text": "args.member"}],
            })),
            insert("page:members/section:list", "item", "card",
                json!({"component": "member_card", "args": {"member": "row.name"}})),
        ],
    });
    valid(&doc, &batch).unwrap_or_else(|why| panic!("the schema refuses it: {why}"));
    assert_eq!(admitted(&doc, &batch), Ok(Vec::new()));
}

/// An empty node is refused in a batch (`batch_patches_carry_nodes`); the same empty node at the
/// top level, as a section or as an item, is a patch that holds no node all the same.
#[test]
fn an_empty_node_is_refused_by_the_schema_at_the_top_level_too() {
    let doc = library();
    let mut accepted = Vec::new();
    for patch in [
        insert("page:loans", "section", "more", json!({})),
        insert("page:loans/section:list", "item", "more", json!({})),
        json!({"op": "replace", "target": "page:overview/section:on_loan", "node": {}}),
    ] {
        if valid(&doc, &patch).is_ok() {
            accepted.push(format!("{patch}\n  uilab: {:?}", admitted(&doc, &patch)));
        }
    }
    assert!(
        accepted.is_empty(),
        "the schema accepts an empty node outside a batch:\n{}",
        accepted.join("\n")
    );
}

/// An overlay naming `same_as` need not repeat `kind` (the schema requires only `component`
/// then): one that leaves `kind` out is admitted.
#[test]
fn an_overlay_same_as_another_without_kind_is_admitted() {
    let doc = library();
    let patch = insert(
        "page:members",
        "overlay",
        "extend",
        json!({"component": "form", "same_as": "loans.edit"}),
    );
    valid(&doc, &patch).unwrap_or_else(|why| panic!("the schema refuses it: {why}"));
    assert_eq!(admitted(&doc, &patch), Ok(Vec::new()));
}

/// A section the page kind contributes, refined without its `component` (the schema's "refines
/// the node of the same name its page kind contributes"): `list_page` contributes `filters`.
#[test]
fn a_section_refining_an_inherited_one_is_admitted() {
    let doc = library();
    let patch = insert("page:loans", "section", "filters", json!({"reset": true}));
    valid(&doc, &patch).unwrap_or_else(|why| panic!("the schema refuses it: {why}"));
    assert_eq!(admitted(&doc, &patch), Ok(Vec::new()));
}

/// The other direction, on nodes ESS 0.48.0 checks with 0 findings (`ess ui check`): an overlay
/// with `visible` and `degrades` (`no_drawer` applies to overlays, `Degrades.capabilities`), and
/// a nested primitive with `degrades` and `state` (ESS's loader takes a node's common keys on
/// every node). Written back unchanged as a replace, each must be valid against its schema, or
/// the agent can never edit that node without dropping what it holds.
#[test]
fn nodes_ess_admits_are_valid_against_their_schema_when_written_back() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library");
    let text = std::fs::read_to_string(dir.join("library.ui.yaml"))
        .unwrap()
        .replacen(
            "        kind: drawer\n",
            "        kind: drawer\n        visible: \"true\"\n        degrades: {no_drawer: dialog}\n",
            1,
        )
        .replacen(
            "        row_actions: [{opens: edit, label: Extend}]\n",
            "        row_actions: [{opens: edit, label: Extend}]\n        item: [{name: hint, primitive: text, text: x, degrades: {no_live: poll}, state: {open: {type: boolean, class: component_state}}}]\n",
            1,
        );
    let doc = Document::from_yaml_in(&text, &dir).unwrap();
    assert!(
        errors(&doc).is_empty(),
        "ESS admits it: {:#?}",
        errors(&doc)
    );

    let mut refused = Vec::new();
    for at in [
        "page:loans/overlay:edit",
        "page:loans/section:list/item:hint",
    ] {
        let node = match uilab_doc::resolve(&doc, &path(at)).unwrap() {
            uilab_doc::path::NodeRef::Overlay(o) => serde_json::to_value(o).unwrap(),
            uilab_doc::path::NodeRef::Primitive(p) => serde_json::to_value(p).unwrap(),
            other => panic!("{at}: {other:?}"),
        };
        let patch = json!({"op": "replace", "target": at, "node": node});
        if let Err(why) = valid(&doc, &patch) {
            refused.push(format!("{at}: {why}"));
        }
    }
    assert!(
        refused.is_empty(),
        "the schema refuses nodes ESS admits:\n{}",
        refused.join("\n")
    );
}

/// The pinned eval case `retarget-none` now says "sort this table by the due date" at
/// `page:loans/section:list`. The answer ESS admits (`ess ui check`: 0 findings, and uilab's
/// admit) is a replace adding `sort: {by: due}`: ESS's loader defaults `Sort.allowed` to empty.
/// The agent's schema must accept it.
#[test]
fn sorting_the_loans_table_as_ess_admits_is_valid_against_the_schema() {
    let doc = library();
    let at = path("page:loans/section:list");
    let mut node =
        serde_json::to_value(uilab_doc::resolve(&doc, &at).unwrap().composite().unwrap()).unwrap();
    node.as_object_mut().unwrap().remove("name");
    node["sort"] = json!({"by": "due"});
    let patch = json!({"op": "replace", "target": at.to_string(), "node": node});
    assert_eq!(admitted(&doc, &patch), Ok(Vec::new()), "ESS admits it");
    valid(&doc, &patch).unwrap_or_else(|why| panic!("the schema refuses it: {why}"));
}
