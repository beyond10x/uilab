use std::path::{Path, PathBuf};

use serde_json::json;
use uilab_doc::{
    CHECKS, Child, Document, Fixtures, Layer, NodePath, Patch, Severity, admit, check,
    node_context, outline, patch_schema, resolve,
};

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
        json!(["replace", "remove", "batch"])
    );

    let root = patch_schema(&doc, &NodePath::root()).unwrap();
    assert_eq!(root["properties"]["op"]["enum"], json!(["insert", "batch"]));
    assert_eq!(layers(root.clone()), ["shell", "page"]);
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
        component: uilab_doc::model::CompositeKind::Chart,
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
