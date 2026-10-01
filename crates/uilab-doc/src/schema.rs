//! The JSON Schema of a patch at one node: only the operations the node takes, only the child
//! layers it can hold, and only names that resolve in the document. It is what an agent's
//! structured output is held to; [`crate::admit`] still decides.
//!
//! Every field shape is ESS's, read from the schema ESS embeds ([`ess_ui::SCHEMA`]): the composite
//! kinds and their fields, the primitive kinds and their fields, overlays, regions, shells, pages,
//! menu sections, widget declarations and widget instances, with each field's `note` as its
//! description. The child layers a node can hold are [`allowed_children`]'s, which reads the same
//! schema. Nothing here lists a field.
//!
//! A composite is offered at three positions, each an ESS frame around one member of the composite
//! union: a section (`Section`), an overlay (`overlay`) and a nested node (`Composite`; a nested
//! node may also be one of the leaves, `Primitive`). Each position offers one closed variant per
//! member and per widget the document declares, chosen by `component` (by `primitive` for a
//! leaf): the frame's fields and the member's, and no other key.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde_json::{Map, Value, json};
use serde_yaml::Value as Yaml;

use crate::model::Document;
use crate::path::{Layer, NodePath, PathError, allowed_children};

/// The names uilab gives a node it inserts ([`crate::patch::valid_name`]).
const CHILD_NAME: &str = "^[a-z][a-z0-9_.-]*$";

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
    ops.push("decline");

    let mut properties = Map::new();
    properties.insert("op".into(), json!({"enum": ops, "description": "insert adds a child under target; replace swaps the node at target; remove deletes it; batch applies `patches` together, for one instruction that must change more than one node; decline changes nothing, with a `reason`, when the words are not a request to change the UI"}));
    properties.insert(
        "reason".into(),
        json!({"type": "string", "description": "for decline only: why nothing is proposed, or the answer to a question"}),
    );
    properties.insert(
        "patches".into(),
        json!({
            "description": "for batch only: the patches, applied in order and checked together; each names its own target and carries its own child or node",
            "type": "array",
            "minItems": 2,
            "items": {"$ref": "#/$defs/patch"},
        }),
    );
    properties.insert("target".into(), json!({"const": path.to_string()}));
    let inherited = Inherited::at(doc, path);
    if !children.is_empty() {
        // The layer is chosen by `if`/`then` on `layer`, not `oneOf`, so a refusal names the field
        // at fault inside the node rather than "valid under none of these".
        let layers: Vec<&str> = children.iter().map(|l| l.as_str()).collect();
        let choose: Vec<Value> = children
            .iter()
            .map(|l| {
                json!({
                    "if": {"properties": {"layer": {"const": l.as_str()}}, "required": ["layer"]},
                    "then": child_variant(doc, *l, &inherited),
                })
            })
            .collect();
        properties.insert(
            "child".into(),
            json!({
                "description": "the new node, for insert",
                "type": "object",
                "required": ["layer", "name", "node"],
                "properties": {"layer": {"enum": layers}},
                "allOf": choose,
            }),
        );
    }
    if changeable {
        let node = match (layer, inherited.this) {
            (Layer::Section, true) => "#/$defs/section_node_inherited".to_owned(),
            (Layer::Overlay, true) => "#/$defs/overlay_inherited".to_owned(),
            _ => def_ref(layer),
        };
        properties.insert(
            "node".into(),
            json!({"description": "the replacement node, for replace", "$ref": node}),
        );
    }

    let mut schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["op", "target"],
        "additionalProperties": false,
        "properties": properties,
        "$defs": Builder::new(doc).defs(),
    });
    // The Messages API refuses a tool schema with `oneOf`, `allOf` or `anyOf` at its top level,
    // so the operations are one `if`/`then`/`else` chain there rather than an `allOf` of them.
    if let (Value::Object(schema), Value::Object(chain)) = (&mut schema, carried(&ops)) {
        schema.extend(chain);
    }
    prune(&mut schema);
    Ok(schema)
}

/// What each operation carries, required when that operation is chosen: an insert its `child`, a
/// replace its `node`, a batch its `patches`, a decline its `reason`. One `if`/`then`/`else`
/// chain, an `else` per further operation; `{}` when none of `ops` carries anything.
fn carried(ops: &[&str]) -> Value {
    [
        ("insert", "child"),
        ("replace", "node"),
        ("batch", "patches"),
        ("decline", "reason"),
    ]
    .into_iter()
    .rev()
    .filter(|(op, _)| ops.contains(op))
    .fold(json!({}), |rest, (op, field)| {
        let mut step = json!({
            "if": {"properties": {"op": {"const": op}}, "required": ["op"]},
            "then": {"required": [field]},
        });
        if rest.as_object().is_some_and(|rest| !rest.is_empty()) {
            step["else"] = rest;
        }
        step
    })
}

