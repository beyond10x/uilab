//! story:essui-agent-schema: patches the agent's schema accepts are patches ESS admits, and a
//! batch must carry its nodes. Values are validated with the harness's own validator
//! ([`OutputSchema::validate`]), the one the agent's answer is held to; that is why these tests
//! live in `uilab-agent` and not beside `schema_from_ess` in `uilab-doc`, which has no validator.

use std::path::Path;

use harness_loop::OutputSchema;
use serde_json::{Value, json};
use serde_yaml::Value as Yaml;
use uilab_doc::{Document, NodePath, Patch, Severity, admit, check, ess_ui, patch_schema};

fn library() -> Document {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library");
    let text = std::fs::read_to_string(dir.join("library.ui.yaml")).unwrap();
    Document::from_yaml_in(&text, &dir).unwrap()
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

fn ess() -> Yaml {
    serde_yaml::from_str(ess_ui::SCHEMA).expect("the embedded ESS schema is YAML")
}

fn strings(value: &Yaml) -> Vec<String> {
    value
        .as_sequence()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

/// Every error finding of the document: ESS's checker and uilab's own checks.
fn errors(doc: &Document) -> Vec<String> {
    check(doc)
        .into_iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| format!("{} at {}: {}", f.check, f.path, f.message))
        .collect()
}

/// The patch is valid against the schema the agent is sent at its target.
fn valid(doc: &Document, patch: &Value) -> Result<(), String> {
    let target = path(patch["target"].as_str().unwrap());
    let schema = patch_schema(doc, &target).map_err(|e| e.to_string())?;
    OutputSchema::new(schema)
        .map_err(|e| e.to_string())?
        .validate(patch)
}

/// Valid against the schema, admitted, and leaving no error finding.
fn passes(doc: &Document, patch: &Value) -> Document {
    if let Err(why) = valid(doc, patch) {
        panic!("the schema refuses {patch}: {why}");
    }
    let parsed: Patch = serde_json::from_value(patch.clone())
        .unwrap_or_else(|e| panic!("not a patch: {patch}: {e}"));
    let (next, _) = admit(doc, &parsed).unwrap_or_else(|r| panic!("refused {patch}: {r:?}"));
    let errors = errors(&next);
    assert!(errors.is_empty(), "{patch} leaves errors: {errors:#?}");
    next
}

fn def<'a>(schema: &'a Value, reference: &Value) -> &'a Value {
    let name = reference
        .as_str()
        .and_then(|r| r.strip_prefix("#/$defs/"))
        .unwrap_or_else(|| panic!("not a local $ref: {reference}"));
    schema["$defs"]
        .get(name)
        .unwrap_or_else(|| panic!("no $defs entry `{name}`"))
}

fn child_node<'a>(schema: &'a Value, layer: &str) -> &'a Value {
    let variant = schema["properties"]["child"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["properties"]["layer"]["const"] == layer)
        .unwrap_or_else(|| panic!("no `{layer}` child offered"));
    def(schema, &variant["properties"]["node"]["$ref"])
}

/// The variant of a discriminated definition for `tag == value`.
fn variant<'a>(schema: &'a Value, union: &'a Value, tag: &str, value: &str) -> &'a Value {
    let then = union["allOf"]
        .as_array()
        .unwrap_or_else(|| panic!("no `allOf` in {union}"))
        .iter()
        .find(|entry| entry["if"]["properties"][tag]["const"] == value)
        .map(|entry| &entry["then"])
        .unwrap_or_else(|| panic!("`{tag}: {value}` is not offered"));
    // A variant written in place names its fields; one that is only a reference is elsewhere.
    match (then.get("properties"), then.get("$ref")) {
        (None, Some(reference)) => def(schema, reference),
        _ => then,
    }
}

