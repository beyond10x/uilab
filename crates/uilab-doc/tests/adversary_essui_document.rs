//! Adversary pass 1 on story:essui-document.
//!
//! Each case holds one document ESS 0.48.0 checks with 0 errors, or one bound uilab claims to
//! keep, and asserts what uilab does with it.

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use serde_yaml::Value as Yaml;
use uilab_doc::{Document, Layer, NodePath, Severity, check, outline, resolve};

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
shells:
  app:
    regions:
      main: {kind: page_outlet}
";

/// Asserts ESS reads `text` clean, then that uilab reads it and writes it back as authored.
fn uilab_reads_what_ess_reads(text: &str) {
    assert_eq!(
        ess_errors(text),
        Vec::<String>::new(),
        "precondition: ESS checks the document with 0 errors"
    );
    ess_ui_loads(text);
    let doc = Document::from_yaml(text)
        .unwrap_or_else(|e| panic!("uilab refuses an ess-ui/1 document ESS reads clean: {e}"));
    let written: Yaml = serde_yaml::from_str(&doc.to_yaml().unwrap()).unwrap();
    let authored: Yaml = serde_yaml::from_str(text).unwrap();
    assert_eq!(written, authored, "written back as authored");
}

fn ess_ui_loads(text: &str) {
    uilab_doc::ess_ui::load_str(text).unwrap_or_else(|e| panic!("precondition: ESS loads it: {e}"));
}

/// The ess-ui/1 inheritance rule: a page names a section its kind contributes and writes only
/// what differs (`shorthands.inheritance.named_lists`, "Start from a page kind and declare only
/// what differs"). The merged section takes `component` from the kind.
#[test]
fn a_page_section_that_overrides_its_kinds_section_is_read() {
    let text = format!(
        "{HEAD}navigation:\n  home: a\n  sections: [{{name: s, pages: [a]}}]\npages:\n  a:\n    kind: list_page\n    sections:\n      - {{name: list, columns: [title]}}\n"
    );
    uilab_reads_what_ess_reads(&text);
}

/// `overlay.same_as` reuses another overlay; `component` comes from it (`merge_under`).
#[test]
fn an_overlay_with_same_as_is_read() {
    let text = format!(
        "{HEAD}navigation:\n  home: a\n  sections: [{{name: s, pages: [a, b]}}]\npages:\n  a:\n    kind: list_page\n    sections:\n      - {{name: list, component: collection}}\n    overlays:\n      edit: {{kind: drawer, component: form, does: x.Edit}}\n  b:\n    kind: list_page\n    sections:\n      - {{name: list, component: collection}}\n    overlays:\n      edit: {{kind: drawer, same_as: a.edit}}\n"
    );
    uilab_reads_what_ess_reads(&text);
}

/// `board.widgets` is `{map: {key: name, value: Node}}`, and a Node may be a primitive.
#[test]
fn a_board_widget_that_is_a_primitive_is_read() {
    let text = format!(
        "{HEAD}navigation:\n  home: a\n  sections: [{{name: s, pages: [a]}}]\npages:\n  a:\n    kind: dashboard_page\n    sections:\n      - name: board\n        component: board\n        reads: {{view: x.All}}\n        widgets:\n          hint: {{primitive: text, text: hi}}\n"
    );
    uilab_reads_what_ess_reads(&text);
}

/// The `Reads` shorthand of `shorthands.index`: `reads: <view>` is `reads: {view: <view>}`.
#[test]
fn the_reads_shorthand_is_read() {
    let text = format!(
        "{HEAD}navigation:\n  home: a\n  sections: [{{name: s, pages: [a]}}]\npages:\n  a:\n    kind: list_page\n    sections:\n      - {{name: list, component: collection, reads: x.All}}\n"
    );
    uilab_reads_what_ess_reads(&text);
}