/// The definition a node of `layer` has.
fn def_ref(layer: Layer) -> String {
    let name = match layer {
        Layer::Section => "section_node",
        Layer::Widget | Layer::Item | Layer::Child | Layer::Part | Layer::Tool => "node",
        Layer::Choice => "choice_entry",
        Layer::Node => "body_node",
        Layer::Component => "widget_declaration",
        other => other.as_str(),
    };
    format!("#/$defs/{name}")
}

/// What a patch's target inherits from its page's kind: the names of the sections and overlays the
/// kind contributes (a page's children of those names may refine them without their tag), and
/// whether the target is itself such a section or overlay.
struct Inherited {
    sections: Vec<String>,
    overlays: Vec<String>,
    this: bool,
}

impl Inherited {
    fn at(doc: &Document, path: &NodePath) -> Self {
        let mut out = Inherited {
            sections: Vec::new(),
            overlays: Vec::new(),
            this: false,
        };
        let page = match (path.layer(), path.parent()) {
            (Layer::Page, _) => path.name().to_owned(),
            (Layer::Section | Layer::Overlay, Some(parent)) if parent.layer() == Layer::Page => {
                parent.name().to_owned()
            }
            _ => return out,
        };
        let Some(kind) = doc.pages.get(&page).map(|p| p.kind.clone()) else {
            return out;
        };
        // The kind, then each kind it `extends`: the document's own, else ESS's built-in ones.
        let mut next = Some(kind);
        let mut seen = BTreeSet::new();
        while let Some(kind) = next.take() {
            if !seen.insert(kind.clone()) {
                break;
            }
            let def = match doc.page_kinds.get(&kind) {
                Some(def) => def.clone(),
                None => to_json(&construct("PageKind")["builtins"][kind.as_str()]),
            };
            for section in def["sections"].as_array().into_iter().flatten() {
                if let Some(name) = section["name"].as_str()
                    && section.get("remove") != Some(&json!(true))
                {
                    out.sections.push(name.to_owned());
                }
            }
            for name in def["overlays"]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(k, _)| k)
            {
                out.overlays.push(name.clone());
            }
            next = def["extends"].as_str().map(str::to_owned);
        }
        out.this = match path.layer() {
            Layer::Section => out.sections.iter().any(|s| s == path.name()),
            Layer::Overlay => out.overlays.iter().any(|o| o == path.name()),
            _ => false,
        };
        out
    }
}

fn child_variant(doc: &Document, layer: Layer, inherited: &Inherited) -> Value {
    let mut properties = Map::new();
    properties.insert("layer".into(), json!({"const": layer.as_str()}));
    properties.insert(
        "name".into(),
        json!({"type": "string", "pattern": CHILD_NAME}),
    );
    // A child named as one the page's kind contributes may refine it without its tag; any other
    // must carry its tag.
    let (names, refined) = match layer {
        Layer::Section => (&inherited.sections, "#/$defs/section_node_inherited"),
        Layer::Overlay => (&inherited.overlays, "#/$defs/overlay_inherited"),
        _ => (&Vec::new(), ""),
    };
    if names.is_empty() {
        properties.insert("node".into(), json!({"$ref": def_ref(layer)}));
    } else {
        properties.insert(
            "node".into(),
            json!({"$ref": refined, "description": format!("without its tag only when named one of {}", names.join(", "))}),
        );
    }
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
    let mut variant = json!({"type": "object", "required": required, "additionalProperties": false, "properties": properties});
    if !names.is_empty() {
        let tag = composite_tag();
        variant["if"] = json!({"not": {"properties": {"name": {"enum": names}}}});
        variant["then"] = json!({"properties": {"node": {"required": [tag]}}});
    }
    variant
}

/// ESS's schema, parsed once.
fn ess() -> &'static Yaml {
    static SCHEMA: OnceLock<Yaml> = OnceLock::new();
    SCHEMA.get_or_init(|| serde_yaml::from_str(ess_ui::SCHEMA).expect("ESS embeds a YAML schema"))
}

fn construct(name: &str) -> &'static Yaml {
    &ess()["constructs"][name]
}

/// The fields a construct declares, in order; keys in parentheses describe the construct's value
/// and are no field.
fn fields_of(construct: &'static Yaml) -> Vec<(&'static str, &'static Yaml)> {
    construct["fields"]
        .as_mapping()
        .into_iter()
        .flatten()
        .filter_map(|(key, spec)| key.as_str().map(|key| (key, spec)))
        .filter(|(key, _)| !key.starts_with('('))
        .collect()
}

