//! What the operator sees and what the agent is told about one node.

use serde::Serialize;

use crate::check::{Instance, reaches, widget_uses};
use crate::model::{CompositeKind, Document};
use crate::path::{Layer, NodePath, NodeRef, PathError, allowed_children, children, resolve};

/// One node of the document tree, with its children in document order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutlineNode {
    /// Where the node sits.
    pub path: String,
    /// Its layer.
    pub layer: Layer,
    /// Its name; empty for the root and `nav`.
    pub name: String,
    /// Its kind: page kind, region kind, composite kind.
    pub kind: String,
    /// Its title or label, where it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The view it reads: a composite's or an overlay's `reads`, a region's `props.reads`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    /// A composite's props other than `component`, `reads`, `widgets` and `item`: what a renderer
    /// needs to draw it (columns, title, from, fields, a widget instance's args). A primitive's
    /// props; a widget's params, arrangement and `uses`: each instance of it as `{path, trail?}`,
    /// the node that holds it and the way through that node's untyped data, as the widget checks
    /// and the docs find them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Value>,
    /// Its children.
    pub children: Vec<OutlineNode>,
}

/// The whole document as a tree, from the root.
pub fn outline(doc: &Document) -> OutlineNode {
    node(doc, &widget_uses(doc), &NodePath::root()).expect("the root resolves")
}

