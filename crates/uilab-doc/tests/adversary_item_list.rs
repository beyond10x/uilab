//! Adversarial cases for story:item-list, driven from the ess shape on integrate/ess-ui-1
//! (schemas/ui/ess-ui.schema.yaml: collection, record, Node, Primitive, button, Action).

use serde_json::json;
use uilab_doc::path::{NodeRef, children};
use uilab_doc::{Child, Document, Layer, NodePath, Patch, Severity, admit, check, resolve};

const DOC: &str = r#"
format: ess-ui/1
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
      loan:
        kind: drawer
        component: record
        reads: {view: loans.All}
        item:
          - {name: title, primitive: text, text: row.title}
navigation:
  home: overview
  sections:
    - {name: circulation, label: Circulation, pages: [overview]}
widgets:
  loan_card:
    summary: A loan as a card.
    params:
      loan: {type: Loan, required: true, note: the loan row}
    body:
      - {name: title, primitive: text, text: args.loan.title, style: heading}
      - name: rows
        component: collection
        reads: {view: loans.All}
        item:
          - {name: due, primitive: text, text: row.due}
pages:
  overview:
    kind: dashboard_page
    title: Overview
    sections:
      - {name: board, remove: true}
      - name: list
        component: collection
        reads: {view: loans.All}
        columns: [{field: title}]
        item:
          - {name: cover, primitive: image, src: row.cover_url, alt: Book cover}
          - {name: card, component: loan_card, args: {loan: row}}
          - {name: tag, primitive: badge, text: row.state}
    overlays:
      extend: {kind: dialog, component: record, reads: {view: loans.All}}
"#;

fn doc() -> Document {
    Document::from_yaml(DOC).unwrap_or_else(|e| panic!("{e}"))
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

fn errors(doc: &Document) -> Vec<(&'static str, String)> {
    check(doc)
        .into_iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| (f.check, f.path))
        .collect()
}

fn names(doc: &Document, at: &str) -> Vec<String> {
    children(doc, &path(at))
        .unwrap()
        .into_iter()
        .map(|(_, n)| n)
        .collect()
}

fn with_list_item(item: &str) -> String {
    DOC.replace(
        "          - {name: tag, primitive: badge, text: row.state}\n",
        &format!(
            "          - {{name: tag, primitive: badge, text: row.state}}\n          - {item}\n"
        ),
    )
}

/// ess `button.action: Action`, and an Action `opens` an overlay. `opens_resolves` holds every
/// `opens` of a page's composites to an overlay of the page or its shell, and it does so for a
/// button written in a section's `children`. A button written as an `item` node, which this story
/// made possible, must be held to the same check.
#[test]
fn a_button_item_that_opens_a_missing_overlay_is_reported() {
    let base = doc();
    assert_eq!(errors(&base), [], "the fixture is clean");

    let in_children = Document::from_yaml(&DOC.replace(
        "        columns: [{field: title}]\n",
        "        columns: [{field: title}]\n        children: [{name: go, primitive: button, label: Go, action: {name: go, opens: nowhere}}]\n",
    ))
    .unwrap();
    assert!(
        errors(&in_children).contains(&(
            "opens_resolves",
            "page:overview/section:list/child:go".to_owned()
        )),
        "control: a button in `children` is checked: {:?}",
        errors(&in_children)
    );

    let reaching = Document::from_yaml(&with_list_item(
        "{name: go, primitive: button, label: Go, action: {name: go, opens: extend}}",
    ))
    .unwrap();
    assert_eq!(
        errors(&reaching),
        [],
        "a button item opening a page overlay is fine"
    );

    let broken = Document::from_yaml(&with_list_item(
        "{name: go, primitive: button, label: Go, action: {name: go, opens: nowhere}}",
    ))
    .unwrap();
    let found = errors(&broken);
    assert!(
        found.iter().any(|(c, _)| *c == "opens_resolves"),
        "a button item opens `nowhere`, which neither the page nor its shell declares, and no \
         check says so: {found:?}"
    );

    let insert = Patch::Insert {
        target: path("page:overview/section:list"),
        child: Child {
            layer: Layer::Item,
            name: "go".into(),
            node: json!({"primitive": "button", "label": "Go", "action": {"name": "go", "opens": "nowhere"}}),
            nav_section: None,
        },
    };
    match admit(&base, &insert) {
        Err(refused) => assert_eq!(refused.check, "opens_resolves", "{refused}"),
        Ok(_) => panic!(
            "a patch inserting a button item that opens a missing overlay is admitted without a \
             finding"
        ),
    }
}