fn strings(value: &'static Yaml) -> Vec<&'static str> {
    value
        .as_sequence()
        .into_iter()
        .flatten()
        .filter_map(Yaml::as_str)
        .collect()
}

fn to_json(value: &Yaml) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

/// The `$defs` key a construct is offered under: the patch layers keep uilab's names, the node
/// positions are built here, every other construct keeps ESS's name.
fn def_name(construct: &str) -> &str {
    match construct {
        "Node" => "node",
        "Section" => "section",
        "Composite" => "composite",
        "Primitive" => "primitive",
        "Widget" => "widget_declaration",
        "Shell" => "shell",
        "Region" => "region",
        "NavSection" => "nav_section",
        "Page" => "page",
        other => other,
    }
}

/// The node positions [`Builder::positions`] builds rather than [`Builder::construct_def`].
const POSITIONS: [&str; 5] = ["Node", "Section", "overlay", "Composite", "Primitive"];

struct Builder<'a> {
    doc: &'a Document,
    defs: Map<String, Value>,
    queued: Vec<&'static str>,
    seen: BTreeSet<&'static str>,
    /// Definitions whose every field also takes a short form: `(definition, form)`.
    widened: Vec<(String, &'static Yaml)>,
    /// The construct being built: a widget declaration's body nodes are named, not typed; a
    /// page's sections and overlays may refine its kind's.
    building: &'static str,
}

impl<'a> Builder<'a> {
    fn new(doc: &'a Document) -> Self {
        Self {
            doc,
            defs: Map::new(),
            queued: Vec::new(),
            seen: BTreeSet::new(),
            widened: Vec::new(),
            building: "",
        }
    }

    fn defs(mut self) -> Map<String, Value> {
        self.positions();
        for layer in ["Shell", "Region", "NavSection", "Page", "Widget"] {
            self.reference(layer);
        }
        self.defs.insert("patch".into(), patch_def());
        self.drain();
        let mut done = BTreeSet::new();
        while let Some((base, form)) = self.widened.pop() {
            let name = format!("{base}.short");
            if !done.insert(name.clone()) {
                continue;
            }
            let mut def = self.defs.get(&base).cloned().unwrap_or(Value::Null);
            let short = self.ty(form);
            if let Some(properties) = def.get_mut("properties").and_then(Value::as_object_mut) {
                for field in properties.values_mut() {
                    let long = field.take();
                    *field = json!({"anyOf": [short.clone(), long]});
                }
            }
            self.defs.insert(name, def);
            self.drain();
        }
        self.defs
    }

    /// Builds every construct referred to and not built yet.
    fn drain(&mut self) {
        while let Some(name) = self.queued.pop() {
            self.building = name;
            let built = self.construct_def(name);
            self.building = "";
            self.defs.insert(def_name(name).to_owned(), built);
        }
    }

    /// A `$ref` to the construct's definition, built once.
    fn reference(&mut self, name: &'static str) -> Value {
        if !POSITIONS.contains(&name) && self.seen.insert(name) {
            self.queued.push(name);
        }
        json!({"$ref": format!("#/$defs/{}", def_name(name))})
    }

    /// The JSON Schema of an ESS type expression (`type_rule`).
    fn ty(&mut self, ty: &'static Yaml) -> Value {
        match ty {
            Yaml::String(name) => self.named(name),
            Yaml::Mapping(_) => {
                if let Some(element) = ty.get("list") {
                    let mut list = json!({"type": "array", "items": self.element(element)});
                    if ty.get("unique").and_then(Yaml::as_bool) == Some(true) {
                        list["uniqueItems"] = json!(true);
                    }
                    list
                } else if let Some(map) = ty.get("map") {
                    let mut object =
                        json!({"type": "object", "additionalProperties": self.ty(&map["value"])});
                    if map["key"].as_str() != Some("string") {
                        object["propertyNames"] = self.ty(&map["key"]);
                    }
                    object
                } else if let Some(inner) = ty.get("optional") {
                    json!({"anyOf": [self.ty(inner), {"type": "null"}]})
                } else if let Some(values) = ty.get("enum") {
                    json!({"enum": to_json(values)})
                } else if let Some(alternatives) = ty.get("one_of").and_then(Yaml::as_sequence) {
                    let alternatives: Vec<Value> =
                        alternatives.iter().map(|alt| self.ty(alt)).collect();
                    json!({"anyOf": alternatives})
                } else if let Some(record) = ty.get("record") {
                    self.record(record)
                } else if let Some(kind) = ty.get("ref").and_then(Yaml::as_str) {
                    self.named_ref(kind)
                } else if let Some(value) = ty.get("const") {
                    json!({"const": to_json(value)})
                } else {
                    json!({})
                }
            }
            _ => json!({}),
        }
    }

    /// A type by name: a primitive of `type_rule.primitives`, a construct, or a type of the ESS
    /// model or of the document's `types`, which the document resolves and the schema cannot.
    fn named(&mut self, name: &'static str) -> Value {
        // A page's own overlay may refine the one its kind contributes.
        if name == "overlay" && self.building == "Page" {
            return json!({"$ref": "#/$defs/overlay_inherited"});
        }
        let primitive = &ess()["type_rule"]["primitives"][name];
        if !primitive.is_null() {
            let mut out = match name {
                "integer" => json!({"type": "integer"}),
                "number" => json!({"type": "number"}),
                "boolean" => json!({"type": "boolean"}),
                "json" => json!({}),
                // `Expr`: written as a string; a bare number or boolean is read as its text.
                "expr" => json!({"type": ["string", "number", "boolean"]}),
                "name" => json!({
                    "type": "string",
                    "pattern": ess()["constructs"]["NodePath"]["syntax"]["segment_pattern"],
                }),
                _ => json!({"type": "string"}),
            };
            if let Some(pattern) = primitive["pattern"].as_str() {
                out["pattern"] = json!(pattern);
            }
            return out;
        }
        if !construct(name).is_null() {
            return self.reference(name);
        }
        json!({"description": format!("`{name}`: a type of the ESS model or of the document's `types`")})
    }

    /// The element of a list: a node in a list carries its `name` (`Node`), as does every
    /// construct whose `name` is required.
    fn element(&mut self, element: &'static Yaml) -> Value {
        if let Some(name) = element.as_str() {
            match name {
                "Node" if self.building == "Widget" => {
                    return json!({"$ref": "#/$defs/named_body_node"});
                }
                "Node" => return json!({"$ref": "#/$defs/named_node"}),
                "Section" => return json!({"$ref": "#/$defs/named_section"}),
                _ => {}
            }
            let named = construct(name)["fields"]["name"]["required"].as_bool() == Some(true);
            if named {
                let reference = self.reference(name);
                return json!({"allOf": [{"type": "object", "required": ["name"]}, reference]});
            }
        }
        self.ty(element)
    }

    /// A name that must resolve: the document's names where they are a closed set the same
    /// answer cannot add to, otherwise a string.
    fn named_ref(&self, kind: &str) -> Value {
        let doc = self.doc;
        let names: Vec<String> = match kind {
            "page" => doc.pages.keys().cloned().collect(),
            "shell" => doc.shells.keys().cloned().collect(),
            "widget" => doc.widgets.keys().cloned().collect(),
            "composite_kind" => members().iter().map(|m| (*m).to_owned()).collect(),
            "page_kind" => construct("PageKind")["builtins"]
                .as_mapping()
                .into_iter()
                .flatten()
                .filter_map(|(k, _)| k.as_str().map(str::to_owned))
                .chain(doc.page_kinds.keys().cloned())
                .collect(),
            _ => Vec::new(),
        };
        let mut out = json!({"type": "string", "description": format!("a {kind} name")});
        if !names.is_empty() {
            out["enum"] = json!(names);
        }
        out
    }

    /// `{record: {field: T}}`: its fields, no other key, none required. ESS's records mark a field
    /// `{optional: T}` or not, but its loader defaults some of the others (`Sort.allowed`), so the
    /// schema guides here and ESS's admission judges (beyond10x/ess#305).
    fn record(&mut self, record: &'static Yaml) -> Value {
        let mut properties = Map::new();
        for (key, ty) in record.as_mapping().into_iter().flatten() {
            let Some(key) = key.as_str() else { continue };
            let ty = ty.get("optional").unwrap_or(ty);
            properties.insert(key.to_owned(), self.ty(ty));
        }
        json!({"type": "object", "properties": properties, "additionalProperties": false})
    }

    /// One field: its type, described by its `note`.
    fn field(&mut self, spec: &'static Yaml) -> Value {
        let mut out = self.ty(&spec["type"]);
        if let (Some(note), Some(object)) = (spec["note"].as_str(), out.as_object_mut()) {
            object.entry("description").or_insert_with(|| json!(note));
        }
        out
    }

    /// Field `key` of `construct`, widened by every short form the construct declares at it
    /// (`shorthand` entries with `at`): `key` itself, each entry of the list `key[]`, or each
    /// value of the map `key.*`. A form the field's type already includes adds nothing.
    fn field_in(&mut self, construct: &'static Yaml, key: &str, spec: &'static Yaml) -> Value {
        let mut out = self.field(spec);
        let alternatives = spec["type"].get("one_of").and_then(Yaml::as_sequence);
        for entry in construct["shorthand"].as_sequence().into_iter().flatten() {
            let accepts = &entry["accepts"];
            if accepts.as_str() == Some("absent")
                || alternatives.is_some_and(|alts| alts.contains(accepts))
            {
                continue;
            }
            for at in entry["at"].as_str().unwrap_or_default().split(" | ") {
                let each = at.strip_suffix(".*") == Some(key);
                // Each field of a construct (`header.*`): a definition whose fields take it too.
                if each
                    && let Some(base) = out["$ref"]
                        .as_str()
                        .and_then(|r| r.strip_prefix("#/$defs/"))
                {
                    let base = base.to_owned();
                    out["$ref"] = json!(format!("#/$defs/{base}.short"));
                    self.widened.push((base, accepts));
                    continue;
                }
                let slot = if at == key {
                    Some(&mut out)
                } else if at.strip_suffix("[]") == Some(key) {
                    out.get_mut("items")
                } else if each {
                    out.get_mut("additionalProperties")
                } else {
                    None
                };
                if let Some(slot) = slot {
                    let short = self.ty(accepts);
                    let long = slot.take();
                    *slot = json!({"anyOf": [short, long]});
                }
            }
        }
        out
    }

    /// A closed object of `fields`: each typed, the `required: true` ones required except `name`
    /// (a list requires it where the node is an entry; a patch carries it beside the node), and
    /// `extra` (a variant's discriminator) last.
    fn object(
        &mut self,
        fields: &[(&'static str, &'static Yaml)],
        rules: &'static Yaml,
        description: Option<&str>,
    ) -> Value {
        let mut properties = Map::new();
        let mut required: Vec<&str> = Vec::new();
        for (key, spec) in fields {
            properties.insert((*key).to_owned(), self.field_in(rules, key, spec));
            if spec["required"].as_bool() == Some(true) && *key != "name" && !required.contains(key)
            {
                required.push(key);
            }
        }
        let mut out = json!({
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false,
        });
        if let Some(description) = description {
            out["description"] = json!(description);
        }
        constrain(&mut out, rules);
        out
    }

    /// A construct that is not a node position, by its `form`, its value, or its fields.
    fn construct_def(&mut self, name: &'static str) -> Value {
        let c = construct(name);
        if let Some(form) = c.get("form") {
            return self.ty(form);
        }
        let entries: Vec<(&'static str, &'static Yaml)> = c["fields"]
            .as_mapping()
            .into_iter()
            .flatten()
            .filter_map(|(key, spec)| key.as_str().map(|key| (key, spec)))
            .collect();
        if let [(key, spec)] = entries.as_slice()
            && key.starts_with('(')
        {
            let value = self.ty(&spec["type"]);
            if *key == "(value)" {
                return value;
            }
            // A map keyed by the named value (`Degrades`: a capability, then a fallback name).
            return json!({"type": "object", "propertyNames": value, "additionalProperties": {"type": "string"}});
        }
        let fields = fields_of(c);
        let object = self.object(&fields, c, c["summary"].as_str());
        // A short form the construct also accepts in place of its map (`shorthands.index`).
        let short = c["shorthand"]
            .as_sequence()
            .into_iter()
            .flatten()
            .find(|entry| entry.get("at").is_none() && entry["accepts"].as_str() != Some("absent"));
        match short {
            Some(entry) => json!({"anyOf": [self.ty(&entry["accepts"]), object]}),
            None => object,
        }
    }

    /// The node positions: section, overlay and nested composite, the nested primitive, a nested
    /// node (either), and the same nested positions inside a widget declaration's body. Where a
    /// page kind can contribute the node (a section or an overlay of a page), also the untagged
    /// refinement of it, offered only where the target inherits ([`patch_schema`]).
    fn positions(&mut self) {
        let doc = self.doc;
        let tag = composite_tag();
        let mut section = fields_of(construct("Section"));
        // A section's `reads` is "the composite's own `reads`" (`Section.fields.reads`): offered
        // where the member declares one, by the member. Its `name` is the patch's (`child.name`,
        // or the target's), or a page entry's, never written inside a patch's node.
        section.retain(|(key, spec)| spec["type"].as_str() != Some("Reads") && *key != "name");
        let mut composites: Vec<Kind> = members().into_iter().map(Kind::Construct).collect();
        composites.extend(doc.widgets.keys().map(|w| Kind::Widget(w.as_str())));
        let primitive = construct("Primitive");
        let leaves: Vec<Kind> = strings(&primitive["fields"]["primitive"]["type"]["enum"])
            .into_iter()
            .map(Kind::Construct)
            .collect();

        let positions = [
            (
                "section",
                Position {
                    tag,
                    framing: construct("Section"),
                    frame: section,
                    kinds: &composites,
                    description: "A section of a page: a composite with its own read and loading lifecycle, written inline with the section's fields.",
                    typed: true,
                    inherits: true,
                },
            ),
            (
                "overlay",
                Position {
                    tag,
                    framing: construct("overlay"),
                    frame: fields_of(construct("overlay")),
                    kinds: &composites,
                    description: "A drawer, dialog, fullscreen pane or popover around one composite, written inline with the overlay's fields.",
                    typed: true,
                    inherits: true,
                },
            ),
            (
                "composite",
                Position {
                    tag,
                    framing: construct("Composite"),
                    frame: fields_of(construct("Composite")),
                    kinds: &composites,
                    description: "A composite nested in another node, or a widget instance.",
                    typed: true,
                    inherits: false,
                },
            ),
            (
                "primitive",
                Position {
                    tag: "primitive",
                    framing: primitive,
                    frame: fields_of(primitive),
                    kinds: &leaves,
                    description: primitive["summary"].as_str().unwrap_or_default(),
                    typed: true,
                    inherits: false,
                },
            ),
            // A widget's body may read `args.<param>` in any field: ESS checks it once expanded
            // at each use site, not as declared, so here its fields are named, not typed.
            (
                "body_composite",
                Position {
                    tag,
                    framing: construct("Composite"),
                    frame: fields_of(construct("Composite")),
                    kinds: &composites,
                    description: "A composite or widget instance of a widget's body; any field may be an expression over `args`.",
                    typed: false,
                    inherits: false,
                },
            ),
            (
                "body_primitive",
                Position {
                    tag: "primitive",
                    framing: primitive,
                    frame: fields_of(primitive),
                    kinds: &leaves,
                    description: "A primitive of a widget's body; any field may be an expression over `args`.",
                    typed: false,
                    inherits: false,
                },
            ),
        ];
        for (name, position) in positions {
            let def = self.union(name, &position);
            self.defs.insert(name.to_owned(), def);
        }

        // A nested node is a primitive when it carries `primitive`, else a composite: chosen by
        // the tag present, so a refusal names the field at fault rather than "none of these".
        let leaf_tag = "primitive";
        for (node, composite, leaf) in [
            ("node", "composite", "primitive"),
            ("body_node", "body_composite", "body_primitive"),
        ] {
            self.defs.insert(
                node.into(),
                json!({
                    "description": construct("Node")["summary"],
                    "if": {"required": [leaf_tag]},
                    "then": {"$ref": format!("#/$defs/{leaf}")},
                    "else": {"$ref": format!("#/$defs/{composite}")},
                }),
            );
        }
        let name_type = self.named("name");
        for (named, of) in [
            ("named_node", "node"),
            // A page's own entry may refine the section its kind contributes.
            ("named_section", "section_inherited"),
            ("named_body_node", "body_node"),
        ] {
            self.defs.insert(
                named.into(),
                json!({
                    "description": "an entry of a list of nodes, which carries its `name`",
                    "allOf": [
                        {"type": "object", "required": ["name"], "properties": {"name": name_type}},
                        {"$ref": format!("#/$defs/{of}")},
                    ],
                }),
            );
        }
        // A section as a patch's node: named by the patch, so no `name` inside it.
        for (node, of) in [
            ("section_node", "section"),
            ("section_node_inherited", "section_inherited"),
        ] {
            self.defs.insert(
                node.into(),
                json!({
                    "description": "a section; its name is the patch's (`child.name`, or the target's), not written in the node",
                    "allOf": [{"not": {"required": ["name"]}}, {"$ref": format!("#/$defs/{of}")}],
                }),
            );
        }

        // A filter bar's choice may be the marker that removes an inherited one
        // (`filter_bar` shorthand at `choices[]`); as a patch's node it carries no `name`.
        let mut removal = Value::Null;
        for entry in construct("filter_bar")["shorthand"]
            .as_sequence()
            .into_iter()
            .flatten()
        {
            if entry["at"].as_str() == Some("choices[]") {
                removal = self.ty(&entry["accepts"]);
            }
        }
        if let Some(object) = removal.as_object_mut() {
            if let Some(properties) = object.get_mut("properties").and_then(Value::as_object_mut) {
                properties.remove("name");
            }
            if let Some(required) = object.get_mut("required").and_then(Value::as_array_mut) {
                required.retain(|r| r != "name");
            }
        }
        self.defs.insert(
            "choice_entry".into(),
            json!({"anyOf": [removal, {"$ref": "#/$defs/node"}]}),
        );
    }

    /// One position. A variant per kind, chosen by `tag`, names the fields it takes (the frame's
    /// and the kind's), requires ESS's required ones, fixes `tag` to the kind and refuses the
    /// other tag. Typed, the frame's fields are typed here, once, and each kind's own once in the
    /// kind's definition, shared by every position.
    ///
    /// Two of ESS's merges relax what a node must write. Where a page kind can contribute the node
    /// (`inherits`), one written without its tag refines the inherited node of its name
    /// (`inheritance.named_lists`, `maps`): `<position>_refinement` names what it may write and
    /// `<position>_inherited` takes either, for the targets that inherit. An overlay naming
    /// `same_as` is merged over the overlay it names, so it need not repeat a required field.
    fn union(&mut self, position: &str, p: &Position<'_, 'a>) -> Value {
        let tag = p.tag;
        let mut properties = Map::new();
        if p.typed {
            for (key, spec) in &p.frame {
                properties.insert((*key).to_owned(), self.field_in(p.framing, key, spec));
            }
        }
        // A node has exactly one of its tags (`Node.exactly_one_of`).
        let others: Vec<Value> = strings(&construct("Node")["exactly_one_of"])
            .into_iter()
            .filter(|other| *other != tag)
            .map(|other| json!({"required": [other]}))
            .collect();
        let same_as = p.frame.iter().any(|(key, _)| *key == "same_as");
        let mut offered = Vec::new();
        let mut choose = Vec::new();
        let mut every = Map::new();
        for kind in p.kinds {
            let (name, c) = match kind {
                Kind::Construct(name) => (*name, construct(name)),
                Kind::Widget(name) => (*name, construct("WidgetInstance")),
            };
            let own = fields_of(c);
            let mut names = Map::new();
            let mut required: Vec<Value> = Vec::new();
            for (key, spec) in p.frame.iter().chain(own.iter()) {
                names.insert((*key).to_owned(), json!(true));
                every.insert((*key).to_owned(), json!(true));
                let wanted = json!(key);
                if spec["required"].as_bool() == Some(true)
                    && *key != "name"
                    && *key != tag
                    && !required.contains(&wanted)
                {
                    required.push(wanted);
                }
            }
            names.insert(tag.to_owned(), json!({"const": name}));
            // The fields ESS declares, listed; other keys are not refused here: ESS's loader takes
            // some its schema does not list (an overlay's `visible`, a node's `degrades`), and its
            // admission judges (beyond10x/ess#305). The tags and the required fields hold.
            let mut variant = json!({"properties": names});
            if !others.is_empty() {
                variant["not"] = json!({"anyOf": others});
            }
            match kind {
                Kind::Construct(name) => {
                    if p.typed {
                        variant["$ref"] = json!(format!("#/$defs/{name}"));
                        self.kind_def(name);
                    }
                    constrain(&mut variant, c);
                }
                Kind::Widget(name) => {
                    if p.typed {
                        variant["$ref"] = json!("#/$defs/WidgetInstance");
                        self.kind_def("WidgetInstance");
                    }
                    if let Some(widget) = self.doc.widgets.get(*name) {
                        variant["description"] = json!(widget.summary);
                        variant["properties"]["args"] = args(widget);
                        // "Required params must be given" (`WidgetInstance`): with one, `args` is.
                        if widget.params.values().any(|param| param.is_required()) {
                            required.push(json!("args"));
                        }
                    }
                }
            }
            if same_as && !required.is_empty() {
                variant["required"] = json!([tag]);
                variant["if"] = json!({"not": {"required": ["same_as"]}});
                variant["then"] = json!({"required": required});
            } else {
                required.insert(0, json!(tag));
                variant["required"] = json!(required);
            }
            choose.push(json!({
                "if": {"properties": {tag: {"const": name}}, "required": [tag]},
                "then": variant,
            }));
            offered.push(name);
        }
        if p.inherits {
            let refinement = format!("{position}_refinement");
            self.defs.insert(
                refinement.clone(),
                json!({
                    "type": "object",
                    "description": format!("without `{tag}`: refines the node of the same name its page kind contributes, writing only what differs"),
                    "properties": every,
                    "minProperties": 1,
                    "not": {"anyOf": others},
                }),
            );
            self.defs.insert(
                format!("{position}_inherited"),
                json!({
                    "description": format!("a {position} with `{tag}`, or, refining the one its page kind contributes, without"),
                    "if": {"required": [tag]},
                    "then": {"$ref": format!("#/$defs/{position}")},
                    "else": {"$ref": format!("#/$defs/{refinement}")},
                }),
            );
        }
        properties.insert(
            tag.to_owned(),
            json!({"enum": offered, "description": "the kind, or a widget the document declares; the other fields are the ones that kind takes"}),
        );
        // An empty node is no node, at every position, as in a batch.
        json!({
            "type": "object",
            "description": p.description,
            "required": [tag],
            "minProperties": 1,
            "properties": properties,
            "allOf": choose,
        })
    }

    /// A kind's own fields, typed and described, once for every position it is offered at. Which
    /// of them a node may write, and which it must, is the position's variant's to say.
    fn kind_def(&mut self, name: &'static str) {
        if self.defs.contains_key(name) {
            return;
        }
        self.defs.insert(name.to_owned(), Value::Null);
        let c = construct(name);
        let mut properties = Map::new();
        for (key, spec) in fields_of(c) {
            properties.insert(key.to_owned(), self.field_in(c, key, spec));
        }
        let def = json!({"description": c["summary"], "type": "object", "properties": properties});
        self.defs.insert(name.to_owned(), def);
    }
}

/// What a position offers under its tag: a member of ESS's union or a leaf, by construct, or a
/// widget the document declares.
enum Kind<'a> {
    Construct(&'static str),
    Widget(&'a str),
}

/// A node position: the frame of ESS fields around the kinds it offers.
struct Position<'k, 'a> {
    /// The discriminator: `component`, or `primitive` for a leaf.
    tag: &'static str,
    /// The construct whose fields frame the kind's (and whose shorthands widen them).
    framing: &'static Yaml,
    frame: Vec<(&'static str, &'static Yaml)>,
    kinds: &'k [Kind<'a>],
    description: &'static str,
    /// Whether fields are typed, or only named (a widget's body).
    typed: bool,
    /// Whether a page kind can contribute the node, so one without its tag refines it.
    inherits: bool,
}

/// Only the definitions the schema reaches from outside `$defs`, following `$ref`s.
fn prune(schema: &mut Value) {
    fn refs(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::Object(object) => {
                if let Some(target) = object.get("$ref").and_then(Value::as_str)
                    && let Some(name) = target.strip_prefix("#/$defs/")
                {
                    out.push(name.split('/').next().unwrap_or(name).to_owned());
                }
                object.values().for_each(|v| refs(v, out));
            }
            Value::Array(values) => values.iter().for_each(|v| refs(v, out)),
            _ => {}
        }
    }
    let Some(Value::Object(defs)) = schema.as_object_mut().and_then(|o| o.remove("$defs")) else {
        return;
    };
    let mut todo = Vec::new();
    refs(schema, &mut todo);
    let mut kept = Map::new();
    while let Some(name) = todo.pop() {
        if kept.contains_key(&name) {
            continue;
        }
        if let Some(def) = defs.get(&name) {
            refs(def, &mut todo);
            kept.insert(name, def.clone());
        }
    }
    let ordered: Map<String, Value> = defs
        .into_iter()
        .filter(|(name, _)| kept.contains_key(name))
        .collect();
    schema["$defs"] = Value::Object(ordered);
}

