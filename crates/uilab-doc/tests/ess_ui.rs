//! story:essui-document: uilab reads, patches and writes `ess-ui/1`, and ESS decides.
//!
//! Each test is named after the acceptance line it holds.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::json;
use serde_yaml::Value as Yaml;
use uilab_doc::ess_ui;
use uilab_doc::ess_ui_check;
use uilab_doc::model::{BUILTIN_PAGE_KINDS, Composite, CompositeKind, RegionKind};
use uilab_doc::{
    CHECKS, Child, Document, Fixtures, Layer, NodePath, Patch, Severity, admit, allowed_children,
    check, outline, resolve,
};

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn library_file() -> PathBuf {
    examples().join("library/library.ui.yaml")
}

fn library_text() -> String {
    std::fs::read_to_string(library_file()).unwrap()
}

fn library() -> Document {
    Document::from_yaml_in(&library_text(), &examples().join("library")).unwrap()
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

/// What a patch case does to the authored YAML, applied to get the expected saved file.
type Expect = Box<dyn Fn(&mut Yaml)>;

fn yaml(text: &str) -> Yaml {
    serde_yaml::from_str(text).unwrap()
}

fn section(name: &str, node: serde_json::Value) -> Child {
    Child {
        layer: Layer::Section,
        name: name.into(),
        node,
        nav_section: None,
    }
}

/// A fresh directory for one test, under cargo's per-target temporary directory.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Every node path of the outline, depth first.
fn outline_paths(doc: &Document) -> Vec<String> {
    fn walk(node: &uilab_doc::OutlineNode, out: &mut Vec<String>) {
        out.push(node.path.clone());
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(&outline(doc), &mut out);
    out
}

/// What each path resolves to, as text, so two documents can be compared node by node.
fn node_text(doc: &Document, at: &str) -> String {
    format!("{:?}", resolve(doc, &path(at)).unwrap())
}

#[test]
fn loads_only_ess_ui() {
    let text = library_text();
    assert!(
        text.starts_with("format: ess-ui/1\n"),
        "the example is ess-ui/1"
    );
    Document::from_yaml(&text).expect("the library example loads");

    // The draft marker is refused, and the refusal names the format uilab reads.
    let draft = text.replacen("format: ess-ui/1", "format: ui-spec/1", 1);
    let refused = Document::from_yaml(&draft).unwrap_err();
    assert!(
        refused.to_string().contains("ess-ui/1"),
        "the refusal names ess-ui/1: {refused}"
    );

    // Whatever ESS's loader refuses, uilab refuses with ESS's message and ESS's path.
    let broken = [
        // a section title, which ess-ui/1 does not have
        text.replacen(
            "      - name: list\n        component: collection\n        reads: {view: members.All}",
            "      - name: list\n        component: collection\n        title: Members\n        reads: {view: members.All}",
            1,
        ),
        // the draft's map of sections
        text.replacen(
            "    sections:\n      - name: list\n        component: collection\n        reads: {view: members.All}\n        columns: [{field: name}, {field: joined}, {field: loans}, {field: standing, as: tag}]",
            "    sections:\n      list:\n        component: collection\n        reads: {view: members.All}\n        columns: [{field: name}, {field: joined}, {field: loans}, {field: standing, as: tag}]",
            1,
        ),
        // a board without its read
        text.replacen(
            "      - {name: board, remove: true}\n",
            "      - {name: board, component: board}\n",
            1,
        ),
        draft,
    ];
    for doc in broken {
        assert_ne!(doc, text, "each broken document differs from the example");
        let ess = ess_ui::load_str(&doc).expect_err("ESS's loader refuses it");
        let ours = Document::from_yaml(&doc).expect_err("uilab refuses it");
        assert_eq!(ours.path, ess.path().to_string(), "ESS's path");
        assert_eq!(ours.message, ess.message(), "ESS's message");
    }
}

#[test]
fn sections_are_named_lists() {
    let doc = library();
    let written = yaml(&doc.to_yaml().unwrap());
    for (page, sections) in written["pages"].as_mapping().unwrap() {
        let list = sections["sections"]
            .as_sequence()
            .unwrap_or_else(|| panic!("page {page:?}: sections are written as a list"));
        assert!(!list.is_empty());
        for entry in list {
            assert!(
                entry["name"].as_str().is_some(),
                "every section names itself: {entry:?}"
            );
        }
    }
    let overview: Vec<&str> = doc.pages["overview"]
        .sections
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        overview,
        ["board", "on_loan", "recent"],
        "read in written order"
    );
    assert!(
        doc.pages["overview"].sections["board"].is_none(),
        "`remove: true` is read"
    );

    // Insert, replace, remove and reorder each keep every other node's path.
    let reordered = {
        let page = &written["pages"]["overview"];
        let mut page: serde_json::Value = serde_json::to_value(page).unwrap();
        page["sections"].as_array_mut().unwrap().reverse();
        page
    };
    let patches = [
        (
            Patch::Insert {
                target: path("page:overview"),
                child: section(
                    "late",
                    json!({"component": "collection", "reads": {"view": "loans.All"}}),
                ),
            },
            "page:overview/section:late",
        ),
        (
            Patch::Replace {
                target: path("page:overview/section:recent"),
                node: json!({"component": "collection", "reads": {"view": "loans.All"}, "columns": [{"field": "title"}]}),
            },
            "page:overview/section:recent",
        ),
        (
            Patch::Remove {
                target: path("page:overview/section:on_loan"),
            },
            "page:overview/section:on_loan",
        ),
        (
            Patch::Replace {
                target: path("page:overview"),
                node: reordered,
            },
            "page:overview",
        ),
    ];
    let before = outline_paths(&doc);
    for (patch, changed) in patches {
        let (next, _) = admit(&doc, &patch).unwrap_or_else(|r| panic!("{patch:?}: {r}"));
        for at in &before {
            if at == "/" || at.starts_with(changed) || changed.starts_with(at.as_str()) {
                continue;
            }
            assert_eq!(
                node_text(&next, at),
                node_text(&doc, at),
                "{at} after {patch:?}"
            );
        }
        let saved = yaml(&next.to_yaml().unwrap());
        assert!(saved["pages"]["overview"]["sections"].is_sequence());
    }
    let (reversed, _) = admit(
        &doc,
        &Patch::Replace {
            target: path("page:overview"),
            node: {
                let mut page = serde_json::to_value(&written["pages"]["overview"]).unwrap();
                page["sections"].as_array_mut().unwrap().reverse();
                page
            },
        },
    )
    .unwrap();
    let order: Vec<&str> = reversed.pages["overview"]
        .sections
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(order, ["recent", "on_loan", "board"], "a reorder is kept");
    for at in [
        "page:overview/section:recent",
        "page:overview/section:on_loan",
    ] {
        assert_eq!(
            node_text(&reversed, at),
            node_text(&doc, at),
            "{at} after a reorder"
        );
    }

    // The written YAML keeps the authored key order, also where it is not the usual one.
    let authored = library_text()
        .replacen(
            "      - name: list\n        component: collection\n        reads: {view: members.All}\n        columns: [{field: name}, {field: joined}, {field: loans}, {field: standing, as: tag}]",
            "      - columns: [{field: name}, {field: joined}, {field: loans}, {field: standing, as: tag}]\n        reads: {view: members.All}\n        name: list\n        component: collection",
            1,
        )
        .replacen("model: library\n", "", 1)
        .replacen("app: library\n", "app: library\nmodel: library\n", 1);
    let doc = Document::from_yaml(&authored).unwrap();
    let (next, _) = admit(
        &doc,
        &Patch::Insert {
            target: path("page:members"),
            child: section(
                "recent",
                json!({"component": "collection", "reads": {"view": "members.All"}}),
            ),
        },
    )
    .unwrap();
    let saved = yaml(&next.to_yaml().unwrap());
    let keys = |value: &Yaml| -> Vec<String> {
        value
            .as_mapping()
            .unwrap()
            .keys()
            .map(|k| k.as_str().unwrap().to_owned())
            .collect()
    };
    let original = yaml(&authored);
    assert_eq!(keys(&saved), keys(&original), "root key order");
    assert_eq!(
        keys(&saved["pages"]["members"]["sections"][0]),
        ["columns", "reads", "name", "component"],
        "an authored section keeps its key order"
    );
    assert_eq!(
        keys(&saved["pages"]["members"]["sections"][1]),
        ["name", "component", "reads"],
        "a new section is written name first"
    );
}

/// Removes the section `name` from a page's written list.
fn without_section(doc: &mut Yaml, page: &str, name: &str) {
    let list = doc["pages"][page]["sections"].as_sequence_mut().unwrap();
    list.retain(|s| s["name"].as_str() != Some(name));
}

#[test]
fn saves_authored_only() {
    let authored = yaml(&library_text());
    let doc = library();
    let loans_list = &authored["pages"]["loans"]["sections"][0];

    let card: serde_json::Value = json!({
        "summary": "A loan as a card.",
        "params": {"loan": {"type": "Loan", "required": true, "note": "the loan row"}},
        "body": [{"name": "title", "primitive": "text", "text": "args.loan.title"}]
    });
    let late = json!({"component": "collection", "reads": {"view": "loans.All"}, "columns": [{"field": "title"}]});
    let mut replaced_list: serde_json::Value = serde_json::to_value(loans_list).unwrap();
    replaced_list.as_object_mut().unwrap().remove("name");
    replaced_list["columns"] = json!([{"field": "title"}, {"field": "member"}, {"field": "due"}, {"field": "state", "as": "tag"}, {"field": "id"}]);

    let cases: Vec<(&str, Patch, Expect)> = vec![
        (
            "insert",
            Patch::Insert {
                target: path("page:loans"),
                child: section("late", late.clone()),
            },
            Box::new(|d: &mut Yaml| {
                let mut entry = serde_yaml::to_value(json!({"name": "late"})).unwrap();
                let body = serde_yaml::to_value(json!({"component": "collection", "reads": {"view": "loans.All"}, "columns": [{"field": "title"}]})).unwrap();
                for (k, v) in body.as_mapping().unwrap() {
                    entry.as_mapping_mut().unwrap().insert(k.clone(), v.clone());
                }
                d["pages"]["loans"]["sections"]
                    .as_sequence_mut()
                    .unwrap()
                    .push(entry);
            }),
        ),
        (
            "replace",
            Patch::Replace {
                target: path("page:loans/section:list"),
                node: replaced_list.clone(),
            },
            Box::new(|d: &mut Yaml| {
                d["pages"]["loans"]["sections"][0]["columns"] = yaml(
                    "[{field: title}, {field: member}, {field: due}, {field: state, as: tag}, {field: id}]",
                );
            }),
        ),
        (
            "remove",
            Patch::Remove {
                target: path("page:overview/section:on_loan"),
            },
            Box::new(|d: &mut Yaml| without_section(d, "overview", "on_loan")),
        ),
        (
            "batch",
            Patch::Batch {
                target: NodePath::root(),
                patches: vec![
                    Patch::Insert {
                        target: NodePath::root(),
                        child: Child {
                            layer: Layer::Component,
                            name: "loan_card".into(),
                            node: card.clone(),
                            nav_section: None,
                        },
                    },
                    Patch::Insert {
                        target: path("page:loans/section:list"),
                        child: Child {
                            layer: Layer::Item,
                            name: "card".into(),
                            node: json!({"component": "loan_card", "args": {"loan": "row"}}),
                            nav_section: None,
                        },
                    },
                    Patch::Remove {
                        target: path("page:members/section:list"),
                    },
                    Patch::Insert {
                        target: path("page:members"),
                        child: section(
                            "list",
                            json!({"component": "collection", "reads": {"view": "members.All"}}),
                        ),
                    },
                ],
            },
            Box::new(move |d: &mut Yaml| {
                let widgets = serde_yaml::to_value(json!({"loan_card": {
                    "summary": "A loan as a card.",
                    "params": {"loan": {"type": "Loan", "required": true, "note": "the loan row"}},
                    "body": [{"name": "title", "primitive": "text", "text": "args.loan.title"}]
                }}))
                .unwrap();
                d.as_mapping_mut()
                    .unwrap()
                    .insert("widgets".into(), widgets);
                d["pages"]["loans"]["sections"][0]
                    .as_mapping_mut()
                    .unwrap()
                    .insert(
                        "item".into(),
                        yaml("[{name: card, component: loan_card, args: {loan: row}}]"),
                    );
                d["pages"]["members"]["sections"][0] =
                    yaml("{name: list, component: collection, reads: {view: members.All}}");
            }),
        ),
    ];
    for (kind, patch, edit) in cases {
        let (next, _) = admit(&doc, &patch).unwrap_or_else(|r| panic!("{kind}: {r}"));
        let saved_text = next.to_yaml().unwrap();
        let saved = yaml(&saved_text);
        let mut expected = authored.clone();
        edit(&mut expected);
        assert_eq!(
            saved, expected,
            "{kind}: the saved file is the authored document plus the patch"
        );

        // Nothing a page kind or a widget contributes is written.
        let loans = &saved["pages"]["loans"];
        let names: Vec<&str> = loans["sections"]
            .as_sequence()
            .unwrap()
            .iter()
            .filter_map(|s| s["name"].as_str())
            .collect();
        assert!(
            !names.contains(&"filters"),
            "{kind}: no inherited filters section"
        );
        for key in ["header", "state", "layout", "shell"] {
            assert!(loans.get(key).is_none(), "{kind}: no inherited `{key}`");
        }
        if let Some(item) = loans["sections"][0].get("item") {
            assert!(
                item[0].get("body").is_none(),
                "{kind}: no widget body under an instance"
            );
        }
        // And ESS reads what uilab wrote.
        ess_ui::load_str(&saved_text).unwrap_or_else(|e| panic!("{kind}: {e}"));
    }
}

#[test]
fn fixtures_index() {
    let doc = library();
    let fixtures = Fixtures::load(&doc, &examples().join("library")).unwrap();
    let file: Yaml =
        yaml(&std::fs::read_to_string(examples().join("library/fixtures/loans.yaml")).unwrap());
    let rows: Vec<serde_json::Value> =
        serde_json::from_value(serde_json::to_value(&file["views"]["loans.All"]["rows"]).unwrap())
            .unwrap();
    assert_eq!(
        fixtures.rows("loans.All").rows,
        rows,
        "rows come from the file the index names"
    );
    assert!(fixtures.has("members.All") && fixtures.has("staff.Me"));
    let index = std::fs::read_to_string(examples().join("library/fixtures/index.yaml")).unwrap();
    assert!(
        index.contains("loans.All: loans.yaml"),
        "the index lists the views"
    );

    // An index that names another file is read from that file.
    let dir = scratch("fixtures_index");
    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(
        dir.join("data/index.yaml"),
        "views:\n  loans.All: elsewhere.yaml\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("data/elsewhere.yaml"),
        "view: loans.All\nrows:\n  - {title: Elsewhere}\n",
    )
    .unwrap();
    let text = library_text().replacen(
        "fixtures: {dir: fixtures, index: fixtures/index.yaml}",
        "fixtures: {dir: data, index: data/index.yaml}",
        1,
    );
    let moved = Document::from_yaml_in(&text, &dir).unwrap();
    let fixtures = Fixtures::load(&moved, &dir).unwrap();
    assert_eq!(
        fixtures.rows("loans.All").rows,
        [json!({"title": "Elsewhere"})]
    );

    // The draft's document-level `views:` is refused by ESS's loader, and uilab relays it.
    let inline = format!(
        "{}views:\n  loans.All:\n    rows: [{{title: Inline}}]\n",
        library_text()
    );
    let ess = ess_ui::load_str(&inline).expect_err("ESS refuses document-level views");
    let ours = Document::from_yaml(&inline).expect_err("uilab refuses it");
    assert_eq!(
        (ours.path, ours.message),
        (ess.path().to_string(), ess.message().to_owned())
    );

    // `fixtures.views`: uilab's verdict is ESS's loader's, whatever that is.
    let mapped = library_text().replacen(
        "fixtures: {dir: fixtures, index: fixtures/index.yaml}",
        "fixtures: {dir: fixtures, views: {loans.All: loans.yaml}}",
        1,
    );
    match (ess_ui::load_str(&mapped), Document::from_yaml(&mapped)) {
        (Err(ess), Err(ours)) => {
            assert_eq!(
                (ours.path, ours.message),
                (ess.path().to_string(), ess.message().to_owned())
            )
        }
        (Ok(_), Ok(doc)) => {
            let fixtures = Fixtures::load(&doc, &examples().join("library")).unwrap();
            assert_eq!(fixtures.rows("loans.All").rows, rows);
        }
        (ess, ours) => panic!("ESS {:?}, uilab {:?}", ess.is_ok(), ours.is_ok()),
    }
}

#[test]
fn placeholder_reads() {
    let dir = scratch("placeholder_reads");
    std::fs::create_dir_all(dir.join("fixtures")).unwrap();
    for name in ["index.yaml", "loans.yaml", "members.yaml", "staff.yaml"] {
        std::fs::copy(
            examples().join("library/fixtures").join(name),
            dir.join("fixtures").join(name),
        )
        .unwrap();
    }
    std::fs::write(
        dir.join("fixtures/late.yaml"),
        "views:\n  loans.Late:\n    total: 1\n    rows:\n      - {title: Middlemarch, days: 3}\n",
    )
    .unwrap();
    let text = library_text().replacen(
        "    sections:\n      - name: list\n        component: collection\n        reads: {view: members.All}",
        "    sections:\n      - name: late\n        component: collection\n        reads: {placeholder: loans.Late, fixture: fixtures/late.yaml}\n        columns: [{field: title}, {field: days}]\n      - name: soon\n        component: collection\n        reads: {placeholder: loans.Soon, fixture: fixtures/soon.yaml}\n        columns: [{field: title}, {field: due}]\n      - name: list\n        component: collection\n        reads: {view: members.All}",
        1,
    );
    let doc = Document::from_yaml_in(&text, &dir).unwrap();
    let late = doc.pages["members"].sections["late"].as_ref().unwrap();
    let reads = late.reads.as_ref().unwrap();
    assert_eq!(
        (
            reads.view.as_deref(),
            reads.placeholder.as_deref(),
            reads.fixture.as_deref()
        ),
        (None, Some("loans.Late"), Some("fixtures/late.yaml"))
    );
    assert_eq!(reads.name(), "loans.Late");

    let fixtures = Fixtures::load(&doc, &dir).unwrap();
    let found = fixtures.rows_for(&doc, "loans.Late");
    assert!(
        !found.sample,
        "a placeholder whose fixture exists is answered by it"
    );
    assert_eq!(found.rows, [json!({"title": "Middlemarch", "days": 3})]);
    assert_eq!(found.total, Some(1));

    let made_up = fixtures.rows_for(&doc, "loans.Soon");
    assert!(
        made_up.sample,
        "a placeholder without its file gets sample rows, marked"
    );
    assert!(!made_up.rows.is_empty());
    for row in &made_up.rows {
        assert!(
            row.get("title").is_some() && row.get("due").is_some(),
            "{row}"
        );
    }
    let real = fixtures.rows_for(&doc, "loans.All");
    assert!(!real.sample && real.rows.len() == 4);

    // ESS reports the placeholders, at their uilab nodes.
    let findings = check(&doc);
    for at in ["page:members/section:late", "page:members/section:soon"] {
        assert!(
            findings.iter().any(|f| f.check == "unbound_placeholder"
                && f.path == at
                && f.severity == Severity::Warning),
            "{at}: {findings:#?}"
        );
    }

    // No `draft.` view is left in uilab's document or server code.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut hits = Vec::new();
    for dir in ["crates/uilab-doc/src", "crates/uilab-app/src"] {
        let mut stack = vec![root.join(dir)];
        while let Some(at) = stack.pop() {
            for entry in std::fs::read_dir(&at).unwrap() {
                let file = entry.unwrap().path();
                if file.is_dir() {
                    stack.push(file);
                } else {
                    let text = std::fs::read_to_string(&file).unwrap_or_default();
                    for (n, line) in text.lines().enumerate() {
                        if line.contains("\"draft.") {
                            hits.push(format!("{}:{}: {line}", file.display(), n + 1));
                        }
                    }
                }
            }
        }
    }
    assert!(hits.is_empty(), "draft views left: {hits:#?}");
}

/// The library with one more section on the loans page.
fn admit_section(
    doc: &Document,
    name: &str,
    node: serde_json::Value,
) -> Result<(), uilab_doc::Refusal> {
    admit(
        doc,
        &Patch::Insert {
            target: path("page:loans"),
            child: section(name, node),
        },
    )
    .map(|_| ())
}

#[test]
fn ess_decides_admission() {
    let doc = library();
    let classes: Vec<(&str, &str, &str, Result<(), uilab_doc::Refusal>)> = vec![
        (
            "a section naming a widget that does not exist",
            "widget_expands",
            "page:loans/section:intro",
            admit_section(&doc, "intro", json!({"component": "page_intro"})),
        ),
        (
            "a widget param without note",
            "document_loads",
            "component:tag",
            admit(
                &doc,
                &Patch::Insert {
                    target: NodePath::root(),
                    child: Child {
                        layer: Layer::Component,
                        name: "tag".into(),
                        node: json!({"summary": "A tag.", "params": {"text": {"type": "string"}}, "body": [{"name": "t", "primitive": "text", "text": "args.text"}]}),
                        nav_section: None,
                    },
                },
            )
            .map(|_| ()),
        ),
        (
            "a board without reads",
            "document_loads",
            "page:loans/section:board",
            admit_section(&doc, "board", json!({"component": "board"})),
        ),
        (
            "a filter_bar with reads",
            "document_loads",
            "page:loans/section:find",
            admit_section(
                &doc,
                "find",
                json!({"component": "filter_bar", "binds": ["state.search"], "reads": {"view": "loans.All"}}),
            ),
        ),
        (
            "a form without does",
            "document_loads",
            "page:loans/section:extend",
            admit_section(&doc, "extend", json!({"component": "form", "fields": ["due"]})),
        ),
        (
            "a collection with title",
            "document_loads",
            "page:loans/section:list",
            admit(
                &doc,
                &Patch::Replace {
                    target: path("page:loans/section:list"),
                    node: json!({"component": "collection", "title": "Loans", "reads": {"view": "loans.All"}}),
                },
            )
            .map(|_| ()),
        ),
        (
            "a navigation entry naming a page that does not exist",
            "nav_resolves",
            "nav/nav_section:circulation",
            admit(
                &doc,
                &Patch::Replace {
                    target: path("nav/nav_section:circulation"),
                    node: json!({"label": "Circulation", "icon": "books", "pages": ["overview", "loans", "ghost"]}),
                },
            )
            .map(|_| ()),
        ),
    ];
    for (class, id, node, result) in classes {
        let refusal = result.expect_err(class);
        assert_eq!(refusal.check, id, "{class}: {refusal}");
        assert!(
            ess_ui_check::CHECKS.iter().any(|c| c.id == refusal.check),
            "{class}: `{}` is an ESS check",
            refusal.check
        );
        assert!(
            refusal.message.starts_with(&format!("{node}: ")),
            "{class}: the refusal names the uilab node `{node}`: {}",
            refusal.message
        );
    }

    // A patch that adds no new ESS error is admitted.
    admit_section(
        &doc,
        "late",
        json!({"component": "collection", "reads": {"view": "loans.All"}, "columns": ["title", "due"]}),
    )
    .expect("a valid section is admitted");
}

/// The keys of a schema construct whose fields hold named nodes (a list or a map of `Node`), each
/// with the uilab layer that addresses them.
fn node_lists(schema: &Yaml, construct: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let fields = schema["constructs"][construct]["fields"]
        .as_mapping()
        .unwrap();
    for (key, spec) in fields {
        let key = key.as_str().unwrap();
        let ty = &spec["type"];
        let element = ty
            .get("list")
            .or_else(|| ty.get("map").and_then(|m| m.get("value")))
            .map(|v| v.get("optional").unwrap_or(v))
            .and_then(Yaml::as_str);
        let layer = match (construct, key, element) {
            ("WidgetInstance", "body", _) => None, // written by expansion, never by an author
            ("Document", "shells", Some("Shell")) => Some("shell"),
            ("Document", "pages", Some("Page")) => Some("page"),
            ("Document", "widgets", Some("Widget")) => Some("component"),
            ("Shell", "regions", Some("Region")) => Some("region"),
            (_, "overlays", Some("overlay")) => Some("overlay"),
            ("Navigation", "sections", Some("NavSection")) => Some("nav_section"),
            (_, "sections", Some("Section")) => Some("section"),
            ("Widget", "body", Some("Node")) => Some("node"),
            (_, "item", Some("Node")) => Some("item"),
            (_, "widgets", Some("Node")) => Some("widget"),
            (_, "children", Some("Node")) => Some("child"),
            (_, "parts", Some("Node")) => Some("part"),
            (_, "choices", Some("Node")) => Some("choice"),
            (_, "toolbar", Some("Node")) => Some("tool"),
            (
                _,
                _,
                Some(
                    "Node" | "Section" | "overlay" | "Shell" | "Page" | "Region" | "Widget"
                    | "NavSection",
                ),
            ) => {
                panic!("{construct}.{key} holds nodes uilab has no layer for")
            }
            _ => None,
        };
        out.extend(layer.map(str::to_owned));
    }
    out
}

fn offered(doc: &Document, at: &str) -> BTreeSet<String> {
    allowed_children(doc, &path(at))
        .unwrap_or_else(|e| panic!("{at}: {e}"))
        .into_iter()
        .map(|l| l.as_str().to_owned())
        .collect()
}

#[test]
fn children_from_ess() {
    let schema = yaml(ess_ui::SCHEMA);
    let strings = |value: &Yaml| -> Vec<String> {
        value
            .as_sequence()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect()
    };

    // The kinds uilab knows are the schema's.
    let members = strings(&schema["constructs"]["Composite"]["union"]["members"]);
    let ours: Vec<String> = CompositeKind::ALL
        .iter()
        .map(|k| k.as_str().to_owned())
        .collect();
    assert_eq!(ours, members, "composite kinds");
    let builtins: Vec<String> = schema["constructs"]["PageKind"]["builtins"]
        .as_mapping()
        .unwrap()
        .keys()
        .map(|k| k.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(BUILTIN_PAGE_KINDS.to_vec(), builtins, "page kinds");
    let regions = strings(&schema["constructs"]["Region"]["fields"]["kind"]["type"]["enum"]);
    let ours: Vec<String> = RegionKind::ALL
        .iter()
        .map(|k| k.as_str().to_owned())
        .collect();
    assert_eq!(ours, regions, "region kinds");

    let mut doc = library();
    assert_eq!(
        offered(&doc, "/"),
        node_lists(&schema, "Document"),
        "the root"
    );
    assert_eq!(
        offered(&doc, "shell:app"),
        node_lists(&schema, "Shell"),
        "a shell"
    );
    assert_eq!(
        offered(&doc, "nav"),
        node_lists(&schema, "Navigation"),
        "the navigation"
    );

    // Every composite kind: as a section, as an overlay, and nested as an item.
    let section_lists = node_lists(&schema, "Section");
    for kind in &members {
        let expected = node_lists(&schema, kind);
        let composite: Composite = serde_json::from_value(json!({"component": kind})).unwrap();
        let page = doc.pages.get_mut("loans").unwrap();
        page.sections
            .insert(format!("as_{kind}"), Some(composite.clone()));
        let list = page.sections.get_mut("list").unwrap().as_mut().unwrap();
        list.item.retain(|n| n.name != format!("as_{kind}"));
        list.item.push(
            uilab_doc::model::Node::named(&format!("as_{kind}"), &json!({"component": kind}))
                .unwrap(),
        );
        doc.pages.get_mut("loans").unwrap().overlays.insert(
            format!("as_{kind}"),
            Some(serde_json::from_value(json!({"kind": "drawer", "component": kind})).unwrap()),
        );

        let mut as_section = expected.clone();
        as_section.extend(section_lists.iter().cloned());
        assert_eq!(
            offered(&doc, &format!("page:loans/section:as_{kind}")),
            as_section,
            "section {kind}"
        );
        assert_eq!(
            offered(&doc, &format!("page:loans/overlay:as_{kind}")),
            expected,
            "overlay {kind}"
        );
        assert_eq!(
            offered(&doc, &format!("page:loans/section:list/item:as_{kind}")),
            expected,
            "item {kind}"
        );
    }

    // A widget instance holds nothing an author writes; a widget declaration holds its body.
    let widget = json!({"summary": "s", "body": [{"name": "t", "primitive": "text", "text": "x"}]});
    doc.widgets
        .insert("card".into(), serde_json::from_value(widget).unwrap());
    doc.pages.get_mut("loans").unwrap().sections.insert(
        "with_card".into(),
        Some(serde_json::from_value(json!({"component": "card"})).unwrap()),
    );
    let mut instance = node_lists(&schema, "WidgetInstance");
    instance.extend(section_lists.iter().cloned());
    assert_eq!(
        offered(&doc, "page:loans/section:with_card"),
        instance,
        "a section that is an instance"
    );
    assert_eq!(
        offered(&doc, "component:card"),
        node_lists(&schema, "Widget")
    );
    assert!(
        offered(&doc, "component:card/node:t").is_empty(),
        "a primitive holds nothing"
    );

    // Every page kind.
    for kind in &builtins {
        let page = serde_json::from_value(json!({"kind": kind, "sections": []})).unwrap();
        doc.pages.insert(format!("as_{kind}"), page);
        assert_eq!(
            offered(&doc, &format!("page:as_{kind}")),
            node_lists(&schema, "Page"),
            "page kind {kind}"
        );
    }

    // Every region kind.
    for kind in &regions {
        let region = serde_json::from_value(json!({"kind": kind})).unwrap();
        doc.shells
            .get_mut("app")
            .unwrap()
            .regions
            .insert(format!("as_{kind}"), region);
        assert_eq!(
            offered(&doc, &format!("shell:app/region:as_{kind}")),
            node_lists(&schema, "Region"),
            "region kind {kind}"
        );
    }
    assert!(offered(&doc, "nav/nav_section:circulation").is_empty());
}

#[test]
fn paths_round_trip() {
    use uilab_doc::ess::{from_ess, node_at, to_ess};

    // Every layer, written out.
    for text in [
        "/",
        "shell:app",
        "shell:app/region:nav",
        "shell:app/overlay:help",
        "nav",
        "nav/nav_section:circulation",
        "page:loans.detail",
        "page:loans/section:list",
        "page:loans/section:list/item:card",
        "page:loans/section:list/item:card/item:inner",
        "page:loans/section:board/widget:metric/item:x",
        "page:loans/section:list/child:hint",
        "page:loans/section:form/part:help",
        "page:loans/section:filters/choice:state",
        "page:loans/section:graph/tool:zoom",
        "page:loans/overlay:edit/item:x",
        "component:loan_card",
        "component:loan_card/node:title",
        "component:loan_card/node:box/item:x",
    ] {
        let ours = path(text);
        let ess = to_ess(&ours);
        assert_eq!(from_ess(&ess), Some(ours.clone()), "{text} → {ess} → back");
    }
    assert_eq!(
        to_ess(&path("page:loans/section:list/item:card")),
        "pages/loans/sections/list/item/card"
    );
    assert_eq!(
        to_ess(&path("component:loan_card/node:title")),
        "widgets/loan_card/body/title"
    );
    assert_eq!(
        to_ess(&path("nav/nav_section:people")),
        "navigation/sections/people"
    );

    for file in [
        examples().join("library/library.ui.yaml"),
        examples().join("empty.ui.yaml"),
    ] {
        let text = std::fs::read_to_string(&file).unwrap();
        let doc = Document::from_yaml_in(&text, file.parent().unwrap()).unwrap();
        let ess_doc = ess_ui::load_str(&text).unwrap();
        let ess_paths: BTreeSet<String> =
            ess_doc.nodes().iter().map(|l| l.path.to_string()).collect();

        // Every uilab node maps to its ESS canonical path and back, and ESS has that node.
        for at in outline_paths(&doc) {
            let ours = path(&at);
            let ess = to_ess(&ours);
            assert_eq!(
                from_ess(&ess),
                Some(ours.clone()),
                "{}: {at}",
                file.display()
            );
            assert_eq!(node_at(&doc, &ess), ours, "{}: {at}", file.display());
            if !matches!(ours.layer(), Layer::Root | Layer::Nav) {
                assert!(
                    ess_paths.contains(&ess),
                    "{}: ESS has no node {ess}",
                    file.display()
                );
            }
        }
        // Every ESS node, inherited ones included, maps to an existing uilab node.
        for ess in &ess_paths {
            let ours = node_at(&doc, ess);
            assert!(resolve(&doc, &ours).is_ok(), "{ess} → {ours}");
        }
        // Every ESS finding maps to an existing uilab node, on the example and on broken copies.
        let mut texts = vec![text.clone()];
        texts.push(text.replacen("home:", "home: ghost\n  # was:", 1));
        for variant in texts {
            let report = ess_ui_check::check_source(
                &variant,
                "doc",
                file.parent().unwrap(),
                None,
                &Default::default(),
            );
            if variant == text {
                assert_eq!(
                    (report.errors(), report.warnings()),
                    (0, 0),
                    "{}: {:#?}",
                    file.display(),
                    report.findings
                );
            }
            for finding in &report.findings {
                let ours = node_at(&doc, &finding.path);
                assert!(resolve(&doc, &ours).is_ok(), "{} → {ours}", finding.path);
            }
        }
    }
}

/// One document per uilab check that fails that check and nothing else.
const SURVIVOR_CASES: [(&str, &str); 17] = [
    ("format_marker", "format: ui-spec/1\n{rest}"),
    (
        "nav_resolves",
        "{head}{shells}navigation:\n  home: ghost\n  hidden: [a]\n  sections: []\n{page}",
    ),
    (
        "page_reachable",
        "{head}{shells}{nav}{page}  b:\n    kind: list_page\n    sections:\n      - {name: list, component: collection}\n",
    ),
    (
        "nav_unique",
        "{head}{shells}navigation:\n  home: a\n  sections: [{name: s, pages: [a]}]\n  hidden: [a]\n{page}",
    ),
    (
        "shell_refs",
        "{head}{shells}{nav}pages:\n  a:\n    kind: list_page\n    shell: nope\n    sections:\n      - {name: list, component: collection}\n",
    ),
    (
        "page_kind_known",
        "{head}{shells}{nav}pages:\n  a:\n    kind: nope_page\n    sections:\n      - {name: list, component: collection}\n",
    ),
    (
        "page_outlet",
        "{head}shells:\n  app:\n    regions:\n      nav: {kind: navigation}\n{nav}{page}",
    ),
    (
        "opens_resolves",
        "{head}{shells}{nav}pages:\n  a:\n    kind: list_page\n    sections:\n      - {name: list, component: collection, row_actions: [{opens: ghost, label: Edit}]}\n",
    ),
    (
        "section_refs",
        "{head}{shells}{nav}pages:\n  a:\n    kind: list_page\n    sections:\n      - {name: list, component: collection, depends_on: ghost}\n",
    ),
    (
        "fixture_per_view",
        "{head}{shells}{nav}pages:\n  a:\n    kind: list_page\n    sections:\n      - {name: list, component: collection, reads: {view: x.All}}\n",
    ),
    (
        "draft_read",
        "{head}{shells}{nav}pages:\n  a:\n    kind: list_page\n    sections:\n      - {name: list, component: collection, reads: {placeholder: x.All, fixture: x.yaml}}\n",
    ),
    (
        "unmapped_reported",
        "{head}{shells}{nav}pages:\n  a:\n    kind: list_page\n    title: 'UNMAPPED: the title is computed'\n    sections:\n      - {name: list, component: collection}\n",
    ),
    (
        "widget_named_like_builtin",
        "{head}{shells}{nav}widgets:\n  metric:\n    summary: A metric.\n    body:\n      - {name: t, primitive: text, text: hi}\n{page}",
    ),
    (
        "widget_args",
        "{head}{shells}{nav}widgets:\n  tag:\n    summary: A tag.\n    params:\n      text: {type: string, required: true, note: the text}\n    body:\n      - {name: t, primitive: text, text: args.text}\n{page}      - {name: t, component: tag, args: {colour: red}}\n",
    ),
    (
        "widget_recursion",
        "{head}{shells}{nav}widgets:\n  loop:\n    summary: Contains itself.\n    body:\n      - {name: again, component: loop}\n{page}",
    ),
    (
        "widget_resolves",
        "{head}{shells}{nav}pages:\n  a:\n    kind: list_page\n    sections:\n      - {name: list, component: collection, item: [{name: card, component: ghost_card}]}\n",
    ),
    (
        "names_unique",
        "{head}{shells}{nav}pages:\n  a:\n    kind: list_page\n    sections:\n      - name: list\n        component: collection\n        item:\n          - {name: t, primitive: text, text: one}\n          - {name: t, primitive: text, text: two}\n",
    ),
];

/// The 18 checks `check.rs` ran before this story.
const BEFORE: [(&str, Severity); 18] = [
    ("format_marker", Severity::Error),
    ("nav_resolves", Severity::Error),
    ("page_reachable", Severity::Error),
    ("nav_unique", Severity::Error),
    ("shell_refs", Severity::Error),
    ("page_kind_known", Severity::Error),
    ("page_outlet", Severity::Error),
    ("opens_resolves", Severity::Error),
    ("section_refs", Severity::Error),
    ("fixture_per_view", Severity::Warning),
    ("draft_read", Severity::Warning),
    ("unmapped_reported", Severity::Warning),
    ("widget_named_like_builtin", Severity::Error),
    ("widget_args", Severity::Error),
    ("widget_recursion", Severity::Error),
    ("widget_resolves", Severity::Error),
    ("names_unique", Severity::Error),
    ("replace_drops", Severity::Warning),
];

fn survivor_doc(template: &str) -> String {
    let head = "format: ess-ui/1\napp: t\nmodel: t\nplacement_profile: fat\n";
    let shells = "shells:\n  app:\n    regions:\n      main: {kind: page_outlet}\n";
    let nav = "navigation:\n  home: a\n  sections: [{name: s, pages: [a]}]\n";
    let page = "pages:\n  a:\n    kind: list_page\n    sections:\n      - {name: list, component: collection}\n";
    let rest = format!("app: t\nmodel: t\nplacement_profile: fat\n{shells}{nav}{page}");
    template
        .replace("{rest}", &rest)
        .replace("{head}", head)
        .replace("{shells}", shells)
        .replace("{nav}", nav)
        .replace("{page}", page)
}

/// Whether ESS reports `document` failing: a finding at least as severe as the uilab check's own.
fn ess_covers(document: &str, severity: Severity) -> bool {
    let report =
        ess_ui_check::check_source(document, "case", Path::new("."), None, &Default::default());
    match severity {
        Severity::Error => report.errors() > 0,
        Severity::Warning => !report.findings.is_empty(),
    }
}

#[test]
fn uilab_checks_survivors() {
    let own: Vec<&str> = CHECKS.iter().map(|(id, _, _)| *id).collect();
    let minimal = survivor_doc("{head}{shells}{nav}{page}");
    assert!(
        !ess_covers(&minimal, Severity::Warning),
        "the base document is clean in ESS"
    );
    let base = Document::from_yaml(&minimal).unwrap();
    assert!(
        check(&base).is_empty(),
        "and clean in uilab: {:#?}",
        check(&base)
    );

    let mut survivors = Vec::new();
    for (id, severity) in BEFORE {
        if id == "replace_drops" {
            continue;
        }
        let template = SURVIVOR_CASES.iter().find(|(c, _)| *c == id).unwrap().1;
        let document = survivor_doc(template);
        if ess_covers(&document, severity) {
            assert!(
                !own.contains(&id),
                "`{id}`: ESS reports it, so uilab no longer runs it"
            );
        } else {
            assert!(
                own.contains(&id),
                "`{id}`: ESS does not report it, so uilab still runs it"
            );
            let doc = Document::from_yaml(&document).unwrap();
            let ids: Vec<&str> = uilab_doc::check::own(&doc)
                .iter()
                .map(|f| f.check)
                .collect();
            assert_eq!(
                ids,
                [id],
                "`{id}`: the document fails that check and nothing else"
            );
            survivors.push(id);
        }
    }

    // `replace_drops` is a rule about a patch: a replace that drops a column. ESS sees a valid
    // document after it, so the rule survives, reported by `admit`.
    let doc = library();
    let patch = Patch::Replace {
        target: path("page:loans/section:list"),
        node: json!({"component": "collection", "reads": {"view": "loans.All", "paging": "server"}, "columns": [{"field": "title"}], "row_actions": [{"opens": "edit", "label": "Extend"}]}),
    };
    let (next, findings) = admit(&doc, &patch).unwrap();
    let report = uilab_doc::ess::report(&next);
    assert!(
        report.findings.is_empty(),
        "ESS sees a clean document: {:#?}",
        report.findings
    );
    assert!(own.contains(&"replace_drops"));
    assert!(
        findings
            .iter()
            .any(|f| f.check == "replace_drops" && f.path == "page:loans/section:list"),
        "{findings:#?}"
    );
    survivors.push("replace_drops");

    assert_eq!(
        own, survivors,
        "CHECKS lists exactly the survivors, in order"
    );
    assert_eq!(BEFORE.len(), SURVIVOR_CASES.len() + 1);
}

/// ESS 0.48.0's own reference example, `examples/partner-portal/ui.yaml` with its `fixtures/`,
/// copied unchanged into `tests/fixtures/partner-portal/` (invented data, `example.com` only).
/// uilab reads it with no error, every node ESS's loader addresses lands on a uilab node, and a
/// save without edits writes it back YAML-equivalent: the same YAML value as the file and the
/// same document to ESS's loader (not the same bytes: flow and block style are not kept).
#[test]
fn ess_reference_example_loads() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/partner-portal");
    let text = std::fs::read_to_string(dir.join("ui.yaml")).unwrap();
    let ess = ess_ui::load_str(&text).expect("ESS loads its own example");
    let report = ess_ui_check::check_source(&text, "ui.yaml", &dir, None, &Default::default());
    assert_eq!(report.errors(), 0, "{:#?}", report.findings);

    let doc = Document::from_yaml_in(&text, &dir).unwrap_or_else(|e| panic!("{e}"));
    let missing: Vec<String> = ess
        .nodes()
        .iter()
        .map(|l| l.path.to_string())
        .filter(|at| resolve(&doc, &uilab_doc::ess::node_at(&doc, at)).is_err())
        .collect();
    assert!(
        missing.is_empty(),
        "ESS nodes with no uilab node: {missing:#?}"
    );
    assert!(
        check(&doc).iter().all(|f| f.severity != Severity::Error),
        "{:#?}",
        check(&doc)
    );

    let written = doc.to_yaml().unwrap();
    assert_eq!(yaml(&written), yaml(&text), "written back as authored");
    assert_eq!(
        ess_ui::load_str(&written).unwrap(),
        ess,
        "the same document to ESS"
    );
    Fixtures::load(&doc, &dir).expect("its fixtures are read");
}
