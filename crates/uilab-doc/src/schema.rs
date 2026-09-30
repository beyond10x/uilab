//! The JSON Schema of a patch at one node: only the operations the node takes, only the child
//! layers it can hold, and only names that resolve in the document. It is what an agent's
//! structured output is held to; [`crate::admit`] still decides.

use serde_json::{Value, json};

use crate::model::{BUILTIN_PAGE_KINDS, CompositeKind, Document, OverlayKind, RegionKind};
use crate::path::{Layer, NodePath, PathError, allowed_children};

/// The schema of a patch whose target is `path`.
pub fn patch_schema(doc: &Document, path: &NodePath) -> Result<Value, PathError> {
    let children = allowed_children(doc, path)?;
    let layer = path.layer();
    let changeable = !matches!(layer, Layer::Root | Layer::Nav);

    let mut ops = Vec::new();
    if !children.is_empty() {
        ops.push("insert");
    }
    if changeable {
        ops.push("replace");
        ops.push("remove");
    }
    ops.push("batch");

    let mut properties = serde_json::Map::new();
    properties.insert("op".into(), json!({"enum": ops, "description": "insert adds a child under target; replace swaps the node at target; remove deletes it; batch applies `patches` together, for one instruction that must change more than one node"}));
    properties.insert(
        "patches".into(),
        json!({
            "description": "for batch only: the patches, applied in order and checked together; each names its own target",
            "type": "array",
            "minItems": 2,
            "items": {"$ref": "#/$defs/patch"},
        }),
    );
    properties.insert("target".into(), json!({"const": path.to_string()}));
    if !children.is_empty() {
        let variants: Vec<Value> = children.iter().map(|l| child_variant(doc, *l)).collect();
        properties.insert(
            "child".into(),
            json!({"description": "the new node, for insert", "oneOf": variants}),
        );
    }
    if changeable {
        properties.insert(
            "node".into(),
            json!({"description": "the replacement node, for replace", "$ref": def_ref(layer)}),
        );
    }

    Ok(json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["op", "target"],
        "additionalProperties": false,
        "properties": properties,
        "$defs": defs(doc),
    }))
}

fn def_ref(layer: Layer) -> String {
    let name = match layer {
        Layer::Section | Layer::Widget | Layer::Item => "composite",
        other => other.as_str(),
    };
    format!("#/$defs/{name}")
}

fn child_variant(doc: &Document, layer: Layer) -> Value {
    let mut properties = serde_json::Map::new();
    properties.insert("layer".into(), json!({"const": layer.as_str()}));
    properties.insert(
        "name".into(),
        json!({"type": "string", "pattern": "^[a-z][a-z0-9_.-]*$"}),
    );
    properties.insert("node".into(), json!({"$ref": def_ref(layer)}));
    let mut required = vec!["layer", "name", "node"];
    if layer == Layer::Page {
        let sections: Vec<&str> = doc
            .navigation
            .sections
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        let mut nav = json!({"type": "string", "description": "menu section to list the page in; omit to list it as hidden"});
        if !sections.is_empty() {
            nav["enum"] = json!(sections);
        }
        properties.insert("nav_section".into(), nav);
    } else {
        required.truncate(3);
    }
    json!({"type": "object", "required": required, "additionalProperties": false, "properties": properties})
}

fn defs(doc: &Document) -> Value {
    let composite_kinds: Vec<&str> = CompositeKind::ALL.iter().map(|k| k.as_str()).collect();
    let region_kinds: Vec<&str> = RegionKind::ALL.iter().map(|k| k.as_str()).collect();
    let overlay_kinds = [
        OverlayKind::Drawer,
        OverlayKind::Dialog,
        OverlayKind::Fullscreen,
        OverlayKind::Popover,
    ]
    .map(|k| serde_json::to_value(k).expect("an enum serializes"));
    let mut page_kinds: Vec<&str> = BUILTIN_PAGE_KINDS.to_vec();
    page_kinds.extend(doc.page_kinds.keys().map(String::as_str));
    let shells: Vec<&str> = doc.shells.keys().map(String::as_str).collect();
    let pages: Vec<&str> = doc.pages.keys().map(String::as_str).collect();

    let mut shell_prop = json!({"type": "string"});
    if !shells.is_empty() {
        shell_prop["enum"] = json!(shells);
    }
    let mut page_names = json!({"type": "string"});
    if !pages.is_empty() {
        page_names["enum"] = json!(pages);
    }

    json!({
        "patch": {
            "type": "object",
            "required": ["op", "target"],
            "additionalProperties": false,
            "properties": {
                "op": {"enum": ["insert", "replace", "remove"]},
                "target": {"type": "string", "description": "path of the node this patch acts on: the parent for insert"},
                "child": {
                    "type": "object",
                    "required": ["layer", "name", "node"],
                    "properties": {
                        "layer": {"enum": ["shell", "region", "nav_section", "page", "section", "overlay", "widget", "item"]},
                        "name": {"type": "string", "pattern": "^[a-z][a-z0-9_.-]*$"},
                        "node": {"type": "object"},
                        "nav_section": {"type": "string"}
                    }
                },
                "node": {"type": "object"}
            }
        },
        "reads": {
            "type": "object",
            "required": ["view"],
            "properties": {
                "view": {"type": "string", "description": "ESS view `<domain>.<View>`; use `draft.<Name>` while no model view exists"}
            }
        },
        "composite": {
            "type": "object",
            "required": ["component"],
            "properties": {
                "component": {"enum": composite_kinds},
                "reads": {"$ref": "#/$defs/reads"},
                "widgets": {"type": "object", "description": "board only: composite per widget kind", "additionalProperties": {"$ref": "#/$defs/composite"}},
                "item": {"type": "object", "description": "collection only: composites nested per row", "additionalProperties": {"$ref": "#/$defs/composite"}}
            }
        },
        "overlay": {
            "type": "object",
            "required": ["kind", "component"],
            "properties": {
                "kind": {"enum": overlay_kinds},
                "component": {"enum": composite_kinds},
                "title": {"type": "string"},
                "reads": {"$ref": "#/$defs/reads"}
            }
        },
        "region": {
            "type": "object",
            "required": ["kind"],
            "properties": {
                "kind": {"enum": region_kinds},
                "props": {"type": "object"}
            }
        },
        "shell": {
            "type": "object",
            "required": ["regions"],
            "properties": {
                "regions": {"type": "object", "additionalProperties": {"$ref": "#/$defs/region"}},
                "overlays": {"type": "object", "additionalProperties": {"$ref": "#/$defs/overlay"}}
            }
        },
        "page": {
            "type": "object",
            "required": ["kind", "sections"],
            "properties": {
                "kind": {"enum": page_kinds},
                "title": {"type": "string"},
                "shell": shell_prop,
                "sections": {"type": "object", "additionalProperties": {"$ref": "#/$defs/composite"}},
                "overlays": {"type": "object", "additionalProperties": {"$ref": "#/$defs/overlay"}}
            }
        },
        "nav_section": {
            "type": "object",
            "required": ["pages"],
            "additionalProperties": false,
            "properties": {
                "label": {"type": "string"},
                "icon": {"type": "string"},
                "pages": {"type": "array", "items": page_names}
            }
        }
    })
}
