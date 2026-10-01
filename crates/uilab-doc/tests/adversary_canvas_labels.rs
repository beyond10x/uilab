//! Adversary pass 1 for story:canvas-shows-labels: the page header and menu entry the browser is
//! shown (`outline::rendered`, a page node's `props.header` / `props.nav`) are "as ESS renders
//! them". These cases hold them to ESS's own loader (`ess_ui::load_str`) for a document's own page
//! kinds, one extending another, including the headline `metrics` a kind contributes; and hold a
//! choice's options to what ESS renders for a named enum type.

use serde_json::{Value, json};
use uilab_doc::outline::{outline, rendered};
use uilab_doc::{Document, OutlineNode, ess_ui};

/// A lending-library document whose own page kinds carry the header: `desk_page` extends
/// `counter_page`, which extends the built-in `list_page` (`title: from_page`, `total`,
/// `filters`). `counter_page` contributes a headline metric and an action; `desk_page` adds help.
/// `front` writes no header; `back` writes one metric of its own.
const DOC: &str = r#"format: ess-ui/1
app: library
title: Lending library
model: library
placement_profile: fat
types:
  LoanState: {enum: [out, returned, overdue]}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: front
  sections:
    - {name: desk, label: Desk, pages: [front, back]}
page_kinds:
  counter_page:
    extends: list_page
    header:
      metrics:
        - {name: out, component: metric, label: Copies out, reads: {view: loans.Summary}, from: on_loan}
      actions: [{name: lend, does: loans.Lend, label: Lend a copy}]
  desk_page:
    extends: counter_page
    header:
      help: {text: What is out and what is due back}
pages:
  front:
    kind: desk_page
    title: Front desk
    sections:
      - name: list
        component: collection
        reads: {view: loans.All}
        columns: [{field: title}]
  back:
    kind: desk_page
    title: Back office
    header:
      metrics:
        - {name: overdue, component: metric, label: Overdue copies, reads: {view: loans.Summary}, from: overdue}
    sections:
      - name: list
        component: collection
        reads: {view: loans.All}
        columns: [{field: title}]
      - name: state
        component: choice
        options: LoanState
"#;

fn page<'o>(root: &'o OutlineNode, name: &str) -> &'o OutlineNode {
    root.children
        .iter()
        .find(|c| c.path == format!("page:{name}"))
        .unwrap_or_else(|| panic!("no page:{name} in the outline"))
}

fn header(root: &OutlineNode, name: &str) -> Value {
    page(root, name)
        .props
        .as_ref()
        .and_then(|p| p.get("header"))
        .cloned()
        .unwrap_or(Value::Null)
}

/// The names of the headline metrics ESS's loader renders in `page`'s header.
fn ess_metric_names(page: &str) -> Vec<String> {
    let ess = ess_ui::load_str(DOC).expect("ESS loads the document");
    let header = ess.pages[page]
        .header
        .as_ref()
        .expect("ESS renders a header");
    header
        .metrics
        .iter()
        .map(|m| m.common.name.clone().expect("a metric in a list is named"))
        .collect()
}

/// The names of the headline metrics the browser is shown in `page`'s header.
fn shown_metric_names(root: &OutlineNode, page: &str) -> Vec<String> {
    header(root, page)
        .get("metrics")
        .and_then(Value::as_array)
        .map(|ms| {
            ms.iter()
                .filter_map(|m| m.get("name").and_then(Value::as_str).map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// The header's title, help and actions merge through two of the document's own page kinds as
/// ESS's loader merges them (the control: this half holds).
#[test]
fn a_header_through_two_document_kinds_is_the_one_ess_renders() {
    let doc = Document::from_yaml(DOC).expect("uilab reads the document");
    let ess = ess_ui::load_str(DOC).expect("ESS loads the document");
    let shown = rendered(&doc);
    for name in ["front", "back"] {
        let h = header(&shown, name);
        let e = ess.pages[name]
            .header
            .as_ref()
            .expect("ESS renders a header");
        assert_eq!(h["title"], json!(e.title), "{name}: title");
        assert_eq!(
            h["help"]["text"],
            json!(e.help.as_ref().and_then(|h| h.text.clone())),
            "{name}: help"
        );
        let actions: Vec<Option<String>> = e.actions.iter().map(|a| a.label.clone()).collect();
        let shown_actions: Vec<Option<String>> = h["actions"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|x| x["label"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        assert_eq!(shown_actions, actions, "{name}: actions");
    }
    assert_eq!(header(&shown, "front")["title"], json!("Front desk"));
}

/// A page that writes no header is shown the headline metric its page kind contributes, as
/// ESS renders it: `Copies out`, the metric's label, is text ESS draws on that page.
#[test]
fn a_page_without_a_header_is_shown_the_metrics_its_kind_contributes() {
    let doc = Document::from_yaml(DOC).expect("uilab reads the document");
    assert_eq!(ess_metric_names("front"), vec!["out".to_owned()]);
    assert_eq!(
        shown_metric_names(&rendered(&doc), "front"),
        ess_metric_names("front"),
        "the browser's header metrics for `front` are not ESS's: {}",
        header(&rendered(&doc), "front")
    );
}

/// A page that writes a headline metric of its own keeps the kind's too: ESS merges named lists
/// by name, inherited first.
#[test]
fn a_page_header_metric_is_merged_with_the_kinds_by_name() {
    let doc = Document::from_yaml(DOC).expect("uilab reads the document");
    assert_eq!(
        ess_metric_names("back"),
        vec!["out".to_owned(), "overdue".to_owned()]
    );
    assert_eq!(
        shown_metric_names(&rendered(&doc), "back"),
        ess_metric_names("back"),
        "the browser's header metrics for `back` are not ESS's: {}",
        header(&rendered(&doc), "back")
    );
}

/// The agent is given the header as the author wrote it: `back` its one metric, `front` none.
#[test]
fn the_agent_outline_keeps_the_header_as_written() {
    let doc = Document::from_yaml(DOC).expect("uilab reads the document");
    let written = outline(&doc);
    assert_eq!(
        shown_metric_names(&written, "back"),
        vec!["overdue".to_owned()]
    );
    assert_eq!(header(&written, "front"), Value::Null);
}

/// A choice over a named enum type (`options: LoanState`, the form of the schema's own example
/// `{component: choice, multiple: true, options: DealStage}`): ESS renders one option per value,
/// each labelled with it. The browser is shown the options as a list it can label; as the
/// written string it can only draw a placeholder.
#[test]
fn a_choice_over_an_enum_type_is_shown_the_options_ess_renders() {
    let doc = Document::from_yaml(DOC).expect("uilab reads the document");
    let shown = rendered(&doc);
    let choice = page(&shown, "back")
        .children
        .iter()
        .find(|c| c.name == "state")
        .expect("the choice section is in the outline");
    let options = choice
        .props
        .as_ref()
        .and_then(|p| p.get("options"))
        .cloned();
    assert_eq!(
        options,
        Some(json!([
            {"value": "out", "label": "out"},
            {"value": "returned", "label": "returned"},
            {"value": "overdue", "label": "overdue"}
        ])),
        "the browser is not shown ESS's options for `options: LoanState`"
    );
}
