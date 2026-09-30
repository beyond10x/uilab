//! Adversary pass 2 on story:draft-sample-rows: the name rules of `sample_rows` against names
//! the base commit shaped, and the quantity rule where composites share a draft view.

use serde_json::Value;
use uilab_doc::{Document, sample_rows};

const DOC: &str = r#"format: ui-spec/1
app: app
title: Sample rows
model: app
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
  home: home
  sections: []
  hidden: [home]
widgets:
  overdue_card:
    summary: Overdue members at a glance.
    body:
      - {name: count, component: metric, reads: {view: draft.Shared}, from: in_widget}
pages:
  home:
    kind: dashboard_page
    title: Home
    sections:
      on_loan:
        component: metric
        reads: {view: draft.Shared}
        from: on_loan
      members:
        component: metric
        reads: {view: draft.Shared}
        from: members
      per_state:
        component: chart
        reads: {view: draft.Shared}
        x: shelf
        series: [{field: returned}]
      table:
        component: collection
        reads: {view: draft.Shared}
        columns: [{field: returned}, {field: members}, {field: shelf}]
      by_state:
        component: chart
        reads: {view: draft.LoansByState}
        x: state
        series: [{field: loans}]
      people:
        component: collection
        reads: {view: draft.People}
        columns:
          - {field: date_of_birth}
          - {field: date_created}
          - {field: date_added}
          - {field: date_returned}
    overlays:
      detail:
        kind: drawer
        component: metric
        reads: {view: draft.Shared}
        from: in_overlay
"#;

fn doc() -> Document {
    Document::from_yaml(DOC).unwrap()
}

fn is_date(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        let b = s.as_bytes();
        s.len() == 10 && s.starts_with("2026-") && b[7] == b'-'
    })
}

/// A `date_…` name was a date at the base commit (`has(&["date", …])`); the rule now reads only
/// the last word, so `date_of_birth` is `"date of birth 1"`. Django-style names
/// (`date_joined`, `date_created`) put the kind first.
#[test]
fn a_name_that_starts_with_date_is_still_a_date() {
    let rows = sample_rows(&doc(), "draft.People");
    let row = &rows[0];
    let wrong: Vec<String> = [
        "date_of_birth",
        "date_created",
        "date_added",
        "date_returned",
    ]
    .into_iter()
    .filter(|f| !is_date(&row[*f]))
    .map(|f| format!("{f} = {}", row[f]))
    .collect();
    assert!(
        wrong.is_empty(),
        "dates at the base commit, not dates now: {wrong:?}"
    );
}

/// A chart's `x` is the category each bar stands for: the operator's Overview reads
/// `draft.LoansByState` with `x: state`, and five sample rows over three state values draw
/// `overdue, closed, open, overdue, closed`, two bars for one category.
#[test]
fn a_chart_draws_one_bar_per_x_label() {
    let rows = sample_rows(&doc(), "draft.LoansByState");
    let labels: Vec<&str> = rows.iter().map(|r| r["state"].as_str().unwrap()).collect();
    let mut distinct = labels.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(distinct.len(), labels.len(), "x labels: {labels:?}");
}

/// Two metrics, a chart series that is also a column, a metric in a widget body and one in an
/// overlay, all over one draft view: every quantity is an integer, the x label is text.
#[test]
fn every_quantity_over_a_shared_draft_view_is_an_integer_wherever_it_is_read() {
    let rows = sample_rows(&doc(), "draft.Shared");
    assert_eq!(rows.len(), 5);
    for row in &rows {
        for field in ["on_loan", "members", "returned", "in_widget", "in_overlay"] {
            assert!(row[field].is_i64(), "`{field}` in {row}");
        }
        assert!(row["shelf"].is_string(), "{row}");
    }
}
