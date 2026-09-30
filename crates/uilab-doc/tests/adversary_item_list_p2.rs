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
fn an_opens_in_a_widget_body_is_checked_at_each_use_site() {
    let clean = doc();
    assert_eq!(
        opens_at(&clean),
        Vec::<String>::new(),
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
    let refused = match admit(&clean, &insert) {
        Ok(_) => panic!(
            "an instance whose body opens `extend` is admitted onto `members`, which declares \
             no `extend` overlay"
        ),
        Err(refused) => refused,
    };
    assert_eq!(refused.check, "opens_resolves", "{refused}");
    assert!(
        refused
            .message
            .starts_with("page:members/section:list/item:card/body/extend: "),
        "{refused}"
    );
}

/// The paths of every `opens_resolves` finding, in order.
fn opens_at(doc: &Document) -> Vec<String> {
    check(doc)
        .into_iter()
        .filter(|f| f.check == "opens_resolves")
        .map(|f| f.path)
        .collect()
}

fn with_widgets(text: &str, widgets: &str) -> String {
    text.replace("widgets:\n", &format!("widgets:\n{widgets}"))
}

fn with_member_item(text: &str, item: &str) -> String {
    text.replace(
        "          - {name: tag, primitive: badge, text: row.state}\n",
        &format!(
            "          - {{name: tag, primitive: badge, text: row.state}}\n          - {item}\n"
        ),
    )
}

/// One widget used on two pages: `overview` declares the overlay its body opens and `members`
/// does not. Exactly one finding, an error at the use on `members`, naming the widget and node.
#[test]
fn a_widget_used_on_two_pages_is_reported_only_where_its_opens_does_not_resolve() {
    let text = with_member_item(DOC, "{name: card, component: loan_card, args: {loan: row}}");
    let doc = Document::from_yaml(&text).unwrap();
    let found: Vec<_> = check(&doc)
        .into_iter()
        .filter(|f| f.check == "opens_resolves")
        .collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        found[0].path,
        "page:members/section:list/item:card/body/extend"
    );
    assert_eq!(found[0].severity, uilab_doc::Severity::Error);
    assert!(
        found[0].message.contains("`loan_card`") && found[0].message.contains("`extend`"),
        "{}",
        found[0].message
    );
}

/// A page's header, a board widget and a shell overlay are use sites too: each is checked against
/// the overlays of the page and its shell, or of the shell alone.
#[test]
fn a_widget_in_a_header_a_board_or_a_shell_overlay_is_checked_where_it_sits() {
    let text = DOC
        .replace(
            "      main: {kind: page_outlet}\n",
            "      main: {kind: page_outlet}\n    overlays:\n      quick: {kind: drawer, component: loan_card, args: {loan: rows.first}}\n",
        )
        .replace(
            "    kind: list_page\n    title: Members\n",
            "    kind: list_page\n    title: Members\n    header: {metrics: [{name: due, component: loan_card, args: {loan: rows.first}}]}\n",
        )
        .replace(
            "    sections:\n      list:\n",
            "    sections:\n      board:\n        component: board\n        widgets:\n          top: {component: loan_card, args: {loan: rows.first}}\n      list:\n",
        );
    let doc = Document::from_yaml(&text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        opens_at(&doc),
        [
            "shell:app/overlay:quick/body/extend",
            "page:members/section:board/widget:top/body/extend",
            "page:members/header/metrics/due/body/extend",
        ]
    );
}

/// A widget inside a widget's body, as a typed item and in a button's `choice`, is expanded at the
/// outer widget's use site, and only there.
#[test]
fn a_nested_widget_body_is_checked_at_the_outer_use_site() {
    let shelf = "  shelf:\n    summary: Loans on a shelf.\n    body:\n      - name: rows\n        component: collection\n        reads: {view: loans.All}\n        item:\n          - {name: card, component: loan_card, args: {loan: row}}\n      - {name: pick, primitive: button, label: Pick, action: {name: pick, does: loans.Pick, choice: {component: loan_card, args: {loan: rows.first}}}}\n";
    let text = with_widgets(DOC, shelf).replace(
        "      latest: {component: loan_card, args: {loan: rows.first}}\n",
        "      latest: {component: loan_card, args: {loan: rows.first}}\n      shelf: {component: shelf}\n",
    );
    let on_overview = Document::from_yaml(&text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(opens_at(&on_overview), Vec::<String>::new());

    let on_members =
        Document::from_yaml(&with_member_item(&text, "{name: shelf, component: shelf}")).unwrap();
    assert_eq!(
        opens_at(&on_members),
        [
            "page:members/section:list/item:shelf/body/rows/item:card/body/extend",
            "page:members/section:list/item:shelf/body/pick/action/choice/body/extend",
        ]
    );
}

/// An `opens` in a body that no page or shell uses reports nothing: a widget used nowhere, one
/// used only in the body of another unused widget, and one used only in `page_kinds`, which is
/// data no page is checked through.
#[test]
fn an_opens_in_a_widget_body_with_no_use_reports_nothing() {
    let widgets = "  orphan:\n    summary: Used nowhere.\n    body:\n      - {name: go, primitive: button, label: Go, action: {name: go, opens: nowhere}}\n  holder:\n    summary: Holds orphan, used nowhere.\n    body:\n      - {name: inner, component: orphan}\n";
    let text = with_widgets(DOC, widgets).replace(
        "widgets:\n",
        "page_kinds:\n  gallery: {sections: {main: {component: orphan}}}\nwidgets:\n",
    );
    let doc = Document::from_yaml(&text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(opens_at(&doc), Vec::<String>::new());
}

/// Two widgets that contain each other: the expansion stops where a widget recurs, the recursion
/// is reported, and the `opens` of the outer body is reported once.
#[test]
fn a_recursive_widget_body_is_expanded_once() {
    let widgets = "  loop_a:\n    summary: Holds loop_b.\n    body:\n      - {name: go, primitive: button, label: Go, action: {name: go, opens: nowhere}}\n      - {name: b, component: loop_b}\n  loop_b:\n    summary: Holds loop_a.\n    body:\n      - {name: a, component: loop_a}\n";
    let text = with_member_item(
        &with_widgets(DOC, widgets),
        "{name: loop, component: loop_a}",
    );
    let doc = Document::from_yaml(&text).unwrap();
    assert!(check(&doc).iter().any(|f| f.check == "widget_recursion"));
    assert_eq!(
        opens_at(&doc),
        ["page:members/section:list/item:loop/body/go"]
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
