//! Adversarial cases, pass 2, for story:replace-drops (89d2c46): the batch judged by its result.

use std::path::Path;

use serde_json::json;
use uilab_doc::model::Composite;
use uilab_doc::{Child, Document, Layer, NodePath, Patch, Severity, admit, check};

fn library() -> Document {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    Document::from_yaml(&std::fs::read_to_string(file).unwrap()).unwrap()
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

fn replace(target: &str, node: serde_json::Value) -> Patch {
    Patch::Replace {
        target: path(target),
        node,
    }
}

fn remove(target: &str) -> Patch {
    Patch::Remove {
        target: path(target),
    }
}

fn batch(target: &str, patches: Vec<Patch>) -> Patch {
    Patch::Batch {
        target: path(target),
        patches,
    }
}

/// The `replace_drops` findings of an admitted patch as `(path, message)`.
fn drops(doc: &Document, patch: &Patch) -> Vec<(String, String)> {
    let (next, findings) = admit(doc, patch).unwrap_or_else(|r| panic!("refused: {r}"));
    assert!(
        !check(&next).iter().any(|f| f.check == "replace_drops"),
        "check never reports replace_drops"
    );
    findings
        .into_iter()
        .filter(|f| f.check == "replace_drops")
        .map(|f| {
            assert_eq!(f.severity, Severity::Warning);
            (f.path, f.message)
        })
        .collect()
}

/// The library overview's `board`, which `dashboard_page` contributes and the overview removes.
fn no_board() -> serde_json::Value {
    json!({"name": "board", "remove": true})
}

fn on_loan() -> serde_json::Value {
    json!({"name": "on_loan", "component": "metric", "label": "Copies on loan",
        "reads": {"view": "loans.Summary"}, "from": "on_loan"})
}

fn recent() -> serde_json::Value {
    json!({"name": "recent", "component": "collection",
        "reads": {"view": "loans.All", "params": {"size": 5}},
        "columns": [{"field": "title"}, {"field": "member"}, {"field": "due"}]})
}

/// The batch removes a section by name and then rewrites the page without it. The section went
/// by the remove the operator asked for; the replace dropped nothing, and the warning says the
/// replace did. `replace at <path> drops <what>` is only true of what the replace removed.
#[test]
fn a_remove_then_a_replace_of_the_parent_does_not_blame_the_replace() {
    let doc = library();
    let patch = batch(
        "page:overview",
        vec![
            remove("page:overview/section:recent"),
            replace(
                "page:overview",
                json!({"kind": "dashboard_page", "title": "Overview",
                    "sections": [no_board(), on_loan()]}),
            ),
        ],
    );
    assert_eq!(drops(&doc, &patch), []);
}

/// The same with the order reversed: the replace keeps the section and only retitles the page;
/// a later remove takes the section.
#[test]
fn a_replace_then_a_remove_of_its_child_does_not_blame_the_replace() {
    let doc = library();
    let patch = batch(
        "page:overview",
        vec![
            replace(
                "page:overview",
                json!({"kind": "dashboard_page", "title": "Home",
                    "sections": [no_board(), on_loan(), recent()]}),
            ),
            remove("page:overview/section:recent"),
        ],
    );
    assert_eq!(drops(&doc, &patch), []);
}

/// Control for the two above: a remove elsewhere in the batch does not hide what the replace
/// itself dropped (the drawer goes by remove, the row action that opened it by the replace).
#[test]
fn a_remove_elsewhere_does_not_hide_what_the_replace_dropped() {
    let doc = library();
    let patch = batch(
        "page:loans",
        vec![
            remove("page:loans/overlay:edit"),
            replace(
                "page:loans/section:list",
                json!({"component": "collection",
                    "reads": {"view": "loans.All", "paging": "server"},
                    "columns": [{"field": "title"}, {"field": "member"}, {"field": "due"},
                        {"field": "state", "as": "tag"}]}),
            ),
        ],
    );
    assert_eq!(
        drops(&doc, &patch),
        [(
            "page:loans/section:list".to_owned(),
            "replace at page:loans/section:list drops row_actions Extend".to_owned()
        )]
    );
}

/// One target replaced twice is judged once: a single finding, not none (the `j < i` guard
/// turned into `j != i`) and not two (the guard dropped).
#[test]
fn the_same_target_replaced_twice_is_judged_once() {
    let doc = library();
    let patch = batch(
        "page:members/section:list",
        vec![
            replace(
                "page:members/section:list",
                json!({"component": "collection",
                    "reads": {"view": "members.All"},
                    "columns": [{"field": "name"}, {"field": "joined"}, {"field": "loans"},
                        {"field": "standing", "as": "tag"}]}),
            ),
            replace(
                "page:members/section:list",
                json!({"component": "collection",
                    "reads": {"view": "members.All"}, "columns": [{"field": "name"}]}),
            ),
        ],
    );
    assert_eq!(
        drops(&doc, &patch),
        [(
            "page:members/section:list".to_owned(),
            "replace at page:members/section:list drops columns joined, loans, standing".to_owned()
        )]
    );
}

/// A stored document may carry `item` in the older map form; a replace written in the list form
/// is compared node by node, so only the item that goes is named.
#[test]
fn items_stored_as_a_map_are_matched_against_a_list() {
    let mut doc = library();
    let section: Composite = serde_json::from_value(json!({"component": "collection",
        "reads": {"view": "members.All"},
        "columns": [{"field": "name"}, {"field": "joined"}, {"field": "loans"},
            {"field": "standing", "as": "tag"}],
        "item": {"tag": {"primitive": "badge", "text": "row.standing"},
            "who": {"primitive": "text", "text": "row.name"}}}))
    .unwrap();
    assert_eq!(section.item.len(), 2, "the map form is read");
    doc.pages["members"].sections["list"] = Some(section);
    let patch = replace(
        "page:members/section:list",
        json!({"component": "collection", "reads": {"view": "members.All"},
            "columns": [{"field": "name"}, {"field": "joined"}, {"field": "loans"},
                {"field": "standing", "as": "tag"}],
            "item": [{"name": "who", "primitive": "text", "text": "row.name"}]}),
    );
    assert_eq!(
        drops(&doc, &patch),
        [(
            "page:members/section:list".to_owned(),
            "replace at page:members/section:list drops item tag".to_owned()
        )]
    );
}

/// An overlay inserted and then replaced in one batch: the stored document never had it, so
/// nothing it lost on the way is named.
#[test]
fn an_insert_then_a_replace_of_the_new_node_names_nothing() {
    let doc = library();
    let patch = batch(
        "page:members",
        vec![
            Patch::Insert {
                target: path("page:members"),
                child: Child {
                    layer: Layer::Overlay,
                    name: "invite".into(),
                    node: json!({"kind": "drawer", "component": "form", "title": "Invite", "does": "members.Invite",
                        "fields": ["name", "joined"]}),
                    nav_section: None,
                },
            },
            replace(
                "page:members/overlay:invite",
                json!({"kind": "drawer", "component": "form", "title": "Invite", "does": "members.Invite",
                    "fields": ["name"]}),
            ),
        ],
    );
    assert_eq!(drops(&doc, &patch), []);
}
