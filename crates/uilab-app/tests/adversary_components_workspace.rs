//! Adversarial cases for story:components-workspace: the Components tab derives each widget's use
//! sites from the outline the browser receives (`widget/src/lib/components.ts` `useSites`), while
//! `/api/docs.md` lists the use sites the widget checks see (`widget_uses`). These cases hold the
//! outline to the docs' account.

use std::path::Path;

use serde_json::Value;
use uilab_doc::{Document, Fixtures, docs_markdown, outline};

const WIDGETS: &str = "widgets:
  loan_card:
    summary: A loan as a card.
    params:
      loan: {type: Loan, required: true}
    body:
      - {name: title, primitive: text, text: args.loan.title, style: heading}
      - {name: due, primitive: badge, text: args.loan.due}
  badge:
    summary: A toned tag.
    params:
      label: {type: string, required: true}
    body:
      - {name: tag, primitive: badge, text: args.label}
pages:
";

const OVERVIEW: &str =
    "  overview:\n    kind: dashboard_page\n    title: Overview\n    sections:\n";

/// The library example with the widgets `loan_card` and `badge`: `loan_card` is used once, as a
/// metric of the overview page's header (ess `header.metrics: {list: Node}`); `badge` is used
/// once, as the overview section `latest`.
fn library() -> Document {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    let text = std::fs::read_to_string(file).unwrap();
    assert!(
        text.contains("\npages:\n") && text.contains(OVERVIEW),
        "fixture anchors are missing"
    );
    let text = text.replacen("\npages:\n", &format!("\n{WIDGETS}"), 1).replacen(
        OVERVIEW,
        "  overview:\n    kind: dashboard_page\n    title: Overview\n    header: {metrics: [{name: due, component: loan_card, args: {loan: rows.first}}]}\n    sections:\n      latest: {component: badge, args: {label: rows.first}}\n",
        1,
    );
    Document::from_yaml(&text).unwrap()
}

/// The `Used at:` line the docs print for `widget`.
fn docs_uses(doc: &Document, widget: &str) -> String {
    let docs = docs_markdown(doc, &Fixtures::default(), &[]);
    let section = docs
        .split(&format!("### {widget}\n"))
        .nth(1)
        .unwrap_or_else(|| panic!("the docs have no section for `{widget}`"));
    section
        .lines()
        .find(|l| l.starts_with("Used at: ") || l.starts_with("Not used yet."))
        .unwrap_or_else(|| panic!("no use line for `{widget}`"))
        .to_owned()
}

fn child<'v>(node: &'v Value, path: &str) -> &'v Value {
    node["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["path"] == path)
        .unwrap_or_else(|| panic!("no child `{path}` in {}", node["path"]))
}

/// Every node of the outline below the root and outside a widget declaration whose kind is
/// `widget`: the typed half of the browser's use-site walk, and the brief's fallback rule.
fn typed_uses(node: &Value, widget: &str, out: &mut Vec<String>) {
    let layer = node["layer"].as_str().unwrap_or("");
    if layer != "root" && layer != "component" && node["kind"] == widget {
        out.push(node["path"].as_str().unwrap().to_owned());
    }
    for c in node["children"].as_array().unwrap() {
        typed_uses(c, widget, out);
    }
}

/// A widget used only in a page header is listed by the docs, and the tab can only find it if the
/// outline carries the header: the page node's props hold nothing but its shell today.
#[test]
fn the_outline_carries_a_widget_instance_held_in_a_page_header() {
    let doc = library();
    assert_eq!(
        docs_uses(&doc, "loan_card"),
        "Used at: `page:overview` (`header/metrics/due`)"
    );
    let root = serde_json::to_value(outline(&doc)).unwrap();
    let page = child(&root, "page:overview");
    let props = serde_json::to_string(&page["props"]).unwrap();
    assert!(
        props.contains("\"component\":\"loan_card\""),
        "the outline the browser gets has no trace of the header instance, so the Components tab \
         shows `loan_card` as not used; page:overview props = {props}"
    );
}

/// A widget named like a primitive: the docs list one use; the outline gives the primitive nodes
/// the same kind, so any walk of the outline by kind (the tab's, and the brief's fallback) counts
/// every primitive `badge` of the document as a use of the widget `badge`.
#[test]
fn the_outline_tells_a_primitive_from_an_instance_of_a_widget_named_like_it() {
    let doc = library();
    assert_eq!(
        docs_uses(&doc, "badge"),
        "Used at: `page:overview/section:latest`"
    );
    let root = serde_json::to_value(outline(&doc)).unwrap();
    let mut found = Vec::new();
    typed_uses(&root, "badge", &mut found);
    assert_eq!(found, ["page:overview/section:latest"]);
}
