//! Adversary pass 2 for story:canvas-shows-labels. Pass 1's fix (a582f42) reproduces ESS's first
//! loading step on the document JSON (`outline.rs` `merged_document`: each page merged over its
//! page kind, each `same_as` overlay over the one it names) so every node the browser is shown
//! carries the props ESS renders it with. Held to ESS's own loader (`ess_ui::load_str` and
//! `Document::nodes`) on documents `ess ui check` (0.48.0) passes, that merge agreed with ESS on
//! every text of every node for kinds extending kinds, named-list order, `{name, remove: true}`,
//! `null` removal of overlays and titles, a different component replacing, maps merged deep and
//! `same_as` chains. What is left is a `null` ESS accepts and uilab's own model refuses before the
//! merge runs (a page removing one of its kind's board widgets), and a `{name, remove: true}` in a
//! nested named list (a filter bar's `choices`, a collection's `item`), which ESS applies and the
//! browser's tree shows as a node of its own. ESS's own example (`examples/partner-portal`,
//! `users.list` removes the kind's `tags` choice) writes the second. For every section, overlay
//! and nested node ESS renders, and only those, the texts ESS gives it (an overlay's title, the
//! labels of its columns, fields, tabs, groups, options and actions, a metric's caption, a
//! primitive's text) must be the texts the browser's node carries.

use std::collections::BTreeMap;

use serde_json::Value;
use uilab_doc::outline::rendered;
use uilab_doc::{Document, Layer, OutlineNode, ess_ui};

/// A kind's board section with two widgets; the page removes one with `null` (ESS's
/// `inheritance.maps.null_value: remove_inherited`) and relabels the other.
/// `ess ui check` (0.48.0): 0 errors (2 warnings, missing fixtures only).
const WALL: &str = r#"format: ess-ui/1
app: library
title: Lending library
model: library
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: wall
  sections:
    - {name: desk, label: Desk, pages: [wall]}
page_kinds:
  wall_page:
    extends: dashboard_page
    sections:
      - name: board
        component: board
        reads: {view: walls.Mine}
        widgets:
          out: {component: metric, reads: {view: loans.Summary}, label: Copies out}
          late: {component: metric, reads: {view: loans.Summary}, label: Late copies}
pages:
  wall:
    kind: wall_page
    title: Wall
    sections:
      - name: board
        widgets:
          late: null
          out: {label: Copies on loan}
"#;

/// A kind's filter bar with two choices and a collection with two item nodes; the page removes
/// one of each by name (`inheritance.named_lists.remove`).
/// `ess ui check` (0.48.0): 0 errors (1 warning, a missing fixture).
const SHELF: &str = r#"format: ess-ui/1
app: library
title: Lending library
model: library
placement_profile: fat
types:
  LoanState: {enum: [out, returned, overdue]}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: front
  sections:
    - {name: desk, label: Desk, pages: [front]}
page_kinds:
  shelf_page:
    extends: list_page
    state:
      loan_state: {type: {optional: LoanState}, class: page_state, store: url}
      branch: {type: {optional: string}, class: page_state, store: url}
    sections:
      - name: filters
        binds: [state.search, state.loan_state, state.branch]
        choices:
          - {name: state, component: choice, binds: state.loan_state, options: LoanState}
          - {name: branch, component: choice, binds: state.branch, options: [Main, East]}
      - name: list
        component: collection
        columns: [title, due]
        item:
          - {name: cover, primitive: image, src: row.cover, alt: Cover}
          - {name: lend, primitive: button, label: Lend, action: {does: loans.Lend}}
pages:
  front:
    kind: shelf_page
    title: Front desk
    sections:
      - name: filters
        choices:
          - {name: branch, remove: true}
      - name: list
        reads: {view: loans.All}
        item:
          - {name: lend, remove: true}
"#;

// ── what ESS renders ─────────────────────────────────────────────────────────────────────────

fn put(out: &mut BTreeMap<String, String>, key: String, text: Option<&str>) {
    if let Some(text) = text {
        out.insert(key, text.to_owned());
    }
}

fn ess_fields(out: &mut BTreeMap<String, String>, key: &str, fields: &[ess_ui::Field]) {
    for (i, f) in fields.iter().enumerate() {
        put(out, format!("{key}[{i}]"), f.label.as_deref());
    }
}

fn ess_actions(out: &mut BTreeMap<String, String>, key: &str, actions: &[ess_ui::Action]) {
    for (i, a) in actions.iter().enumerate() {
        put(out, format!("{key}[{i}]"), a.label.as_deref());
    }
}

