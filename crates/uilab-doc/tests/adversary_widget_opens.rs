//! Adversarial cases for story:widget-opens-at-use (dba026d): `opens_resolves` in widget bodies at
//! each use site, driven from ess ui-spec/1 on plan/ess-ui-implemented
//! (schemas/ui/ess-ui.schema.yaml Widget :850, WidgetInstance :877) and from the `NodePath`
//! contract in `crates/uilab-doc/src/path.rs`.

use std::time::{Duration, Instant};

use serde_json::json;
use uilab_doc::{Child, Document, Layer, NodePath, Patch, admit, check, resolve};

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
    overlays:
      help: {kind: drawer, component: record, reads: {view: loans.All}}
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
  helper:
    summary: A button that opens the shell's help drawer.
    body:
      - {name: ask, primitive: button, label: Help, action: {name: ask, opens: help}}
  opener:
    summary: A button that opens the overlay it is given.
    params:
      target: {type: {ref: overlay}, required: true}
    body:
      - {name: go, primitive: button, label: Open, action: {name: go, opens: args.target}}
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

fn parse(text: &str) -> Document {
    Document::from_yaml(text).unwrap_or_else(|e| panic!("{e}"))
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

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

fn with_member_items(text: &str, items: &str) -> String {
    text.replace(
        "          - {name: tag, primitive: badge, text: row.state}\n",
        &format!("          - {{name: tag, primitive: badge, text: row.state}}\n{items}"),
    )
}

fn item(name: &str, node: serde_json::Value) -> Patch {
    Patch::Insert {
        target: path("page:members/section:list"),
        child: Child {
            layer: Layer::Item,
            name: name.into(),
            node,
            nav_section: None,
        },
    }
}

/// ess WidgetInstance.expansion: "substitute: args.<param> in the body is replaced by the bound
/// expression", then "validate: the expanded nodes are checked like built-ins". `opener`'s body
/// opens `args.target`; bound to `extend` on `overview`, which declares `extend`, it resolves.
/// Bound to `extend` on `members`, which does not, the finding names `extend`, not `args.target`.
#[test]
fn an_opens_bound_through_args_is_checked_against_the_bound_overlay() {
    let text = DOC.replace(
        "      latest: {component: loan_card, args: {loan: rows.first}}\n",
        "      latest: {component: loan_card, args: {loan: rows.first}}\n      open_extend: {component: opener, args: {target: extend}}\n",
    );
    let doc = parse(&text);
    let found: Vec<_> = check(&doc)
        .into_iter()
        .filter(|f| f.check == "opens_resolves")
        .map(|f| (f.path, f.message))
        .collect();
    assert_eq!(
        found,
        [],
        "`opener` bound to `extend` sits on `overview`, which declares `extend`"
    );

    let on_members = admit(
        &doc,
        &item(
            "open",
            json!({"component": "opener", "args": {"target": "extend"}}),
        ),
    );
    let refused = on_members.expect_err("members declares no `extend` overlay");
    assert!(
        refused.message.contains("`extend`") && !refused.message.contains("args.target"),
        "{refused}"
    );
}

/// `NodePath` (path.rs): "A path is `/`-separated segments of `<layer>:<name>`"; a finding's path
/// is what the operator clicks to select the node (SidebarPanel.vue `select(f.path)`, and
/// uilab-behaviour `select_node` answers `NotFound` for a path that does not parse or resolve).
/// The brief: "If NodePath cannot carry that suffix, use the instance path and name the body node
/// in the message".
#[test]
fn a_widget_body_finding_path_parses_and_resolves_as_a_node_path() {
    let doc = parse(&with_member_items(
        DOC,
        "          - {name: card, component: loan_card, args: {loan: row}}\n",
    ));
    let found: Vec<_> = check(&doc)
        .into_iter()
        .filter(|f| f.check == "opens_resolves")
        .collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    for f in &found {
        let parsed = f.path.parse::<NodePath>();
        assert!(
            parsed.as_ref().is_ok_and(|p| resolve(&doc, p).is_ok()),
            "finding path `{}` does not select a node: {:?}",
            f.path,
            parsed.map(|p| resolve(&doc, &p).map(|_| ()))
        );
    }
}

/// Two widgets that contain each other (a `widget_recursion` document, which admit lets the
/// operator keep editing elsewhere). `loop_b`'s body opens `nowhere` and holds `loop_a`;
/// `loop_a`'s body holds `loop_b`. A use of `loop_a` on `members` carries `loop_b`'s button, as
/// `a_recursive_widget_body_is_expanded_once` shows for the other order. Whether it is reported
/// must not depend on whether a use of `loop_b` comes earlier in the document.
#[test]
fn a_recursive_widget_reached_first_inside_another_is_still_expanded_at_its_own_use() {
    let widgets = "  loop_a:\n    summary: Holds loop_b.\n    body:\n      - {name: b, component: loop_b}\n  loop_b:\n    summary: Holds loop_a and opens nowhere.\n    body:\n      - {name: go, primitive: button, label: Go, action: {name: go, opens: nowhere}}\n      - {name: a, component: loop_a}\n";
    let alone = parse(&with_member_items(
        &with_widgets(DOC, widgets),
        "          - {name: la, component: loop_a}\n",
    ));
    assert_eq!(
        opens_at(&alone),
        ["page:members/section:list/item:la/body/b/body/go"],
        "loop_a used alone"
    );

    let after_b = parse(&with_member_items(
        &with_widgets(DOC, widgets),
        "          - {name: lb, component: loop_b}\n          - {name: la, component: loop_a}\n",
    ));
    assert_eq!(
        opens_at(&after_b),
        [
            "page:members/section:list/item:lb/body/go",
            "page:members/section:list/item:la/body/b/body/go",
        ],
        "loop_a used after loop_b"
    );

    let with_b = parse(&with_member_items(
        &with_widgets(DOC, widgets),
        "          - {name: lb, component: loop_b}\n",
    ));
    let admitted = admit(&with_b, &item("la", json!({"component": "loop_a"})));
    assert!(
        admitted.is_err(),
        "an instance of loop_a, whose body carries loop_b's `opens: nowhere`, is admitted onto \
         `members`"
    );
}

/// A document that already has an unresolved `opens` in a body at a use site: an unrelated insert
/// elsewhere is admitted, and the finding keeps its path and message.
#[test]
fn an_unresolved_body_opens_already_present_does_not_block_an_unrelated_insert() {
    let doc = parse(&with_member_items(
        DOC,
        "          - {name: card, component: loan_card, args: {loan: row}}\n",
    ));
    let before: Vec<_> = check(&doc)
        .into_iter()
        .filter(|f| f.check == "opens_resolves")
        .collect();
    assert_eq!(before.len(), 1);
    let (next, _) = admit(&doc, &item("extra", json!({"primitive": "divider"})))
        .unwrap_or_else(|e| panic!("{e}"));
    let after: Vec<_> = check(&next)
        .into_iter()
        .filter(|f| f.check == "opens_resolves")
        .collect();
    assert_eq!(before, after);
}

/// path.rs: "keyed by name and never by position. Adding a sibling never changes an existing
/// path." An instance in an unnamed header metric already carries an unresolved body `opens`.
/// A replace of the page that puts a clean metric in front of it brings no new unresolved
/// `opens` onto the page, and is refused all the same, because the old error's path moved.
#[test]
fn a_clean_metric_added_in_front_of_a_broken_one_is_not_refused_for_the_old_error() {
    let text = DOC.replace(
        "    kind: list_page\n    title: Members\n",
        "    kind: list_page\n    title: Members\n    header: {metrics: [{component: loan_card, args: {loan: rows.first}}]}\n",
    );
    let doc = parse(&text);
    assert_eq!(opens_at(&doc).len(), 1, "{:?}", opens_at(&doc));
    let mut page = serde_json::to_value(&doc.pages["members"]).unwrap();
    page["header"]["metrics"] = json!([
        {"component": "helper"},
        {"component": "loan_card", "args": {"loan": "rows.first"}},
    ]);
    let replace = Patch::Replace {
        target: path("page:members"),
        node: page,
    };
    admit(&doc, &replace).unwrap_or_else(|refused| {
        panic!("a clean metric in front of the pre-existing error is refused: {refused}")
    });
}

/// Use sites the implementation's own tests do not name: a collection item inside a board widget,
/// a page overlay whose body is the instance, and a page-level button's `choice`. Each is on
/// `members` and each is reported once. An instance whose body opens the shell's overlay, and an
/// instance whose overlay the page gains in the same batch, are clean.
#[test]
fn every_page_position_is_a_use_site_and_shell_or_batch_overlays_resolve() {
    let text = with_member_items(
        &DOC.replace(
            "    sections:\n      list:\n",
            "    sections:\n      board:\n        component: board\n        widgets:\n          top:\n            component: collection\n            reads: {view: loans.All}\n            item:\n              - {name: c, component: loan_card, args: {loan: row}}\n      list:\n",
        )
        .replace(
            "        item:\n          - {name: tag, primitive: badge, text: row.state}\n",
            "        item:\n          - {name: tag, primitive: badge, text: row.state}\n    overlays:\n      peek: {kind: drawer, component: loan_card, args: {loan: rows.first}}\n",
        ),
        "          - {name: pick, primitive: button, label: Pick, action: {name: pick, does: loans.Pick, choice: {component: loan_card, args: {loan: row}}}}\n          - {name: ask, component: helper}\n",
    );
    let doc = parse(&text);
    let mut found = opens_at(&doc);
    found.sort();
    assert_eq!(
        found,
        [
            "page:members/overlay:peek/body/extend",
            "page:members/section:board/widget:top/item:c/body/extend",
            "page:members/section:list/item:pick/action/choice/body/extend",
        ]
    );

    let clean = parse(DOC);
    let batch = Patch::Batch {
        target: path("page:members"),
        patches: vec![
            Patch::Insert {
                target: path("page:members"),
                child: Child {
                    layer: Layer::Overlay,
                    name: "extend".into(),
                    node: json!({"kind": "dialog", "component": "record", "reads": {"view": "loans.All"}}),
                    nav_section: None,
                },
            },
            item(
                "card",
                json!({"component": "loan_card", "args": {"loan": "row"}}),
            ),
        ],
    };
    let (next, _) = admit(&clean, &batch).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(opens_at(&next), Vec::<String>::new());
}

/// A chain of 13 widgets, each using the next twice, the last opening `nowhere`: per ess the use
/// expands to 2^12 buttons, each unresolved. Checking it and admitting an unrelated insert into a
/// document that already has the use stay within the bounds of `checking_a_large_document_stays_fast`.
#[test]
fn a_chain_doubling_at_each_of_twelve_levels_is_checked_in_bounded_time() {
    let depth = 12;
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
    let doc = parse(&with_member_items(
        &with_widgets(DOC, &widgets),
        "          - {name: deep, component: d0}\n",
    ));

    let started = Instant::now();
    let found = opens_at(&doc).len();
    let checked = started.elapsed();
    assert_eq!(found, 1 << depth);

    let started = Instant::now();
    admit(&doc, &item("extra", json!({"primitive": "divider"}))).unwrap_or_else(|e| panic!("{e}"));
    let admitted = started.elapsed();
    eprintln!("depth {depth}: {found} findings, check {checked:?}, admit {admitted:?}");
    assert!(
        checked < Duration::from_secs(5) && admitted < Duration::from_secs(10),
        "check took {checked:?}, admit took {admitted:?}"
    );
}
