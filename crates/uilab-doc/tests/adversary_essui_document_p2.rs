//! Adversary pass 2 on story:essui-document (commit 0d731f9).
//!
//! Each read case holds one document ESS 0.48.0 checks with 0 errors and asserts that uilab reads
//! it and writes it back as authored. Each patch case asserts what a patch on such a document
//! does. Each `expansion_bound` case asserts that a document whose expansion ESS performs past
//! `EXPANSION_LIMIT` is refused by the bound quickly, or that one ESS reads in milliseconds is not.

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use serde_yaml::Value as Yaml;
use uilab_doc::{Child, Document, Layer, NodePath, Patch, admit};

/// ESS's own verdict on a document: its error findings, as `check path: message`.
fn ess_errors(text: &str) -> Vec<String> {
    uilab_doc::ess_ui_check::check_source(text, "case", Path::new("."), None, &Default::default())
        .findings
        .into_iter()
        .filter(|f| f.severity == uilab_doc::ess_ui_check::Severity::Error)
        .map(|f| format!("{} {}: {}", f.check, f.path, f.message))
        .collect()
}

const HEAD: &str = "format: ess-ui/1
app: t
model: t
placement_profile: fat
navigation:
  home: a
  sections: [{name: s, pages: [a]}]
";

const SHELL: &str = "shells:
  app:
    regions:
      main: {kind: page_outlet}
";

