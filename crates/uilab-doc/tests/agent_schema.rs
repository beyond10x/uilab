//! story:essui-agent-schema: the patch schema the agent's answer is held to takes every field shape
//! from ESS's schema, `ess_ui::SCHEMA`. The expected field sets are read from that schema here, on
//! their own, and the variants are found by following the patch schema the agent is sent, so an
//! ESS release that adds or drops a field fails this test until uilab follows it.
//!
//! The two tests that validate a value against the schema, `schema_patches_pass_ess` and
//! `batch_patches_carry_nodes`, are in `crates/uilab-agent/tests/agent_schema.rs`: they need the
//! validator the agent's harness runs, which `uilab-doc` does not depend on.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::{Value, json};
use serde_yaml::Value as Yaml;
use uilab_doc::{Document, NodePath, Patch, admit, ess_ui, patch_schema};

fn ess() -> Yaml {
    serde_yaml::from_str(ess_ui::SCHEMA).expect("the embedded ESS schema is YAML")
}

fn examples() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples"))
}

fn library() -> Document {
    let dir = examples().join("library");
    let text = std::fs::read_to_string(dir.join("library.ui.yaml")).unwrap();
    Document::from_yaml_in(&text, &dir).unwrap()
}

fn path(text: &str) -> NodePath {
    text.parse().unwrap()
}

/// The library with one widget declared, so the schema offers a widget instance.
fn with_widget() -> Document {
    let declare: Patch = serde_json::from_value(json!({
        "op": "insert",
        "target": "/",
        "child": {
            "layer": "component",
            "name": "due_badge",
            "node": {
                "summary": "A loan's due date in a toned pill.",
                "params": {
                    "due": {"type": "date", "required": true, "note": "the due date"},
                    "tone": {"type": "string", "note": "the pill's tone", "default": "info"},
                },
                "body": [{"name": "pill", "primitive": "badge", "text": "args.due"}],
            },
        },
    }))
    .unwrap();
    admit(&library(), &declare)
        .unwrap_or_else(|refusal| panic!("{refusal:?}"))
        .0
}

fn strings(value: &Yaml) -> Vec<String> {
    value
        .as_sequence()
        .unwrap_or_else(|| panic!("not a list: {value:?}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

/// A construct's fields as ESS declares them: every key, and the keys marked `required: true`.
/// Keys in parentheses describe the construct's value, not a field.
struct Fields {
    all: BTreeSet<String>,
    required: BTreeSet<String>,
}

fn fields(construct: &Yaml) -> Fields {
    let mut out = Fields {
        all: BTreeSet::new(),
        required: BTreeSet::new(),
    };
    for (key, spec) in construct["fields"].as_mapping().unwrap() {
        let key = key.as_str().unwrap();
        if key.starts_with('(') {
            continue;
        }
        out.all.insert(key.to_owned());
        if spec["required"].as_bool() == Some(true) {
            out.required.insert(key.to_owned());
        }
    }
    out
}

/// A field set and the construct's other fields together; a node's `name` is never required in
/// the node itself: a list entry requires it there, and a patch carries it beside the node.
fn joined(frame: &Fields, own: &Fields) -> (BTreeSet<String>, BTreeSet<String>) {
    let all = frame.all.union(&own.all).cloned().collect();
    let mut required: BTreeSet<String> = frame.required.union(&own.required).cloned().collect();
    required.remove("name");
    (all, required)
}

/// The definition a `$ref` names, in `schema`'s `$defs`.
fn def<'a>(schema: &'a Value, reference: &Value) -> &'a Value {
    let name = reference
        .as_str()
        .and_then(|r| r.strip_prefix("#/$defs/"))
        .unwrap_or_else(|| panic!("not a local $ref: {reference}"));
    schema["$defs"]
        .get(name)
        .unwrap_or_else(|| panic!("no $defs entry `{name}`"))
}

/// The `node` the schema offers for inserting a `layer` child at the patch's target.
fn child_node<'a>(schema: &'a Value, layer: &str) -> &'a Value {
    let variant = schema["properties"]["child"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["properties"]["layer"]["const"] == layer)
        .unwrap_or_else(|| panic!("no `{layer}` child offered"));
    def(schema, &variant["properties"]["node"]["$ref"])
}

/// The variant a discriminated definition holds for `tag == value`: the `then` of the one
/// `allOf` entry whose `if` names exactly that value.
fn variant<'a>(schema: &'a Value, union: &'a Value, tag: &str, value: &str) -> &'a Value {
    let found: Vec<&Value> = union["allOf"]
        .as_array()
        .unwrap_or_else(|| panic!("no `allOf` in {union}"))
        .iter()
        .filter(|entry| entry["if"]["properties"][tag]["const"] == value)
        .collect();
    assert_eq!(found.len(), 1, "`{tag}: {value}` is offered once");
    let then = &found[0]["then"];
    // A variant written in place names its fields; one that is only a reference is elsewhere.
    match (then.get("properties"), then.get("$ref")) {
        (None, Some(reference)) => def(schema, reference),
        _ => then,
    }
}

