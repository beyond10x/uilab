//! Adversarial cases, pass 2, for story:widget-opens-at-use against the correction 8cf170a: args
//! substitution through nested instances, index-free instance keys, the uncached expansion of a
//! body that recurs. Driven from ess WidgetInstance.expansion as quoted in
//! `adversary_widget_opens.rs`: "substitute: args.<param> in the body is replaced by the bound
//! expression", then "validate: the expanded nodes are checked like built-ins".

use std::time::{Duration, Instant};

use serde_json::json;
use uilab_doc::{Child, Document, Layer, NodePath, Patch, admit, check};

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
  opener:
    summary: A button that opens the overlay it is given.
    params:
      target: {type: {ref: overlay}, required: true}
    body:
      - {name: go, primitive: button, label: Open, action: {name: go, opens: args.target}}
  relay:
    summary: Hands its own target to an opener.
    params:
      target: {type: {ref: overlay}, required: true}
    body:
      - {name: inner, component: opener, args: {target: args.target}}
  relay_default:
    summary: Hands its own target, by default nowhere, to an opener.
    params:
      target: {type: {ref: overlay}, default: nowhere}
    body:
      - {name: inner, component: opener, args: {target: args.target}}
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

fn opens_found(doc: &Document) -> Vec<(String, String)> {
    check(doc)
        .into_iter()
        .filter(|f| f.check == "opens_resolves")
        .map(|f| (f.path, f.message))
        .collect()
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

/// `relay`'s body binds `opener`'s `target` to `args.target`, relay's own param. Substituted at
/// each level, a use of `relay` bound to `nowhere` expands to a button that opens `nowhere`, which
/// `members` does not declare. The same use bound to `extend` on `overview` resolves.
#[test]
fn a_pass_through_bound_to_a_literal_at_the_outer_use_is_checked_against_that_literal() {
    let text = DOC.replace(
        "      latest: {component: loan_card, args: {loan: rows.first}}\n",
        "      latest: {component: loan_card, args: {loan: rows.first}}\n      relayed: {component: relay, args: {target: extend}}\n",
    );
    let doc = parse(&with_member_items(
        &text,
        "          - {name: p, component: relay, args: {target: nowhere}}\n",
    ));
    let found = opens_found(&doc);
    assert!(
        found.len() == 1
            && found[0].0 == "page:members/section:list/item:p"
            && found[0].1.contains("opens `nowhere`"),
        "relay bound to `nowhere` on members: {found:#?}"
    );

    let clean = parse(&text);
    let refused = admit(
        &clean,
        &item(
            "p",
            json!({"component": "relay", "args": {"target": "nowhere"}}),
        ),
    )
    .expect_err("a relay bound to `nowhere` brings an unresolved opens onto members");
    assert_eq!(refused.check, "opens_resolves", "{refused}");
}

/// `relay_default` binds nothing at its use, so its `target` is its default `nowhere`, which the
/// nested opener receives through `args.target`.
#[test]
fn a_pass_through_of_an_outer_default_is_checked_against_that_default() {
    let doc = parse(&with_member_items(
        DOC,
        "          - {name: d, component: relay_default}\n",
    ));
    let found = opens_found(&doc);
    assert!(
        found.len() == 1
            && found[0].0 == "page:members/section:list/item:d"
            && found[0].1.contains("opens `nowhere`"),
        "relay_default on members: {found:#?}"
    );
}

/// A param whose default is a runtime reference cannot be judged statically and reports nothing;
/// the same widget bound to a literal the page lacks is reported. Positive control included so
/// the silence is not the absence of any check.
#[test]
fn a_runtime_reference_default_is_skipped_and_a_bound_literal_the_page_lacks_is_reported() {
    let widgets = "  by_row:\n    summary: Opens the overlay its row names.\n    params:\n      target: {type: {ref: overlay}, default: row.overlay}\n    body:\n      - {name: go, primitive: button, label: Go, action: {name: go, opens: args.target}}\n";
    let text = DOC.replace("widgets:\n", &format!("widgets:\n{widgets}"));
    let doc = parse(&with_member_items(
        &text,
        "          - {name: r, component: by_row}\n          - {name: s, component: by_row, args: {target: extend}}\n",
    ));
    let found = opens_found(&doc);
    assert!(
        found.len() == 1
            && found[0].0 == "page:members/section:list/item:s"
            && found[0].1.contains("opens `extend`"),
        "{found:#?}"
    );
}

/// `members` already carries one header metric whose `loan_card` body opens `extend`, which the
/// page lacks. A replace of the page that adds a second, different `loan_card` metric brings a
/// second unresolved `opens` onto the page. At dba026d the two were `header/metrics/0` and
/// `header/metrics/1`; with index-free keys both findings are equal, and admit's set lookup
/// takes the new one for the old one.
#[test]
fn a_second_broken_unnamed_instance_beside_an_equal_one_is_refused() {
    let text = DOC.replace(
        "    kind: list_page\n    title: Members\n",
        "    kind: list_page\n    title: Members\n    header: {metrics: [{component: loan_card, args: {loan: rows.first}}]}\n",
    );
    let doc = parse(&text);
    assert_eq!(opens_found(&doc).len(), 1, "{:#?}", opens_found(&doc));

    let mut page = serde_json::to_value(&doc.pages["members"]).unwrap();
    page["header"]["metrics"] = json!([
        {"component": "loan_card", "args": {"loan": "rows.first"}},
        {"component": "loan_card", "args": {"loan": "rows.last"}},
    ]);
    let replace = Patch::Replace {
        target: path("page:members"),
        node: page,
    };
    match admit(&doc, &replace) {
        Err(refused) => assert_eq!(refused.check, "opens_resolves", "{refused}"),
        Ok((next, _)) => panic!(
            "a second broken loan_card metric was admitted; its findings: {:#?}",
            opens_found(&next)
        ),
    }
}

/// A chain of widgets each using the next twice, whose last widget holds a widget that holds
/// itself (a `widget_recursion` error, which admit must still judge). No body holds an `opens`,
/// so there is nothing to report and nothing to expand into; before 8cf170a each widget was
/// expanded once. Checking must stay within the bounds of `checking_a_large_document_stays_fast`.
#[test]
fn a_recursive_leaf_under_a_doubling_chain_is_checked_in_bounded_time() {
    let mut last = Duration::ZERO;
    for depth in [12, 14, 22] {
        let mut widgets = String::new();
        for i in 0..=depth {
            let body = if i < depth {
                let next = i + 1;
                format!(
                    "      - {{name: l, component: d{next}}}\n      - {{name: r, component: d{next}}}\n"
                )
            } else {
                "      - {name: s, component: selfish}\n".to_owned()
            };
            widgets.push_str(&format!(
                "  d{i}:\n    summary: Level {i}.\n    body:\n{body}"
            ));
        }
        widgets.push_str(
            "  selfish:\n    summary: Holds itself.\n    body:\n      - {name: again, component: selfish}\n",
        );
        let doc = parse(&with_member_items(
            &DOC.replace("widgets:\n", &format!("widgets:\n{widgets}")),
            "          - {name: deep, component: d0}\n",
        ));
        let started = Instant::now();
        let found = check(&doc);
        last = started.elapsed();
        let recursion = found
            .iter()
            .filter(|f| f.check == "widget_recursion")
            .count();
        eprintln!("depth {depth}: check {last:?}, widget_recursion {recursion}");
        assert_eq!(
            found.iter().filter(|f| f.check == "opens_resolves").count(),
            0
        );
    }
    assert!(
        last < Duration::from_secs(5),
        "check at depth 22 took {last:?}"
    );
}

/// Control for the case above: the same chain at depth 22 with a plain leaf, no recursion.
#[test]
fn a_doubling_chain_without_recursion_is_checked_in_bounded_time() {
    let depth = 22;
    let mut widgets = String::new();
    for i in 0..=depth {
        let body = if i < depth {
            let next = i + 1;
            format!(
                "      - {{name: l, component: d{next}}}\n      - {{name: r, component: d{next}}}\n"
            )
        } else {
            "      - {name: t, primitive: text, text: Leaf}\n".to_owned()
        };
        widgets.push_str(&format!(
            "  d{i}:\n    summary: Level {i}.\n    body:\n{body}"
        ));
    }
    let doc = parse(&with_member_items(
        &DOC.replace("widgets:\n", &format!("widgets:\n{widgets}")),
        "          - {name: deep, component: d0}\n",
    ));
    let started = Instant::now();
    check(&doc);
    let took = started.elapsed();
    eprintln!("control depth {depth}: check {took:?}");
    assert!(took < Duration::from_secs(5), "control took {took:?}");
}
