//! Adversarial cases for story:item-list, pass 2: the `nodes` walk of 9b1e617, driven from the
//! ess shape on integrate/ess-ui-1 (schemas/ui/ess-ui.schema.yaml: Widget, WidgetInstance,
//! button, Action).

use std::time::{Duration, Instant};

use serde_json::json;
use uilab_doc::{Child, Document, Fixtures, Layer, NodePath, Patch, admit, check, docs_markdown};

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
    - {name: circulation, label: Circulation, pages: [overview, members]}
widgets:
  loan_card:
    summary: A loan as a card with an extend button.
    params:
      loan: {type: Loan, required: true}
    body:
      - {name: title, primitive: text, text: args.loan.title}
      - {name: extend, primitive: button, label: Extend, action: {name: extend, opens: extend}}
  state_badge:
    summary: A loan state as a toned badge.
    params:
      state: {type: string, required: true}
    body:
      - {name: badge, primitive: badge, text: args.state}
pages:
  overview:
    kind: dashboard_page
    title: Overview
    sections:
      latest: {component: loan_card, args: {loan: rows.first}}
    overlays:
      extend: {kind: dialog, component: record, reads: {view: loans.All}}
  members:
    kind: list_page
    title: Members
    sections:
      list:
        component: collection
        reads: {view: loans.All}
        item:
          - {name: tag, primitive: badge, text: row.state}
"#;

