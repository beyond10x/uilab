//! Adversarial cases for story:widget-model, driven from the ess shape on integrate/ess-ui-1
//! (schemas/ui/ess-ui.schema.yaml: Widget, WidgetInstance, Node, collection, section).

use serde_json::json;
use uilab_doc::model::{Component, Widget};
use uilab_doc::{Child, Document, Layer, NodePath, Patch, Severity, admit, check, node_context};

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
  loan_card:
    summary: A loan as a card.
    params:
      loan: {type: Loan, required: true}
      compact: {type: boolean, default: false}
    body:
      - {name: title, primitive: text, text: args.loan.title, style: heading}
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
      list:
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
    let text =
        "{summary: s, params: {state: {type: string, required: true, label: State}}, body: []}";
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
        "{summary: s, params: {note: {type: string, default: null}}, body: []}",
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
/// Today's state, pinned until story:item-list lands: this subset reads `item` as a map, so the
/// ess list form is refused with its current parse error. When story:item-list lands, flip this
/// case back: the list parses, and `page:overview/section:list/item:card` resolves to a
/// `loan_card` instance.
#[test]
fn an_ess_collection_item_list_with_a_widget_instance_parses_and_resolves() {
    let text = DOC.replace(
        "        columns: [{field: title}]\n",
        "        columns: [{field: title}]\n        item: [{name: card, component: loan_card, args: {loan: row}}]\n",
    );
    match Document::from_yaml(&text) {
        Err(refused) => assert!(
            refused
                .to_string()
                .contains("item: invalid type: sequence, expected a map"),
            "story:item-list has not landed, so the list form of `item` is refused, but with a \
             different error than pinned: {refused}"
        ),
        Ok(doc) => {
            let at = path("page:overview/section:list/item:card");
            let composite = uilab_doc::resolve(&doc, &at)
                .ok()
                .and_then(|n| n.composite().cloned());
            panic!(
                "the ess list form of `item` now parses (item:card resolves to {:?}): \
                 story:item-list has landed, so flip this case to assert that it parses and \
                 resolves to Component::Widget(\"loan_card\")",
                composite.map(|c| c.component == Component::Widget("loan_card".into()))
            );
        }
    }
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
        Err("widget_resolves".to_owned()),
        "a widget still used by a section child was removed"
    );
}

/// Tests build with serde_json `arbitrary_precision`; a node goes through a JSON map, so a number
/// in a node's props must still be written as a number.
#[test]
fn a_number_in_a_body_node_round_trips_as_a_number() {
    let text = DOC.replace(
        "      - {name: title, primitive: text, text: args.loan.title, style: heading}\n",
        "      - {name: title, primitive: text, text: args.loan.title, style: heading, max_lines: 2}\n      - {name: state, component: state_badge, args: {state: args.loan.state, size: 3}}\n",
    );
    let doc = Document::from_yaml(&text).unwrap();
    let title = doc.widgets["loan_card"].node("title").unwrap();
    assert_eq!(title.primitive().unwrap().props["max_lines"], json!(2));
    let yaml = doc.to_yaml().unwrap();
    assert!(yaml.contains("max_lines: 2"), "{yaml}");
    assert!(yaml.contains("size: 3"), "{yaml}");
    assert!(!yaml.contains("serde_json"), "{yaml}");
    assert_eq!(Document::from_yaml(&yaml).unwrap(), doc);
}

/// Recursion hidden one level down: loan_card's body holds a board whose widget instantiates
/// state_badge, and state_badge instantiates loan_card. Both instance paths are reported.
#[test]
fn indirect_recursion_through_a_board_node_is_found_at_both_instances() {
    let mut doc = doc();
    doc.widgets["loan_card"].body.push(
        serde_json::from_value(json!({
            "name": "grid", "component": "board",
            "widgets": {"state": {"component": "state_badge", "args": {"state": "x"}}}
        }))
        .unwrap(),
    );
    doc.widgets["state_badge"].body.push(
        serde_json::from_value(
            json!({"name": "back", "component": "loan_card", "args": {"loan": "x"}}),
        )
        .unwrap(),
    );
    assert_eq!(
        errors(&doc),
        [
            (
                "widget_recursion",
                "component:loan_card/node:grid/widget:state".to_owned()
            ),
            (
                "widget_recursion",
                "component:state_badge/node:back".to_owned()
            ),
        ]
    );
}

/// A board node inside a widget body takes a board widget by patch, and that widget can be an
/// instance, checked for its args.
#[test]
fn a_board_node_in_a_body_takes_a_widget_instance_by_patch() {
    let mut doc = doc();
    doc.widgets["loan_card"]
        .body
        .push(serde_json::from_value(json!({"name": "grid", "component": "board"})).unwrap());
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
        "widget_args"
    );
}

/// Replacing a widget so that it no longer declares a param an instance passes is refused.
#[test]
fn replacing_a_widget_to_drop_a_param_its_instance_passes_is_refused() {
    let mut doc = doc();
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
    assert_eq!(refused.check, "widget_args");
    assert!(
        refused
            .message
            .starts_with("component:loan_card/node:state"),
        "{}",
        refused.message
    );
}