/// Every text ESS renders a body with, keyed by where it sits.
fn ess_texts(body: &ess_ui::Body) -> BTreeMap<String, String> {
    use ess_ui::Composite as C;
    use ess_ui::Primitive as P;
    let mut out = BTreeMap::new();
    match body {
        ess_ui::Body::Composite(C::Collection(c)) => {
            match &c.columns {
                Some(ess_ui::Columns::Fixed(fields)) => ess_fields(&mut out, "columns", fields),
                Some(ess_ui::Columns::Selectable(s)) => ess_fields(&mut out, "columns.all", &s.all),
                _ => {}
            }
            ess_actions(&mut out, "row_actions", &c.row_actions);
            ess_actions(&mut out, "bulk_actions", &c.bulk_actions);
            ess_actions(&mut out, "actions", &c.actions);
        }
        ess_ui::Body::Composite(C::Record(r)) => {
            ess_fields(&mut out, "fields", &r.fields);
            for (i, t) in r.tabs.iter().enumerate() {
                put(&mut out, format!("tabs[{i}]"), t.label.as_deref());
            }
            ess_actions(&mut out, "actions", &r.actions);
        }
        ess_ui::Body::Composite(C::Form(f)) => {
            ess_fields(&mut out, "fields", &f.fields);
            for (i, g) in f.groups.iter().enumerate() {
                put(&mut out, format!("groups[{i}]"), g.label.as_deref());
                ess_fields(&mut out, &format!("groups[{i}].fields"), &g.fields);
            }
            for (i, t) in f.tabs.iter().enumerate() {
                put(&mut out, format!("tabs[{i}]"), t.label.as_deref());
            }
            ess_actions(&mut out, "actions", &f.actions);
            put(
                &mut out,
                "submit.label".into(),
                f.submit.as_ref().and_then(|s| s.label.as_deref()),
            );
        }
        ess_ui::Body::Composite(C::Choice(c)) => {
            for (i, o) in c.options.iter().enumerate() {
                put(&mut out, format!("options[{i}]"), Some(&o.label));
            }
        }
        ess_ui::Body::Composite(C::FilterBar(f)) => {
            put(
                &mut out,
                "search.placeholder".into(),
                f.search.as_ref().and_then(|s| s.placeholder.as_deref()),
            );
            ess_fields(&mut out, "inputs", &f.inputs);
            ess_actions(&mut out, "actions", &f.actions);
        }
        ess_ui::Body::Composite(C::Confirm(c)) => {
            put(&mut out, "body".into(), c.body.as_deref());
            put(&mut out, "confirm_label".into(), c.confirm_label.as_deref());
            put(
                &mut out,
                "input.label".into(),
                c.input.as_ref().map(|i| i.label.as_str()),
            );
            ess_actions(&mut out, "alternatives", &c.alternatives);
        }
        ess_ui::Body::Composite(C::Metric(m)) => put(&mut out, "label".into(), m.label.as_deref()),
        ess_ui::Body::Composite(C::References(r)) => ess_fields(&mut out, "columns", &r.columns),
        ess_ui::Body::Composite(C::GraphEditor(g)) => {
            ess_actions(&mut out, "node_actions", &g.node_actions);
            ess_actions(&mut out, "edge_actions", &g.edge_actions);
        }
        ess_ui::Body::Composite(C::Board(b)) => {
            ess_actions(&mut out, "item_actions", &b.item_actions)
        }
        ess_ui::Body::Primitive(P::Button(b)) => put(&mut out, "label".into(), Some(&b.label)),
        ess_ui::Body::Primitive(P::Toggle(t)) => put(&mut out, "label".into(), Some(&t.label)),
        ess_ui::Body::Primitive(P::Icon(i)) => put(&mut out, "label".into(), Some(&i.label)),
        ess_ui::Body::Primitive(P::Input(i)) => {
            put(&mut out, "placeholder".into(), i.placeholder.as_deref())
        }
        ess_ui::Body::Primitive(P::Text(t)) => put(
            &mut out,
            "text".into(),
            t.text.as_ref().map(|e| e.0.as_str()),
        ),
        ess_ui::Body::Primitive(P::Badge(b)) => put(
            &mut out,
            "text".into(),
            b.text.as_ref().map(|e| e.0.as_str()),
        ),
        ess_ui::Body::Primitive(P::Link(l)) => put(&mut out, "text".into(), Some(&l.text.0)),
        ess_ui::Body::Primitive(P::Image(i)) => put(&mut out, "alt".into(), Some(&i.alt)),
        _ => {}
    }
    out
}