fn keys(object: &Value) -> BTreeSet<String> {
    object
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

fn required(variant: &Value) -> BTreeSet<String> {
    variant["required"]
        .as_array()
        .map(|r| r.iter().map(|v| v.as_str().unwrap().to_owned()).collect())
        .unwrap_or_default()
}

/// One variant offers exactly `all`, requires exactly `required`, refuses any other key, fixes
/// its discriminator, and says "exactly one of" as ESS does.
fn assert_variant(
    at: &str,
    variant: &Value,
    tag: &str,
    value: &str,
    (all, wanted): (BTreeSet<String>, BTreeSet<String>),
    exactly_one_of: Option<&Yaml>,
    same_as: bool,
) {
    assert_eq!(keys(&variant["properties"]), all, "{at}: fields offered");
    if same_as && wanted.len() > 1 {
        // An overlay naming `same_as` is merged over the one it names (`merge_under`): what it
        // must write otherwise, it need not write then.
        assert_eq!(
            required(variant),
            BTreeSet::from([tag.to_owned()]),
            "{at}: with same_as"
        );
        assert_eq!(
            variant["if"],
            json!({"not": {"required": ["same_as"]}}),
            "{at}"
        );
        let mut otherwise = required(&variant["then"]);
        otherwise.insert(tag.to_owned());
        assert_eq!(otherwise, wanted, "{at}: fields required without same_as");
    } else {
        assert_eq!(required(variant), wanted, "{at}: fields required");
    }
    assert_eq!(
        variant["additionalProperties"],
        json!(false),
        "{at}: a field ESS does not declare is refused"
    );
    assert_eq!(variant["properties"][tag], json!({"const": value}), "{at}");
    let one_of: Vec<Value> = exactly_one_of
        .map(|keys| {
            strings(keys)
                .into_iter()
                .map(|k| json!({"required": [k]}))
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        variant["oneOf"].as_array().cloned().unwrap_or_default(),
        one_of,
        "{at}: exactly one of"
    );
}

/// Every field a variant offers is typed: the frame's on the position (which types exactly the
/// frame and the tag), the kind's in the definition the variant refers to (which types exactly
/// the kind's own fields), or in place.
fn assert_typed(
    at: &str,
    schema: &Value,
    position: &Value,
    variant: &Value,
    tag: &str,
    frame: &Fields,
    own: &Fields,
) {
    let mut framed = frame.all.clone();
    framed.insert(tag.to_owned());
    assert_eq!(
        keys(&position["properties"]),
        framed,
        "{at}: the frame's types"
    );
    let kind = def(schema, &variant["$ref"]);
    assert_eq!(keys(&kind["properties"]), own.all, "{at}: the kind's types");
    for (key, offered) in variant["properties"].as_object().unwrap() {
        if *offered == json!(true) {
            assert!(
                kind["properties"].get(key).is_some() || position["properties"].get(key).is_some(),
                "{at}: `{key}` is typed nowhere"
            );
        }
    }
}

#[test]
fn schema_from_ess() {
    let ess = ess();
    let constructs = &ess["constructs"];
    let union = &constructs["Composite"]["union"];
    let tag = union["tag"].as_str().unwrap();
    let members = strings(&union["members"]);
    assert_eq!(members.len(), 12, "ESS 0.48.0 has 12 composite kinds");

    // A section's own fields; its `reads` is the composite's (Section.reads: "it is the
    // composite's own `reads`"), so a section offers one only where the member declares one.
    let mut section = fields(&constructs["Section"]);
    for (key, spec) in constructs["Section"]["fields"].as_mapping().unwrap() {
        if spec["type"].as_str() == Some("Reads") {
            let key = key.as_str().unwrap();
            section.all.remove(key);
            section.required.remove(key);
        }
    }
    let frames = [
        ("page:loans", "section", section),
        ("page:loans", "overlay", fields(&constructs["overlay"])),
        (
            "page:loans/section:list",
            "item",
            fields(&constructs["Composite"]),
        ),
    ];

    let doc = with_widget();
    for (target, layer, frame) in &frames {
        let schema = patch_schema(&doc, &path(target)).unwrap();
        let mut position = child_node(&schema, layer);
        if *layer == "item" {
            // A nested node is a composite, a widget instance or a primitive.
            position = position["oneOf"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| def(&schema, &r["$ref"]))
                .find(|d| d["properties"].get(tag).is_some())
                .expect("a nested node may be a composite");
        }
        let mut offered: Vec<String> = members.clone();
        offered.push("due_badge".into());
        assert_eq!(
            position["properties"][tag]["enum"],
            json!(offered),
            "{layer}: the kinds and the document's widgets"
        );
        // A page kind may contribute the node: one without its tag refines the inherited node of
        // its name, naming any field a kind or the frame takes and requiring none.
        assert!(
            required(position).is_empty(),
            "{layer}: the tag may be inherited"
        );
        let refines = position["allOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["if"] == json!({"not": {"required": [tag]}}))
            .map(|e| &e["then"])
            .expect("a node without its tag refines an inherited one");
        let mut every = frame.all.clone();
        for kind in members.iter().map(String::as_str).chain(["WidgetInstance"]) {
            every.extend(fields(&constructs[kind]).all);
        }
        assert_eq!(
            keys(&refines["properties"]),
            every,
            "{layer}: what a refinement names"
        );
        assert_eq!(refines["additionalProperties"], json!(false));
        assert!(refines.get("required").is_none());

        for kind in &members {
            let at = format!("{layer} {kind}");
            assert_variant(
                &at,
                variant(&schema, position, tag, kind),
                tag,
                kind,
                joined(frame, &fields(&constructs[kind.as_str()])),
                constructs[kind.as_str()].get("exactly_one_of"),
                frame.all.contains("same_as"),
            );
            let own = fields(&constructs[kind.as_str()]);
            let v = variant(&schema, position, tag, kind);
            assert_typed(&at, &schema, position, v, tag, frame, &own);
        }

        // A widget instance: ESS's WidgetInstance fields, and `args` holding the widget's params;
        // `due` is required, and "required params must be given" (WidgetInstance), so `args` is.
        let instance = variant(&schema, position, tag, "due_badge");
        let (all, mut wanted) = joined(frame, &fields(&constructs["WidgetInstance"]));
        wanted.insert("args".into());
        assert_variant(
            &format!("{layer} due_badge"),
            instance,
            tag,
            "due_badge",
            (all, wanted),
            None,
            frame.all.contains("same_as"),
        );
        let own = fields(&constructs["WidgetInstance"]);
        assert_typed(
            &format!("{layer} due_badge"),
            &schema,
            position,
            instance,
            tag,
            frame,
            &own,
        );
        let args = &instance["properties"]["args"];
        assert_eq!(
            keys(&args["properties"]),
            BTreeSet::from(["due".into(), "tone".into()])
        );
        assert_eq!(required(args), BTreeSet::from(["due".to_owned()]));
        assert_eq!(args["additionalProperties"], json!(false));
    }

    // Every primitive kind, nested as an item.
    let primitive = &constructs["Primitive"];
    let kinds = strings(&primitive["fields"]["primitive"]["type"]["enum"]);
    assert_eq!(kinds.len(), 9, "ESS 0.48.0 has 9 primitive kinds");
    let schema = patch_schema(&doc, &path("page:loans/section:list")).unwrap();
    let position = child_node(&schema, "item")["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| def(&schema, &r["$ref"]))
        .find(|d| d["properties"].get("primitive").is_some())
        .expect("a nested node may be a primitive");
    assert_eq!(position["properties"]["primitive"]["enum"], json!(kinds));
    for kind in &kinds {
        assert_variant(
            &format!("primitive {kind}"),
            variant(&schema, position, "primitive", kind),
            "primitive",
            kind,
            joined(&fields(primitive), &fields(&constructs[kind.as_str()])),
            constructs[kind.as_str()].get("exactly_one_of"),
            false,
        );
        let v = variant(&schema, position, "primitive", kind);
        let own = fields(&constructs[kind.as_str()]);
        assert_typed(
            &format!("primitive {kind}"),
            &schema,
            position,
            v,
            "primitive",
            &fields(primitive),
            &own,
        );
    }

    // Overlay and region kinds are ESS's enums.
    let overlay_kind = &constructs["overlay"]["fields"]["kind"]["type"]["enum"];
    let page = patch_schema(&doc, &path("page:loans")).unwrap();
    let overlay = child_node(&page, "overlay");
    assert_eq!(
        overlay["properties"]["kind"]["enum"],
        serde_json::to_value(overlay_kind).unwrap()
    );
    let shell = patch_schema(&doc, &path("shell:app")).unwrap();
    let region = child_node(&shell, "region");
    assert_eq!(
        region["properties"]["kind"]["enum"],
        serde_json::to_value(&constructs["Region"]["fields"]["kind"]["type"]["enum"]).unwrap()
    );
    assert_eq!(
        keys(&region["properties"]),
        fields(&constructs["Region"]).all
    );
}