/// Paths stay `item:<name>`: remove and replace at the start, the middle and the end keep every
/// other node where it was, in a page section, a shell overlay and a widget body.
#[test]
fn item_nodes_are_removed_and_replaced_at_every_position() {
    let base = doc();
    let list = "page:overview/section:list";
    for (name, left) in [
        ("cover", ["card", "tag"]),
        ("card", ["cover", "tag"]),
        ("tag", ["cover", "card"]),
    ] {
        let remove = Patch::Remove {
            target: path(&format!("{list}/item:{name}")),
        };
        let (next, _) = admit(&base, &remove).unwrap_or_else(|e| panic!("remove {name}: {e}"));
        assert_eq!(names(&next, list), left, "remove {name}");
        let replace = Patch::Replace {
            target: path(&format!("{list}/item:{name}")),
            node: json!({"primitive": "divider"}),
        };
        let (next, _) = admit(&base, &replace).unwrap_or_else(|e| panic!("replace {name}: {e}"));
        assert_eq!(
            names(&next, list),
            ["cover", "card", "tag"],
            "replace {name}"
        );
        assert!(matches!(
            resolve(&next, &path(&format!("{list}/item:{name}"))).unwrap(),
            NodeRef::Primitive(p) if p.primitive == uilab_doc::model::PrimitiveKind::Divider
        ));
    }

    for (holder, first) in [
        ("shell:app/overlay:loan", "title"),
        ("component:loan_card/node:rows", "due"),
    ] {
        let insert = Patch::Insert {
            target: path(holder),
            child: Child {
                layer: Layer::Item,
                name: "extra".into(),
                node: json!({"primitive": "text", "text": "row.id"}),
                nav_section: None,
            },
        };
        let (next, _) = admit(&base, &insert).unwrap_or_else(|e| panic!("insert at {holder}: {e}"));
        assert_eq!(names(&next, holder), [first, "extra"], "{holder}");
        let remove = Patch::Remove {
            target: path(&format!("{holder}/item:{first}")),
        };
        let (after, _) =
            admit(&next, &remove).unwrap_or_else(|e| panic!("remove at {holder}: {e}"));
        assert_eq!(names(&after, holder), ["extra"], "{holder}");
    }
}

/// An empty list reads as no items and writes nothing. An empty map and a null are not an `item`
/// list in ess-ui/1: ESS's loader refuses both, and uilab relays the refusal.
#[test]
fn empty_item_forms_read_as_no_items() {
    for (form, at) in [
        ("{}", "pages/overview/sections/list"),
        ("~", "pages/overview/sections/list"),
    ] {
        let text = DOC.replace(
            "        columns: [{field: title}]\n        item:\n          - {name: cover, primitive: image, src: row.cover_url, alt: Book cover}\n          - {name: card, component: loan_card, args: {loan: row}}\n          - {name: tag, primitive: badge, text: row.state}\n",
            &format!("        columns: [{{field: title}}]\n        item: {form}\n"),
        );
        assert_ne!(text, DOC, "the fixture changed");
        let refused = Document::from_yaml(&text).expect_err(form);
        assert_eq!(refused.path, at, "{form}: {refused}");
    }
    {
        let empty = "[]";
        let text = DOC.replace(
            "        columns: [{field: title}]\n        item:\n          - {name: cover, primitive: image, src: row.cover_url, alt: Book cover}\n          - {name: card, component: loan_card, args: {loan: row}}\n          - {name: tag, primitive: badge, text: row.state}\n",
            &format!("        columns: [{{field: title}}]\n        item: {empty}\n"),
        );
        assert_ne!(text, DOC, "the fixture changed");
        let doc = Document::from_yaml(&text).unwrap_or_else(|e| panic!("item: {empty}: {e}"));
        assert!(
            names(&doc, "page:overview/section:list").is_empty(),
            "{empty}"
        );
        assert_eq!(errors(&doc), [], "{empty}");
        let written = doc.to_yaml().unwrap();
        assert_eq!(Document::from_yaml(&written).unwrap(), doc, "{empty}");
    }
}

/// ess: in lists every node carries `name`, and a Node is a map with exactly one of `component` and
/// `primitive`. Each wrong entry is refused, naming what is wrong.
#[test]
fn a_malformed_item_entry_is_refused_with_its_reason() {
    for (entry, says) in [
        ("{primitive: divider}", "needs a `name`"),
        ("{name: '', primitive: divider}", "needs a `name`"),
        ("card", "Composite shorthand"),
        (
            "{name: x, component: metric, primitive: divider}",
            "exactly one",
        ),
        (
            "{name: x, text: row.id}",
            "needs `component` or `primitive`",
        ),
    ] {
        match Document::from_yaml(&with_list_item(entry)) {
            Ok(_) => panic!("`{entry}` in an item list is accepted"),
            Err(e) => assert!(e.to_string().contains(says), "`{entry}`: {e}"),
        }
    }
}

/// Numbers in an item node are numbers after a round trip, integers and floats alike.
#[test]
fn numbers_in_item_nodes_survive_a_round_trip() {
    let doc = Document::from_yaml(&with_list_item(
        "{name: due, component: choice, options: [{value: 2, label: two}, {value: 1.5, label: one and a half}]}",
    ))
    .unwrap();
    let written = doc.to_yaml().unwrap();
    let yaml: serde_yaml::Value = serde_yaml::from_str(&written).unwrap();
    let due = &yaml["pages"]["overview"]["sections"][1]["item"][3];
    assert_eq!(due["name"].as_str(), Some("due"), "{written}");
    assert_eq!(due["options"][0]["value"].as_u64(), Some(2), "{written}");
    assert_eq!(due["options"][1]["value"].as_f64(), Some(1.5), "{written}");
    assert_eq!(Document::from_yaml(&written).unwrap(), doc);
}

/// A widget whose body holds a collection whose item instantiates the widget contains itself.
#[test]
fn a_widget_containing_itself_through_an_item_is_refused() {
    let insert = Patch::Insert {
        target: path("component:loan_card/node:rows"),
        child: Child {
            layer: Layer::Item,
            name: "again".into(),
            node: json!({"component": "loan_card", "args": {"loan": "row"}}),
            nav_section: None,
        },
    };
    let refused = admit(&doc(), &insert).unwrap_err();
    assert_eq!(refused.check, "widget_expands", "{refused}");
}