/// The use sites of the widget `name`, one per instance, in the order [`widget_uses`] finds them.
fn uses_of(uses: &[(NodePath, Instance<'_>)], name: &str) -> serde_json::Value {
    let mut out: Vec<serde_json::Value> = Vec::new();
    for (path, instance) in uses.iter().filter(|(_, i)| i.widget == name) {
        let site = match &instance.trail {
            Some(trail) => serde_json::json!({"path": path.to_string(), "trail": trail}),
            None => serde_json::json!({"path": path.to_string()}),
        };
        out.push(site);
    }
    serde_json::Value::Array(out)
}

fn node(
    doc: &Document,
    uses: &[(NodePath, Instance<'_>)],
    path: &NodePath,
) -> Result<OutlineNode, PathError> {
    let found = resolve(doc, path)?;
    // Composites carry their own props. The menu, its sections and pages carry what a renderer
    // needs to place them: the home page, a section's pages, a page's shell.
    let props = match found {
        NodeRef::Nav(n) => Some(serde_json::json!({"home": n.home})),
        NodeRef::NavSection(s) => Some(match &s.pages {
            crate::model::NavPages::Fixed(pages) => serde_json::json!({"pages": pages}),
            crate::model::NavPages::Dynamic(entries) => {
                serde_json::json!({"from_view": entries.get("from_view"), "page": entries.get("page")})
            }
        }),
        NodeRef::Page(p) => doc
            .shell_of(p)
            .map(|shell| serde_json::json!({"shell": shell})),
        NodeRef::Component(w) => Some(serde_json::json!({
            "params": w.params,
            "arrange": w.arrangement(),
            "uses": uses_of(uses, path.name()),
        })),
        NodeRef::Primitive(p) => Some(serde_json::Value::Object(
            p.props.clone().into_iter().collect(),
        )),
        _ => found
            .composite()
            .filter(|c| !c.props.is_empty())
            .map(|c| serde_json::Value::Object(c.props.clone().into_iter().collect())),
    };
    let (title, view) = match found {
        NodeRef::Root(d) => (d.title.clone(), None),
        NodeRef::Page(p) => (p.title.clone(), None),
        NodeRef::NavSection(s) => (s.label.clone(), None),
        NodeRef::Component(w) => (Some(w.summary.clone()), None),
        NodeRef::Overlay(o) => (
            o.body
                .props
                .get("title")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            o.body.reads.as_ref().map(|r| r.name().to_owned()),
        ),
        NodeRef::Composite(c) => (
            c.props
                .get("title")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            c.reads.as_ref().map(|r| r.name().to_owned()),
        ),
        NodeRef::Region(r) => (
            None,
            r.props
                .get("reads")
                .and_then(|reads| reads.get("view"))
                .and_then(|v| v.as_str())
                .map(str::to_owned),
        ),
        _ => (None, None),
    };
    let mut kids = Vec::new();
    for (layer, name) in children(doc, path)? {
        kids.push(node(doc, uses, &path.child(layer, &name))?);
    }
    Ok(OutlineNode {
        path: path.to_string(),
        layer: path.layer(),
        name: path.name().to_owned(),
        kind: found.kind_label(),
        title,
        view,
        props,
        children: kids,
    })
}

/// What an agent needs to know to propose a patch at one node.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NodeContext<'a> {
    /// The node.
    pub path: String,
    /// Its layer.
    pub layer: Layer,
    /// Its kind.
    pub kind: String,
    /// From the root down to its parent, each as `path (kind)`.
    pub ancestors: Vec<String>,
    /// Layers a new child can take here.
    pub allowed_children: Vec<Layer>,
    /// What a new composite child can be: every built-in kind, then every declared widget that
    /// would not make a widget contain itself.
    pub composite_kinds: Vec<&'a str>,
    /// Existing children, as `layer:name`.
    pub children: Vec<String>,
    /// The node itself, as YAML.
    pub yaml: String,
}

/// The context of the node at `path`.
pub fn node_context<'a>(doc: &'a Document, path: &NodePath) -> Result<NodeContext<'a>, PathError> {
    let found = resolve(doc, path)?;
    let allowed = allowed_children(doc, path)?;
    let composite_kinds = if allowed.iter().any(|l| l.is_composite()) {
        let within = path
            .0
            .first()
            .filter(|s| s.layer == Layer::Component)
            .map(|s| s.name.as_str());
        CompositeKind::ALL
            .iter()
            .map(|k| k.as_str())
            .chain(
                doc.widgets
                    .keys()
                    .map(String::as_str)
                    .filter(|w| within.is_none_or(|own| *w != own && !reaches(doc, w, own))),
            )
            .collect()
    } else {
        Vec::new()
    };
    let mut ancestors = Vec::new();
    for ancestor in path
        .lineage()
        .into_iter()
        .rev()
        .skip(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let kind = resolve(doc, &ancestor)?.kind_label();
        ancestors.push(format!("{ancestor} ({kind})"));
    }
    let children = children(doc, path)?
        .into_iter()
        .map(|(layer, name)| {
            if layer == Layer::Nav {
                "nav".to_owned()
            } else {
                format!("{layer}:{name}")
            }
        })
        .collect();
    Ok(NodeContext {
        path: path.to_string(),
        layer: path.layer(),
        kind: found.kind_label(),
        ancestors,
        allowed_children: allowed,
        composite_kinds,
        children,
        yaml: node_yaml(found),
    })
}

/// The node at `path` as YAML, or `None` when no node has that path.
pub fn yaml_at(doc: &Document, path: &NodePath) -> Option<String> {
    resolve(doc, path).ok().map(node_yaml)
}

fn node_yaml(node: NodeRef<'_>) -> String {
    let text = match node {
        NodeRef::Root(d) => crate::model::to_yaml(&serde_json::json!({
            "app": d.app, "title": d.title, "model": d.model,
            "shells": d.shells.keys().collect::<Vec<_>>(),
            "pages": d.pages.keys().collect::<Vec<_>>(),
        })),
        NodeRef::Shell(s) => crate::model::to_yaml(s),
        NodeRef::Region(r) => crate::model::to_yaml(r),
        NodeRef::Nav(n) => crate::model::to_yaml(n),
        NodeRef::NavSection(s) => crate::model::to_yaml(s),
        NodeRef::Page(p) => crate::model::to_yaml(p),
        NodeRef::Overlay(o) => crate::model::to_yaml(o),
        NodeRef::Composite(c) => crate::model::to_yaml(c),
        NodeRef::Component(w) => crate::model::to_yaml(w),
        NodeRef::Primitive(p) => crate::model::to_yaml(p),
    };
    text.unwrap_or_default()
}

/// Words the speech model should expect at this node: layer and kind names and the names of the
/// node's ancestors and children. A speech model given these as a prompt spells them as written.
pub fn vocabulary(doc: &Document, path: &NodePath) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut add = |w: &str| {
        let w = w.replace('_', " ");
        if !w.is_empty() && !words.contains(&w) {
            words.push(w);
        }
    };
    if let Ok(context) = node_context(doc, path) {
        context
            .allowed_children
            .iter()
            .for_each(|l| add(l.as_str()));
        context.composite_kinds.iter().for_each(|k| add(k));
        for child in &context.children {
            add(child.split_once(':').map_or(child.as_str(), |(_, n)| n));
        }
    }
    for segment in &path.0 {
        add(&segment.name);
    }
    for page in doc.pages.keys() {
        add(page);
    }
    words
}