fn doc() -> Document {
    Document::from_yaml(DOC).unwrap_or_else(|e| panic!("{e}"))
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

/// ess Widget: "A widget is expanded at its use site and then checked like a built-in";
/// WidgetInstance.expansion: "the expanded nodes are checked like built-ins; findings are reported
/// at <instance path>/body/<node name>". `loan_card`'s extend button opens `extend`, which page
/// `overview` declares and page `members` does not. Used on `overview` it is fine; used in the
/// `members` item list, its button opens an overlay that page and its shell do not declare.
#[test]
fn an_opens_in_a_widget_body_is_not_yet_checked_at_its_use_site() {
    let clean = doc();
    let on_overview: Vec<_> = check(&clean)
        .into_iter()
        .filter(|f| f.check == "opens_resolves")
        .collect();
    assert_eq!(
        on_overview,
        [],
        "used where `extend` is declared, the body's button resolves"
    );

    let insert = Patch::Insert {
        target: path("page:members/section:list"),
        child: Child {
            layer: Layer::Item,
            name: "card".into(),
            node: json!({"component": "loan_card", "args": {"loan": "row"}}),
            nav_section: None,
        },
    };
    // Pinned to today's behaviour: the gap is pre-existing (8b337a1) and open as
    // story:widget-opens-at-use. When that story lands this case must flip to asserting a
    // refusal or an opens_resolves finding at the instance.
    let (used_on_members, _) = admit(&clean, &insert).unwrap_or_else(|refused| {
        panic!(
            "the insert is now refused ({refused}): story:widget-opens-at-use has landed, flip \
             this case to assert the refusal"
        )
    });
    let found: Vec<_> = check(&used_on_members)
        .into_iter()
        .filter(|f| f.check == "opens_resolves")
        .map(|f| (f.path, f.message))
        .collect();
    assert!(
        found.is_empty(),
        "an opens_resolves finding now appears for the widget body on `members` ({found:?}): \
         story:widget-opens-at-use has landed, flip this case to assert it"
    );
}

/// The two walks (`nodes` for widget uses, `check_composite` for opens) meet at every primitive
/// item and every widget-body primitive: each fault is reported once, and each use site is listed
/// once in the docs.
#[test]
fn a_fault_in_a_primitive_node_is_reported_once_and_a_use_site_listed_once() {
    let text = DOC
        .replace(
            "          - {name: tag, primitive: badge, text: row.state}\n",
            "          - {name: tag, primitive: badge, text: row.state}\n          - {name: go, primitive: button, label: Go, action: {name: go, opens: nowhere, choice: {component: missing}}}\n          - {name: pick, primitive: button, label: Pick, action: {name: pick, does: loans.Pick, choice: {component: state_badge, args: {state: row.state}}}}\n",
        )
        .replace(
            "      - {name: badge, primitive: badge, text: args.state}\n",
            "      - {name: badge, primitive: badge, text: args.state}\n      - {name: more, primitive: button, label: More, action: {name: more, does: loans.More, choice: {component: missing}}}\n",
        );
    let doc = Document::from_yaml(&text).unwrap();
    let findings: Vec<_> = check(&doc)
        .into_iter()
        .map(|f| (f.check, f.path, f.message))
        .collect();
    for (i, f) in findings.iter().enumerate() {
        assert!(
            !findings[..i].contains(f),
            "reported twice: {f:?}\n{findings:#?}"
        );
    }
    let at = |check: &str, p: &str| {
        findings
            .iter()
            .filter(|(c, fp, _)| *c == check && fp == p)
            .count()
    };
    assert_eq!(at("opens_resolves", "page:members/section:list/item:go"), 1);
    assert_eq!(
        at("widget_resolves", "page:members/section:list/item:go"),
        1
    );
    assert_eq!(at("widget_resolves", "component:state_badge/node:more"), 1);

    let docs = docs_markdown(&doc, &Fixtures::default(), &check(&doc));
    let site = "`page:members/section:list/item:pick` (`action/choice`)";
    assert_eq!(docs.matches(site).count(), 1, "{docs}");
}

/// A large document: a collection with 3000 item nodes and a chain of 200 widgets, each using the
/// next in a body collection item and in a body button's choice. Checking it and admitting one
/// insert stay fast.
#[test]
fn checking_a_large_document_stays_fast() {
    let mut text = String::from(DOC);
    let mut widgets = String::new();
    for i in 0..200 {
        let body = if i < 199 {
            let next = i + 1;
            format!(
                "      - name: rows\n        component: collection\n        reads: {{view: loans.All}}\n        item:\n          - {{name: next, component: chain{next}}}\n      - {{name: go, primitive: button, label: Go, action: {{name: go, does: loans.Pick, choice: {{component: chain{next}}}}}}}\n"
            )
        } else {
            "      - {name: end, primitive: text, text: end}\n".to_owned()
        };
        widgets.push_str(&format!(
            "  chain{i}:\n    summary: Link {i} of a chain.\n    body:\n{body}"
        ));
    }
    text = text.replace("widgets:\n", &format!("widgets:\n{widgets}"));
    let mut items = String::new();
    for i in 0..3000 {
        if i % 2 == 0 {
            items.push_str(&format!(
                "          - {{name: n{i}, primitive: button, label: Go, action: {{name: go, opens: nowhere}}}}\n"
            ));
        } else {
            items.push_str(&format!("          - {{name: n{i}, component: chain0}}\n"));
        }
    }
    text = text.replace(
        "          - {name: tag, primitive: badge, text: row.state}\n",
        &format!("          - {{name: tag, primitive: badge, text: row.state}}\n{items}"),
    );
    let doc = Document::from_yaml(&text).unwrap_or_else(|e| panic!("{e}"));

    let started = Instant::now();
    let findings = check(&doc);
    let checked = started.elapsed();
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.check == "opens_resolves")
            .count(),
        1500
    );

    let started = Instant::now();
    let insert = Patch::Insert {
        target: path("page:members/section:list"),
        child: Child {
            layer: Layer::Item,
            name: "extra".into(),
            node: json!({"primitive": "divider"}),
            nav_section: None,
        },
    };
    admit(&doc, &insert).unwrap_or_else(|e| panic!("{e}"));
    let admitted = started.elapsed();
    assert!(
        checked < Duration::from_secs(5) && admitted < Duration::from_secs(10),
        "check took {checked:?}, admit took {admitted:?}"
    );
}
