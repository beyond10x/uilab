//! Adversarial cases for story:replace-drops (0a61776): what `admit` warns a replace drops.

use std::path::Path;

use serde_json::json;
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

/// The library example writes the drawer's form field as the string `due`; the renderer
/// (`widget/src/lib/outline.ts` `fieldsOf`) and this crate's own fixture reader
/// (`src/fixtures.rs`) read `due` and `{field: due}` as the same field. Giving it a label keeps
/// the field; nothing is dropped.
#[test]
fn a_string_field_rewritten_as_an_object_with_the_same_field_is_not_a_drop() {
    let doc = library();
    let labelled = replace(
        "page:loans/overlay:edit",
        json!({"kind": "drawer", "component": "form", "title": "Extend loan",
            "does": "loans.ExtendLoan", "fields": [{"field": "due", "label": "Due date"}]}),
    );
    assert_eq!(drops(&doc, &labelled), []);
}

/// A batch whose later replace puts back what an earlier replace took: the admitted document
/// still has every column the stored one had, and the proposal card should not say otherwise.
#[test]
fn a_batch_whose_later_replace_restores_the_columns_warns_nothing() {
    let doc = library();
    let batch = Patch::Batch {
        target: path("page:members/section:list"),
        patches: vec![
            replace(
                "page:members/section:list",
                json!({"component": "collection", "reads": {"view": "members.All"},
                    "columns": [{"field": "name"}]}),
            ),
            replace(
                "page:members/section:list",
                json!({"component": "collection", "reads": {"view": "members.All"},
                    "columns": [{"field": "name"}, {"field": "joined"}, {"field": "loans"},
                        {"field": "standing", "as": "tag"}, {"field": "email"}]}),
            ),
        ],
    };
    assert_eq!(drops(&doc, &batch), []);
}

/// An insert then a replace of its parent that leaves the new child out: the stored document
/// never had `edit_member`, so the replace drops nothing the operator has.
#[test]
fn an_insert_then_a_replace_without_it_names_nothing_the_document_had() {
    let doc = library();
    let batch = Patch::Batch {
        target: path("page:members"),
        patches: vec![
            Patch::Insert {
                target: path("page:members"),
                child: Child {
                    layer: Layer::Overlay,
                    name: "edit_member".into(),
                    node: json!({"kind": "drawer", "component": "form", "fields": ["name"]}),
                    nav_section: None,
                },
            },
            replace(
                "page:members",
                json!({"kind": "list_page", "title": "People", "sections": {"list": {
                    "component": "collection", "reads": {"view": "members.All"},
                    "columns": [{"field": "name"}, {"field": "joined"}, {"field": "loans"},
                        {"field": "standing", "as": "tag"}]}}}),
            ),
        ],
    };
    assert_eq!(drops(&doc, &batch), []);
}

/// Two columns over one field (the text and a tag); a replace keeping one of them drops the
/// other, and a column that goes is named.
#[test]
fn one_of_two_columns_over_the_same_field_dropped_is_named() {
    let mut doc = library();
    doc.pages["members"].sections["list"]
        .as_mut()
        .unwrap()
        .props
        .insert(
            "columns".into(),
            json!([{"field": "name"}, {"field": "standing"}, {"field": "standing", "as": "tag"}]),
        );
    let one = replace(
        "page:members/section:list",
        json!({"component": "collection", "reads": {"view": "members.All"},
            "columns": [{"field": "name"}, {"field": "standing", "as": "tag"}]}),
    );
    assert_eq!(
        drops(&doc, &one),
        [(
            "page:members/section:list".to_owned(),
            "replace at page:members/section:list drops columns standing".to_owned()
        )]
    );
}

/// Drops under children that stay, three layers down: page, board section, widget, item.
#[test]
fn a_drop_deep_under_children_that_stay_is_named_at_its_node() {
    let doc = Document::from_yaml(
        r#"
format: ui-spec/1
app: library
model: library
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: overview
  sections:
    - {name: circulation, pages: [overview]}
pages:
  overview:
    kind: dashboard_page
    sections:
      board:
        component: board
        widgets:
          due:
            component: collection
            reads: {view: loans.All}
            columns: [{field: title}, {field: due}]
            item:
              - {name: info, component: record, fields: [title, member]}
"#,
    )
    .unwrap();
    let page = replace(
        "page:overview",
        json!({"kind": "dashboard_page", "title": "Overview", "sections": {"board": {
            "component": "board", "widgets": {"due": {
                "component": "collection", "reads": {"view": "loans.All"},
                "columns": [{"field": "due"}, {"field": "title"}],
                "item": [{"name": "info", "component": "record", "fields": ["title"]}]}}}}}),
    );
    assert_eq!(
        drops(&doc, &page),
        [(
            "page:overview/section:board/widget:due/item:info".to_owned(),
            "replace at page:overview/section:board/widget:due/item:info drops fields member"
                .to_owned()
        )]
    );
}

/// Reordering, changing `as`, and re-stating the menu in another order drop nothing.
#[test]
fn reordering_and_changing_as_warn_nothing() {
    let doc = library();
    let reordered = replace(
        "page:loans/section:list",
        json!({"component": "collection", "reads": {"view": "loans.All", "paging": "server"},
            "columns": [{"field": "state", "as": "badge"}, {"field": "due"}, {"field": "member"},
                {"field": "title"}],
            "row_actions": [{"opens": "edit", "label": "Extend"}]}),
    );
    assert_eq!(drops(&doc, &reordered), []);
    let menu = replace(
        "nav/nav_section:circulation",
        json!({"label": "Circulation", "icon": "books", "pages": ["loans", "overview"]}),
    );
    assert_eq!(drops(&doc, &menu), []);
}
