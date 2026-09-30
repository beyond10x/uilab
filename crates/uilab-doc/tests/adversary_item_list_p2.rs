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
            .starts_with("page:members/section:list/item:card: ")
            && refused.message.contains("body `body/extend`"),
        "{refused}"
    );
}

/// The paths of every `opens_resolves` finding, in order.
fn opens_at(doc: &Document) -> Vec<String> {
    opens_found(doc).into_iter().map(|(p, _)| p).collect()
}

/// Every `opens_resolves` finding as (path, message), in order.
fn opens_found(doc: &Document) -> Vec<(String, String)> {
    check(doc)
        .into_iter()
        .filter(|f| f.check == "opens_resolves")
        .map(|f| (f.path, f.message))
        .collect()
}

/// Asserts the `opens_resolves` findings are at `expected` instance paths, in order, and each
/// message names its body trail.
fn assert_opens(doc: &Document, expected: &[(&str, &str)]) {
    let found = opens_found(doc);
    assert_eq!(
        found.iter().map(|(p, _)| p.as_str()).collect::<Vec<_>>(),
        expected.iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        "{found:#?}"
    );
    for ((_, message), (_, trail)) in found.iter().zip(expected) {
        assert!(message.contains(trail), "`{message}` lacks `{trail}`");
    }
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
    assert_eq!(found[0].path, "page:members/section:list/item:card");
    assert_eq!(found[0].severity, uilab_doc::Severity::Error);
    assert!(
        found[0].message.contains("`loan_card`")
            && found[0].message.contains("body `body/extend`")
            && found[0].message.contains("opens `extend`"),
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
    assert_opens(
        &doc,
        &[
            ("shell:app/overlay:quick", "body `body/extend`"),
            (
                "page:members/section:board/widget:top",
                "body `body/extend`",
            ),
            (
                "page:members",
                "at `header/metrics/due`, body `body/extend`",
            ),
        ],
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
    assert_opens(
        &on_members,
        &[
            (
                "page:members/section:list/item:shelf",
                "body `body/rows/item:card/body/extend`",
            ),
            (
                "page:members/section:list/item:shelf",
                "body `body/pick/action/choice/body/extend`",
            ),
        ],
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
    assert_opens(
        &doc,
        &[("page:members/section:list/item:loop", "body `body/go`")],
    );
}

const OPENER: &str = "  opener:\n    summary: A button that opens the overlay it is given.\n    params:\n      target: {type: {ref: overlay}}\n    body:\n      - {name: go, primitive: button, label: Open, action: {name: go, opens: args.target}}\n  opens_extend:\n    summary: An opener that opens extend by default.\n    params:\n      target: {type: {ref: overlay}, default: extend}\n    body:\n      - {name: go, primitive: button, label: Open, action: {name: go, opens: args.target}}\n";

/// ess WidgetInstance.expansion substitutes `args.<param>` before the body is checked. An
/// `opens: args.target` is held to the overlay the instance binds, or the param's default when it
/// binds none. The same widget bound differently at two uses is judged per use, in either order.
/// An unbound param with no default, or one bound to a runtime reference or a non-string, cannot
/// be judged statically and reports nothing.
#[test]
fn an_args_opens_is_held_to_the_bound_literal_or_the_default_and_skipped_otherwise() {
    let items = [
        "{name: a, component: opener, args: {target: extend}}",
        "{name: b, component: opener, args: {target: help}}",
        "{name: c, component: opener}",
        "{name: d, component: opener, args: {target: row.overlay}}",
        "{name: e, component: opener, args: {target: rows.first}}",
        "{name: f, component: opener, args: {target: args.other}}",
        "{name: g, component: opener, args: {target: 3}}",
        "{name: h, component: opens_extend}",
        "{name: i, component: opens_extend, args: {target: help}}",
    ]
    .join("\n          - ");
    let with_help = DOC.replace(
        "      main: {kind: page_outlet}\n",
        "      main: {kind: page_outlet}\n    overlays:\n      help: {kind: drawer, component: record, reads: {view: loans.All}}\n",
    );
    let used = |overview: &str| {
        let text = with_member_item(&with_widgets(&with_help, OPENER), &items).replace(
            "      latest: {component: loan_card, args: {loan: rows.first}}\n",
            &format!(
                "      latest: {{component: loan_card, args: {{loan: rows.first}}}}\n{overview}"
            ),
        );
        Document::from_yaml(&text).unwrap_or_else(|e| panic!("{e}"))
    };
    let expected = [
        (
            "page:members/section:list/item:a",
            "widget `opener`, body `body/go`, opens `extend`,",
        ),
        (
            "page:members/section:list/item:h",
            "widget `opens_extend`, body `body/go`, opens `extend`,",
        ),
    ];
    assert_opens(&used(""), &expected);
    assert_opens(
        &used(
            "      o1: {component: opener, args: {target: extend}}\n      o2: {component: opens_extend}\n",
        ),
        &expected,
    );
}

/// A widget in another widget's body is expanded with the args that body binds: a literal is held
/// to the overlays of the outer use site; a pass-through of the outer widget's own `args.<param>`
/// reports nothing.
#[test]
fn a_nested_instance_is_expanded_with_the_args_its_holder_binds() {
    let holders = "  holds_literal:\n    summary: Opens nowhere through an opener.\n    body:\n      - {name: inner, component: opener, args: {target: nowhere}}\n  passes_through:\n    summary: Hands its target to an opener.\n    params:\n      target: {type: {ref: overlay}}\n    body:\n      - {name: inner, component: opener, args: {target: args.target}}\n";
    let widgets = format!("{OPENER}{holders}");
    let text = with_member_item(
        &with_member_item(
            &with_widgets(DOC, &widgets),
            "{name: p, component: passes_through, args: {target: nowhere}}",
        ),
        "{name: l, component: holds_literal}",
    );
    let doc = Document::from_yaml(&text).unwrap_or_else(|e| panic!("{e}"));
    assert_opens(
        &doc,
        &[(
            "page:members/section:list/item:l",
            "body `body/inner/body/go`, opens `nowhere`,",
        )],
    );
}

/// path.rs: "Adding a sibling never changes an existing path." An unnamed header metric holding an
/// undeclared widget, and an unnamed header action whose `choice` holds a widget whose body opens
/// an overlay the page lacks, are errors already there. A replace of the page that puts a clean
/// entry in front of each brings nothing new and is admitted.
#[test]
fn a_clean_unnamed_entry_in_front_of_a_broken_one_is_not_refused_for_the_old_error() {
    let text = DOC.replace(
        "    kind: list_page\n    title: Members\n",
        "    kind: list_page\n    title: Members\n    header: {metrics: [{component: missing}], actions: [{label: Pick, choice: {component: loan_card, args: {loan: rows.first}}}]}\n",
    );
    let doc = Document::from_yaml(&text).unwrap_or_else(|e| panic!("{e}"));
    let errors: Vec<_> = check(&doc)
        .into_iter()
        .filter(|f| f.check == "widget_resolves" || f.check == "opens_resolves")
        .collect();
    assert_eq!(errors.len(), 2, "{errors:#?}");
    let mut page = serde_json::to_value(&doc.pages["members"]).unwrap();
    page["header"]["metrics"] = json!([{"component": "state_badge", "args": {"state": "rows.first"}}, {"component": "missing"}]);
    page["header"]["actions"] = json!([
        {"label": "Other"},
        {"label": "Pick", "choice": {"component": "loan_card", "args": {"loan": "rows.first"}}},
    ]);
    let replace = Patch::Replace {
        target: path("page:members"),
        node: page,
    };
    admit(&doc, &replace).unwrap_or_else(|refused| {
        panic!("a clean entry in front of a pre-existing error is refused: {refused}")
    });
}

/// A chain of 15 widgets, each using the next twice, the last opening `nowhere`: the use expands
/// to 2^14 unresolved buttons. Admitting an unrelated insert compares the findings before and
/// after, and stays within a small multiple of one check, not the square of the finding count.
#[test]
fn admitting_beside_sixteen_thousand_body_findings_stays_near_the_cost_of_a_check() {
    let depth = 14;
    let mut widgets = String::new();
    for i in 0..=depth {
        let body = if i < depth {
            let next = i + 1;
            format!(
                "      - {{name: l, component: d{next}}}\n      - {{name: r, component: d{next}}}\n"
            )
        } else {
            "      - {name: go, primitive: button, label: Go, action: {name: go, opens: nowhere}}\n"
                .to_owned()
        };
        widgets.push_str(&format!(
            "  d{i}:\n    summary: Level {i}.\n    body:\n{body}"
        ));
    }
    let doc = Document::from_yaml(&with_member_item(
        &with_widgets(DOC, &widgets),
        "{name: deep, component: d0}",
    ))
    .unwrap_or_else(|e| panic!("{e}"));

    let started = Instant::now();
    let found = opens_at(&doc).len();
    let checked = started.elapsed();
    assert_eq!(found, 1 << depth);

    let insert = Patch::Insert {
        target: path("page:members/section:list"),
        child: Child {
            layer: Layer::Item,
            name: "extra".into(),
            node: json!({"primitive": "divider"}),
            nav_section: None,
        },
    };
    let started = Instant::now();
    admit(&doc, &insert).unwrap_or_else(|e| panic!("{e}"));
    let admitted = started.elapsed();
    eprintln!("depth {depth}: {found} findings, check {checked:?}, admit {admitted:?}");
    assert!(
        admitted < checked * 10 + Duration::from_millis(300),
        "check took {checked:?}, admit took {admitted:?}"
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
