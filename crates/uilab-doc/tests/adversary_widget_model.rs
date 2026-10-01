//! Adversarial cases for story:widget-model, driven from the ess shape on integrate/ess-ui-1
//! (schemas/ui/ess-ui.schema.yaml: Widget, WidgetInstance, Node, collection, section).

use serde_json::json;
use uilab_doc::model::{Component, Widget};
use uilab_doc::{Child, Document, Layer, NodePath, Patch, Severity, admit, check, node_context};

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
navigation:
  home: overview
  sections:
    - {name: circulation, label: Circulation, pages: [overview]}
widgets:
  loan_card:
    summary: A loan as a card.
    params:
      loan: {type: Loan, required: true, note: the loan row}
      compact: {type: boolean, default: false, note: compact param}
    body:
      - {name: title, primitive: text, text: args.loan.title, style: heading}
  state_badge:
    summary: A loan state as a toned badge.
    params:
      state: {type: string, required: true, note: state param}
    body:
      - {name: badge, primitive: badge, text: args.state}
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
"#;

fn doc() -> Document {
    Document::from_yaml(DOC).unwrap()
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

/// ess `Widget.params` values are records {type, required, default, note}. A key the subset does
/// not type must either be kept (the model's own round-trip promise) or refused (ess closes the
/// record); silently dropping it is neither.
#[test]
fn a_param_key_the_subset_does_not_type_is_kept_or_refused() {
    let text = "{summary: s, params: {state: {type: string, required: true, label: State, note: state param}}, body: []}";
    match serde_yaml::from_str::<Widget>(text) {
        Err(_) => {}
        Ok(widget) => {
            let yaml = uilab_doc::model::to_yaml(&widget).unwrap();
            assert!(
                yaml.contains("label: State"),
                "the param's `label` was dropped on the round trip:\n{yaml}"
            );
        }
    }
}

/// ess `default: {optional: json}`: `null` is a JSON value, and an explicit `default: null` is a
/// different declaration from no default at all.
#[test]
fn an_explicit_null_param_default_survives_a_round_trip() {
    let widget: Widget = serde_yaml::from_str(
        "{summary: s, params: {note: {type: string, default: null, note: note param}}, body: []}",
    )
    .unwrap();
    let yaml = uilab_doc::model::to_yaml(&widget).unwrap();
    assert!(
        yaml.contains("default: null"),
        "`default: null` was dropped on the round trip:\n{yaml}"
    );
}

/// The ess example for `collection` writes `item` as a list of named nodes, and uses a widget
/// instance there: `item: [{name: card, component: partner_card, args: {partner: row}}]`.
///
/// story:item-list: the list parses, `page:overview/section:list/item:card` resolves to a
/// `loan_card` instance, and the widget checks hold the instance to the widget's params.
#[test]
fn an_ess_collection_item_list_with_a_widget_instance_parses_and_resolves() {
    let with_item = |item: &str| {
        DOC.replace(
            "        columns: [{field: title}]\n",
            &format!("        columns: [{{field: title}}]\n        item: {item}\n"),
        )
    };
    let text = with_item("[{name: card, component: loan_card, args: {loan: row}}]");
    let doc = Document::from_yaml(&text).unwrap_or_else(|e| panic!("the ess list form: {e}"));
    let at = path("page:overview/section:list/item:card");
    let composite = uilab_doc::resolve(&doc, &at)
        .unwrap_or_else(|e| panic!("{at}: {e}"))
        .composite()
        .cloned()
        .expect("item:card is a composite");
    assert_eq!(composite.component, Component::Widget("loan_card".into()));
    assert_eq!(
        errors(&doc),
        [],
        "a well-formed instance passes every check"
    );

    // An instance passing a param the widget does not declare: ESS refuses the document.
    let missing = with_item("[{name: card, component: loan_card, args: {cover: row.cover}}]");
    let refused = Document::from_yaml(&missing).expect_err("ESS refuses an undeclared param");
    assert_eq!(
        refused.path, "pages/overview/sections/list/item/card/args/cover",
        "the instance in the item list is checked against its widget: {refused}"
    );
    let found = uilab_doc::ess_ui_check::check_source(
        &missing,
        "case",
        std::path::Path::new("."),
        None,
        &Default::default(),
    );
    assert!(
        found.findings.iter().all(|f| f.check == "widget_expands"),
        "{:?}",
        found.findings
    );
}

/// The agent is told "Composite kinds a new composite child can be" from `composite_kinds`; a
/// widget is usable wherever a composite kind is, so a declared widget belongs in that list.
#[test]
fn the_agent_context_offers_declared_widgets_where_a_composite_can_go() {
    let doc = doc();
    let context = node_context(&doc, &path("page:overview")).unwrap();
    assert!(
        context.composite_kinds.contains(&"loan_card"),
        "a page's context omits declared widgets: {:?}",
        context.composite_kinds
    );
}

/// ess section `children: {list: Node}` — "`children` adds widgets or primitives rendered with
/// the section". Removing the widget that such a child instantiates leaves a dangling instance.
#[test]
fn removing_a_widget_a_section_child_still_uses_is_refused() {
    let text = DOC.replace(
        "        columns: [{field: title}]\n",
        "        columns: [{field: title}]\n        children: [{name: badge, component: state_badge, args: {state: row.state}}]\n",
    );
    let doc = Document::from_yaml(&text).unwrap();
    assert!(errors(&doc).is_empty(), "{:?}", errors(&doc));
    let refused = admit(
        &doc,
        &Patch::Remove {
            target: path("component:state_badge"),
        },
    );
    assert_eq!(
        refused.map(|_| ()).map_err(|r| r.check),
        Err("widget_expands".to_owned()),
        "a widget still used by a section child was removed"
    );
}

/// Tests build with serde_json `arbitrary_precision`; a node goes through a JSON map, so a number
/// in a node's props must still be written as a number.
#[test]
fn a_number_in_a_body_node_round_trips_as_a_number() {
    let text = DOC.replace(
        "      - {name: title, primitive: text, text: args.loan.title, style: heading}\n",
        "      - {name: title, primitive: text, text: args.loan.title, style: heading}\n      - {name: state, component: state_badge, args: {state: args.loan.state, size: 3, ratio: 1.5}}\n",
    )
    .replace(
        "      state: {type: string, required: true, note: state param}\n",
        "      state: {type: string, required: true, note: state param}\n      size: {type: integer, note: badge size}\n      ratio: {type: number, note: badge ratio}\n",
    );
    let doc = Document::from_yaml(&text).unwrap();
    // ess-ui/1 gives no primitive a numeric prop; the numbers ride in a widget instance's args.
    let state = doc.widgets["loan_card"].node("state").unwrap();
    let args = &state.composite().unwrap().props["args"];
    assert_eq!((&args["size"], &args["ratio"]), (&json!(3), &json!(1.5)));
    let yaml = doc.to_yaml().unwrap();
    assert!(yaml.contains("ratio: 1.5"), "{yaml}");
    assert!(yaml.contains("size: 3"), "{yaml}");
    assert!(!yaml.contains("serde_json"), "{yaml}");
    assert_eq!(Document::from_yaml(&yaml).unwrap(), doc);
}

/// Recursion hidden one level down: loan_card's body holds a board whose widget instantiates
/// state_badge, and state_badge instantiates loan_card. ESS reports the loop at both widgets
/// (`widget_expands`); closing it by patch from either side is refused.
#[test]
fn indirect_recursion_through_a_board_node_is_found_at_both_instances() {
    let grid = json!({
        "name": "grid", "component": "board", "reads": {"view": "loans.All"},
        "widgets": {"state": {"component": "state_badge", "args": {"state": "x"}}}
    });
    let back = json!({"name": "back", "component": "loan_card", "args": {"loan": "x"}});
    let mut doc = doc();
    doc.widgets["loan_card"]
        .body
        .push(serde_json::from_value(grid.clone()).unwrap());
    doc.widgets["state_badge"]
        .body
        .push(serde_json::from_value(back.clone()).unwrap());
    assert_eq!(
        errors(&doc),
        [
            ("widget_expands", "component:loan_card".to_owned()),
            ("widget_expands", "component:state_badge".to_owned()),
        ]
    );

    for (half, owner, closing, other) in [
        (&grid, "loan_card", &back, "state_badge"),
        (&back, "state_badge", &grid, "loan_card"),
    ] {
        let mut open = self::doc();
        open.widgets[owner]
            .body
            .push(serde_json::from_value(half.clone()).unwrap());
        let mut node = closing.clone();
        let name = node["name"].as_str().unwrap().to_owned();
        node.as_object_mut().unwrap().remove("name");
        let close = Patch::Insert {
            target: path(&format!("component:{other}")),
            child: Child {
                layer: Layer::Node,
                name,
                node,
                nav_section: None,
            },
        };
        assert_eq!(
            admit(&open, &close).unwrap_err().check,
            "widget_expands",
            "closing the loop at {other}"
        );
    }
}

/// `doc()` with `loan_card` used as section `card` of the overview. ESS checks the args of an
/// instance in a widget body where that widget is used (`widget_expands`, "at each use").
fn with_card_used() -> Document {
    let mut doc = doc();
    let card =
        serde_json::from_value(json!({"component": "loan_card", "args": {"loan": "rows.first"}}))
            .unwrap();
    doc.pages["overview"]
        .sections
        .insert("card".into(), Some(card));
    doc
}

/// A board node inside a widget body takes a board widget by patch, and that widget can be an
/// instance, checked for its args where the widget is used.
#[test]
fn a_board_node_in_a_body_takes_a_widget_instance_by_patch() {
    let mut doc = with_card_used();
    doc.widgets["loan_card"].body.push(
        serde_json::from_value(
            json!({"name": "grid", "component": "board", "reads": {"view": "loans.All"}}),
        )
        .unwrap(),
    );
    let at = path("component:loan_card/node:grid");
    let insert = |node: serde_json::Value| Patch::Insert {
        target: at.clone(),
        child: Child {
            layer: Layer::Widget,
            name: "state".into(),
            node,
            nav_section: None,
        },
    };
    let (next, _) = admit(
        &doc,
        &insert(json!({"component": "state_badge", "args": {"state": "args.loan.state"}})),
    )
    .unwrap();
    assert!(uilab_doc::resolve(&next, &path("component:loan_card/node:grid/widget:state")).is_ok());
    assert_eq!(
        admit(&doc, &insert(json!({"component": "state_badge"})))
            .unwrap_err()
            .check,
        "widget_expands"
    );
}

/// Replacing a widget so that it no longer declares a param an instance passes is refused, at the
/// use of the widget whose body holds the instance.
#[test]
fn replacing_a_widget_to_drop_a_param_its_instance_passes_is_refused() {
    let mut doc = with_card_used();
    doc.widgets["loan_card"].body.push(
        serde_json::from_value(
            json!({"name": "state", "component": "state_badge", "args": {"state": "args.loan.state"}}),
        )
        .unwrap(),
    );
    let refused = admit(
        &doc,
        &Patch::Replace {
            target: path("component:state_badge"),
            node: json!({"summary": "s", "body": [{"name": "b", "primitive": "divider"}]}),
        },
    )
    .unwrap_err();
    assert_eq!(refused.check, "widget_expands");
    assert!(
        refused
            .message
            .starts_with("page:overview/section:card: widget `loan_card`, `pages/overview/sections/card/body/state"),
        "{}",
        refused.message
    );
}