/// A page that takes every section from its kind writes no `sections`; the save writes nothing
/// the author did not (`saves_authored_only`).
#[test]
fn a_page_that_inherits_every_section_is_written_as_authored() {
    let text = format!(
        "{HEAD}navigation:\n  home: a\n  sections: [{{name: s, pages: [a]}}]\npages:\n  a:\n    kind: list_page\n    title: Loans\n"
    );
    uilab_reads_what_ess_reads(&text);
}

/// `Page.shell` defaults to `first_present: [shells.app, only_shell]`, not to the first shell.
/// A page without `shell` in a document whose first shell is a sign-in frame renders in `app`;
/// ESS reports nothing, so uilab's surviving `page_outlet` check must not either.
#[test]
fn a_page_without_shell_renders_in_shells_app_not_the_first_shell() {
    let text = "format: ess-ui/1
app: t
model: t
placement_profile: fat
shells:
  auth:
    regions:
      nav: {kind: navigation}
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: a
  sections: [{name: s, pages: [a]}]
pages:
  a:
    kind: detail_page
    sections:
      - {name: summary, component: record}
";
    assert_eq!(ess_errors(text), Vec::<String>::new(), "precondition");
    let doc = Document::from_yaml(text).unwrap();
    let errors: Vec<_> = check(&doc)
        .into_iter()
        .filter(|f| f.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "uilab reports an error ESS does not, on a shell the page does not render in: {errors:#?}"
    );
}

/// ESS merges a page kind into every page of that kind and expands its widget uses there, once
/// per page (`expand.rs`: page kinds first, then widget uses under `pages`). uilab weighs a use
/// in `page_kinds` once. Two pages of a kind whose section expands to 98 302 nodes cost ESS
/// 196 604, past `EXPANSION_LIMIT`; uilab counts 98 302 and hands the document to ESS.
#[test]
fn expansion_bound_counts_a_page_kinds_widget_once_per_page() {
    let pages = 2;
    let text = page_kind_chain(15, pages);
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(Document::from_yaml(&text).map(|_| ()));
    });
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(Err(refused)) => assert!(
            refused
                .message
                .contains(&uilab_doc::EXPANSION_LIMIT.to_string()),
            "refused, but not by the bound: {refused}"
        ),
        Ok(Ok(())) => panic!(
            "admitted: ESS expanded {pages} × 98302 nodes, past the {} uilab hands to ESS",
            uilab_doc::EXPANSION_LIMIT
        ),
        Err(_) => panic!(
            "not refused within 10s: the bound let the document through and ESS is expanding \
             {pages} × 98302 nodes"
        ),
    }
}

/// Evidence for the case above, run on its own (`--ignored`): without a deadline the same
/// document is read, so the bound did let it through. Slow by construction.
#[test]
#[ignore = "probe: runs ESS's expansion to the end"]
fn expansion_probe_the_two_page_document_is_read_in_the_end() {
    let started = std::time::Instant::now();
    let read = Document::from_yaml(&page_kind_chain(15, 2));
    eprintln!("read in {:?}: {:?}", started.elapsed(), read.as_ref().err());
    assert!(read.is_ok());
}

/// Probe, run on its own (`--ignored`) with `ESS_UI_EXAMPLE` naming ESS 0.48.0's
/// `examples/partner-portal/ui.yaml`: uilab reads ESS's own reference example.
#[test]
#[ignore = "probe: needs ESS_UI_EXAMPLE"]
fn probe_uilab_reads_ess_reference_example() {
    let file = std::env::var("ESS_UI_EXAMPLE").expect("ESS_UI_EXAMPLE");
    let text = std::fs::read_to_string(file).unwrap();
    uilab_reads_what_ess_reads(&text);
}

