//! story:essui-agent-schema: the instruction suite `uilab op eval` runs is stated in `ess-ui/1`
//! terms. It reads `evals/library.yaml` directly, since `uilab-app`, which runs it, is a binary
//! crate, and resolves each target in the committed library example with uilab's own resolver.

use std::path::Path;

use serde_yaml::Value;
use uilab_doc::{Document, Layer, NodePath, resolve};

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn suite() -> Value {
    let text = std::fs::read_to_string(root().join("evals/library.yaml")).unwrap();
    serde_yaml::from_str(&text).unwrap()
}

/// The document the suite names (`document:`), relative to the repository.
fn library(suite: &Value) -> Document {
    let named = suite["document"]
        .as_str()
        .expect("the suite names its document");
    let file = root().join(named);
    let text = std::fs::read_to_string(&file)
        .unwrap_or_else(|error| panic!("{}: {error}", file.display()));
    let doc = Document::from_yaml_in(&text, file.parent().unwrap())
        .unwrap_or_else(|error| panic!("{}: {error}", file.display()));
    assert!(
        text.lines().any(|line| line.trim() == "format: ess-ui/1"),
        "{} is not an ess-ui/1 document",
        file.display()
    );
    doc
}

#[test]
fn eval_suite_is_ess_ui() {
    let suite = suite();
    let doc = library(&suite);
    let cases = suite["cases"].as_sequence().expect("the suite has cases");
    assert!(!cases.is_empty(), "the suite has no cases");
    let mut wrong = Vec::new();
    for case in cases {
        let id = case["id"].as_str().expect("every case has an id");
        let target = case["target"].as_str().expect("every case has a target");
        let say = case["say"].as_str().expect("every case has a say");
        let path: NodePath = match target.parse() {
            Ok(path) => path,
            Err(error) => {
                wrong.push(format!("{id}: `{target}` is no node path: {error}"));
                continue;
            }
        };
        if let Err(error) = resolve(&doc, &path) {
            wrong.push(format!("{id}: the target does not resolve: {error}"));
        }
        let expect = serde_yaml::to_string(&case["expect"]).unwrap();
        if expect.contains("draft.") {
            wrong.push(format!("{id}: `expect` names a `draft.` view: {expect}"));
        }
        if say.to_lowercase().contains("title")
            && !matches!(path.layer(), Layer::Page | Layer::Overlay)
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