/// The definitions a nested node may be: the composite one and the primitive one.
fn nested<'a>(schema: &'a Value, node: &'a Value, tag: &str) -> &'a Value {
    node["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| def(schema, &r["$ref"]))
        .find(|d| d["properties"].get(tag).is_some())
        .unwrap_or_else(|| panic!("a nested node may carry `{tag}`"))
}

/// The smallest node a position's variant accepts. A variant names its fields (`true`) and types
/// none of the frame's: those are typed on the position, the kind's own in the definition the
/// variant refers to. Each required field is generated from whichever types it.
fn example_variant(schema: &Value, position: &Value, variant: &Value) -> Value {
    let kind = variant
        .get("$ref")
        .map(|r| def(schema, r))
        .unwrap_or(&Value::Null);
    let mut typed = variant.clone();
    let properties = typed["properties"].as_object_mut().unwrap();
    for (key, field) in properties.iter_mut() {
        if *field == json!(true) {
            *field = [&kind["properties"][key], &position["properties"][key]]
                .into_iter()
                .find(|t| !t.is_null())
                .cloned()
                .unwrap_or_else(|| panic!("`{key}` is typed nowhere"));
        }
    }
    typed.as_object_mut().unwrap().remove("$ref");
    example(schema, &typed)
}

/// The smallest value `at` accepts, generated from the schema alone: every required field, the
/// first of every choice and of every "exactly one of", the first value of every enum.
fn example(schema: &Value, at: &Value) -> Value {
    if let Some(reference) = at.get("$ref") {
        return example(schema, def(schema, reference));
    }
    if let Some(value) = at.get("const") {
        return value.clone();
    }
    if let Some(values) = at.get("enum") {
        return values[0].clone();
    }
    if at.get("type").is_none() && at.get("properties").is_none() {
        for union in ["anyOf", "oneOf"] {
            if let Some(first) = at.get(union).and_then(|u| u.get(0)) {
                return example(schema, first);
            }
        }
    }
    let kind = match &at["type"] {
        Value::String(kind) => kind.as_str(),
        Value::Array(kinds) => kinds[0].as_str().unwrap(),
        _ if at.get("properties").is_some() => "object",
        _ => return json!("sample"),
    };
    match kind {
        "object" => {
            let mut wanted: Vec<String> = at["required"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect();
            if let Some(first) = at["oneOf"].get(0) {
                for key in first["required"].as_array().into_iter().flatten() {
                    wanted.push(key.as_str().unwrap().to_owned());
                }
            }
            // A record (closed, `required` left out: ESS's records do not say which fields its
            // loader defaults) is written with every field it names.
            if at.get("required").is_none() && at["additionalProperties"] == json!(false) {
                for key in at["properties"]
                    .as_object()
                    .into_iter()
                    .flatten()
                    .map(|(k, _)| k)
                {
                    wanted.push(key.clone());
                }
            }
            // `if not <keys> then <required>`: none of those keys is generated, so the `then` holds.
            if at["if"]["not"].get("required").is_some() {
                for key in at["then"]["required"].as_array().into_iter().flatten() {
                    wanted.push(key.as_str().unwrap().to_owned());
                }
            }
            let mut out = serde_json::Map::new();
            for (key, field) in at["properties"].as_object().into_iter().flatten() {
                if wanted.contains(key) {
                    out.insert(key.clone(), example(schema, field));
                }
            }
            for key in wanted {
                out.entry(key).or_insert_with(|| json!("sample"));
            }
            Value::Object(out)
        }
        "array" => {
            let n = at["minItems"].as_u64().unwrap_or(0);
            (0..n).map(|_| example(schema, &at["items"])).collect()
        }
        "string" => match at["pattern"].as_str() {
            Some(pattern) if pattern.contains("ms|s") => json!("30s"),
            _ => json!("sample"),
        },
        "integer" | "number" => json!(1),
        "boolean" => json!(false),
        _ => json!("sample"),
    }
}

/// The library with one widget declared, declared by a patch the root's schema accepts.
fn with_widget() -> Document {
    passes(
        &library(),
        &json!({
            "op": "insert",
            "target": "/",
            "child": {
                "layer": "component",
                "name": "due_badge",
                "node": {
                    "summary": "A loan's due date in a toned pill.",
                    "params": {
                        "due": {"type": "date", "required": true, "note": "the due date"},
                    },
                    "body": [{"name": "pill", "primitive": "badge", "text": "args.due"}],
                },
            },
        }),
    )
}

fn insert(target: &str, layer: &str, name: &str, node: Value) -> Value {
    json!({"op": "insert", "target": target, "child": {"layer": layer, "name": name, "node": node}})
}

#[test]
fn schema_patches_pass_ess() {
    let ess = ess();
    let members = strings(&ess["constructs"]["Composite"]["union"]["members"]);
    let primitives =
        strings(&ess["constructs"]["Primitive"]["fields"]["primitive"]["type"]["enum"]);
    let doc = with_widget();
    let mut kinds: Vec<String> = members.clone();
    kinds.push("due_badge".into());
    let mut passed = 0;

    // Every composite kind and the widget: as a section and as an overlay of a page.
    let page = patch_schema(&doc, &path("page:loans")).unwrap();
    for layer in ["section", "overlay"] {
        let position = child_node(&page, layer);
        for kind in &kinds {
            let node =
                example_variant(&page, position, variant(&page, position, "component", kind));
            passes(
                &doc,
                &insert("page:loans", layer, &format!("as_{kind}"), node),
            );
            passed += 1;
        }
    }

    // Every composite kind, the widget and every primitive kind: as an item of a collection.
    let list = patch_schema(&doc, &path("page:loans/section:list")).unwrap();
    let node = child_node(&list, "item");
    for (tag, kinds) in [("component", &kinds), ("primitive", &primitives)] {
        let position = nested(&list, node, tag);
        for kind in kinds {
            let node = example_variant(&list, position, variant(&list, position, tag, kind));
            passes(
                &doc,
                &insert(
                    "page:loans/section:list",
                    "item",
                    &format!("as_{kind}"),
                    node,
                ),
            );
            passed += 1;
        }
    }
    assert_eq!(passed, 2 * 13 + 13 + 9);

    // A valid batch: a drawer and the row action that opens it, each patch carrying its node.
    let overview = patch_schema(&doc, &path("page:overview")).unwrap();
    let overlay = child_node(&overview, "overlay");
    let drawer = example_variant(
        &overview,
        overlay,
        variant(&overview, overlay, "component", "record"),
    );
    let recent = path("page:overview/section:recent");
    let mut recent_node = serde_json::to_value(
        uilab_doc::resolve(&doc, &recent)
            .unwrap()
            .composite()
            .unwrap(),
    )
    .unwrap();
    recent_node["row_actions"] = json!([{"name": "peek", "opens": "peek", "label": "Peek"}]);
    let batch = json!({
        "op": "batch",
        "target": "page:overview",
        "patches": [
            insert("page:overview", "overlay", "peek", drawer),
            {"op": "replace", "target": recent.to_string(), "node": recent_node},
        ],
    });
    let next = passes(&doc, &batch);
    assert!(uilab_doc::resolve(&next, &path("page:overview/overlay:peek")).is_ok());

    // A field ESS does not declare is not offered, and ESS's admission refuses it: the schema
    // guides, admission judges (correction 1, beyond10x/ess#305).
    let section = child_node(&page, "section");
    let metric = variant(&page, section, "component", "metric");
    assert!(
        metric["properties"].get("title").is_none(),
        "a section `title` is not offered"
    );
    let mut titled = example_variant(&page, section, metric);
    titled["title"] = json!("Copies on loan");
    let patch = insert("page:loans", "section", "titled", titled);
    let parsed: Patch = serde_json::from_value(patch.clone()).unwrap();
    let refused = admit(&doc, &parsed);
    assert!(refused.is_err(), "ESS refuses a section `title`: {patch}");
}

/// Round 4, case `creative` ("show me how creative you can be" at `page:overview`): the answer
/// was a batch whose patches held no node, reported as "batch holds no node". The raw answer was
/// not recorded (`evals/reports/library-round-4.json` keeps the reason only); this is its shape.
#[test]
fn batch_patches_carry_nodes() {
    let doc = library();
    let round_4 = json!({
        "op": "batch",
        "target": "page:overview",
        "patches": [
            {"op": "insert", "target": "page:overview"},
            {"op": "replace", "target": "page:overview/section:recent"},
        ],
    });
    let refused = valid(&doc, &round_4);
    assert!(refused.is_err(), "the round-4 answer is refused: {round_4}");

    // The class: every insert needs its `child` and every child its `node`, every replace its
    // `node`, every batch its `patches`, every decline its `reason`; in a batch and at the top.
    let metric =
        json!({"component": "metric", "reads": {"view": "loans.Summary"}, "from": "on_loan"});
    let refused_alone = [
        json!({"op": "insert", "target": "page:overview"}),
        json!({"op": "insert", "target": "page:overview", "child": {"layer": "section", "name": "more"}}),
        json!({"op": "replace", "target": "page:overview/section:on_loan"}),
        json!({"op": "batch", "target": "page:overview"}),
        json!({"op": "decline", "target": "page:overview"}),
    ];
    for patch in &refused_alone {
        assert!(valid(&doc, patch).is_err(), "refused at the top: {patch}");
    }
    let carried = [
        insert("page:overview", "section", "more", metric.clone()),
        json!({"op": "replace", "target": "page:overview/section:on_loan", "node": metric}),
    ];
    for (i, patch) in refused_alone.iter().take(3).enumerate() {
        let batch = json!({
            "op": "batch",
            "target": "page:overview",
            "patches": [carried[(i + 1) % 2].clone(), patch],
        });
        assert!(valid(&doc, &batch).is_err(), "refused in a batch: {patch}");
    }

    // A node in a batch is not held to its layer's shape (a batch may declare a widget and use it,
    // which a shape naming only the declared widgets would refuse); admission checks it. It must
    // hold something.
    for empty in [
        insert("page:overview", "section", "more", json!({})),
        json!({"op": "replace", "target": "page:overview/section:on_loan", "node": {}}),
    ] {
        let batch = json!({"op": "batch", "target": "page:overview", "patches": [carried[0].clone(), empty]});
        assert!(
            valid(&doc, &batch).is_err(),
            "an empty node in a batch: {batch}"
        );
    }

    // And the same batch with its nodes is accepted and admitted.
    let batch = json!({"op": "batch", "target": "page:overview", "patches": carried});
    passes(&doc, &batch);
}

/// A node as the document holds it, in the shape a replace writes.
fn written(node: uilab_doc::path::NodeRef<'_>) -> Option<Value> {
    use uilab_doc::path::NodeRef;
    let value = match node {
        NodeRef::Root(_) | NodeRef::Nav(_) => return None,
        NodeRef::Shell(shell) => serde_json::to_value(shell),
        NodeRef::Region(region) => serde_json::to_value(region),
        NodeRef::NavSection(section) => serde_json::to_value(section),
        NodeRef::Page(page) => serde_json::to_value(page),
        NodeRef::Overlay(overlay) => serde_json::to_value(overlay),
        NodeRef::Composite(composite) => serde_json::to_value(composite),
        NodeRef::Component(widget) => serde_json::to_value(widget),
        NodeRef::Primitive(primitive) => serde_json::to_value(primitive),
    };
    Some(value.unwrap())
}

fn paths(node: &uilab_doc::OutlineNode, out: &mut Vec<String>) {
    out.push(node.path.clone());
    for child in &node.children {
        paths(child, out);
    }
}

/// Every node of `doc` but the root and `nav`, written back unchanged as a replace at its path:
/// how many were checked, and the ones the schema refuses.
fn as_written(doc: &Document) -> (usize, Vec<String>) {
    let mut all = Vec::new();
    paths(&uilab_doc::outline(doc), &mut all);
    let mut checked = 0;
    let mut refused = Vec::new();
    for at in &all {
        let Some(node) = written(uilab_doc::resolve(doc, &path(at)).unwrap()) else {
            continue;
        };
        let patch = json!({"op": "replace", "target": at, "node": node});
        if let Err(why) = valid(doc, &patch) {
            refused.push(format!("{at}: {why}"));
        }
        checked += 1;
    }
    assert_eq!(checked, all.len() - 2, "every node but the root and `nav`");
    (checked, refused)
}

/// The other direction: the schema is no stricter than ESS. Every node of a document ESS loads
/// and checks without an error is valid against the schema at that node when written back
/// unchanged, as a replace that keeps every child does: ESS's own reference example, which
/// "exercises every construct of the schema, and most shorthands", and the committed library.
#[test]
fn the_library_as_written_is_valid_against_its_schema() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../uilab-doc/tests/fixtures/partner-portal");
    let text = std::fs::read_to_string(dir.join("ui.yaml")).unwrap();
    let portal = Document::from_yaml_in(&text, &dir).unwrap();
    let library = with_widget();
    assert!(errors(&library).is_empty(), "{:#?}", errors(&library));

    let mut refused = Vec::new();
    for (name, doc, least) in [("partner-portal", &portal, 60), ("library", &library, 15)] {
        let (checked, mut wrong) = as_written(doc);
        assert!(checked >= least, "{name}: {checked} nodes checked");
        refused.extend(wrong.drain(..).map(|w| format!("{name} {w}")));
    }
    assert!(
        refused.is_empty(),
        "the schema refuses nodes ESS admits:\n{}",
        refused.join("\n")
    );
}
