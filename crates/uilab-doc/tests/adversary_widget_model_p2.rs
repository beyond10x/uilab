//! Adversarial cases, pass 2, for story:widget-model (correction 44880c1). Driven from the ess
//! shape on integrate/ess-ui-1: `header.metrics: {list: Node}` (header, order 6) and the docs
//! requirement "docs list widgets with params and use sites".

use serde_json::json;
use uilab_doc::{Document, Fixtures, NodePath, Patch, Severity, admit, check};

const DOC: &str = r#"
format: ui-spec/1
app: library
title: Lending library
model: library
placement_profile: fat
fixtures:
  views:
    loans.All: loans.yaml
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: overview
  sections:
    - {name: circulation, label: Circulation, pages: [overview]}
widgets:
  state_badge:
    summary: A loan state as a toned badge.
    params:
      state: {type: string, required: true}
      tone: {type: string, required: false}
    body:
      - {name: badge, primitive: badge, text: args.state}
pages:
  overview:
    kind: dashboard_page
    title: Overview
    sections:
      list:
        component: collection
        reads: {view: loans.All}
        columns: [{field: title}]
"#;

fn with(from: &str, to: &str) -> Document {
    assert!(DOC.contains(from), "fixture anchor `{from}` is missing");
    Document::from_yaml(&DOC.replacen(from, to, 1)).unwrap()
}

fn errors(doc: &Document) -> Vec<(&'static str, String)> {
    check(doc)
        .into_iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| (f.check, f.path))
        .collect()
}

const PAGE: &str = "    title: Overview\n";

/// ess `header.metrics: {list: Node}`: an instance there names no declared widget.
#[test]
fn an_unresolved_instance_in_a_page_header_metric_is_reported() {
    let doc = with(
        PAGE,
        "    title: Overview\n    header: {metrics: [{name: due, component: loan_tile}]}\n",
    );
    let found = errors(&doc);
    assert!(
        found.iter().any(|(c, _)| *c == "widget_resolves"),
        "an instance of an undeclared widget in header.metrics goes unreported: {found:?}"
    );
}

/// Removing the only widget a page header metric uses leaves a dangling instance.
#[test]
fn removing_a_widget_a_page_header_metric_still_uses_is_refused() {
    let doc = with(
        PAGE,
        "    title: Overview\n    header: {metrics: [{name: due, component: state_badge, args: {state: row.state}}]}\n",
    );
    assert!(errors(&doc).is_empty(), "{:?}", errors(&doc));
    let removed = admit(
        &doc,
        &Patch::Remove {
            target: "component:state_badge".parse::<NodePath>().unwrap(),
        },
    );
    assert_eq!(
        removed.map(|_| ()).map_err(|r| r.check),
        Err("widget_resolves".to_owned()),
        "a widget still used by a page header metric was removed"
    );
}

/// The Widgets section lists use sites. After 44880c1 an instance in a section's `children` is a
/// use the checks see (removing the widget is refused), so the docs must list it too.
#[test]
fn docs_list_a_use_site_in_an_untyped_node_position() {
    let doc = with(
        "        columns: [{field: title}]\n",
        "        columns: [{field: title}]\n        children: [{name: badge, component: state_badge, args: {state: row.state}}]\n",
    );
    assert!(errors(&doc).is_empty(), "{:?}", errors(&doc));
    let docs = uilab_doc::docs_markdown(&doc, &Fixtures::default(), &check(&doc));
    let start = docs.find("### state_badge").expect("a state_badge entry");
    let end = docs[start + 4..]
        .find("\n#")
        .map_or(docs.len(), |i| start + 4 + i);
    let entry = &docs[start..end];
    assert!(
        entry.contains("`page:overview/section:list`") && !entry.contains("Not used yet."),
        "docs miss the use in `children`:\n{entry}"
    );
}

/// `required: false` written explicitly is kept and still means not required.
#[test]
fn an_explicit_required_false_is_kept_and_means_optional() {
    let doc = with(
        "        columns: [{field: title}]\n",
        "        columns: [{field: title}]\n        item: {badge: {component: state_badge, args: {state: row.state}}}\n",
    );
    assert!(errors(&doc).is_empty(), "{:?}", errors(&doc));
    let param = &doc.widgets["state_badge"].params["tone"];
    assert!(!param.is_required());
    assert_eq!(
        serde_json::to_value(param).unwrap()["required"],
        json!(false)
    );
    assert!(doc.to_yaml().unwrap().contains("required: false"));
}