/// The members of ESS's composite union.
fn members() -> Vec<&'static str> {
    strings(&construct("Composite")["union"]["members"])
}

/// The key the composite union is discriminated by.
fn composite_tag() -> &'static str {
    construct("Composite")["union"]["tag"]
        .as_str()
        .expect("ESS's composite union has a tag")
}

/// A construct's rules over its fields: `exactly_one_of` as one alternative per field, and every
/// `when <field> present, requires <field> present` constraint.
fn constrain(object: &mut Value, rules: &Yaml) {
    let exactly = strings_of(rules.get("exactly_one_of"));
    if !exactly.is_empty() {
        object["oneOf"] = exactly
            .iter()
            .map(|key| json!({"required": [key]}))
            .collect();
    }
    let mut dependent = Map::new();
    for constraint in rules
        .get("constraints")
        .and_then(Yaml::as_sequence)
        .into_iter()
        .flatten()
    {
        let present = |side: &Yaml| -> Option<String> {
            let map = side.as_mapping()?;
            let (key, value) = map.iter().next()?;
            (map.len() == 1 && value.as_str() == Some("present"))
                .then(|| key.as_str().map(str::to_owned))
                .flatten()
        };
        if let (Some(when), Some(then)) = (
            present(&constraint["when"]),
            present(&constraint["requires"]),
        ) {
            dependent.insert(when, json!([then]));
        }
    }
    if !dependent.is_empty() {
        object["dependentRequired"] = Value::Object(dependent);
    }
}

