//! Adversary pass 1 for story:essui-app-widget: `outline::rendered` is "what ESS renders"
//! (epic:ess-ui-adoption D6). These cases hold it to ESS's own expansion: every node ESS's loader
//! yields (`ess_ui::load_str` + `Document::nodes`) under `pages/` and `shells/` that names a uilab
//! node is in the rendered outline, and the rendered outline holds nothing ESS does not render.

use std::collections::BTreeSet;

use uilab_doc::outline::{authored_at, rendered};
use uilab_doc::{Document, Layer, NodePath, OutlineNode, ess_ui};

/// The uilab layer an ESS container key holds under a node of `parent`'s layer, as far as uilab
/// addresses nodes; a widget instance's expanded `body` is uilab's `node:` at the instance.
fn layer_under(parent: Layer, key: &str) -> Option<Layer> {
    use Layer::*;
    Some(match (parent, key) {
        (Root, "pages") => Page,
        (Root, "shells") => Shell,
        (Shell, "regions") => Region,
        (Shell, "overlays") => Overlay,
        (Page, "sections") => Section,
        (Page, "overlays") => Overlay,
        (Section | Overlay | Widget | Item | Child | Part | Choice | Tool | Node, key) => match key
        {
            "body" => Node,
            "widgets" => Widget,
            "item" => Item,
            "children" if parent == Section => Child,
            "parts" => Part,
            "choices" => Choice,
            "toolbar" => Tool,
            _ => return None,
        },
        _ => return None,
    })
}

/// The uilab path of an ESS canonical path, when every segment pair names a uilab node.
fn uilab_path(segments: &[String]) -> Option<String> {
    let mut path = NodePath::root();
    let mut i = 0;
    while i < segments.len() {
        let layer = layer_under(path.layer(), &segments[i])?;
        let name = segments.get(i + 1)?;
        path = path.child(layer, name);
        i += 2;
    }
    (!segments.is_empty()).then(|| path.to_string())
}

/// Every node ESS renders under `pages/` and `shells/` that names a uilab node, as uilab paths.
fn ess_nodes(text: &str) -> BTreeSet<String> {
    let ess = ess_ui::load_str(text).expect("ESS loads the document");
    ess.nodes()
        .iter()
        .filter(|l| {
            l.path
                .segments()
                .first()
                .is_some_and(|s| s == "pages" || s == "shells")
        })
        .filter_map(|l| uilab_path(l.path.segments()))
        .collect()
}

/// Every node of `rendered` under pages and shells.
fn rendered_nodes(doc: &Document) -> BTreeSet<String> {
    fn walk(node: &OutlineNode, out: &mut BTreeSet<String>) {
        if matches!(node.layer, Layer::Component | Layer::Nav) {
            return;
        }
        if node.layer != Layer::Root {
            out.insert(node.path.clone());
        }
        node.children.iter().for_each(|c| walk(c, out));
    }
    let mut out = BTreeSet::new();
    walk(&rendered(doc), &mut out);
    out
}

fn assert_rendered_is_ess(text: &str) {
    let doc = Document::from_yaml(text).expect("uilab loads the document");
    let ess = ess_nodes(text);
    let ours = rendered_nodes(&doc);
    let missing: Vec<_> = ess.difference(&ours).collect();
    let extra: Vec<_> = ours.difference(&ess).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "ESS renders and `rendered` misses: {missing:?}\n`rendered` holds and ESS does not render: {extra:?}"
    );
}

const HEAD: &str = "format: ess-ui/1
app: library
title: Lending library
model: library
placement_profile: fat
fixtures: {dir: fixtures, index: fixtures/index.yaml}
shells:
  app:
    regions:
      nav:
        kind: navigation
      main:
        kind: page_outlet
      overlay:
        kind: overlay_outlet
navigation:
  home: loans
  sections:
    - name: circulation
      label: Circulation
      pages: [loans]
";

/// A declared page kind over `list_page` whose `filters` holds a `status` choice and which adds a
/// `create` drawer; the Loans page refines `filters` with a `member` choice of its own. ESS merges
/// named lists by name (`inheritance.named_lists`), so the rendered `filters` holds both choices.
fn refined_kind() -> String {
    format!(
        "{HEAD}page_kinds:
  loan_list:
    extends: list_page
    purpose: loans with a status choice and a create drawer
    sections:
      - name: filters
        component: filter_bar
        binds: [state.search]
        search: {{binds: state.search}}
        choices: [{{name: status, component: choice, options: [open, returned]}}]
    overlays:
      create: {{kind: drawer, component: form, title: New loan, does: loans.ExtendLoan, fields: [due]}}
pages:
  loans:
    kind: loan_list
    title: Loans
    sections:
      - name: filters
        component: filter_bar
        binds: [state.search]
        choices: [{{name: member, component: choice, options: [ada, bob]}}]
      - name: list
        component: collection
        reads: {{view: loans.All, paging: server}}
        columns: [{{field: title}}, {{field: due}}]
"
    )
}