/// Asserts ESS reads `text` with 0 errors, then that uilab reads it and writes it back as
/// authored (the same YAML value).
fn uilab_reads_what_ess_reads(text: &str) -> Document {
    assert_eq!(
        ess_errors(text),
        Vec::<String>::new(),
        "precondition: ESS checks the document with 0 errors"
    );
    let doc = Document::from_yaml(text)
        .unwrap_or_else(|e| panic!("uilab refuses an ess-ui/1 document ESS reads clean: {e}"));
    let written: Yaml = serde_yaml::from_str(&doc.to_yaml().unwrap()).unwrap();
    let authored: Yaml = serde_yaml::from_str(text).unwrap();
    assert_eq!(written, authored, "written back as authored");
    doc
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

// ── shapes ESS reads clean ───────────────────────────────────────────────────────────────────

/// `Kinds::merge` (ess-ui `expand.rs`): a page field set to `null` removes what the kind
/// contributes (`null_value: remove_inherited`). A page writes `overlays: null` to drop every
/// overlay its kind brings.
#[test]
fn a_page_that_removes_every_inherited_overlay_is_read() {
    let text = format!(
        "{HEAD}{SHELL}pages:\n  a:\n    kind: list_page\n    title: A\n    sections:\n      - {{name: list, component: collection, reads: x.All}}\n    overlays: null\n"
    );
    uilab_reads_what_ess_reads(&text);
}

/// `shorthands.inheritance.maps: {merge: deep}`: a page refines one board widget its kind
/// contributes by writing only what differs; the widget takes `component` from the kind. This is
/// the board's map of nodes, the one place `item_nodes`' refined-entry rule does not reach.
#[test]
fn a_board_widget_that_refines_its_kinds_widget_is_read() {
    let text = format!(
        "{HEAD}{SHELL}page_kinds:\n  dash:\n    extends: dashboard_page\n    sections:\n      - name: board\n        component: board\n        reads: x.All\n        widgets:\n          k: {{component: metric, from: row.n, label: N}}\npages:\n  a:\n    kind: dash\n    title: A\n    sections:\n      - name: board\n        widgets:\n          k: {{label: Total}}\n"
    );
    uilab_reads_what_ess_reads(&text);
}

/// The `Composite` shorthand of `shorthands.index` (`accepts: {ref: composite_kind}, expands_to:
/// {component: $value}`): where a Node is expected, a bare member name is the composite. In the
/// board's map of nodes the key is the name, so `k: rich_text` is a whole node.
#[test]
fn a_board_widget_written_as_the_composite_shorthand_is_read() {
    let text = format!(
        "{HEAD}{SHELL}pages:\n  a:\n    kind: dashboard_page\n    title: A\n    sections:\n      - name: board\n        component: board\n        reads: x.All\n        widgets:\n          k: rich_text\n"
    );
    uilab_reads_what_ess_reads(&text);
}

/// A shell overlay may be `same_as` a page overlay (`resolve_same_as` walks `pages` and
/// `shells`), with neither `kind` nor `component` of its own.
#[test]
fn a_shell_overlay_same_as_a_page_overlay_is_read() {
    let text = format!(
        "{HEAD}shells:\n  app:\n    regions:\n      main: {{kind: page_outlet}}\n    overlays:\n      e: {{same_as: a.e}}\npages:\n  a:\n    kind: list_page\n    title: A\n    sections:\n      - {{name: list, component: collection, reads: x.All}}\n    overlays:\n      e: {{kind: dialog, component: confirm, does: x.Del}}\n"
    );
    uilab_reads_what_ess_reads(&text);
}

/// A refined `item` entry and a `choices` removal (`{name, remove: true}`, the filter_bar
/// shorthand) on a page whose kind contributes both.
#[test]
fn a_refined_item_and_a_removed_choice_are_read() {
    let text = format!(
        "{HEAD}{SHELL}page_kinds:\n  k:\n    extends: list_page\n    sections:\n      - name: filters\n        component: filter_bar\n        choices: [{{name: c, component: choice, options: [a, b]}}]\n      - name: list\n        component: collection\n        reads: x.All\n        item: [{{name: t, primitive: text, text: hi}}]\npages:\n  a:\n    kind: k\n    title: A\n    sections:\n      - name: filters\n        choices: [{{name: c, remove: true}}]\n      - name: list\n        item: [{{name: t, text: bye}}]\n"
    );
    uilab_reads_what_ess_reads(&text);
}

/// A region's `props` are data (`OPAQUE` in ESS); `.inf` is a YAML number ESS keeps. uilab holds
/// props as JSON, which has no infinity.
#[test]
#[ignore = "props are JSON; .inf/.nan not kept (accepted, essui-document adversary-2)"]
fn a_non_finite_number_in_region_props_is_written_back() {
    let text = format!(
        "{HEAD}shells:\n  app:\n    regions:\n      main: {{kind: page_outlet, props: {{ratio: .inf}}}}\npages:\n  a:\n    kind: list_page\n    title: A\n    sections:\n      - {{name: list, component: collection, reads: x.All}}\n"
    );
    uilab_reads_what_ess_reads(&text);
}

// ── patches on refined entries ──────────────────────────────────────────────────────────────

/// A list page that refines its kind's `list` section without naming `component`.
fn refined_list() -> String {
    format!(
        "{HEAD}{SHELL}pages:\n  a:\n    kind: list_page\n    title: A\n    sections:\n      - {{name: list, columns: [title]}}\n"
    )
}

/// The refined `list` is a `collection` (inherited from `list_page`), and a collection holds
/// `item` (`collection.item: {list: Node}`). Inserting an item there is a document ESS reads
/// clean; uilab must offer and admit it.
#[test]
fn an_item_can_be_inserted_into_a_section_that_refines_its_kinds_collection() {
    let doc = uilab_reads_what_ess_reads(&refined_list());
    let patch = Patch::Insert {
        target: path("page:a/section:list"),
        child: Child {
            layer: Layer::Item,
            name: "t".into(),
            node: serde_json::json!({"primitive": "text", "text": "hi"}),
            nav_section: None,
        },
    };
    let expected = refined_list().replace(
        "{name: list, columns: [title]}",
        "{name: list, columns: [title], item: [{name: t, primitive: text, text: hi}]}",
    );
    assert_eq!(
        ess_errors(&expected),
        Vec::<String>::new(),
        "precondition: the patched document is one ESS checks with 0 errors"
    );
    let (next, _) = admit(&doc, &patch)
        .unwrap_or_else(|r| panic!("uilab refuses an insert whose result ESS reads clean: {r}"));
    let written: Yaml = serde_yaml::from_str(&next.to_yaml().unwrap()).unwrap();
    let want: Yaml = serde_yaml::from_str(&expected).unwrap();
    assert_eq!(written, want);
}

/// Replacing the refined section with another refinement, and inserting a refinement of the
/// `filters` section the kind contributes: each written as the patch says, and ESS reads it clean.
#[test]
fn refinements_inserted_or_replaced_are_written_as_patched_and_read_clean_by_ess() {
    let doc = uilab_reads_what_ess_reads(&refined_list());
    let patch = Patch::Batch {
        target: path("page:a"),
        patches: vec![
            Patch::Replace {
                target: path("page:a/section:list"),
                node: serde_json::json!({"columns": ["title", "author"]}),
            },
            Patch::Insert {
                target: path("page:a"),
                child: Child {
                    layer: Layer::Section,
                    name: "filters".into(),
                    node: serde_json::json!({"search": {"binds": "state.search"}}),
                    nav_section: None,
                },
            },
        ],
    };
    let (next, _) = admit(&doc, &patch).unwrap_or_else(|r| panic!("{r}"));
    let text = next.to_yaml().unwrap();
    assert_eq!(ess_errors(&text), Vec::<String>::new(), "{text}");
    let written: Yaml = serde_yaml::from_str(&text).unwrap();
    let want: Yaml = serde_yaml::from_str(&refined_list().replace(
        "{name: list, columns: [title]}",
        "{name: list, columns: [title, author]}\n      - {name: filters, search: {binds: state.search}}",
    ))
    .unwrap();
    assert_eq!(written, want);
}

// ── expansion_bound ─────────────────────────────────────────────────────────────────────────

/// `widgets:` with a doubling chain `w0..=w<depth>`: `w<i>` uses `w<i-1>` twice, so one use of
/// `w<depth>` is 2^depth leaves plus 2·(2^depth − 1) inner nodes (depth 14: 49 150 nodes).
fn chain(depth: usize) -> String {
    let mut text = String::from(
        "widgets:\n  w0:\n    summary: Leaf.\n    body:\n      - {name: t, primitive: text, text: hi}\n",
    );
    for i in 1..=depth {
        text.push_str(&format!(
            "  w{i}:\n    summary: Level.\n    body:\n      - {{name: a, component: w{p}}}\n      - {{name: b, component: w{p}}}\n",
            p = i - 1
        ));
    }
    text
}

const FOUR_PAGES: &str = "format: ess-ui/1
app: t
model: t
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: p0
  sections: [{name: s, pages: [p0, p1, p2, p3]}]
";

/// Reads `text` on a thread and returns what `from_yaml` said within `secs`, or `None`.
fn read_within(text: String, secs: u64) -> Option<Result<(), uilab_doc::LoadError>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(Document::from_yaml(&text).map(|_| ()));
    });
    rx.recv_timeout(Duration::from_secs(secs)).ok()
}

