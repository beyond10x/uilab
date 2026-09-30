//! Adversarial cases, pass 2, for story:agent-retarget: whether the Components-tab guard still
//! lets a move leave the widgets when the instruction names no page, after round 2 made a widget's
//! own name stop naming one.

use std::path::Path;

use uilab_agent::{Workspace, check_retarget};
use uilab_doc::Document;

/// The library example with the widgets the operator's own document declares (`member_card`,
/// `loan_card`, `stat_tile`, `page_intro`), each a one-node body.
fn library_with_the_operators_widgets() -> Document {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library/library.ui.yaml");
    let text = std::fs::read_to_string(file).unwrap();
    assert!(text.contains("\npages:\n"), "fixture anchor is missing");
    let widgets: String = ["member_card", "loan_card", "stat_tile", "page_intro"]
        .iter()
        .map(|name| {
            format!(
                "  {name}:\n    summary: A widget.\n    body:\n      - {{name: title, primitive: text, text: Title, style: heading}}\n"
            )
        })
        .collect();
    let text = text.replacen("\npages:\n", &format!("\nwidgets:\n{widgets}pages:\n"), 1);
    Document::from_yaml(&text).unwrap()
}

fn components_move(doc: &Document, from: &str, to: &str, utterance: &str) -> Result<(), String> {
    check_retarget(
        doc,
        &from.parse().unwrap(),
        &to.parse().unwrap(),
        utterance,
        false,
        Workspace::Components,
    )
    .map_err(|refusal| refusal.check)
}

/// Design (story:agent-retarget): "On the Components tab the move stays inside `component:`
/// paths unless the instruction names a page." Round 2 stopped the widget's own name `loan_card`
/// from naming the page `loans`, but a second mention of the entity the widget shows ("the
/// loan's due date") is outside the widget's name, and the plural rule of `same_word` reads it as
/// the page `loans`. The instruction is about the widget alone.
#[test]
fn a_second_mention_of_the_widgets_entity_names_no_page() {
    let doc = library_with_the_operators_widgets();
    assert_eq!(
        components_move(
            &doc,
            "component:loan_card",
            "page:loans",
            "make the loan card bigger and show the loan's due date"
        ),
        Err("retarget_workspace".to_owned()),
        "the instruction names the widget loan_card and a field of its loan, not the loans page"
    );
}

/// The same hole through a field the widget shows: the loan card's member. "member" is the
/// singular of the page `members`, so an instruction about what a widget displays names a page
/// whenever it mentions one of the library's nouns (loans, members, overview), which every
/// instruction about these widgets does.
#[test]
fn a_field_the_widget_shows_names_no_page() {
    let doc = library_with_the_operators_widgets();
    assert_eq!(
        components_move(
            &doc,
            "component:loan_card",
            "page:members",
            "make the loan card show the member's name"
        ),
        Err("retarget_workspace".to_owned()),
        "\"the member's name\" is a field of the loan card, not the members page"
    );
    assert_eq!(
        components_move(
            &doc,
            "component:member_card",
            "page:loans",
            "list the member's current loans on the member card"
        ),
        Err("retarget_workspace".to_owned()),
        "\"current loans\" is what the member card lists, not the loans page"
    );
}

/// The word "page" outside a widget's name counts as naming a page, including as a layout
/// measure ("as wide as the page"), which names no page at all.
#[test]
fn page_as_a_measure_names_no_page() {
    let doc = library_with_the_operators_widgets();
    assert_eq!(
        components_move(
            &doc,
            "component:stat_tile",
            "page:overview",
            "make the stat tile as wide as the page"
        ),
        Err("retarget_workspace".to_owned()),
        "\"as wide as the page\" is a size, not a destination"
    );
}

/// Probes that hold: a multi-word page title said beside a widget, a widget named like a page
/// (`page_intro`), the plural of a widget's name, and the word "page" beside a page's name.
#[test]
fn a_page_said_beside_a_widget_still_names_it() {
    let doc = library_with_the_operators_widgets();
    assert_eq!(
        components_move(
            &doc,
            "component:page_intro",
            "page:loans",
            "make the page intro shorter"
        ),
        Err("retarget_workspace".to_owned())
    );
    assert_eq!(
        components_move(
            &doc,
            "component:page_intro",
            "page:loans",
            "put the page intro on the loans page"
        ),
        Ok(())
    );
    assert_eq!(
        components_move(
            &doc,
            "component:member_card",
            "page:members",
            "use the member cards in members"
        ),
        Ok(())
    );
    assert_eq!(
        components_move(
            &doc,
            "component:member_card",
            "page:overview",
            "make the member cards smaller"
        ),
        Err("retarget_workspace".to_owned())
    );
}
