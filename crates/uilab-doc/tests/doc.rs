use std::path::{Path, PathBuf};

use serde_json::json;
use uilab_doc::model::{Component, Composite, CompositeKind, PrimitiveKind, Widget};
use uilab_doc::path::NodeRef;
use uilab_doc::{
    CHECKS, Child, Document, Fixtures, Layer, NodePath, Patch, Severity, admit, allowed_children,
    check, node_context, outline, patch_schema, resolve,
};

fn widget(yaml: &str) -> Widget {
    serde_yaml::from_str(yaml).unwrap()
}

fn composite(value: serde_json::Value) -> Composite {
    serde_json::from_value(value).unwrap()
}

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn library() -> Document {
    let text = std::fs::read_to_string(examples().join("library/library.ui.yaml")).unwrap();
    Document::from_yaml(&text).unwrap()
}

fn empty() -> Document {
    let text = std::fs::read_to_string(examples().join("empty.ui.yaml")).unwrap();
    Document::from_yaml(&text).unwrap()
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

fn ids(doc: &Document) -> Vec<&'static str> {
    check(doc).iter().map(|f| f.check).collect()
}

#[test]
fn examples_pass_every_check_and_round_trip() {
    for doc in [library(), empty()] {
        assert!(check(&doc).is_empty(), "{:#?}", check(&doc));
        let again = Document::from_yaml(&doc.to_yaml().unwrap()).unwrap();
        assert_eq!(again, doc);
    }
}

#[test]
fn round_trip_keeps_section_order_and_untyped_props() {
    let doc = library();
    let yaml = doc.to_yaml().unwrap();
    let on_loan = yaml.find("on_loan:").unwrap();
    let recent = yaml.find("recent:").unwrap();
    assert!(on_loan < recent, "section order is layout order");
    assert!(yaml.contains("row_actions"), "an untyped prop survives");
    assert!(yaml.contains("size: 5"), "a number is written as a number");
    assert!(
        !yaml.contains("serde_json"),
        "no serde_json internals in the file"
    );
}

#[test]
fn paths_parse_print_and_refuse_misplaced_layers() {
    for text in [
        "/",
        "shell:app/region:nav",
        "nav/nav_section:sales",
        "page:loans/section:list/item:status",
    ] {
        assert_eq!(path(text).to_string(), text);
    }
    assert!("page:loans/page:other".parse::<NodePath>().is_err());
    assert!("shell:app/section:x".parse::<NodePath>().is_err());
    assert!("nav:x".parse::<NodePath>().is_err());
    assert!("page:".parse::<NodePath>().is_err());
}

#[test]
fn a_path_survives_a_sibling_insert() {
    let doc = library();
    let recent = path("page:overview/section:recent");
    let before = format!("{:?}", resolve(&doc, &recent).unwrap());
    let patch = Patch::Insert {
        target: path("page:overview"),
        child: Child {
            layer: Layer::Section,
            name: "alerts".into(),
            node: json!({"component": "record"}),
            nav_section: None,
        },
    };
    let (next, _) = admit(&doc, &patch).unwrap();
    assert_eq!(format!("{:?}", resolve(&next, &recent).unwrap()), before);
    assert!(resolve(&next, &path("page:overview/section:alerts")).is_ok());
}

