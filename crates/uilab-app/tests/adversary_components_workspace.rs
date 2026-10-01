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
      loan: {type: Loan, required: true, note: loan param}
    body:
      - {name: title, primitive: text, text: args.loan.title, style: heading}
      - {name: due, primitive: badge, text: args.loan.due}
  badge:
    summary: A toned tag.
    params:
      label: {type: string, required: true, note: label param}
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
        "  overview:\n    kind: dashboard_page\n    title: Overview\n    header: {metrics: [{name: due, component: loan_card, args: {loan: rows.first}}]}\n    sections:\n      - {name: latest, component: badge, args: {label: rows.first}}\n",
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

/// The paths of the use sites the outline carries on the component node of `widget`: what the
/// Components tab lists.
fn outline_uses(doc: &Document, widget: &str) -> Vec<String> {
    let root = serde_json::to_value(outline(doc)).unwrap();
    let node = child(&root, &format!("component:{widget}"));
    node["props"]["uses"]
        .as_array()
        .unwrap_or_else(|| panic!("component:{widget} carries no uses: {}", node["props"]))
        .iter()
        .map(|u| match u["trail"].as_str() {
            Some(trail) => format!("{} ({trail})", u["path"].as_str().unwrap()),
            None => u["path"].as_str().unwrap().to_owned(),
        })
        .collect()
}

/// A widget used only in a page header is listed by the docs, and the outline the browser gets
/// carries that use, so the tab does not show `loan_card` as not used.
#[test]
fn the_outline_carries_a_widget_instance_held_in_a_page_header() {
    let doc = library();
    assert_eq!(
        docs_uses(&doc, "loan_card"),
        "Used at: `page:overview` (`header/metrics/due`)"
    );
    assert_eq!(
        outline_uses(&doc, "loan_card"),
        ["page:overview (header/metrics/due)"]
    );
}

/// A widget named like a primitive: the docs list one use, and the outline carries that one use
/// only, although primitive nodes of the document share the kind `badge`.
#[test]
fn the_outline_tells_a_primitive_from_an_instance_of_a_widget_named_like_it() {
    let doc = library();
    assert_eq!(
        docs_uses(&doc, "badge"),
        "Used at: `page:overview/section:latest`"
    );
    assert_eq!(
        outline_uses(&doc, "badge"),
        ["page:overview/section:latest"]
    );
}

/// The use sites the docs print for `widget`, one entry per instance, in the outline's format.
fn docs_sites(doc: &Document, widget: &str) -> Vec<String> {
    let line = docs_uses(doc, widget);
    let Some(list) = line.strip_prefix("Used at: ") else {
        return Vec::new();
    };
    list.split(", ").map(|site| site.replace('`', "")).collect()
}

/// The library example with `loan_card` and `badge` placed in every kind of spot `widget_uses`
/// walks: a board widget, a collection's item, an item action's `choice`, a widget body, two
/// toolbar entries of one section, and a page header whose `metrics` list holds two instances
/// (named apart: ess-ui/1 refuses two metrics with one `name`).
fn everywhere() -> Document {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    let text = std::fs::read_to_string(file).unwrap();
    assert!(
        text.contains("\npages:\n") && text.contains(OVERVIEW),
        "fixture anchors are missing"
    );
    let widgets = "widgets:
  loan_card:
    summary: A loan as a card.
    params:
      loan: {type: Loan, required: true, note: loan param}
    body:
      - {name: title, primitive: text, text: args.loan.title, style: heading}
      - {name: state, component: badge, args: {label: args.loan.state}}
  badge:
    summary: A toned tag.
    params:
      label: {type: string, required: true, note: label param}
    body:
      - {name: tag, primitive: badge, text: args.label}
pages:
";
    let overview = "  overview:
    kind: dashboard_page
    title: Overview
    header:
      metrics:
        - {name: due, component: loan_card, args: {loan: rows.first}}
        - {name: due_last, component: loan_card, args: {loan: rows.last}}
    sections:
      - name: tiles
        component: board
        reads: {view: loans.All}
        widgets:
          first: {component: loan_card, args: {loan: rows.first}}
      - name: flow
        component: graph_editor
        reads: {view: loans.All}
        toolbar:
          - {name: a, component: badge, args: {label: one}}
          - {name: b, component: badge, args: {label: two}}
      - name: cards
        component: collection
        reads: {view: loans.All}
        item:
          - {name: card, component: loan_card, args: {loan: row}}
          - {name: go, primitive: button, label: Go, action: {name: go, does: loans.Pick, choice: {component: badge, args: {label: row.state}}}}
";
    let text = text
        .replacen("\npages:\n", &format!("\n{widgets}"), 1)
        .replacen(OVERVIEW, overview, 1);
    Document::from_yaml(&text).unwrap()
}

/// Every instance `widget_uses` finds is one use site in the docs, and the Components tab lists
/// the outline's `uses`: the two must agree in number as well as in paths, or the tab's count
/// ("used N times") is not the docs' count.
#[test]
fn the_outline_lists_every_instance_the_docs_list_one_entry_each() {
    let doc = everywhere();
    let mut differ = Vec::new();
    for widget in ["loan_card", "badge"] {
        let docs = docs_sites(&doc, widget);
        assert!(!docs.is_empty(), "`{widget}`: the fixture places it");
        let listed = outline_uses(&doc, widget);
        if listed != docs {
            differ.push(format!("`{widget}`: outline {listed:?}, docs {docs:?}"));
        }
    }
    assert!(
        differ.is_empty(),
        "the outline's uses and the docs' use sites differ:\n{}",
        differ.join("\n")
    );
}
