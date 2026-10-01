//! Adversary pass 1 on story:essui-agent-schema: the schema is no stricter than ESS. Every
//! construct of `ess_ui::SCHEMA` carries an `example` taken from ESS's reference document; each is
//! validated, with the validator the agent's answer is held to, against the definition the patch
//! schema offers for that construct, built over the same reference document (so the widgets and
//! pages the examples name are declared).

use std::collections::BTreeMap;
use std::path::Path;

use harness_loop::OutputSchema;
use serde_json::{Map, Value, json};
use serde_yaml::Value as Yaml;
use uilab_doc::{Document, NodePath, ess_ui, patch_schema};

fn portal() -> Document {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../uilab-doc/tests/fixtures/partner-portal");
    let text = std::fs::read_to_string(dir.join("ui.yaml")).unwrap();
    Document::from_yaml_in(&text, &dir).unwrap()
}

/// Every definition the schemas at `/` and at `nav` reach.
fn defs(doc: &Document) -> Map<String, Value> {
    let mut out = Map::new();
    for at in ["/", "nav"] {
        let path: NodePath = at.parse().unwrap();
        let schema = patch_schema(doc, &path).unwrap();
        for (k, v) in schema["$defs"].as_object().unwrap() {
            out.insert(k.clone(), v.clone());
        }
    }
    out
}

fn validate(defs: &Map<String, Value>, def: &str, value: &Value) -> Result<(), String> {
    let schema = json!({
        "type": "object",
        "required": ["v"],
        "properties": {"v": {"$ref": format!("#/$defs/{def}")}},
        "$defs": defs,
    });
    OutputSchema::new(schema)
        .map_err(|e| e.to_string())?
        .validate(&json!({"v": value}))
}

#[test]
fn every_ess_example_is_valid_against_the_schema() {
    let ess: Yaml = serde_yaml::from_str(ess_ui::SCHEMA).unwrap();
    let constructs = ess["constructs"].as_mapping().unwrap();
    let members: Vec<String> = ess["constructs"]["Composite"]["union"]["members"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|m| m.as_str().unwrap().to_owned())
        .collect();
    let leaves: Vec<String> = ess["constructs"]["Primitive"]["fields"]["primitive"]["type"]["enum"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|m| m.as_str().unwrap().to_owned())
        .collect();
    let doc = portal();
    let defs = defs(&doc);

    let mut checked = 0;
    let mut refused: BTreeMap<String, String> = BTreeMap::new();
    for (name, construct) in constructs {
        let name = name.as_str().unwrap();
        let Some(example) = construct.get("example") else {
            continue;
        };
        let example: Value = serde_json::to_value(example).unwrap();
        // The definition the patch schema offers this construct under, and the values to check.
        let (def, values): (&str, Vec<Value>) = match name {
            // A document fragment, a path, and two whose example is a `State`.
            "Document" | "PlacementProfile" | "NodePath" | "StateClass" | "Store" => continue,
            // A widget body node reads `args.<param>` in any field (`body_primitive`).
            n if leaves.iter().any(|l| l == n) && example.to_string().contains("args.") => {
                ("body_primitive", vec![example])
            }
            n if members.iter().any(|m| m == n) || n == "WidgetInstance" || n == "Composite" => {
                // An example that carries the overlay frame (`kind`) is an overlay's body.
                let def = if example.get("kind").is_some() {
                    "overlay"
                } else {
                    "composite"
                };
                (def, vec![example])
            }
            n if leaves.iter().any(|l| l == n) || n == "Primitive" => ("primitive", vec![example]),
            "overlay" => ("overlay", vec![example]),
            "Node" => ("node", vec![example]),
            "Section" => ("section", vec![example]),
            "Page" => ("page", vec![example]),
            "Shell" => ("shell", vec![example]),
            "Region" => ("region", vec![example]),
            "NavSection" => ("nav_section", vec![example]),
            "Widget" => ("widget_declaration", vec![example]),
            // Examples written as a list or a map of the construct.
            "Action" => (name, example.as_array().unwrap().clone()),
            "State" | "Type" => (
                name,
                example.as_object().unwrap().values().cloned().collect(),
            ),
            other => (other, vec![example]),
        };
        if !defs.contains_key(def) {
            continue;
        }
        for (i, value) in values.iter().enumerate() {
            checked += 1;
            if let Err(why) = validate(&defs, def, value) {
                refused.insert(
                    format!("{name}[{i}] as {def}"),
                    format!("{value}\n    {why}"),
                );
            }
        }
    }
    assert!(checked >= 40, "{checked} examples checked");
    assert!(
        refused.is_empty(),
        "ESS's own examples the agent's schema refuses ({} of {checked}):\n{}",
        refused.len(),
        refused
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The schema at `page:loans` grows linearly in the widgets a document declares: the second ten
/// widgets cost no more than the first ten (within 10%).
#[test]
fn the_schema_grows_linearly_in_declared_widgets() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library");
    let base = std::fs::read_to_string(dir.join("library.ui.yaml")).unwrap();
    let with = |n: usize| -> usize {
        let mut text = base.clone();
        if n > 0 {
            text.push_str("widgets:\n");
            for i in 0..n {
                text.push_str(&format!(
                    "  w{i}:\n    summary: Widget {i}.\n    params:\n      a: {{type: string, required: true, note: the a}}\n      b: {{type: string, note: the b}}\n    body: [{{name: t, primitive: text, text: args.a}}]\n"
                ));
            }
        }
        let doc = Document::from_yaml_in(&text, &dir).unwrap();
        let path: NodePath = "page:loans".parse().unwrap();
        serde_json::to_string(&patch_schema(&doc, &path).unwrap())
            .unwrap()
            .len()
    };
    let (s0, s10, s20) = (with(0), with(10), with(20));
    eprintln!("schema at page:loans: 0 widgets {s0} bytes, 10 widgets {s10}, 20 widgets {s20}");
    assert!(
        (s20 - s10) as f64 <= 1.1 * (s10 - s0) as f64,
        "0: {s0}, 10: {s10}, 20: {s20}"
    );
}
