//! The deadline around every call into ESS (beyond10x/ess#300): the weighing in front of ESS is a
//! first filter, and a document it lets through that ESS takes longer than the deadline on is
//! refused as `expansion_bound` within the deadline and 2 s, on load and on admit. A test binary
//! of its own, because the deadline is set for the whole process.

use std::time::{Duration, Instant};

use serde_json::json;
use uilab_doc::{Child, Document, EXPANSION_LIMIT, Layer, Patch, admit};

const DEADLINE: Duration = Duration::from_millis(300);

const DOC: &str = "format: ess-ui/1
app: t
model: t
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: a
  sections: [{name: s, pages: [a]}]
pages:
  a:
    kind: list_page
    title: A
    sections:
      - {name: list, component: collection, reads: {view: x.All}}
";

/// `widgets:` with a doubling chain `d0..=d14` whose leaf is a button: one use of `d14` expands
/// to 2^14 buttons, 65 534 maps, under `EXPANSION_LIMIT`, and takes ESS far longer than
/// [`DEADLINE`] in a test build.
fn chain() -> String {
    let mut text = String::from(
        "widgets:\n  d0:\n    summary: Leaf.\n    body:\n      - {name: go, primitive: button, label: Go, action: {name: go, does: x.Go}}\n",
    );
    for i in 1..=14 {
        text.push_str(&format!(
            "  d{i}:\n    summary: Level.\n    body:\n      - {{name: l, component: d{p}}}\n      - {{name: r, component: d{p}}}\n",
            p = i - 1
        ));
    }
    text
}

#[test]
fn a_document_ess_does_not_read_within_the_deadline_is_refused_on_load_and_on_admit() {
    assert_eq!(uilab_doc::ess::ESS_DEADLINE, Duration::from_secs(30));
    assert_eq!(uilab_doc::ess::deadline(), uilab_doc::ess::ESS_DEADLINE);

    // Within the default deadline the document without a use loads.
    let declared = DOC.replace("pages:\n", &format!("{}pages:\n", chain()));
    let doc = Document::from_yaml(&declared).unwrap_or_else(|e| panic!("{e}"));

    uilab_doc::ess::set_deadline(DEADLINE);

    // On load: the use is under the bound, so only the deadline refuses it.
    let used = declared.replace(
        "      - {name: list, component: collection, reads: {view: x.All}}\n",
        "      - {name: list, component: collection, reads: {view: x.All}}\n      - {name: deep, component: d14}\n",
    );
    let started = Instant::now();
    let refused = Document::from_yaml(&used).expect_err("ESS takes longer than the deadline");
    let took = started.elapsed();
    assert!(
        took < DEADLINE + Duration::from_secs(2),
        "refused after {took:?}"
    );
    assert!(
        refused.message.contains("beyond10x/ess#300")
            && !refused.message.contains(&EXPANSION_LIMIT.to_string()),
        "refused by the deadline, not the weighing: {refused}"
    );

    // On admit: the same use inserted by a patch.
    let insert = Patch::Insert {
        target: "page:a".parse().unwrap(),
        child: Child {
            layer: Layer::Section,
            name: "deep".into(),
            node: json!({"component": "d14"}),
            nav_section: None,
        },
    };
    let started = Instant::now();
    let refusal = admit(&doc, &insert).expect_err("ESS takes longer than the deadline");
    let took = started.elapsed();
    assert_eq!(refusal.check, "expansion_bound", "{refusal}");
    assert!(refusal.message.contains("beyond10x/ess#300"), "{refusal}");
    assert!(
        took < DEADLINE + Duration::from_secs(2),
        "refused after {took:?}"
    );

    // On check: never silent. A document read past the loader (as a patch's result is checked)
    // that ESS does not check in time has one `expansion_bound` error, not no findings.
    let unread: Document = serde_yaml::from_str(&used).unwrap();
    let started = Instant::now();
    let found = uilab_doc::check(&unread);
    let took = started.elapsed();
    let bound: Vec<_> = found
        .iter()
        .filter(|f| f.check == "expansion_bound")
        .collect();
    assert_eq!(bound.len(), 1, "{found:?}");
    assert_eq!(bound[0].severity, uilab_doc::Severity::Error);
    assert!(bound[0].message.contains("beyond10x/ess#300"), "{found:?}");
    assert!(
        took < DEADLINE + Duration::from_secs(2),
        "checked after {took:?}"
    );
}
