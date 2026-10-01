//! Adversary pass 2 for story:essui-app-widget (commit d825897): `outline::rendered` shows each
//! node with the fields ESS renders it with (epic:ess-ui-adoption D6), not only the fields the
//! author wrote at that node. Every document here is checked by ESS 0.48.0 first (0 errors), and
//! every expected value is the one ESS's expansion gives (`ess_ui::load_str`).

use std::path::Path;

use uilab_doc::outline::rendered;
use uilab_doc::{Document, OutlineNode};

/// ESS's own verdict on a document: its error findings, as `check path: message`.
fn ess_errors(text: &str) -> Vec<String> {
    uilab_doc::ess_ui_check::check_source(text, "case", Path::new("."), None, &Default::default())
        .findings
        .into_iter()
        .filter(|f| f.severity == uilab_doc::ess_ui_check::Severity::Error)
        .map(|f| format!("{} {}: {}", f.check, f.path, f.message))
        .collect()
}

/// The node of `root` at `path`, searched depth first.
fn find<'o>(root: &'o OutlineNode, path: &str) -> Option<&'o OutlineNode> {
    if root.path == path {
        return Some(root);
    }
    root.children.iter().find_map(|c| find(c, path))
}

/// `rendered` of `text`, after ESS has checked `text` with 0 errors.
fn shown(text: &str) -> OutlineNode {
    let errors = ess_errors(text);
    assert!(errors.is_empty(), "ESS refuses the case: {errors:?}");
    rendered(&Document::from_yaml(text).expect("uilab loads the document"))
}

fn at<'o>(root: &'o OutlineNode, path: &str) -> &'o OutlineNode {
    find(root, path).unwrap_or_else(|| panic!("`rendered` holds no node `{path}`"))
}

const HEAD: &str = "format: ess-ui/1
app: library
title: Lending library
model: library
placement_profile: fat
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
      pages: [loans, members]
";

/// The shipped library example: its Loans page writes the drawer `edit` (`kind: drawer`,
/// `component: form`). The wire, the widget's tests (`outline.test.ts`, `collab.test.ts`) and the
/// docs spell such an overlay's kind `drawer form`.
#[test]
fn adversary_an_overlay_kind_reads_as_the_document_spells_it_p2() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml"),
    )
    .unwrap();
    let root = shown(&text);
    assert_eq!(at(&root, "page:loans/overlay:edit").kind, "drawer form");
}

/// A Loans page (`list_page`) that refines the kind's `filters` with a choice of its own and
/// writes no `component` there, as `ess-ui/1` allows for a refinement (uilab's
/// `Component::Inherited`). ESS merges the kind's section in by name: it renders `filters` as a
/// `filter_bar`.
#[test]
fn adversary_a_refining_section_shows_the_component_its_kind_gives_p2() {
    let text = format!(
        "{HEAD}pages:
  loans:
    kind: list_page
    title: Loans
    sections:
      - name: filters
        choices: [{{name: status, component: choice, options: [open, returned]}}]
      - name: list
        component: collection
        reads: {{view: loans.All, paging: server}}
        columns: [{{field: title}}, {{field: due}}]
  members:
    kind: list_page
    title: Members
    sections:
      - name: list
        component: collection
        reads: {{view: members.All}}
        columns: [{{field: name}}]
"
    );
    let root = shown(&text);
    assert_eq!(at(&root, "page:loans/section:filters").kind, "filter_bar");
}

/// A declared kind over `list_page` whose `list` reads `loans.All`; the Loans page refines `list`
/// with its own columns. ESS merges the kind's `list` under the page's: the section it renders
/// reads `loans.All`, and the canvas asks for rows by the view the outline carries.
#[test]
fn adversary_a_refining_section_reads_the_view_its_kind_gives_p2() {
    let text = format!(
        "{HEAD}page_kinds:
  loan_list:
    extends: list_page
    purpose: every loan
    sections:
      - name: list
        component: collection
        reads: {{view: loans.All, paging: server}}
        columns: [{{field: title}}, {{field: due}}]
pages:
  loans:
    kind: loan_list
    title: Loans
    sections:
      - name: list
        component: collection
        columns: [{{field: title}}, {{field: member}}]
  members:
    kind: list_page
    title: Members
    sections:
      - name: list
        component: collection
        reads: {{view: members.All}}
        columns: [{{field: name}}]
"
    );
    let root = shown(&text);
    assert_eq!(
        at(&root, "page:loans/section:list").view.as_deref(),
        Some("loans.All")
    );
}