#[test]
fn patch_schema_offers_only_what_the_node_can_take() {
    let doc = library();
    let layers = |schema: serde_json::Value| -> Vec<String> {
        schema["properties"]["child"]["oneOf"]
            .as_array()
            .map(|v| {
                v.iter()
                    .map(|c| {
                        c["properties"]["layer"]["const"]
                            .as_str()
                            .unwrap()
                            .to_owned()
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let page = patch_schema(&doc, &path("page:loans")).unwrap();
    assert_eq!(layers(page), ["section", "overlay"]);

    let collection = patch_schema(&doc, &path("page:loans/section:list")).unwrap();
    assert_eq!(layers(collection.clone()), ["item"]);

    let metric = patch_schema(&doc, &path("page:overview/section:on_loan")).unwrap();
    assert!(metric["properties"].get("child").is_none());
    assert_eq!(
        metric["properties"]["op"]["enum"],
        json!(["replace", "remove", "batch", "decline"])
    );

    let root = patch_schema(&doc, &NodePath::root()).unwrap();
    assert_eq!(
        root["properties"]["op"]["enum"],
        json!(["insert", "batch", "decline"])
    );
    assert_eq!(layers(root.clone()), ["shell", "page", "component"]);
    let nav = &root["properties"]["child"]["oneOf"][1]["properties"]["nav_section"]["enum"];
    assert_eq!(nav, &json!(["circulation", "people"]));
}

#[test]
fn admit_refuses_by_check_id() {
    let doc = library();
    let refused = |patch: Patch| admit(&doc, &patch).unwrap_err().check;

    assert_eq!(
        refused(Patch::Remove {
            target: path("page:overview")
        }),
        "nav_resolves"
    );
    assert_eq!(
        refused(Patch::Insert {
            target: path("page:loans"),
            child: Child {
                layer: Layer::Section,
                name: "list".into(),
                node: json!({"component": "record"}),
                nav_section: None
            },
        }),
        "name_unique"
    );
    assert_eq!(
        refused(Patch::Insert {
            target: path("page:loans"),
            child: Child {
                layer: Layer::Section,
                name: "extra".into(),
                node: json!({"component": "collection", "row_actions": [{"opens": "missing"}]}),
                nav_section: None,
            },
        }),
        "opens_resolves"
    );
    assert_eq!(
        refused(Patch::Insert {
            target: path("page:loans/section:list"),
            child: Child {
                layer: Layer::Widget,
                name: "w".into(),
                node: json!({"component": "chart"}),
                nav_section: None
            },
        }),
        "layer_allowed"
    );
    assert_eq!(
        refused(Patch::Replace {
            target: path("page:loans/section:list"),
            node: json!({"component": "carousel"})
        }),
        "node_shape"
    );
    assert_eq!(
        refused(Patch::Remove {
            target: path("page:nowhere")
        }),
        "path_resolves"
    );
    assert_eq!(
        refused(Patch::Remove {
            target: NodePath::root()
        }),
        "op_allowed"
    );
}

#[test]
fn admit_accepts_a_page_and_lists_it() {
    let doc = empty();
    let patch = Patch::Insert {
        target: NodePath::root(),
        child: Child {
            layer: Layer::Page,
            name: "books".into(),
            node: json!({"kind": "list_page", "title": "Books", "sections": {"list": {"component": "collection", "reads": {"view": "draft.Books"}}}}),
            nav_section: None,
        },
    };
    let (next, findings) = admit(&doc, &patch).unwrap();
    assert!(next.navigation.hidden.contains(&"books".to_owned()));
    assert_eq!(
        findings
            .iter()
            .map(|f| (f.check, f.severity))
            .collect::<Vec<_>>(),
        [("draft_read", Severity::Warning)]
    );

    let removed = admit(
        &next,
        &Patch::Remove {
            target: path("page:books"),
        },
    )
    .unwrap()
    .0;
    assert_eq!(
        removed, doc,
        "removing the page undoes the insert, menu included"
    );
}

type Breaks = Box<dyn Fn(&mut Document)>;

/// One broken document per check id: every check can fail.
#[test]
fn every_check_fails_on_its_own_fixture() {
    let base = library();
    let broken: Vec<(&str, Breaks)> = vec![
        ("format_marker", Box::new(|d| d.format = "ui-spec/0".into())),
        (
            "nav_resolves",
            Box::new(|d| d.navigation.home = "nowhere".into()),
        ),
        (
            "page_reachable",
            Box::new(|d| {
                d.navigation.sections[1].pages = uilab_doc::model::NavPages::Fixed(vec![])
            }),
        ),
        (
            "nav_unique",
            Box::new(|d| d.navigation.hidden.push("loans".into())),
        ),
        (
            "shell_refs",
            Box::new(|d| d.pages["loans"].shell = Some("print".into())),
        ),
        (
            "page_kind_known",
            Box::new(|d| d.pages["loans"].kind = "wizard_page".into()),
        ),
        (
            "page_outlet",
            Box::new(|d| {
                d.shells["app"].regions.shift_remove("main");
            }),
        ),
        (
            "opens_resolves",
            Box::new(|d| {
                d.pages["loans"].overlays.shift_remove("edit");
            }),
        ),
        (
            "section_refs",
            Box::new(|d| {
                let list = d.pages["loans"].sections["list"].as_mut().unwrap();
                list.props.insert("depends_on".into(), json!("filters"));
            }),
        ),
        (
            "fixture_per_view",
            Box::new(|d| {
                d.fixtures
                    .as_mut()
                    .unwrap()
                    .views
                    .shift_remove("members.All");
            }),
        ),
        (
            "draft_read",
            Box::new(|d| {
                d.pages["members"].sections["list"]
                    .as_mut()
                    .unwrap()
                    .reads
                    .as_mut()
                    .unwrap()
                    .view = "draft.Members".into();
            }),
        ),
        (
            "unmapped_reported",
            Box::new(|d| {
                d.pages["loans"]
                    .extra
                    .insert("note".into(), json!("UNMAPPED: nobody said"));
            }),
        ),
        (
            "widget_named_like_builtin",
            Box::new(|d| {
                d.widgets
                    .insert("chart".into(), widget("{summary: s, body: []}"));
            }),
        ),
        (
            "widget_args",
            Box::new(|d| {
                d.widgets.insert(
                    "w".into(),
                    widget("{summary: s, params: {p: {type: string, required: true}}, body: []}"),
                );
                d.pages["overview"]
                    .sections
                    .insert("card".into(), Some(composite(json!({"component": "w"}))));
            }),
        ),
        (
            "widget_recursion",
            Box::new(|d| {
                d.widgets.insert(
                    "w".into(),
                    widget("{summary: s, body: [{name: again, component: w}]}"),
                );
            }),
        ),
        (
            "widget_resolves",
            Box::new(|d| {
                d.pages["overview"].sections.insert(
                    "card".into(),
                    Some(composite(json!({"component": "nowhere"}))),
                );
            }),
        ),
    ];
    assert_eq!(broken.len(), CHECKS.len());
    for (id, breaks) in &broken {
        assert!(
            CHECKS.iter().any(|(c, _, _)| c == id),
            "{id} is not a declared check"
        );
        let mut doc = base.clone();
        breaks(&mut doc);
        assert!(ids(&doc).contains(id), "{id} did not fire: {:?}", ids(&doc));
    }
}

#[test]
fn fixtures_answer_rows_and_drafts_answer_none() {
    let doc = library();
    let fixtures = Fixtures::load(&doc, &examples().join("library")).unwrap();
    assert_eq!(fixtures.rows("loans.All").rows.len(), 4);
    assert_eq!(fixtures.rows("loans.All").total, Some(4));
    assert_eq!(fixtures.rows("members.All").rows.len(), 3);
    assert!(fixtures.rows("draft.Anything").rows.is_empty());
}

#[test]
fn context_names_what_can_go_here() {
    let doc = library();
    let context = node_context(&doc, &path("page:loans")).unwrap();
    assert_eq!(context.allowed_children, [Layer::Section, Layer::Overlay]);
    assert_eq!(context.composite_kinds.len(), 14);
    assert_eq!(context.children, ["section:list", "overlay:edit"]);
    assert_eq!(context.ancestors, ["/ (document)"]);
    let tree = outline(&doc);
    assert_eq!(
        tree.children
            .iter()
            .map(|c| c.path.as_str())
            .collect::<Vec<_>>(),
        [
            "shell:app",
            "nav",
            "page:overview",
            "page:loans",
            "page:members"
        ]
    );
}

#[test]
fn columns_that_name_no_fixture_field_are_warned() {
    let mut doc = library();
    let fixtures = Fixtures::load(&doc, &examples().join("library")).unwrap();
    assert_eq!(
        fixtures.fields()["loans.All"],
        ["id", "title", "member", "due", "state"]
    );
    assert!(uilab_doc::field_findings(&doc, &fixtures).is_empty());
    let list = doc.pages["members"].sections["list"].as_mut().unwrap();
    list.reads.as_mut().unwrap().view = "loans.All".into();
    let found = uilab_doc::field_findings(&doc, &fixtures);
    let named: Vec<&str> = found
        .iter()
        .map(|f| f.message.split('`').nth(1).unwrap())
        .collect();
    assert_eq!(named, ["name", "joined", "loans", "standing"]);
    assert!(
        found
            .iter()
            .all(|f| f.check == "column_fields" && f.severity == Severity::Warning)
    );
}

#[test]
fn a_batch_admits_what_its_parts_cannot_alone() {
    let doc = library();
    let row_action = Patch::Replace {
        target: path("page:members/section:list"),
        node: json!({"component": "collection", "reads": {"view": "members.All"},
            "columns": [{"field": "name"}], "row_actions": [{"opens": "edit_member", "label": "Edit"}]}),
    };
    assert_eq!(
        admit(&doc, &row_action).unwrap_err().check,
        "opens_resolves"
    );
    let drawer = Patch::Insert {
        target: path("page:members"),
        child: Child {
            layer: Layer::Overlay,
            name: "edit_member".into(),
            node: json!({"kind": "drawer", "component": "form", "fields": ["name"]}),
            nav_section: None,
        },
    };
    let batch = Patch::Batch {
        target: path("page:members/section:list"),
        patches: vec![drawer, row_action],
    };
    let (next, _) = admit(&doc, &batch).unwrap();
    assert!(resolve(&next, &path("page:members/overlay:edit_member")).is_ok());
    assert_eq!(batch.changed_path().to_string(), "page:members");
    assert_eq!(batch.op_name(), "Batch");

    let nested = Patch::Batch {
        target: NodePath::root(),
        patches: vec![batch.clone()],
    };
    assert_eq!(admit(&doc, &nested).unwrap_err().check, "batch_shape");
    let wire: Patch = serde_json::from_value(serde_json::to_value(&batch).unwrap()).unwrap();
    assert_eq!(wire, batch);
}

#[test]
fn draft_views_get_sample_rows_shaped_by_their_readers() {
    let mut doc = library();
    let chart = uilab_doc::model::Composite {
        component: uilab_doc::model::CompositeKind::Chart.into(),
        reads: Some(uilab_doc::model::Reads {
            view: "draft.LoansPerMonth".into(),
            extra: Default::default(),
        }),
        widgets: Default::default(),
        item: Default::default(),
        props: [
            ("x".to_owned(), json!("month")),
            ("series".to_owned(), json!([{"field": "loans"}])),
        ]
        .into_iter()
        .collect(),
    };
    doc.pages["overview"]
        .sections
        .insert("trend".into(), Some(chart));
    let rows = uilab_doc::sample_rows(&doc, "draft.LoansPerMonth");
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0]["month"], json!("2026-05"));
    assert!(rows[0]["loans"].is_number());
    assert_eq!(
        uilab_doc::sample_rows(&doc, "draft.Nothing")[0]["name"],
        json!("name 1")
    );
}

#[test]
fn docs_and_help_describe_the_document_and_every_kind() {
    let doc = library();
    let fixtures = Fixtures::load(&doc, &examples().join("library")).unwrap();
    let docs = uilab_doc::docs_markdown(&doc, &fixtures, &check(&doc));
    for heading in [
        "# Lending library",
        "## Navigation",
        "## Pages",
        "## Components",
        "## Data",
        "## Commands",
        "## Interactions",
        "## State and events",
        "## Findings",
    ] {
        assert!(docs.contains(heading), "missing {heading}");
    }
    assert!(docs.contains("| `loans.All` | fixture | id, title, member, due, state |"));
    assert!(docs.contains("`loans.ExtendLoan`"));
    assert!(docs.contains("- `page:loans/section:list` opens `edit`"));
    let help = uilab_doc::help_markdown();
    for kind in uilab_doc::model::CompositeKind::ALL {
        assert!(help.contains(&format!("**{}**", kind.as_str())));
    }
    assert!(help.contains("| page (a route) | section, overlay |"));
}

#[test]
fn a_patch_that_changes_nothing_is_refused() {
    let doc = library();
    let list = path("page:members/section:list");
    let same =
        serde_json::to_value(doc.pages["members"].sections["list"].as_ref().unwrap()).unwrap();
    assert_eq!(
        admit(
            &doc,
            &Patch::Replace {
                target: list,
                node: same
            }
        )
        .unwrap_err()
        .check,
        "no_change"
    );
}

/// A lending-library document that declares two widgets and uses one in every place a composite
/// can go: a section, a board widget, a collection item, a page overlay and a shell overlay.
const WIDGETS: &str = r#"
format: ui-spec/1
app: library
title: Lending library
model: library
placement_profile: fat
fixtures:
  views:
    loans.All: loans.yaml
    loans.Summary: loans.yaml
shells:
  app:
    regions:
      main: {kind: page_outlet}
    overlays:
      loan: {kind: drawer, component: loan_card, args: {loan: state.selected}}
navigation:
  home: overview
  sections:
    - {name: circulation, label: Circulation, pages: [overview]}
widgets:
  loan_card:
    summary: A loan as a card with its cover, title, state and an extend button.
    doc: Used in lists, on the board and in the loan drawer.
    params:
      loan: {type: Loan, required: true, note: the loan row}
      compact: {type: boolean, default: false, note: hides the cover}
    arrange: column
    body:
      - {name: cover, primitive: image, src: args.loan.cover_url, alt: Book cover, fit: contain}
      - {name: title, primitive: text, text: args.loan.title, style: heading}
      - {name: state, component: state_badge, args: {state: args.loan.state}}
      - {name: extend, primitive: button, label: Extend, action: {name: extend, does: loans.ExtendLoan}}
  state_badge:
    summary: A loan state as a toned badge.
    params:
      state: {type: string, required: true}
    arrange: row
    body:
      - {name: badge, primitive: badge, text: args.state, tone_by: {value: args.state, map: {overdue: danger, out: info}}}
pages:
  overview:
    kind: dashboard_page
    title: Overview
    sections:
      latest:
        component: loan_card
        args: {loan: rows.first}
      board:
        component: board
        reads: {view: loans.Summary}
        widgets:
          featured: {component: loan_card, args: {loan: row, compact: true}}
      list:
        component: collection
        reads: {view: loans.All}
        columns: [{field: title}]
        item:
          card: {component: loan_card, args: {loan: row}}
    overlays:
      detail: {kind: dialog, component: loan_card, args: {loan: state.selected}}
"#;

fn with_widgets() -> Document {
    Document::from_yaml(WIDGETS).unwrap()
}

/// The findings of `doc` as `(check, path)`, errors only.
fn errors(doc: &Document) -> Vec<(&'static str, String)> {
    check(doc)
        .into_iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| (f.check, f.path))
        .collect()
}

#[test]
fn a_document_with_widgets_round_trips() {
    let doc = with_widgets();
    assert!(check(&doc).is_empty(), "{:#?}", check(&doc));
    assert_eq!(doc.widgets.len(), 2);
    let card = &doc.widgets["loan_card"];
    assert_eq!(
        card.summary,
        "A loan as a card with its cover, title, state and an extend button."
    );
    assert_eq!(
        card.params.keys().collect::<Vec<_>>(),
        ["loan", "compact"],
        "params keep their order"
    );
    let param = |name: &str| serde_json::to_value(&card.params[name]).unwrap();
    assert_eq!(param("loan")["required"], json!(true));
    assert!(param("compact").get("required").is_none());
    assert_eq!(card.params["compact"].default, Some(json!(false)));
    assert_eq!(
        card.body
            .iter()
            .map(|n| n.name.as_str())
            .collect::<Vec<_>>(),
        ["cover", "title", "state", "extend"]
    );

    let yaml = doc.to_yaml().unwrap();
    assert!(yaml.contains("widgets:"));
    assert!(yaml.contains("primitive: image"));
    assert!(
        yaml.find("name: cover").unwrap() < yaml.find("name: extend").unwrap(),
        "body order is kept"
    );
    assert_eq!(Document::from_yaml(&yaml).unwrap(), doc);
}

#[test]
fn a_widget_instance_parses_and_resolves_wherever_a_composite_can_go() {
    let doc = with_widgets();
    for at in [
        "page:overview/section:latest",
        "page:overview/section:board/widget:featured",
        "page:overview/section:list/item:card",
        "page:overview/overlay:detail",
        "shell:app/overlay:loan",
    ] {
        let node = resolve(&doc, &path(at)).unwrap_or_else(|e| panic!("{at}: {e}"));
        let composite = node.composite().unwrap();
        assert_eq!(
            composite.component,
            Component::Widget("loan_card".into()),
            "{at}"
        );
        assert_eq!(composite.component.kind(), None, "{at}");
        let name = composite.component.widget().unwrap();
        assert!(
            doc.widgets.contains_key(name),
            "{at} names a declared widget"
        );
    }
    let board = resolve(&doc, &path("page:overview/section:board")).unwrap();
    assert_eq!(
        board.composite().unwrap().component.kind(),
        Some(CompositeKind::Board),
        "a built-in keeps its typed kind"
    );
    assert_eq!(
        board.composite().unwrap().component,
        CompositeKind::Board,
        "a component compares with a kind"
    );
}

#[test]
fn primitives_parse_as_nodes_of_a_widget_body() {
    let every = widget(
        r#"
summary: Every primitive once.
body:
  - {name: t, primitive: text, text: hello, style: caption}
  - {name: b, primitive: badge, text: ok, tone: success}
  - {name: i, primitive: icon, icon: books, label: Books}
  - {name: go, primitive: button, label: Go, action: {name: go, does: loans.ExtendLoan}}
  - {name: l, primitive: link, text: Open, to: {to: overview}}
  - {name: q, primitive: input, as: search, binds: state.q}
  - {name: on, primitive: toggle, label: On, binds: state.on}
  - {name: img, primitive: image, src: args.src, alt: Cover}
  - {name: rule, primitive: divider}
"#,
    );
    let kinds: Vec<PrimitiveKind> = every
        .body
        .iter()
        .map(|n| n.primitive().expect("a primitive node").primitive)
        .collect();
    assert_eq!(kinds, PrimitiveKind::ALL);

    let mut doc = with_widgets();
    doc.widgets.insert("every".into(), every);
    let text = resolve(&doc, &path("component:every/node:t")).unwrap();
    match text {
        NodeRef::Primitive(p) => {
            assert_eq!(p.primitive, PrimitiveKind::Text);
            assert_eq!(p.props["style"], json!("caption"));
        }
        other => panic!("not a primitive: {other:?}"),
    }
    let instance = resolve(&doc, &path("component:loan_card/node:state")).unwrap();
    assert_eq!(
        instance.composite().unwrap().component,
        Component::Widget("state_badge".into())
    );

    for (bad, why) in [
        (
            "{summary: s, body: [{name: x, primitive: text, component: metric}]}",
            "both component and primitive",
        ),
        (
            "{summary: s, body: [{name: x}]}",
            "neither component nor primitive",
        ),
        ("{summary: s, body: [{primitive: text}]}", "no name"),
        (
            "{summary: s, body: [{name: x, primitive: carousel}]}",
            "unknown primitive",
        ),
        (
            "{summary: s, body: [{name: x, primitive: divider}, {name: x, primitive: divider}]}",
            "two nodes with one name",
        ),
        ("{body: []}", "no summary"),
        ("{summary: s}", "no body"),
    ] {
        assert!(
            serde_yaml::from_str::<Widget>(bad).is_err(),
            "{why} is refused"
        );
    }
}

#[test]
fn widget_checks_refuse_at_the_instance_path() {
    let base = with_widgets();
    let latest = "page:overview/section:latest";
    let set_latest = |doc: &mut Document, value: serde_json::Value| {
        doc.pages["overview"]
            .sections
            .insert("latest".into(), Some(composite(value)));
    };

    let mut named = base.clone();
    named
        .widgets
        .insert("chart".into(), widget("{summary: s, body: []}"));
    assert_eq!(
        errors(&named),
        [("widget_named_like_builtin", "component:chart".to_owned())]
    );

    let mut unknown = base.clone();
    set_latest(
        &mut unknown,
        json!({"component": "loan_card", "args": {"loan": "row", "colour": "red"}}),
    );
    let found = check(&unknown);
    assert_eq!(errors(&unknown), [("widget_args", latest.to_owned())]);
    assert!(
        found[0].message.contains("unknown") && found[0].message.contains("`colour`"),
        "{}",
        found[0].message
    );

    let mut missing = base.clone();
    set_latest(&mut missing, json!({"component": "loan_card"}));
    let found = check(&missing);
    assert_eq!(errors(&missing), [("widget_args", latest.to_owned())]);
    assert!(
        found[0].message.contains("missing") && found[0].message.contains("`loan`"),
        "{}",
        found[0].message
    );

    let mut direct = base.clone();
    direct.widgets["state_badge"].body.push(
        serde_json::from_value(
            json!({"name": "again", "component": "state_badge", "args": {"state": "x"}}),
        )
        .unwrap(),
    );
    assert_eq!(
        errors(&direct),
        [(
            "widget_recursion",
            "component:state_badge/node:again".to_owned()
        )]
    );

    let mut indirect = base.clone();
    indirect.widgets["state_badge"].body.push(
        serde_json::from_value(
            json!({"name": "back", "component": "loan_card", "args": {"loan": "x"}}),
        )
        .unwrap(),
    );
    assert_eq!(
        errors(&indirect),
        [
            (
                "widget_recursion",
                "component:loan_card/node:state".to_owned()
            ),
            (
                "widget_recursion",
                "component:state_badge/node:back".to_owned()
            ),
        ]
    );

    let mut unresolved = base.clone();
    set_latest(&mut unresolved, json!({"component": "loan_tile"}));
    assert_eq!(
        errors(&unresolved),
        [("widget_resolves", latest.to_owned())]
    );

    for id in [
        "widget_named_like_builtin",
        "widget_args",
        "widget_recursion",
        "widget_resolves",
    ] {
        let declared = CHECKS.iter().find(|(c, _, _)| *c == id);
        assert_eq!(
            declared.map(|(_, s, _)| *s),
            Some(Severity::Error),
            "{id} is a declared error"
        );
    }
}

#[test]
fn widget_paths_resolve_and_name_their_children() {
    let doc = with_widgets();
    for text in ["component:loan_card", "component:loan_card/node:title"] {
        assert_eq!(path(text).to_string(), text);
        assert!(resolve(&doc, &path(text)).is_ok(), "{text} resolves");
    }
    assert!(matches!(
        resolve(&doc, &path("component:loan_card")).unwrap(),
        NodeRef::Component(w) if w.summary.starts_with("A loan as a card")
    ));
    assert!(resolve(&doc, &path("component:nowhere")).is_err());
    assert!(resolve(&doc, &path("component:loan_card/node:nothing")).is_err());
    for misplaced in [
        "node:title",
        "page:overview/component:loan_card",
        "component:loan_card/section:x",
        "component:loan_card/node:title/node:x",
    ] {
        assert!(
            misplaced.parse::<NodePath>().is_err(),
            "{misplaced} is refused"
        );
    }

    assert!(
        allowed_children(&doc, &NodePath::root())
            .unwrap()
            .contains(&Layer::Component)
    );
    assert_eq!(
        allowed_children(&doc, &path("component:loan_card")).unwrap(),
        [Layer::Node]
    );
    assert!(
        allowed_children(&doc, &path("component:loan_card/node:title"))
            .unwrap()
            .is_empty()
    );
    let context = node_context(&doc, &path("component:loan_card")).unwrap();
    assert_eq!(
        context.children,
        ["node:cover", "node:title", "node:state", "node:extend"]
    );
    assert_eq!(context.ancestors, ["/ (document)"]);
    assert_eq!(context.kind, "widget");
}

#[test]
fn widgets_and_their_nodes_are_patched_by_path() {
    let doc = with_widgets();
    let root = NodePath::root();
    let card = path("component:loan_card");

    let insert_widget = Patch::Insert {
        target: root.clone(),
        child: Child {
            layer: Layer::Component,
            name: "member_line".into(),
            node: json!({"summary": "A member as one line.", "params": {"member": {"type": "Member", "required": true}},
                "arrange": "row", "body": [{"name": "name", "primitive": "text", "text": "args.member.name"}]}),
            nav_section: None,
        },
    };
    let (next, _) = admit(&doc, &insert_widget).unwrap();
    assert!(resolve(&next, &path("component:member_line/node:name")).is_ok());

    let insert_node = Patch::Insert {
        target: card.clone(),
        child: Child {
            layer: Layer::Node,
            name: "rule".into(),
            node: json!({"primitive": "divider"}),
            nav_section: None,
        },
    };
    let (next, _) = admit(&doc, &insert_node).unwrap();
    assert_eq!(
        next.widgets["loan_card"].body.last().unwrap().name,
        "rule",
        "a node is appended to the body"
    );

    let replace_node = Patch::Replace {
        target: path("component:loan_card/node:title"),
        node: json!({"primitive": "text", "text": "args.loan.title", "style": "body"}),
    };
    let (next, _) = admit(&doc, &replace_node).unwrap();
    assert_eq!(
        next.widgets["loan_card"].body[1].primitive().unwrap().props["style"],
        json!("body")
    );
    assert_eq!(next.widgets["loan_card"].body[1].name, "title");

    let remove_node = Patch::Remove {
        target: path("component:loan_card/node:cover"),
    };
    let (next, _) = admit(&doc, &remove_node).unwrap();
    assert_eq!(next.widgets["loan_card"].body.len(), 3);

    let replace_widget = Patch::Replace {
        target: path("component:state_badge"),
        node: json!({"summary": "A loan state as text.", "params": {"state": {"type": "string", "required": true}},
            "body": [{"name": "label", "primitive": "text", "text": "args.state"}]}),
    };
    let (next, _) = admit(&doc, &replace_widget).unwrap();
    assert_eq!(next.widgets["state_badge"].summary, "A loan state as text.");

    let refused = |patch: Patch| admit(&doc, &patch).unwrap_err().check;
    assert_eq!(
        refused(Patch::Remove {
            target: path("component:state_badge")
        }),
        "widget_resolves",
        "a widget still in use cannot be removed"
    );
    assert_eq!(
        refused(Patch::Insert {
            target: path("page:overview"),
            child: Child {
                layer: Layer::Section,
                name: "another".into(),
                node: json!({"component": "loan_card"}),
                nav_section: None,
            },
        }),
        "widget_args"
    );
    assert_eq!(
        refused(Patch::Insert {
            target: card.clone(),
            child: Child {
                layer: Layer::Node,
                name: "title".into(),
                node: json!({"primitive": "text", "text": "again"}),
                nav_section: None,
            },
        }),
        "name_unique"
    );
    assert_eq!(
        refused(Patch::Insert {
            target: path("component:state_badge"),
            child: Child {
                layer: Layer::Node,
                name: "loop".into(),
                node: json!({"component": "loan_card", "args": {"loan": "x"}}),
                nav_section: None,
            },
        }),
        "widget_recursion"
    );
    assert_eq!(
        refused(Patch::Insert {
            target: root.clone(),
            child: Child {
                layer: Layer::Component,
                name: "metric".into(),
                node: json!({"summary": "s", "body": []}),
                nav_section: None,
            },
        }),
        "widget_named_like_builtin"
    );
    assert_eq!(
        refused(Patch::Insert {
            target: card.clone(),
            child: Child {
                layer: Layer::Section,
                name: "x".into(),
                node: json!({"component": "record"}),
                nav_section: None,
            },
        }),
        "layer_allowed"
    );
}

#[test]
fn the_patch_schema_offers_widgets_and_nodes() {
    let doc = with_widgets();
    let layers = |schema: &serde_json::Value| -> Vec<String> {
        schema["properties"]["child"]["oneOf"]
            .as_array()
            .map(|v| {
                v.iter()
                    .map(|c| {
                        c["properties"]["layer"]["const"]
                            .as_str()
                            .unwrap()
                            .to_owned()
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let root = patch_schema(&doc, &NodePath::root()).unwrap();
    assert_eq!(layers(&root), ["shell", "page", "component"]);
    let component = &root["properties"]["child"]["oneOf"][2];
    assert_eq!(
        component["properties"]["node"]["$ref"],
        json!("#/$defs/widget_declaration")
    );

    let card = patch_schema(&doc, &path("component:loan_card")).unwrap();
    assert_eq!(layers(&card), ["node"]);
    assert_eq!(
        card["properties"]["op"]["enum"],
        json!(["insert", "replace", "remove", "batch", "decline"])
    );
    assert_eq!(
        card["properties"]["child"]["oneOf"][0]["properties"]["node"]["$ref"],
        json!("#/$defs/node")
    );

    let title = patch_schema(&doc, &path("component:loan_card/node:title")).unwrap();
    assert!(title["properties"].get("child").is_none());
    assert_eq!(title["properties"]["node"]["$ref"], json!("#/$defs/node"));

    let defs = &root["$defs"];
    let components = defs["composite"]["properties"]["component"]["enum"]
        .as_array()
        .unwrap();
    for name in ["collection", "board", "loan_card", "state_badge"] {
        assert!(
            components.contains(&json!(name)),
            "`component` accepts {name}"
        );
    }
    let primitives = defs["primitive"]["properties"]["primitive"]["enum"]
        .as_array()
        .unwrap();
    assert_eq!(primitives.len(), 9);
    assert!(defs["node"]["oneOf"].is_array());
    assert!(defs["widget_declaration"]["properties"]["body"].is_object());
    let patch_layers = defs["patch"]["properties"]["child"]["properties"]["layer"]["enum"]
        .as_array()
        .unwrap();
    assert!(patch_layers.contains(&json!("component")));
    assert!(patch_layers.contains(&json!("node")));
}

#[test]
fn the_outline_lists_widgets_under_the_root() {
    let doc = with_widgets();
    let tree = outline(&doc);
    assert_eq!(
        tree.children
            .iter()
            .map(|c| c.path.as_str())
            .collect::<Vec<_>>(),
        [
            "shell:app",
            "nav",
            "component:loan_card",
            "component:state_badge",
            "page:overview"
        ]
    );
    let card = &tree.children[2];
    assert_eq!(card.layer, Layer::Component);
    assert_eq!(card.kind, "widget");
    assert_eq!(
        card.title.as_deref(),
        Some("A loan as a card with its cover, title, state and an extend button.")
    );
    assert_eq!(
        card.children
            .iter()
            .map(|c| (c.path.as_str(), c.layer, c.kind.as_str()))
            .collect::<Vec<_>>(),
        [
            ("component:loan_card/node:cover", Layer::Node, "image"),
            ("component:loan_card/node:title", Layer::Node, "text"),
            ("component:loan_card/node:state", Layer::Node, "state_badge"),
            ("component:loan_card/node:extend", Layer::Node, "button"),
        ]
    );
    let section = tree.children[4]
        .children
        .iter()
        .find(|c| c.path == "page:overview/section:latest")
        .unwrap();
    assert_eq!(section.kind, "loan_card");
}

#[test]
fn docs_list_widgets_with_params_and_use_sites_and_help_names_them() {
    let doc = with_widgets();
    let docs = uilab_doc::docs_markdown(&doc, &Fixtures::default(), &check(&doc));
    let start = docs.find("## Widgets").expect("a Widgets section");
    let end = docs[start + 3..]
        .find("\n## ")
        .map_or(docs.len(), |i| start + 3 + i);
    let section = &docs[start..end];
    for expected in [
        "### loan_card",
        "A loan as a card with its cover, title, state and an extend button.",
        "Used in lists, on the board and in the loan drawer.",
        "| param | type | required | default | note |",
        "| `loan` | `Loan` | yes | - | the loan row |",
        "| `compact` | `boolean` | no | `false` | hides the cover |",
        "`page:overview/section:latest`",
        "`page:overview/section:board/widget:featured`",
        "`page:overview/section:list/item:card`",
        "`page:overview/overlay:detail`",
        "`shell:app/overlay:loan`",
        "### state_badge",
        "`component:loan_card/node:state`",
    ] {
        assert!(section.contains(expected), "missing {expected}:\n{section}");
    }

    let plain = uilab_doc::docs_markdown(&library(), &Fixtures::default(), &[]);
    assert!(plain.contains("## Widgets"));

    let help = uilab_doc::help_markdown();
    assert!(help.contains("| component ("), "{help}");
    assert!(help.contains("widget"));
    for kind in PrimitiveKind::ALL {
        assert!(
            help.contains(&format!("**{}**", kind.as_str())),
            "help names primitive {}",
            kind.as_str()
        );
    }
}

/// A param keeps what it declares as written: an explicit `required: false`, an explicit
/// `default: null`, and a key this subset does not type.
#[test]
fn a_param_keeps_what_it_declares_as_written() {
    let text = WIDGETS.replace(
        "      state: {type: string, required: true}\n",
        "      state: {type: string, required: true}\n      tone: {type: string, required: false, default: null, label: Tone}\n",
    );
    let doc = Document::from_yaml(&text).unwrap();
    let yaml = doc.to_yaml().unwrap();
    for kept in ["required: false", "default: null", "label: Tone"] {
        assert!(yaml.contains(kept), "`{kept}` was dropped:\n{yaml}");
    }
    assert_eq!(Document::from_yaml(&yaml).unwrap(), doc);
    assert!(check(&doc).is_empty(), "{:#?}", check(&doc));
}

/// Every Node position `ui-spec/1` has that this subset keeps as untyped props: an instance there
/// is checked like one in a typed position, reported at the composite that holds it.
#[test]
fn widget_instances_in_untyped_node_positions_are_checked() {
    let list = "page:overview/section:list";
    let with_prop = |key: &str, value: serde_json::Value| {
        let mut doc = with_widgets();
        doc.pages["overview"].sections["list"]
            .as_mut()
            .unwrap()
            .props
            .insert(key.into(), value);
        doc
    };
    let missing = json!({"component": "loan_tile"});
    for (key, value) in [
        ("children", json!([{"name": "n", "component": "loan_tile"}])),
        ("parts", json!([{"name": "n", "component": "loan_tile"}])),
        ("choices", json!([{"name": "n", "component": "loan_tile"}])),
        ("metrics", json!([{"name": "n", "component": "loan_tile"}])),
        ("toolbar", json!([{"name": "n", "component": "loan_tile"}])),
        ("expand", missing.clone()),
        (
            "columns",
            json!([{"field": "state", "as": "choice", "choice": missing.clone()}]),
        ),
        ("tabs", json!([{"name": "t", "form": missing.clone()}])),
        (
            "children",
            json!([{"name": "box", "component": "record", "parts": [{"name": "deep", "component": "loan_tile"}]}]),
        ),
    ] {
        let doc = with_prop(key, value.clone());
        assert_eq!(
            errors(&doc),
            [("widget_resolves", list.to_owned())],
            "{key}: {value}"
        );
    }

    let doc = with_prop(
        "children",
        json!([{"name": "badge", "component": "state_badge", "args": {"state": "row.state", "colour": "red"}}]),
    );
    assert_eq!(errors(&doc), [("widget_args", list.to_owned())]);
    let message = &check(&doc)[0].message;
    assert!(
        message.starts_with("`children/badge`: unknown arg `colour`"),
        "{message}"
    );

    let doc = with_prop(
        "toolbar",
        json!([{"name": "badge", "component": "state_badge"}]),
    );
    assert_eq!(errors(&doc), [("widget_args", list.to_owned())]);

    let mut literal = with_widgets();
    literal.pages["overview"].sections.insert(
        "latest".into(),
        Some(composite(
            json!({"component": "loan_card", "args": {"loan": {"component": "loan_tile"}}}),
        )),
    );
    assert!(
        errors(&literal).is_empty(),
        "an args literal is data, not a node"
    );

    let mut recursive = with_widgets();
    recursive.widgets["state_badge"].body.push(
        serde_json::from_value(json!({"name": "box", "component": "record",
            "children": [{"name": "back", "component": "state_badge", "args": {"state": "x"}}]}))
        .unwrap(),
    );
    assert_eq!(
        errors(&recursive),
        [(
            "widget_recursion",
            "component:state_badge/node:box".to_owned()
        )]
    );

    let doc = with_prop(
        "parts",
        json!([{"name": "badge", "component": "state_badge", "args": {"state": "row.state"}}]),
    );
    assert!(errors(&doc).is_empty());
    assert_eq!(
        admit(
            &doc,
            &Patch::Remove {
                target: path("component:state_badge")
            }
        )
        .unwrap_err()
        .check,
        "widget_resolves",
        "a widget used only in an untyped position cannot be removed"
    );
}

/// The agent is offered declared widgets wherever a composite child can go, except the ones that
/// would make the widget it is editing contain itself.
#[test]
fn the_agent_context_offers_widgets_that_do_not_recurse() {
    let doc = with_widgets();
    let kinds = |at: &str| node_context(&doc, &path(at)).unwrap().composite_kinds;
    let widgets = |at: &str| -> Vec<&str> {
        kinds(at)
            .into_iter()
            .filter(|k| doc.widgets.contains_key(*k))
            .collect()
    };
    assert_eq!(kinds("page:overview").len(), 16);
    assert_eq!(widgets("page:overview"), ["loan_card", "state_badge"]);
    assert_eq!(
        widgets("page:overview/section:board"),
        ["loan_card", "state_badge"]
    );
    assert_eq!(widgets("component:loan_card"), ["state_badge"]);
    assert!(
        widgets("component:state_badge").is_empty(),
        "loan_card holds state_badge, so neither can go into state_badge"
    );
    assert!(kinds("page:overview/section:latest").is_empty());
}

/// The Node positions outside composites: a page header's `metrics` and an action's `choice`, and
/// a page kind's `sections` and `header`. Instances there are checked, keep their widget from
/// being removed, and are listed as use sites with their trail.
#[test]
fn widget_instances_in_page_headers_and_page_kinds_are_checked_and_documented() {
    let with_header = |header: serde_json::Value| {
        let mut doc = with_widgets();
        doc.pages["overview"].extra.insert("header".into(), header);
        doc
    };
    let with_kind = |kind: serde_json::Value| {
        let mut doc = with_widgets();
        doc.page_kinds.insert("board_page".into(), kind);
        doc
    };
    let missing = json!({"component": "loan_tile"});
    for (doc, at) in [
        (
            with_header(json!({"metrics": [{"name": "due", "component": "loan_tile"}]})),
            "page:overview",
        ),
        (
            with_header(
                json!({"actions": [{"name": "pick", "as": "choice", "choice": missing.clone()}]}),
            ),
            "page:overview",
        ),
        (
            with_kind(json!({"sections": {"summary": missing.clone()}})),
            "/",
        ),
        (
            with_kind(json!({"header": {"metrics": [{"name": "due", "component": "loan_tile"}]}})),
            "/",
        ),
    ] {
        assert_eq!(errors(&doc), [("widget_resolves", at.to_owned())]);
    }
    let doc = with_header(json!({"metrics": [{"name": "due", "component": "state_badge"}]}));
    assert_eq!(errors(&doc), [("widget_args", "page:overview".to_owned())]);
    assert!(
        check(&doc)[0]
            .message
            .starts_with("`header/metrics/due`: missing required arg `state`"),
        "{}",
        check(&doc)[0].message
    );

    let doc = with_kind(json!({"header": {"metrics": [
        {"name": "due", "component": "state_badge", "args": {"state": "row.state"}}
    ]}}));
    assert!(errors(&doc).is_empty(), "{:?}", errors(&doc));
    assert_eq!(
        admit(
            &doc,
            &Patch::Remove {
                target: path("component:state_badge")
            }
        )
        .unwrap_err()
        .check,
        "widget_resolves",
        "a widget used only by a page kind cannot be removed"
    );
    let docs = uilab_doc::docs_markdown(&doc, &Fixtures::default(), &check(&doc));
    assert!(
        docs.contains("`/` (`page_kinds/board_page/header/metrics/due`)"),
        "{docs}"
    );

    let mut doc = with_header(json!({"metrics": [
        {"name": "due", "component": "state_badge", "args": {"state": "row.state"}}
    ]}));
    doc.pages["overview"].sections["list"]
        .as_mut()
        .unwrap()
        .props
        .insert(
            "children".into(),
            json!([{"name": "badge", "component": "state_badge", "args": {"state": "row.state"}}]),
        );
    let docs = uilab_doc::docs_markdown(&doc, &Fixtures::default(), &check(&doc));
    for site in [
        "`component:loan_card/node:state`",
        "`page:overview/section:list` (`children/badge`)",
        "`page:overview` (`header/metrics/due`)",
    ] {
        assert!(docs.contains(site), "missing use site {site}:\n{docs}");
    }
}
