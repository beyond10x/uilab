//! story:essui-agent-schema: the instruction suite `uilab op eval` runs is stated in `ess-ui/1`
//! terms. It reads `evals/library.yaml` directly, since `uilab-app`, which runs it, is a binary
//! crate.

use std::path::{Path, PathBuf};

use serde_yaml::Value;

/// The converted library example the suite's targets resolve in, relative to `$HOME`. Until
/// story:essui-document converts `examples/library/library.ui.yaml`, this is the hand conversion
/// ESS 0.48.0 checks clean, kept outside the repository; the second half of
/// story:essui-agent-schema points the test at the suite's own `document` instead.
const REFERENCE: &str = ".cache/uilab-wave-w10/reference-library.ui.yaml";

fn suite() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../evals/library.yaml");
    serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn library() -> Value {
    let path = PathBuf::from(std::env::var_os("HOME").expect("HOME is set")).join(REFERENCE);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let doc: Value = serde_yaml::from_str(&text).unwrap();
    assert_eq!(
        doc["format"].as_str(),
        Some("ess-ui/1"),
        "{} is not an ess-ui/1 document",
        path.display()
    );
    doc
}

/// The node of a list whose `name` is `name`.
fn named<'a>(list: Option<&'a Value>, name: &str) -> Option<&'a Value> {
    list?
        .as_sequence()?
        .iter()
        .find(|node| node.get("name").and_then(Value::as_str) == Some(name))
}

/// The node a uilab path names in an `ess-ui/1` document: `/` is the root, `nav` the navigation,
/// and each `<layer>:<name>` segment one step down. Sections, nav sections, items and widget
/// body nodes are lists of named nodes; pages, shells, regions, overlays and widgets are maps.
fn resolve<'a>(doc: &'a Value, target: &str) -> Result<&'a Value, String> {
    let mut node = doc;
    let mut parent = "root";
    let trimmed = target.trim().trim_matches('/');
    if trimmed.is_empty() {
        return Ok(doc);
    }
    for part in trimmed.split('/') {
        let (layer, name) = if part == "nav" {
            ("nav", "")
        } else {
            part.split_once(':')
                .ok_or_else(|| format!("`{target}`: `{part}` is not `<layer>:<name>`"))?
        };
        let found = match (parent, layer) {
            ("root", "nav") => node.get("navigation"),
            ("root", "page") => node.get("pages").and_then(|pages| pages.get(name)),
            ("root", "shell") => node.get("shells").and_then(|shells| shells.get(name)),
            ("root", "component") => node.get("widgets").and_then(|widgets| widgets.get(name)),
            ("shell", "region") => node.get("regions").and_then(|regions| regions.get(name)),
            ("page" | "shell", "overlay") => {
                node.get("overlays").and_then(|overlays| overlays.get(name))
            }
            ("page", "section") | ("nav", "nav_section") => named(node.get("sections"), name),
            ("section" | "overlay" | "item", "item") => named(node.get("item"), name),
            ("component", "node") => named(node.get("body"), name),
            _ => return Err(format!("`{target}`: no `{layer}` under a {parent}")),
        };
        node = found.ok_or_else(|| format!("`{target}`: no {layer} `{name}`"))?;
        parent = layer;
    }
    Ok(node)
}

/// The layer of a path's last segment: `root` for `/`.
fn last_layer(target: &str) -> &str {
    match target.trim().trim_matches('/').rsplit('/').next() {
        None | Some("") => "root",
        Some("nav") => "nav",
        Some(part) => part.split_once(':').map_or(part, |(layer, _)| layer),
    }
}

#[test]
fn eval_suite_is_ess_ui() {
    let doc = library();
    let suite = suite();
    let cases = suite["cases"].as_sequence().expect("the suite has cases");
    assert!(!cases.is_empty(), "the suite has no cases");
    let mut wrong = Vec::new();
    for case in cases {
        let id = case["id"].as_str().expect("every case has an id");
        let target = case["target"].as_str().expect("every case has a target");
        let say = case["say"].as_str().expect("every case has a say");
        if let Err(error) = resolve(&doc, target) {
            wrong.push(format!("{id}: the target does not resolve: {error}"));
        }
        let expect = serde_yaml::to_string(&case["expect"]).unwrap();
        if expect.contains("draft.") {
            wrong.push(format!("{id}: `expect` names a `draft.` view: {expect}"));
        }
        if say.to_lowercase().contains("title") && !matches!(last_layer(target), "page" | "overlay")
        {
            wrong.push(format!(
                "{id}: asks for a title at `{target}`, which is not a page or an overlay: {say}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "cases not stated in ess-ui/1 terms:\n{}",
        wrong.join("\n")
    );
}

/// The resolver this test relies on finds what the reference has and refuses what it lacks, so a
/// target that resolves is evidence and not a resolver that accepts everything.
#[test]
fn the_resolver_tells_a_node_from_a_missing_one() {
    let doc = library();
    for target in [
        "/",
        "nav",
        "page:loans",
        "page:loans/section:list",
        "page:loans/overlay:edit",
        "shell:app",
        "shell:app/region:nav",
    ] {
        resolve(&doc, target).unwrap_or_else(|error| panic!("{error}"));
    }
    for target in [
        "page:nowhere",
        "page:loans/section:nowhere",
        "page:loans/overlay:nowhere",
        "shell:app/region:nowhere",
        "page:loans/region:nav",
    ] {
        assert!(resolve(&doc, target).is_err(), "{target} resolved");
    }
    assert_eq!(last_layer("/"), "root");
    assert_eq!(last_layer("nav"), "nav");
    assert_eq!(last_layer("page:loans"), "page");
    assert_eq!(last_layer("page:loans/section:list"), "section");
}
