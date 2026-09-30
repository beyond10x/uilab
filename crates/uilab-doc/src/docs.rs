//! Documentation generated from a document: what a reader of the spec needs without reading the
//! YAML. Every section is derived; nothing here is authored per document.

use std::collections::BTreeMap;
use std::fmt::Write;

use serde_json::Value;

use crate::check::{Finding, Severity, composites, widget_uses};
use crate::fixtures::Fixtures;
use crate::model::{CompositeKind, Document, NavPages, NodeBody, PrimitiveKind};
use crate::path::Layer;

/// The document's documentation as Markdown.
pub fn docs_markdown(doc: &Document, fixtures: &Fixtures, findings: &[Finding]) -> String {
    let mut out = String::new();
    let title = doc.title.as_deref().unwrap_or(&doc.app);
    let all = composites(doc);
    let _ = writeln!(out, "# {title}\n");
    let _ = writeln!(
        out,
        "App `{}` · model `{}` · {} shell(s) · {} page(s) · {} composite(s) · placement `{}`\n",
        doc.app,
        doc.model,
        doc.shells.len(),
        doc.pages.len(),
        all.len(),
        spelled(&doc.placement_profile)
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
                o.as_ref()
                    .map(|o| format!("`{n}` ({} {})", spelled(&o.kind), o.body.component.as_str()))
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

    widgets_markdown(&mut out, doc);

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
            "- {} `{}` at `{}`: {}",
            spelled(&f.severity),
            f.check,
            f.path,
            f.message
        );
    }
    out
}

/// An enum as the document and the wire spell it (its serde name), never its `Debug` form.
fn spelled<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// The Widgets section: each declared widget with its summary, params, body and use sites. The
/// use sites are the instances the widget checks see, typed or held in untyped data.
fn widgets_markdown(out: &mut String, doc: &Document) {
    let uses = widget_uses(doc);
    let _ = writeln!(out, "## Widgets\n");
    if doc.widgets.is_empty() {
        let _ = writeln!(out, "The document declares no widgets.\n");
        return;
    }
    let inline = |value: &Value| match value {
        Value::String(s) => format!("`{s}`"),
        other => format!("`{other}`"),
    };
    for (name, widget) in &doc.widgets {
        let _ = writeln!(out, "### {name}\n");
        let _ = writeln!(out, "{}\n", widget.summary);
        if let Some(text) = &widget.doc {
            let _ = writeln!(out, "{text}\n");
        }
        let arrange = serde_json::to_value(widget.arrangement()).unwrap_or_default();
        let _ = writeln!(
            out,
            "Arranged as a {}.\n",
            arrange.as_str().unwrap_or("column")
        );
        if widget.params.is_empty() {
            let _ = writeln!(out, "No params.\n");
        } else {
            let _ = writeln!(
                out,
                "| param | type | required | default | note |\n|---|---|---|---|---|"
            );
            for (param, declared) in &widget.params {
                let _ = writeln!(
                    out,
                    "| `{param}` | {} | {} | {} | {} |",
                    inline(&declared.ty),
                    if declared.is_required() { "yes" } else { "no" },
                    declared.default.as_ref().map_or("-".to_owned(), inline),
                    declared.note.as_deref().unwrap_or("")
                );
            }
            out.push('\n');
        }
        let body: Vec<String> = widget
            .body
            .iter()
            .map(|n| {
                let kind = match &n.body {
                    NodeBody::Composite(c) => c.component.as_str(),
                    NodeBody::Primitive(p) => p.primitive.as_str(),
                };
                format!("`{}` ({kind})", n.name)
            })
            .collect();
        let _ = writeln!(out, "Body: {}\n", body.join(", "));
        let used: Vec<String> = uses
            .iter()
            .filter(|(_, i)| i.widget == name.as_str())
            .map(|(p, i)| match &i.trail {
                Some(trail) => format!("`{p}` (`{trail}`)"),
                None => format!("`{p}`"),
            })
            .collect();
        if used.is_empty() {
            let _ = writeln!(out, "Not used yet.\n");
        } else {
            let _ = writeln!(out, "Used at: {}\n", used.join(", "));
        }
    }
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
         turn; for work that takes several changes, give a goal.\n"
    );
    let _ = writeln!(out, "## Goals\n");
    let _ = writeln!(
        out,
        "Switch the toggle by the text field from **instruction** to **goal**, type what you want \
         done and press Enter. The agent plans it into steps, then proposes them one at a time at \
         the node each step names; you accept or reject each proposal and the next step follows. \
         The goal panel in the sidebar lists the steps and where each stands, and **Stop** ends the \
         run. While a goal runs, no other instruction or goal is taken. A goal is typed; a spoken \
         instruction is always a single instruction.\n"
    );
    let _ = writeln!(out, "## The Components tab\n");
    let _ = writeln!(
        out,
        "Press `4` (or click **Components**) to see the document's widgets with previews. The tab \
         is a workspace of its own: an instruction given there builds widgets. With a widget or one \
         of its body nodes selected, the agent works under the selected widget; otherwise it \
         declares a new widget at the root, under `widgets:`. A page left selected on the canvas \
         does not count there. A goal given on the tab keeps every step to it. `1` returns to the \
         UI, `2` shows the YAML and `3` the generated docs.\n"
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
            "a region of a page; a board holds widgets, a collection or record holds named item nodes per row",
        ),
        (
            Layer::Component,
            "a widget: an app-defined composite with params and a body",
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
            Layer::Component,
            Layer::Node,
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
    let _ = writeln!(out, "\n## Widgets and primitives\n");
    let _ = writeln!(
        out,
        "A **widget** is an app-defined composite, declared under `widgets:` at the root with a \
         `summary`, typed `params` and a `body` of named nodes. Use it wherever a composite kind \
         goes (a section, an overlay, a board widget, an item) as \
         `component: <widget>` with `args` for its params. A widget is never named like a \
         built-in kind and never contains itself.\n\n\
         A node of a widget body or of an item list is a composite, a widget instance, or one \
         of these primitives:\n"
    );
    for kind in PrimitiveKind::ALL {
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
         `opens` names an overlay that exists, columns name fields the view has. The proposal card \
         lists only the findings the proposal brings: those the document did not have before it, \
         and what a replace would drop. The document's own findings stay in the sidebar."
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