fn strings_of(value: Option<&Yaml>) -> Vec<String> {
    value
        .and_then(Yaml::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}

/// A widget instance's `args`: one entry per param the widget declares, the required ones
/// required, nothing else.
fn args(widget: &crate::model::Widget) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for (name, param) in &widget.params {
        let ty = serde_json::to_string(&param.ty).unwrap_or_default();
        let note = param.note.as_deref().unwrap_or_default();
        properties.insert(
            name.clone(),
            json!({"description": format!("an expression or a literal of type {ty}; {note}")}),
        );
        if param.is_required() {
            required.push(name.clone());
        }
    }
    json!({
        "type": "object",
        "description": "one entry per param of the widget: an expression, or a literal such as a map",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

/// A patch inside a batch. Its node is not held to its layer's shape here: a batch may declare a
/// widget and use it, or add a page and link to it, which a shape naming only the document's
/// widgets and pages would refuse; admission checks the result. It must carry its node.
fn patch_def() -> Value {
    let layers: Vec<&str> = Layer::ALL
        .iter()
        .filter(|l| !matches!(l, Layer::Root | Layer::Nav))
        .map(|l| l.as_str())
        .collect();
    json!({
        "type": "object",
        "required": ["op", "target"],
        "additionalProperties": false,
        "properties": {
            "op": {"enum": ["insert", "replace", "remove"]},
            "target": {"type": "string", "description": "path of the node this patch acts on: the parent for insert"},
            "child": {
                "type": "object",
                "required": ["layer", "name", "node"],
                "additionalProperties": false,
                "properties": {
                    "layer": {"enum": layers},
                    "name": {"type": "string", "pattern": CHILD_NAME},
                    "node": {"type": "object", "minProperties": 1, "description": "the new node, in the shape of its layer"},
                    "nav_section": {"type": "string"}
                }
            },
            "node": {"type": "object", "minProperties": 1, "description": "for replace: the new node, in the shape of the target's layer"}
        },
        "allOf": [carried(&["insert", "replace"])],
    })
}