fn assert_refused_by_the_bound(text: String, what: &str) {
    match read_within(text, 10) {
        Some(Err(refused)) => assert!(
            refused
                .message
                .contains(&uilab_doc::EXPANSION_LIMIT.to_string()),
            "refused, but not by the bound: {refused}"
        ),
        Some(Ok(())) => panic!("admitted: {what}"),
        None => panic!("not refused within 10s: the bound let the document through and {what}"),
    }
}

/// `resolve_same_as` runs before widget expansion (ess-ui `expand.rs`, pass 1 then pass 2): each
/// `same_as` overlay is a copy of its target, and ESS expands the widget in every copy. Three
/// copies of a 49 150-node overlay cost ESS 196 600 nodes (7 s in a release build of the `ess`
/// 0.48.0 CLI, 0 errors); uilab weighs the copies as 0, because their `component` is inherited.
#[test]
fn expansion_bound_counts_a_same_as_overlay_as_the_overlay_it_copies() {
    let mut text = format!("{FOUR_PAGES}{}", chain(14));
    text.push_str("pages:\n  p0:\n    kind: detail_page\n    overlays:\n      e: {kind: drawer, component: w14}\n");
    for p in ["p1", "p2", "p3"] {
        text.push_str(&format!(
            "  {p}:\n    kind: detail_page\n    overlays:\n      e: {{same_as: p0.e}}\n"
        ));
    }
    assert_refused_by_the_bound(
        text,
        "ESS is expanding 4 × 49150 widget nodes through same_as",
    );
}

