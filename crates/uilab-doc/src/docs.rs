//! Documentation generated from a document: what a reader of the spec needs without reading the
//! YAML. Every section is derived; nothing here is authored per document.

use std::collections::BTreeMap;
use std::fmt::Write;

use serde_json::Value;

use crate::check::{Finding, Severity, composites};
use crate::fixtures::Fixtures;
use crate::model::{CompositeKind, Document, NavPages};
use crate::path::Layer;

/// The document's documentation as Markdown.
pub fn docs_markdown(doc: &Document, fixtures: &Fixtures, findings: &[Finding]) -> String {
    let mut out = String::new();
    let title = doc.title.as_deref().unwrap_or(&doc.app);
    let all = composites(doc);
    let _ = writeln!(out, "# {title}\n");
    let _ = writeln!(
        out,
        "App `{}` · model `{}` · {} shell(s) · {} page(s) · {} composite(s) · placement `{:?}`\n",
        doc.app,
        doc.model,
        doc.shells.len(),
        doc.pages.len(),
        all.len(),
        doc.placement_profile
    );

    // Navigation.
    let _ = writeln!(out, "## Navigation\n");
    let _ = writeln!(out, "Home: `{}`\n", doc.navigation.home);
    for section in &doc.navigation.sections {
        let label = section.label.as_deref().unwrap_or(&section.name);
        match &section.pages {
            NavPages::Fixed(pages) => {
                let pages: Vec<String> = pages.iter().map(|p| format!("`{p}`")).collect();
                let _ = writeln!(out, "- **{label}**: {}", pages.join(", "));
            }
            NavPages::Dynamic(entries) => {
                let _ = writeln!(
                    out,
                    "- **{label}**: one entry per row of `{}`",
                    entries
                        .get("from_view")
                        .and_then(Value::as_str)
                        .unwrap_or("?")
                );
            }
        }
    }
    if !doc.navigation.hidden.is_empty() {
        let hidden: Vec<String> = doc
            .navigation
            .hidden
            .iter()
            .map(|p| format!("`{p}`"))
            .collect();
        let _ = writeln!(out, "- hidden (reached by link): {}", hidden.join(", "));
    }
    out.push('\n');

    // Pages.
    let _ = writeln!(out, "## Pages\n");
    for (name, page) in &doc.pages {
        let _ = writeln!(
            out,
            "### {} (`{name}`)\n",
            page.title.as_deref().unwrap_or(name)
        );
        let _ = writeln!(
            out,
            "Kind `{}` · shell `{}`\n",
            page.kind,
            doc.shell_of(page).unwrap_or("-")
        );
        let _ = writeln!(
            out,
            "| section | component | reads | title |\n|---|---|---|---|"
        );
        for (section, composite) in page
            .sections
            .iter()
            .filter_map(|(n, c)| c.as_ref().map(|c| (n, c)))
        {
            let _ = writeln!(
                out,
                "| `{section}` | {} | {} | {} |",
                composite.component.as_str(),
                composite
                    .reads
                    .as_ref()
                    .map_or("-".to_owned(), |r| format!("`{}`", r.view)),
                composite
                    .props
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            );
        }
        let overlays: Vec<String> = page
            .overlays
            .iter()
            .filter_map(|(n, o)| {
                o.as_ref().map(|o| {
                    format!("`{n}` ({:?} {})", o.kind, o.body.component.as_str()).to_lowercase()
                })
            })
            .collect();
        if !overlays.is_empty() {
            let _ = writeln!(out, "\nOverlays: {}", overlays.join(", "));
        }
        out.push('\n');
    }

    // Component catalogue.
    let _ = writeln!(out, "## Components\n");
    let _ = writeln!(
        out,
        "| kind | used | where | what it is for |\n|---|---|---|---|"
    );
    for kind in CompositeKind::ALL {
        let used: Vec<String> = all
            .iter()
            .filter(|(_, c)| c.component == kind)
            .map(|(p, _)| format!("`{p}`"))
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            kind.as_str(),
            used.len(),
            used.join(", "),
            kind.summary()
        );
    }
    out.push('\n');

    // Data.
    let _ = writeln!(out, "## Data\n");
    let _ = writeln!(
        out,
        "| view | status | fields | read by |\n|---|---|---|---|"
    );
    let fields = fixtures.fields();
    let mut readers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (path, composite) in &all {
        if let Some(reads) = &composite.reads {
            readers
                .entry(reads.view.clone())
                .or_default()
                .push(format!("`{path}`"));
        }
    }
    for (view, by) in &readers {
        let status = if view.starts_with(crate::model::DRAFT_VIEW_PREFIX) {
            "draft: no model view yet"
        } else if fixtures.has(view) {
            "fixture"
        } else {
            "no fixture"
        };
        let names = fields.get(view).map(|f| f.join(", ")).unwrap_or_default();
        let _ = writeln!(out, "| `{view}` | {status} | {names} | {} |", by.join(", "));
    }
    out.push('\n');

    // Commands and overlays.
    let mut commands: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut opens: Vec<(String, String)> = Vec::new();
    for (path, composite) in &all {
        let props = Value::Object(composite.props.clone().into_iter().collect());
        collect(&props, &mut |key, value| match key {
            "does" => commands
                .entry(value.to_owned())
                .or_default()
                .push(format!("`{path}`")),
            "opens" => opens.push((path.to_string(), value.to_owned())),
            _ => {}
        });
    }
    let _ = writeln!(out, "## Commands\n");
    if commands.is_empty() {
        let _ = writeln!(out, "No composite runs a command yet.\n");
    } else {
        let _ = writeln!(out, "| command | run from |\n|---|---|");
        for (command, from) in &commands {
            let _ = writeln!(out, "| `{command}` | {} |", from.join(", "));
        }
        out.push('\n');
    }
    if !opens.is_empty() {
        let _ = writeln!(out, "## Interactions\n");
        for (from, overlay) in &opens {
            let _ = writeln!(out, "- `{from}` opens `{overlay}`");
        }
        out.push('\n');
    }

    // State and live data.
    let _ = writeln!(out, "## State and events\n");
    let mut state_lines = Vec::new();
    for (name, shell) in &doc.shells {
        if let Some(Value::Object(state)) = shell.extra.get("state") {
            state_lines.push(format!(
                "- shell `{name}`: {}",
                state
                    .keys()
                    .map(|k| format!("`{k}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    for (name, page) in &doc.pages {
        if let Some(Value::Object(state)) = page.extra.get("state") {
            state_lines.push(format!(
                "- page `{name}`: {}",
                state
                    .keys()
                    .map(|k| format!("`{k}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    if let Some(Value::Object(channels)) = doc.extra.get("channels") {
        state_lines.push(format!(
            "- channels (live events): {}",
            channels
                .keys()
                .map(|k| format!("`{k}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if state_lines.is_empty() {
        let _ = writeln!(
            out,
            "The document declares no UI state and no live channels. Entity state machines and \
             domain events belong to the ESS model `{}`, not to this document.\n",
            doc.model
        );
    } else {
        for line in state_lines {
            let _ = writeln!(out, "{line}");
        }
        out.push('\n');
    }

    // Findings.
    let _ = writeln!(out, "## Findings\n");
    let errors = findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .count();
    let _ = writeln!(
        out,
        "{errors} error(s), {} warning(s).\n",
        findings.len() - errors
    );
    for f in findings {
        let _ = writeln!(
            out,
            "- {:?} `{}` at `{}`: {}",
            f.severity, f.check, f.path, f.message
        );
    }
    out
}

/// Calls `visit(key, value)` for every string value under a JSON value.
fn collect(value: &Value, visit: &mut impl FnMut(&str, &str)) {
    match value {
        Value::Object(map) => {
            for (key, v) in map {
                match v {
                    Value::String(s) => visit(key, s),
                    other => collect(other, visit),
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|v| collect(v, visit)),
        _ => {}
    }
}

/// What uilab and its agent can do, from the same tables the checks and the patch schema use.
pub fn help_markdown() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# What uilab can do\n");
    let _ = writeln!(
        out,
        "Select a node, then say or type one instruction. The agent answers with **one proposal** at \
         that node; you accept or reject it, and undo an accepted one. Each instruction is one agent \
         turn: there is no multi-step goal yet.\n"
    );
    let _ = writeln!(out, "## Where things can go\n");
    let _ = writeln!(out, "| selected | can hold |\n|---|---|");
    for (layer, note) in [
        (Layer::Root, "the document"),
        (Layer::Shell, "an application frame"),
        (Layer::Nav, "the menu"),
        (Layer::Page, "a route"),
        (
            Layer::Section,
            "a region of a page; a board holds widgets, a collection holds items per row",
        ),
    ] {
        let children: Vec<&str> = [
            Layer::Shell,
            Layer::Region,
            Layer::NavSection,
            Layer::Page,
            Layer::Section,
            Layer::Overlay,
            Layer::Widget,
            Layer::Item,
        ]
        .into_iter()
        .filter(|c| crate::path::may_contain(layer, *c))
        .map(Layer::as_str)
        .collect();
        let _ = writeln!(
            out,
            "| {} ({note}) | {} |",
            layer.as_str(),
            children.join(", ")
        );
    }
    let _ = writeln!(out, "\n## Component kinds\n");
    for kind in CompositeKind::ALL {
        let _ = writeln!(out, "- **{}**: {}", kind.as_str(), kind.summary());
    }
    let _ = writeln!(out, "\n## What the agent proposes\n");
    let _ = writeln!(
        out,
        "- **insert** a new child under the selected node\n\
         - **replace** the selected node (change its columns, title, fields, actions)\n\
         - **remove** the selected node\n\
         - **batch**: several of those checked together, when one instruction touches more than one \
         node (a drawer plus the row action that opens it)\n\n\
         Data comes from ESS views. When no view holds what you asked for, the agent reads a \
         placeholder `draft.<Name>` view; the canvas fills it with sample rows and the findings list \
         it until the model has the view.\n\n\
         Every proposal is checked before you see it: names resolve, the menu lists every page, \
         `opens` names an overlay that exists, columns name fields the view has."
    );
    let _ = writeln!(out, "\n## Examples\n");
    for example in [
        "add a table of overdue loans with title, member and due date",
        "also show the member in this table",
        "add a drawer to add a new member with name and email",
        "add an edit action on each row that opens a drawer",
        "add a page for reservations in the circulation menu",
        "remove this",
    ] {
        let _ = writeln!(out, "- \"{example}\"");
    }
    out
}