/// The outline of the subtree at `path`, or `None` when no node has that path.
pub fn outline_at(doc: &Document, path: &NodePath) -> Option<OutlineNode> {
    node(doc, &widget_uses(doc), path).ok()
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::{Fixtures, docs_markdown};

    const WIDGETS: &str = "widgets:
  loan_card:
    summary: A loan as a card.
    params:
      loan: {type: Loan, required: true, note: the loan row}
    body:
      - {name: title, primitive: text, text: args.loan.title, style: heading}
      - {name: due, primitive: badge, text: args.loan.due}
  badge:
    summary: A toned tag.
    params:
      label: {type: string, required: true, note: the tag text}
    body:
      - {name: tag, primitive: badge, text: args.label}
  unused:
    summary: Nothing uses it.
    body:
      - {name: tag, primitive: text, text: nothing}
pages:
";

    /// The library example with the widgets `loan_card`, `badge` and `unused`, none of them used.
    fn library() -> Document {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/library/library.ui.yaml");
        let text = std::fs::read_to_string(file).unwrap();
        assert!(text.contains("\npages:\n"), "fixture anchor is missing");
        Document::from_yaml(&text.replacen("\npages:\n", &format!("\n{WIDGETS}"), 1)).unwrap()
    }

    /// The `uses` prop of the widget `name`'s outline node.
    fn uses(doc: &Document, name: &str) -> Value {
        let root = outline(doc);
        let widget = root
            .children
            .iter()
            .find(|c| c.path == format!("component:{name}"))
            .unwrap_or_else(|| panic!("no widget `{name}` in the outline"));
        widget.props.as_ref().expect("a widget has props")["uses"].clone()
    }

    #[test]
    fn a_widget_used_only_in_a_page_header_carries_that_use() {
        let mut doc = library();
        doc.pages["overview"].extra.insert(
            "header".into(),
            json!({"metrics": [{"name": "due", "component": "loan_card", "args": {"loan": "rows.first"}}]}),
        );
        assert_eq!(
            uses(&doc, "loan_card"),
            json!([{"path": "page:overview", "trail": "header/metrics/due"}])
        );
    }

    #[test]
    fn a_widget_used_only_in_a_page_kind_carries_that_use_at_the_root() {
        let mut doc = library();
        doc.page_kinds.insert(
            "board_page".into(),
            json!({"header": {"metrics": [{"name": "due", "component": "badge", "args": {"label": "row.state"}}]}}),
        );
        assert_eq!(
            uses(&doc, "badge"),
            json!([{"path": "/", "trail": "page_kinds/board_page/header/metrics/due"}])
        );
    }

    /// A primitive `badge` has kind `badge` in the outline, as an instance of the widget `badge`
    /// does; only the instance is a use.
    #[test]
    fn a_widget_named_like_a_primitive_carries_only_its_instances() {
        let mut doc = library();
        let latest: crate::model::Composite =
            serde_json::from_value(json!({"component": "badge", "args": {"label": "rows.first"}}))
                .unwrap();
        doc.pages["overview"]
            .sections
            .insert("latest".into(), Some(latest));
        assert_eq!(
            uses(&doc, "badge"),
            json!([{"path": "page:overview/section:latest"}])
        );
        assert_eq!(uses(&doc, "unused"), json!([]));
    }

    /// A shell region that reads a view (the library's `account` menu reads `staff.Me` through its
    /// props) carries that view in the outline, as a composite does; a region that reads none
    /// carries none.
    #[test]
    fn a_region_carries_the_view_its_props_read() {
        let root = outline(&library());
        let shell = root
            .children
            .iter()
            .find(|c| c.path == "shell:app")
            .expect("the library has the shell `app`");
        let view = |region: &str| {
            shell
                .children
                .iter()
                .find(|c| c.path == format!("shell:app/region:{region}"))
                .unwrap_or_else(|| panic!("no region `{region}`"))
                .view
                .clone()
        };
        assert_eq!(view("account"), Some("staff.Me".to_owned()));
        assert_eq!(view("nav"), None);
        assert_eq!(view("main"), None);
    }

    /// Every widget's `uses` are the use sites the docs list for it, in the same order: the
    /// outline and `/api/docs.md` read one walk.
    #[test]
    fn every_widget_carries_the_use_sites_the_docs_list() {
        let mut doc = library();
        doc.pages["overview"].extra.insert(
            "header".into(),
            json!({"metrics": [
                {"name": "due", "component": "loan_card", "args": {"loan": "rows.first"}},
                {"name": "tag", "component": "badge", "args": {"label": "rows.first"}}
            ]}),
        );
        doc.page_kinds.insert(
            "board_page".into(),
            json!({"sections": {"s": {"component": "loan_card", "args": {"loan": "row"}}}}),
        );
        let docs = docs_markdown(&doc, &Fixtures::default(), &[]);
        for name in doc.widgets.keys() {
            let listed: Vec<String> = uses(&doc, name)
                .as_array()
                .unwrap()
                .iter()
                .map(|u| match u.get("trail") {
                    Some(trail) => format!(
                        "`{}` (`{}`)",
                        u["path"].as_str().unwrap(),
                        trail.as_str().unwrap()
                    ),
                    None => format!("`{}`", u["path"].as_str().unwrap()),
                })
                .collect();
            let line = if listed.is_empty() {
                "Not used yet.".to_owned()
            } else {
                format!("Used at: {}", listed.join(", "))
            };
            let section = docs.split(&format!("### {name}\n")).nth(1).unwrap();
            assert!(
                section.lines().any(|l| l == line),
                "`{name}`: the outline lists {line:?}, the docs do not"
            );
        }
    }
}