/// `substitute` (ess-ui `expand.rs`) replaces a whole `args.<param>` string by the bound value,
/// args included, before the body's own widget uses expand. A widget that passes its arg on twice
/// (`args: {p: {x: args.p, y: args.p}}`) doubles the value at every level: 20 levels are a
/// 2^20-leaf value, though each level writes one node. ESS checks the 18-level document with 0
/// errors in 7 s (release CLI), 4× per two more levels; uilab weighs nodes only and counts 21.
#[test]
fn expansion_bound_counts_what_args_substitution_copies() {
    let depth = 20;
    let mut text = format!(
        "{HEAD}{SHELL}widgets:\n  w0:\n    summary: Leaf.\n    params:\n      p: {{type: json, note: data}}\n    body:\n      - {{name: t, primitive: text, text: hi}}\n"
    );
    for i in 1..=depth {
        text.push_str(&format!(
            "  w{i}:\n    summary: Level.\n    params:\n      p: {{type: json, note: data}}\n    body:\n      - {{name: a, component: w{p}, args: {{p: {{x: args.p, y: args.p}}}}}}\n",
            p = i - 1
        ));
    }
    text.push_str(&format!(
        "pages:\n  a:\n    kind: detail_page\n    title: A\n    sections:\n      - {{name: summary, component: w{depth}, args: {{p: 1}}}}\n"
    ));
    assert_refused_by_the_bound(text, "ESS is substituting a 2^20-leaf argument");
}

/// ESS's `widget_uses` walks every key under `shells` and `pages` but its `OPAQUE` data keys,
/// and expands before the model is read. A widget use under a page's `nav` is expanded in full
/// (2 × 2^17 nodes, 45 s in the release CLI) and only then refused. uilab weighs typed
/// composites, page headers and page kinds only, so it counts 0 and hands the document to ESS.
#[test]
fn expansion_bound_counts_a_widget_use_wherever_ess_expands_it() {
    let mut text = format!("{FOUR_PAGES}{}", chain(17));
    text.push_str("pages:\n  p0:\n    kind: detail_page\n    nav: {label: Big, component: w17}\n");
    for p in ["p1", "p2", "p3"] {
        text.push_str(&format!("  {p}:\n    kind: detail_page\n"));
    }
    assert_refused_by_the_bound(
        text,
        "ESS is expanding about 2^18 widget nodes under pages.p0.nav",
    );
}

/// `different_component: page_entry_replaces`: a kind that extends another and replaces its
/// widget section with a record carries no widget use. ESS reads the four-page document in
/// 0.04 s (release CLI) with 0 errors. uilab follows `extends` from the base kind's use and
/// weighs it on every page of the extending kind: 4 × 98 302 nodes, and refuses it.
#[test]
fn expansion_bound_does_not_count_a_widget_section_an_extending_kind_replaces() {
    let mut text = format!("{FOUR_PAGES}{}", chain(15));
    text.push_str("page_kinds:\n  k1:\n    extends: detail_page\n    sections:\n      - {name: big, component: w15}\n  k2:\n    extends: k1\n    sections:\n      - {name: big, component: record}\npages:\n");
    for p in ["p0", "p1", "p2", "p3"] {
        text.push_str(&format!("  {p}:\n    kind: k2\n"));
    }
    assert_eq!(ess_errors(&text), Vec::<String>::new(), "precondition");
    match read_within(text, 30) {
        Some(Ok(())) => {}
        Some(Err(e)) => panic!("uilab refuses a document ESS reads clean in milliseconds: {e}"),
        None => panic!("not read within 30s"),
    }
}