/// The uilab path of an ESS canonical path under `pages/` or `shells/`, when every step names a
/// node uilab addresses.
fn uilab_path(segments: &[String]) -> Option<String> {
    let mut steps = Vec::new();
    for step in segments.chunks(2) {
        let [key, name] = step else { return None };
        let layer = match key.as_str() {
            "pages" => "page",
            "shells" => "shell",
            "sections" => "section",
            "overlays" => "overlay",
            "item" => "item",
            "children" => "child",
            "parts" => "part",
            "choices" => "choice",
            "toolbar" => "tool",
            "widgets" => "widget",
            "body" => "node",
            _ => return None,
        };
        steps.push(format!("{layer}:{name}"));
    }
    Some(steps.join("/"))
}

/// Every section, overlay and nested node ESS renders, by uilab path, with its texts; an
/// overlay's title under `title`.
fn ess_nodes(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let ess = ess_ui::load_str(text).expect("ESS loads the document");
    let mut out = BTreeMap::new();
    for located in ess.nodes() {
        let segments = located.path.segments();
        if !matches!(
            segments.first().map(String::as_str),
            Some("pages" | "shells")
        ) {
            continue;
        }
        let Some(path) = uilab_path(segments) else {
            continue;
        };
        let texts = match located.node {
            ess_ui::NodeRef::Section(s) => ess_texts(&s.body),
            ess_ui::NodeRef::Node(n) => ess_texts(&n.body),
            ess_ui::NodeRef::Overlay(o) => {
                let mut texts = ess_texts(&o.body);
                put(&mut texts, "title".into(), o.title.as_deref());
                texts
            }
            _ => continue,
        };
        out.insert(path, texts);
    }
    out
}

// ── what the browser is shown ────────────────────────────────────────────────────────────────

fn json_labels(out: &mut BTreeMap<String, String>, key: &str, list: Option<&Value>) {
    for (i, entry) in list
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        put(
            out,
            format!("{key}[{i}]"),
            entry.get("label").and_then(Value::as_str),
        );
    }
}

fn at<'v>(value: &'v Value, keys: &[&str]) -> Option<&'v str> {
    let mut v = value;
    for k in keys {
        v = v.get(*k)?;
    }
    v.as_str()
}

/// The same texts read from a node of `outline::rendered`: its kind (an overlay's last word) and
/// its props, as the canvas reads them.
fn shown_texts(node: &OutlineNode) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let props = node.props.clone().unwrap_or(Value::Null);
    let kind = node.kind.rsplit(' ').next().unwrap_or_default();
    let p = &props;
    match kind {
        "collection" => {
            match p.get("columns") {
                Some(Value::Object(columns)) => {
                    json_labels(&mut out, "columns.all", columns.get("all"))
                }
                other => json_labels(&mut out, "columns", other),
            }
            json_labels(&mut out, "row_actions", p.get("row_actions"));
            json_labels(&mut out, "bulk_actions", p.get("bulk_actions"));
            json_labels(&mut out, "actions", p.get("actions"));
        }
        "record" => {
            json_labels(&mut out, "fields", p.get("fields"));
            json_labels(&mut out, "tabs", p.get("tabs"));
            json_labels(&mut out, "actions", p.get("actions"));
        }
        "form" => {
            json_labels(&mut out, "fields", p.get("fields"));
            json_labels(&mut out, "groups", p.get("groups"));
            for (i, g) in p
                .get("groups")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
            {
                json_labels(&mut out, &format!("groups[{i}].fields"), g.get("fields"));
            }
            json_labels(&mut out, "tabs", p.get("tabs"));
            json_labels(&mut out, "actions", p.get("actions"));
            put(&mut out, "submit.label".into(), at(p, &["submit", "label"]));
        }
        "choice" => json_labels(&mut out, "options", p.get("options")),
        "filter_bar" => {
            put(
                &mut out,
                "search.placeholder".into(),
                at(p, &["search", "placeholder"]),
            );
            json_labels(&mut out, "inputs", p.get("inputs"));
            json_labels(&mut out, "actions", p.get("actions"));
        }
        "confirm" => {
            put(&mut out, "body".into(), at(p, &["body"]));
            put(&mut out, "confirm_label".into(), at(p, &["confirm_label"]));
            put(&mut out, "input.label".into(), at(p, &["input", "label"]));
            json_labels(&mut out, "alternatives", p.get("alternatives"));
        }
        "metric" | "button" | "toggle" | "icon" => put(&mut out, "label".into(), at(p, &["label"])),
        "references" => json_labels(&mut out, "columns", p.get("columns")),
        "graph_editor" => {
            json_labels(&mut out, "node_actions", p.get("node_actions"));
            json_labels(&mut out, "edge_actions", p.get("edge_actions"));
        }
        "board" => json_labels(&mut out, "item_actions", p.get("item_actions")),
        "input" => put(&mut out, "placeholder".into(), at(p, &["placeholder"])),
        "text" | "badge" | "link" => put(&mut out, "text".into(), at(p, &["text"])),
        "image" => put(&mut out, "alt".into(), at(p, &["alt"])),
        _ => {}
    }
    if node.layer == Layer::Overlay {
        put(&mut out, "title".into(), node.title.as_deref());
    }
    out
}