/// A doubling widget chain `depth` deep, used once in a page kind's section, and `pages` pages of
/// that kind.
fn page_kind_chain(depth: usize, pages: usize) -> String {
    let mut text = String::from(HEAD);
    let names: Vec<String> = (0..pages).map(|i| format!("p{i}")).collect();
    text.push_str(&format!(
        "navigation:\n  home: p0\n  sections: [{{name: s, pages: [{}]}}]\n",
        names.join(", ")
    ));
    text.push_str("widgets:\n  w0:\n    summary: Leaf.\n    body:\n      - {name: t, primitive: text, text: hi}\n");
    for i in 1..=depth {
        text.push_str(&format!(
            "  w{i}:\n    summary: Level.\n    body:\n      - {{name: a, component: w{p}}}\n      - {{name: b, component: w{p}}}\n",
            p = i - 1
        ));
    }
    text.push_str(&format!(
        "page_kinds:\n  heavy:\n    extends: detail_page\n    sections:\n      - {{name: big, component: w{depth}}}\npages:\n"
    ));
    for name in &names {
        text.push_str(&format!(
            "  {name}:\n    kind: heavy\n    sections:\n      - {{name: summary, component: record}}\n"
        ));
    }
    text
}

/// A document with every layer uilab addresses, names ESS allows that `valid_name` does not
/// (`Big2`, `v2.list`), and keys uilab does not type.
const RICH: &str = "format: ess-ui/1
app: t
title: Rich
model: t
actor: from_session
placement_profile: hybrid
placement_defaults: {draft: session_storage}
types:
  Window: {record: {from: timestamp, to: timestamp}}
shells:
  app:
    regions:
      main: {kind: page_outlet}
      over: {kind: overlay_outlet, props: {one_at_a_time: true, ratio: 1.5, code: '007'}}
    overlays:
      help:
        kind: drawer
        component: record
        reads: {view: x.All}
        item:
          - {name: tip, primitive: text, text: hi}
navigation:
  home: a
  visibility: by_actor_grants
  sections:
    - {name: s, label: Main, pages: [a, Big2, v2.list]}
widgets:
  card:
    summary: A card.
    params:
      size: {type: integer, default: 3, note: the size}
    body:
      - {name: title, primitive: text, text: hi}
      - name: box
        component: record
        item:
          - {name: x, primitive: text, text: hi}
pages:
  a:
    kind: dashboard_page
    title: Home
    nav: {label: Home, synonyms: [start]}
    sections:
      - name: board
        component: board
        reads: {view: x.All}
        widgets:
          tile:
            component: record
            item:
              - {name: x, primitive: text, text: y}
      - name: list
        load: on_visible
        component: collection
        reads: {view: x.All, params: {limit: 10}}
        item:
          - {name: c, component: card, args: {size: 2}}
        children:
          - {name: hint, primitive: text, text: hi}
    overlays:
      edit:
        kind: drawer
        component: form
        does: x.Edit
        parts:
          - {name: help, primitive: text, text: hi}
  Big2:
    kind: list_page
    sections:
      - name: filters
        component: filter_bar
        binds: [state.search]
        choices:
          - {name: state, component: choice, options: [a, b], binds: state.search}
      - name: list
        component: collection
        reads: {view: x.All}
  v2.list:
    kind: editor_page
    sections:
      - name: editor
        component: graph_editor
        reads: {view: x.All}
        toolbar:
          - {name: zoom, primitive: input, binds: state.zoom}
";

fn key_order(a: &Yaml, b: &Yaml, at: &str, out: &mut Vec<String>) {
    match (a, b) {
        (Yaml::Mapping(a), Yaml::Mapping(b)) => {
            let ka: Vec<_> = a.keys().collect();
            let kb: Vec<_> = b.keys().collect();
            if ka != kb {
                out.push(format!("{at}: {ka:?} vs {kb:?}"));
            }
            for (k, v) in a {
                if let Some(w) = b.get(k) {
                    key_order(v, w, &format!("{at}/{}", k.as_str().unwrap_or("?")), out);
                }
            }
        }
        (Yaml::Sequence(a), Yaml::Sequence(b)) => {
            for (i, (v, w)) in a.iter().zip(b).enumerate() {
                key_order(v, w, &format!("{at}/{i}"), out);
            }
        }
        _ => {}
    }
}

