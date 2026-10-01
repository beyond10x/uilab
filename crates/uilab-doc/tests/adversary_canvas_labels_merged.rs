//! Adversary pass 1 for story:canvas-shows-labels: texts ESS renders on a node the author wrote
//! that come from elsewhere — a column `label` a document's page kind gives the section the page
//! names again, and an overlay's `title` reached through `same_as` (the acceptance names "`title`
//! on an overlay"). The browser is shown `outline::rendered`; these cases hold it to ESS's loader.

use uilab_doc::outline::rendered;
use uilab_doc::{Document, OutlineNode, ess_ui};

/// `shelf_page` gives its `list` section labelled columns; `front` names `list` again with only
/// its read. `back`'s overlay `extend` is `same_as: front.edit`, whose title is "Extend loan".
/// `ess ui check` (0.48.0): 0 errors.
const DOC: &str = r#"format: ess-ui/1
app: library
title: Lending library
model: library
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: front
  sections:
    - {name: desk, label: Desk, pages: [front, back]}
page_kinds:
  shelf_page:
    extends: list_page
    sections:
      - name: list
        component: collection
        columns: [{field: title, label: Book}, {field: due, label: Due back}]
pages:
  front:
    kind: shelf_page
    title: Front desk
    sections:
      - name: list
        component: collection
        reads: {view: loans.All}
    overlays:
      edit: {kind: drawer, component: form, title: Extend loan, does: loans.ExtendLoan, fields: [due]}
  back:
    kind: list_page
    title: Back office
    sections:
      - name: list
        component: collection
        reads: {view: loans.All}
        columns: [{field: title}]
    overlays:
      extend: {same_as: front.edit}
"#;

fn find<'o>(node: &'o OutlineNode, path: &str) -> &'o OutlineNode {
    if node.path == path {
        return node;
    }
    node.children
        .iter()
        .find_map(|c| {
            (c.path == path || path.starts_with(&format!("{}/", c.path))).then(|| find(c, path))
        })
        .unwrap_or_else(|| panic!("no {path} in the outline"))
}

/// The column labels ESS renders for `front`'s `list`.
fn ess_column_labels() -> Vec<Option<String>> {
    let ess = ess_ui::load_str(DOC).expect("ESS loads the document");
    let list = ess.pages["front"]
        .sections
        .iter()
        .find(|s| s.name == "list")
        .expect("ESS renders front's list");
    match &list.body {
        ess_ui::Body::Composite(ess_ui::Composite::Collection(c)) => match &c.columns {
            Some(ess_ui::Columns::Fixed(fields)) => {
                fields.iter().map(|f| f.label.clone()).collect()
            }
            other => panic!("ESS renders columns {other:?}"),
        },
        other => panic!("ESS renders list as {other:?}"),
    }
}

/// ESS merges the kind's `list` into the page's by name: the page's read, the kind's columns.
/// The browser is shown the columns, so "Book" and "Due back" can reach the table head.
#[test]
fn a_section_the_page_names_again_keeps_the_column_labels_its_kind_gives() {
    assert_eq!(
        ess_column_labels(),
        vec![Some("Book".to_owned()), Some("Due back".to_owned())]
    );
    let doc = Document::from_yaml(DOC).expect("uilab reads the document");
    let shown = rendered(&doc);
    let list = find(&shown, "page:front/section:list");
    let labels: Vec<Option<String>> = list
        .props
        .as_ref()
        .and_then(|p| p.get("columns"))
        .and_then(|c| c.as_array())
        .map(|cs| {
            cs.iter()
                .map(|c| c.get("label").and_then(|l| l.as_str()).map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        labels,
        ess_column_labels(),
        "front's list is shown with props {:?}",
        list.props
    );
}

/// An overlay written as `same_as` another is rendered with that one's title; the canvas draws
/// an overlay by `title` (its button, its modal bar), so the browser's node must carry it.
#[test]
fn an_overlay_same_as_another_carries_the_title_ess_renders() {
    let ess = ess_ui::load_str(DOC).expect("ESS loads the document");
    assert_eq!(
        ess.pages["back"].overlays["extend"].title.as_deref(),
        Some("Extend loan")
    );
    let doc = Document::from_yaml(DOC).expect("uilab reads the document");
    let shown = rendered(&doc);
    let extend = find(&shown, "page:back/overlay:extend");
    assert_eq!(
        extend.title.as_deref(),
        Some("Extend loan"),
        "back's overlay `extend` is shown as {extend:?}"
    );
}
