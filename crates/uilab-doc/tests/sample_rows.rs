//! Sample rows for a view without a fixture: the value each field gets.

use std::path::Path;

use serde_json::{Value, json};
use uilab_doc::model::Composite;
use uilab_doc::{Document, sample_rows};

fn library() -> Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    Document::from_yaml(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The library document with `composite` added as a section of the overview page.
fn with_section(name: &str, composite: Value) -> Document {
    let mut doc = library();
    let composite: Composite = serde_json::from_value(composite).unwrap();
    doc.pages["overview"]
        .sections
        .insert(name.into(), Some(composite));
    doc
}

fn is_date(v: &Value) -> bool {
    v.as_str().is_some_and(|s| s.starts_with("2026-"))
}

#[test]
fn a_field_a_metric_reads_as_from_is_an_integer_whatever_its_name() {
    for field in ["on_loan", "members", "title", "joined_at", "status"] {
        let doc = with_section(
            "m",
            json!({"component": "metric", "reads": {"view": "draft.Counts"}, "from": field}),
        );
        let rows = sample_rows(&doc, "draft.Counts");
        assert!(
            rows.iter().all(|r| r[field].is_i64()),
            "`from: {field}` gave {:?}",
            rows[0][field]
        );
    }
}

#[test]
fn a_field_a_chart_reads_as_a_series_is_an_integer_and_its_x_a_label() {
    let doc = with_section(
        "c",
        json!({"component": "chart", "reads": {"view": "draft.Returns"},
               "x": "shelf", "series": [{"field": "returned"}, "on_time"]}),
    );
    let rows = sample_rows(&doc, "draft.Returns");
    assert!(rows.iter().all(|r| r["returned"].is_i64()), "{:?}", rows[0]);
    assert!(rows.iter().all(|r| r["on_time"].is_i64()), "{:?}", rows[0]);
    assert!(rows[0]["shelf"].is_string());
}

#[test]
fn a_word_that_marks_a_kind_at_one_end_of_a_name_does_not_mark_it_at_the_other() {
    let columns = [
        "on_loan",
        "at_risk",
        "time_limit",
        "due_amount",
        "day_count",
        "active_loans",
        "status_changed_at",
        "month_total",
        "created_on",
        "joined_at",
        "due",
        "renewal_date",
        "is_member",
        "account_active",
        "loan_status",
    ];
    let doc = with_section(
        "t",
        json!({"component": "collection", "reads": {"view": "draft.Names"}, "columns": columns}),
    );
    let row = &sample_rows(&doc, "draft.Names")[0];
    for field in ["on_loan", "at_risk", "time_limit"] {
        assert!(
            row[field].is_string() && !is_date(&row[field]),
            "`{field}` is not a date: {:?}",
            row[field]
        );
    }
    for field in ["due_amount", "day_count", "active_loans", "month_total"] {
        assert!(
            row[field].is_i64(),
            "`{field}` is a number: {:?}",
            row[field]
        );
    }
    for field in [
        "status_changed_at",
        "created_on",
        "joined_at",
        "due",
        "renewal_date",
    ] {
        assert!(
            is_date(&row[field]),
            "`{field}` is a date: {:?}",
            row[field]
        );
    }
    for field in ["is_member", "account_active"] {
        assert!(
            row[field].is_boolean(),
            "`{field}` is a flag: {:?}",
            row[field]
        );
    }
    assert!(
        ["open", "overdue", "closed"].contains(&row["loan_status"].as_str().unwrap()),
        "{:?}",
        row["loan_status"]
    );
}