#[test]
fn every_layer_maps_to_ess_and_back_and_is_written_as_authored() {
    use uilab_doc::ess::{from_ess, node_at, to_ess};

    assert_eq!(ess_errors(RICH), Vec::<String>::new(), "precondition");
    let doc = Document::from_yaml(RICH).unwrap();

    // Written back as authored: values and key order.
    let written: Yaml = serde_yaml::from_str(&doc.to_yaml().unwrap()).unwrap();
    let authored: Yaml = serde_yaml::from_str(RICH).unwrap();
    assert_eq!(written, authored, "written back as authored");
    let mut order = Vec::new();
    key_order(&authored, &written, "", &mut order);
    assert!(order.is_empty(), "key order changed: {order:#?}");

    // Every uilab node maps to an ESS node and back.
    let ess_doc = uilab_doc::ess_ui::load_str(RICH).unwrap();
    let ess_paths: std::collections::BTreeSet<String> =
        ess_doc.nodes().iter().map(|l| l.path.to_string()).collect();
    let mut ours = Vec::new();
    fn walk(node: &uilab_doc::OutlineNode, out: &mut Vec<String>) {
        out.push(node.path.clone());
        for child in &node.children {
            walk(child, out);
        }
    }
    walk(&outline(&doc), &mut ours);
    let mut wrong = Vec::new();
    for at in &ours {
        let path: NodePath = at.parse().unwrap();
        let ess = to_ess(&path);
        if from_ess(&ess).as_ref() != Some(&path) || node_at(&doc, &ess) != path {
            wrong.push(format!("{at} → {ess} does not come back"));
        }
        if !matches!(path.layer(), Layer::Root | Layer::Nav) && !ess_paths.contains(&ess) {
            wrong.push(format!("{at} → {ess}: ESS has no such node"));
        }
    }
    // Every node uilab addresses is in the outline.
    for expected in [
        "shell:app/overlay:help/item:tip",
        "page:a/section:board/widget:tile/item:x",
        "page:a/section:list/item:c",
        "page:a/section:list/child:hint",
        "page:a/overlay:edit/part:help",
        "page:Big2/section:filters/choice:state",
        "page:v2.list/section:editor/tool:zoom",
        "component:card/node:box/item:x",
    ] {
        let path: NodePath = expected.parse().unwrap();
        assert!(resolve(&doc, &path).is_ok(), "{expected} resolves");
        if !ours.iter().any(|p| p == expected) {
            wrong.push(format!("{expected}: not in the outline"));
        }
    }
    // Every ESS node lands on an existing uilab node.
    for ess in &ess_paths {
        let at = node_at(&doc, ess);
        if resolve(&doc, &at).is_err() {
            wrong.push(format!("ESS {ess} → {at}: no such uilab node"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The known gap: the acceptance says `fixtures.views` is refused; ESS 0.48.0 reads it. Whatever
/// ESS says, a document carrying it that uilab reads is written back unchanged.
#[test]
fn fixtures_views_is_written_back_unchanged() {
    let text = String::from(
        "format: ess-ui/1\napp: t\nmodel: t\nplacement_profile: fat\nfixtures: {dir: fixtures, views: {x.All: x.yaml}}\nshells:\n  app:\n    regions:\n      main: {kind: page_outlet}\nnavigation:\n  home: a\n  sections: [{name: s, pages: [a]}]\npages:\n  a:\n    kind: list_page\n    sections:\n      - {name: list, component: collection, reads: {view: x.All}}\n",
    );
    match (
        uilab_doc::ess_ui::load_str(&text),
        Document::from_yaml(&text),
    ) {
        (Ok(_), Ok(doc)) => {
            let written: Yaml = serde_yaml::from_str(&doc.to_yaml().unwrap()).unwrap();
            let authored: Yaml = serde_yaml::from_str(&text).unwrap();
            assert_eq!(written, authored);
        }
        (Err(_), Err(_)) => {}
        (ess, ours) => panic!("ESS {:?}, uilab {:?}", ess.is_ok(), ours.is_ok()),
    }
}