/// The Members page's drawer `edit` is `same_as: loans.edit`. ESS copies the Loans drawer
/// (`resolve_same_as`): it renders a drawer form, and the form's part `hint` under it.
fn same_as() -> String {
    format!(
        "{HEAD}pages:
  loans:
    kind: list_page
    title: Loans
    sections:
      - name: list
        component: collection
        reads: {{view: loans.All, paging: server}}
        columns: [{{field: title}}, {{field: due}}]
        row_actions: [{{opens: edit, label: Extend}}]
    overlays:
      edit:
        kind: drawer
        component: form
        title: Extend loan
        does: loans.ExtendLoan
        fields: [due]
        parts:
          - {{name: hint, primitive: text, text: Extends by two weeks}}
  members:
    kind: list_page
    title: Members
    sections:
      - name: list
        component: collection
        reads: {{view: members.All}}
        columns: [{{field: name}}]
        row_actions: [{{opens: edit, label: Extend}}]
    overlays:
      edit:
        same_as: loans.edit
"
    )
}

#[test]
fn adversary_a_same_as_overlay_shows_the_overlay_it_copies_p2() {
    let root = shown(&same_as());
    assert_eq!(at(&root, "page:members/overlay:edit").kind, "drawer form");
}

/// The part ESS renders inside the copied drawer is shown as what it is, a `text`, not as an
/// untyped `node`.
#[test]
fn adversary_a_part_of_a_same_as_overlay_shows_its_kind_p2() {
    let root = shown(&same_as());
    let hint = at(&root, "page:members/overlay:edit/part:hint");
    assert!(
        hint.inherited,
        "the author did not write it at the Members page"
    );
    assert_eq!(hint.kind, "text");
}

/// Reachability of `widget/src/lib/nested.instance.p2.adversary.test.ts`: two widgets whose param
/// is named `loan`, one nested in the other's body. ESS checks the document with 0 errors, and
/// `rendered` sends the nested body node with its declaration's `text: args.loan.title` and the
/// nested instance with `args: {loan: args.loan.renewal}`, as that widget test's outline holds them.
#[test]
fn adversary_a_nested_body_is_sent_as_its_declaration_writes_it_p2() {
    let text = format!(
        "{HEAD}widgets:
  loan_card:
    summary: A loan as a card.
    params:
      loan: {{type: Loan, required: true, note: the loan}}
    body:
      - {{name: title, primitive: text, text: args.loan.title}}
      - {{name: renewal, component: loan_line, args: {{loan: args.loan.renewal}}}}
  loan_line:
    summary: A loan on one line.
    params:
      loan: {{type: Loan, required: true, note: the loan}}
    body:
      - {{name: line, primitive: text, text: args.loan.title}}
pages:
  loans:
    kind: list_page
    title: Loans
    sections:
      - name: list
        component: collection
        reads: {{view: loans.All, paging: server}}
        columns: [{{field: title}}, {{field: due}}]
      - {{name: latest, component: loan_card, args: {{loan: rows.first}}}}
  members:
    kind: list_page
    title: Members
    sections:
      - name: list
        component: collection
        reads: {{view: members.All}}
        columns: [{{field: name}}]
"
    );
    let root = shown(&text);
    let renewal = at(&root, "page:loans/section:latest/node:renewal");
    assert!(renewal.inherited);
    assert_eq!(
        renewal.props.as_ref().map(|p| p["args"].clone()),
        Some(serde_json::json!({"loan": "args.loan.renewal"}))
    );
    let line = at(&root, "page:loans/section:latest/node:renewal/node:line");
    assert!(line.inherited);
    assert_eq!(
        line.props.as_ref().map(|p| p["text"].clone()),
        Some(serde_json::json!("args.loan.title"))
    );
}