/// Widget instances as a collection item, as a section child, nested in another widget's body,
/// and one widget used twice as sections.
fn instances() -> String {
    format!(
        "{HEAD}widgets:
  loan_card:
    summary: A loan as a card.
    params:
      loan: {{type: Loan, required: true, note: the loan row}}
    body:
      - {{name: title, primitive: text, text: args.loan.title, style: heading}}
      - {{name: due, component: due_tag, args: {{when: args.loan.due}}}}
  due_tag:
    summary: A due date.
    params:
      when: {{type: string, required: true, note: the date}}
    body:
      - {{name: tag, primitive: badge, text: args.when}}
pages:
  loans:
    kind: list_page
    title: Loans
    sections:
      - name: list
        component: collection
        reads: {{view: loans.All, paging: server}}
        columns: [{{field: title}}, {{field: due}}]
        item:
          - {{name: when, component: due_tag, args: {{when: row.due}}}}
        children:
          - {{name: first, component: loan_card, args: {{loan: rows.first}}}}
          - {{name: rule, primitive: divider}}
      - {{name: latest, component: loan_card, args: {{loan: rows.first}}}}
      - {{name: second, component: loan_card, args: {{loan: rows.first}}}}
"
    )
}

/// A declared page kind whose section reads a view through ESS's `Reads` shorthand
/// (`reads: loans.Summary`, schema `shorthands.index`). ESS 0.48.0 checks the document with 0
/// errors and 0 warnings.
fn shorthand_kind() -> String {
    format!(
        "{HEAD}page_kinds:
  counted_list:
    extends: list_page
    purpose: a list with a count above it
    sections:
      - {{name: count, component: metric, label: Copies on loan, reads: loans.Summary, from: on_loan}}
pages:
  loans:
    kind: counted_list
    title: Loans
    sections:
      - name: list
        component: collection
        reads: {{view: loans.All, paging: server}}
        columns: [{{field: title}}, {{field: due}}]
"
    )
}

/// `KindParts::read` reads a kind's sections as uilab's `Composite` and leaves out "an entry
/// uilab's model cannot read"; a section ESS renders is then missing from what the canvas shows.
#[test]
fn adversary_rendered_is_what_ess_renders_for_a_kind_section_in_shorthand() {
    assert_rendered_is_ess(&shorthand_kind());
}

/// Control: the library example as shipped.
#[test]
fn adversary_rendered_is_what_ess_renders_for_the_library() {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/library/library.ui.yaml"),
    )
    .unwrap();
    assert_rendered_is_ess(&text);
}

/// Brief: "a page kind section the page refines". The kind's `status` choice is rendered by ESS
/// inside the page's `filters`; `rendered` shows the author's `filters` node with only the author's
/// children.
#[test]
fn adversary_rendered_is_what_ess_renders_when_the_page_refines_a_kind_section() {
    assert_rendered_is_ess(&refined_kind());
}

/// Instances in items, in a section's children, nested, and one widget used twice.
#[test]
fn adversary_rendered_is_what_ess_renders_for_nested_and_repeated_instances() {
    assert_rendered_is_ess(&instances());
}

/// A widget body node inside a widget used twice is answered for by its own instance, and a node
/// of a nested instance's body by the outermost node the author wrote.
#[test]
fn adversary_each_use_of_a_widget_answers_for_its_own_body() {
    let doc = Document::from_yaml(&instances()).unwrap();
    let at = |p: &str| authored_at(&doc, p).map(|n| n.to_string());
    assert_eq!(
        at("page:loans/section:latest/node:title").as_deref(),
        Some("page:loans/section:latest")
    );
    assert_eq!(
        at("page:loans/section:second/node:title").as_deref(),
        Some("page:loans/section:second")
    );
    assert_eq!(
        at("page:loans/section:second/node:due/node:tag").as_deref(),
        Some("page:loans/section:second")
    );
    assert_eq!(
        at("page:loans/section:list/child:first/node:due").as_deref(),
        Some("page:loans/section:list/child:first")
    );
}
