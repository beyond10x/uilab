//! Adversary cases for story:help-accuracy: the help's claims read against the code that makes
//! them true.

/// The body of the help section headed `## <title>`, up to the next `## ` heading.
fn section(help: &str, title: &str) -> String {
    let start = help
        .find(&format!("## {title}\n"))
        .unwrap_or_else(|| panic!("help has no section `{title}`:\n{help}"));
    let body = &help[start + title.len() + 4..];
    let end = body.find("\n## ").unwrap_or(body.len());
    body[..end].to_owned()
}

/// The browser (`widget/src/lib/workspace.ts` `targetIn`) and the server (`app.rs` `placed`) keep a
/// Components-tab instruction at the selection only when it is a `component:` path, a declared
/// widget or its body; anything else, including a board's `widget:` node, sends it to the root.
/// The same help calls a board's children "widgets" (`| section (... a board holds widgets ...) |
/// widget, item |`), and the sidebar tree labels them `wdg`. So "with a widget selected" must say
/// which one: a reader who selects a board widget on the tab is told the agent works under it and
/// gets a new root widget instead.
#[test]
fn the_components_section_names_the_layer_it_keys_on() {
    let help = uilab_doc::help_markdown();
    assert!(
        help.contains("a board holds widgets") && help.contains("| widget, item |"),
        "premise: the help's own table calls a board's children widgets"
    );
    let tab = section(&help, "The Components tab");
    assert!(
        tab.contains("widget or one of its body nodes selected"),
        "premise: the section keys on a selected widget:\n{tab}"
    );
    assert!(
        tab.contains("`component`") || tab.contains("component:"),
        "the Components section says \"with a widget selected\" but never names the `component` \
         layer the target rule checks, while the same help calls a board's `widget` nodes widgets:\n{tab}"
    );
}

/// `help_describes_goals_the_components_workspace_and_card_findings` pins "at the root" against the
/// whole help, where the Widgets section already says "declared under `widgets:` at the root"; it
/// stays green if the Components section loses the claim. The same claims, held to their section.
#[test]
fn the_components_claims_are_in_the_components_section() {
    let help = uilab_doc::help_markdown();
    let tab = section(&help, "The Components tab");
    for expected in [
        "`4`",
        "builds widgets",
        "under the selected widget",
        "at the root",
    ] {
        assert!(
            tab.contains(expected),
            "the Components section misses {expected:?}:\n{tab}"
        );
    }
    let goals = section(&help, "Goals");
    for expected in ["**goal**", "plans it into steps", "**Stop**"] {
        assert!(
            goals.contains(expected),
            "the Goals section misses {expected:?}:\n{goals}"
        );
    }
}
