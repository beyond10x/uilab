//! Adversary pass 2 on story:components-gallery: the region view the outline now carries, checked
//! against the TypeScript fixture `servedLibrary()` in
//! `widget/src/lib/components.gallery.adversary.test.ts` and against `outline_at`.

use std::path::Path;

use serde_json::{Value, json};
use uilab_doc::{Document, NodePath, outline, outline_at};

const OVERLAY: &str = "        kind: notifications
    overlays:
      whoami:
        kind: dialog
        component: staff_badge
        args: {staff: me}
";

const WIDGETS: &str = "
widgets:
  staff_badge:
    summary: Who is signed in
    arrange: row
    params:
      staff: {type: Staff, required: true, note: the signed-in staff row}
    body:
      - {name: name, primitive: text, text: args.staff.name}
      - {name: email, primitive: text, text: args.staff.email}
pages:
";

fn served_library(account_reads: &str) -> Document {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    let text = std::fs::read_to_string(file).unwrap();
    for anchor in [
        "        kind: notifications\n",
        "\npages:\n",
        "reads: {view: staff.Me}",
    ] {
        assert!(text.contains(anchor), "anchor `{anchor}` is missing");
    }
    let text = text
        .replacen("        kind: notifications\n", OVERLAY, 1)
        .replacen("\npages:\n", WIDGETS, 1)
        .replacen("reads: {view: staff.Me}", account_reads, 1);
    Document::from_yaml(&text).unwrap()
}

fn find<'a>(node: &'a Value, path: &str) -> &'a Value {
    if node["path"] == path {
        return node;
    }
    node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|c| {
            let hit = find(c, path);
            (hit["path"] == path).then_some(hit)
        })
        .unwrap_or(&Value::Null)
}

/// The TypeScript fixture of pass 1's case, as amended in e3d970d, is what the server sends: the
/// region node carries `view` and no `props`, the shell overlay carries its args, and the widget's
/// one use site is the overlay.
#[test]
fn the_amended_fixture_matches_the_served_outline() {
    let root = serde_json::to_value(outline(&served_library("reads: {view: staff.Me}"))).unwrap();
    assert_eq!(
        find(&root, "shell:app/region:account"),
        &json!({
            "path": "shell:app/region:account",
            "layer": "region",
            "name": "account",
            "kind": "account_menu",
            "view": "staff.Me",
            "children": [],
        })
    );
    assert_eq!(
        find(&root, "shell:app/overlay:whoami")["props"],
        json!({"args": {"staff": "me"}})
    );
    assert_eq!(
        find(&root, "component:staff_badge")["props"]["uses"],
        json!([{"path": "shell:app/overlay:whoami"}])
    );
}

/// A changed event re-reads one node with `outline_at`; a region read that way carries the same view.
#[test]
fn outline_at_a_region_carries_its_view() {
    let doc = served_library("reads: {view: staff.Me}");
    let at: NodePath = "shell:app/region:account".parse().unwrap();
    assert_eq!(
        outline_at(&doc, &at).unwrap().view.as_deref(),
        Some("staff.Me")
    );
}

/// A region whose `reads` names no view, or a view that is not a string, carries none; a draft read
/// is carried as written (the widget, not the server, leaves drafts out).
#[test]
fn a_region_read_without_a_string_view_carries_none() {
    let at: NodePath = "shell:app/region:account".parse().unwrap();
    for (reads, expected) in [
        ("reads: {params: {id: me}}", None),
        ("reads: {view: 7}", None),
        ("reads: staff.Me", None),
        ("reads: {view: draft.Profile}", Some("draft.Profile")),
    ] {
        let doc = served_library(reads);
        assert_eq!(
            outline_at(&doc, &at).unwrap().view.as_deref(),
            expected,
            "{reads}"
        );
    }
}