/// Every node of `rendered` under a page or a shell that is not a page, shell or region, by path.
fn shown_nodes(root: &OutlineNode) -> BTreeMap<String, &OutlineNode> {
    fn walk<'o>(node: &'o OutlineNode, out: &mut BTreeMap<String, &'o OutlineNode>) {
        if !matches!(
            node.layer,
            Layer::Root
                | Layer::Page
                | Layer::Shell
                | Layer::Region
                | Layer::Nav
                | Layer::NavSection
                | Layer::Component
        ) {
            out.insert(node.path.trim_matches('/').to_owned(), node);
        }
        if node.layer != Layer::Component {
            node.children.iter().for_each(|c| walk(c, out));
        }
    }
    let mut out = BTreeMap::new();
    walk(root, &mut out);
    out
}

/// Where the browser's tree and ESS disagree on `text`: a node one has and the other has not, and
/// every text that differs.
fn disagreements(text: &str) -> Vec<String> {
    let doc = Document::from_yaml(text).expect("uilab reads the document");
    let root = rendered(&doc);
    let shown = shown_nodes(&root);
    let ess = ess_nodes(text);
    let mut out = Vec::new();
    for (path, texts) in &ess {
        match shown.get(path) {
            None => out.push(format!(
                "{path}: ESS renders it, the browser is not shown it"
            )),
            Some(node) => {
                let got = shown_texts(node);
                if &got != texts {
                    out.push(format!(
                        "{path}: ESS renders {texts:?}, the browser is shown {got:?}"
                    ));
                }
            }
        }
    }
    for path in shown.keys() {
        if !ess.contains_key(path) {
            out.push(format!(
                "{path}: the browser is shown it, ESS renders no such node"
            ));
        }
    }
    out
}

// ── cases ────────────────────────────────────────────────────────────────────────────────────

/// A page removes one of its kind's board widgets with `null`, which ESS's loader accepts; uilab
/// must open the document and show the board as ESS renders it, the removed widget absent.
#[test]
fn a_board_widget_the_page_removes_with_null_is_opened_and_not_shown() {
    assert!(ess_ui::load_str(WALL).is_ok(), "ESS loads the document");
    let read = Document::from_yaml(WALL);
    assert!(
        read.is_ok(),
        "uilab refuses a document ESS loads: {:?}",
        read.err()
    );
    let found = disagreements(WALL);
    assert!(
        found.is_empty(),
        "{} disagreement(s):\n{}",
        found.len(),
        found.join("\n")
    );
}

/// A page removes an entry of a nested named list its kind gives (a filter bar's choice, a
/// collection's item node) with `{name, remove: true}`. ESS renders neither; the browser must not
/// be shown them either, as it is not shown a section the page removes.
#[test]
fn an_entry_a_page_removes_from_a_nested_list_is_not_shown() {
    let ess = ess_nodes(SHELF);
    let removed = [
        "page:front/section:filters/choice:branch",
        "page:front/section:list/item:lend",
    ];
    for path in removed {
        assert!(!ess.contains_key(path), "ESS renders {path}");
    }
    let doc = Document::from_yaml(SHELF).expect("uilab reads the document");
    let root = rendered(&doc);
    let shown = shown_nodes(&root);
    let ghosts: Vec<String> = removed
        .iter()
        .filter_map(|p| {
            shown
                .get(*p)
                .map(|n| format!("{p} (kind {}, props {:?})", n.kind, n.props))
        })
        .collect();
    assert!(
        ghosts.is_empty(),
        "the browser is shown what the page removes: {ghosts:?}"
    );
    let found = disagreements(SHELF);
    assert!(
        found.is_empty(),
        "{} disagreement(s):\n{}",
        found.len(),
        found.join("\n")
    );
}
