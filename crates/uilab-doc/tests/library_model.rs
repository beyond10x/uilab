//! The ESS model of the library example invents nothing: every view and command it declares is
//! one the library document names, and every field of every view is a key of that view's fixture
//! rows. What the model needs and neither source says is an `UNMAPPED:` marker, not a declaration.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_yaml::Value;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library")
}

fn read_yaml(path: &Path) -> Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_yaml::from_str(&text)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()))
}

/// Every `.yaml` file below `dir`, sorted.
fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|error| panic!("cannot list {}: {error}", dir.display()));
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn list<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_sequence)
        .map_or(&[], Vec::as_slice)
}

/// What the model declares: its system name, its views with their fields, and its commands.
struct Model {
    system: String,
    views: BTreeMap<String, Vec<String>>,
    commands: BTreeSet<String>,
}

fn model() -> Model {
    let dir = example().join("model");
    let system = read_yaml(&dir.join("system.yaml"));
    let system = text(&system, "system")
        .expect("system.yaml names its system")
        .to_owned();
    let mut views = BTreeMap::new();
    let mut commands = BTreeSet::new();
    for file in yaml_files(&dir) {
        let spec = read_yaml(&file);
        for view in list(&spec, "views") {
            let name = text(view, "name").expect("a view has a name").to_owned();
            let fields = list(view, "fields")
                .iter()
                .map(|field| text(field, "name").expect("a field has a name").to_owned())
                .collect();
            views.insert(name, fields);
        }
        for command in list(&spec, "commands") {
            commands.insert(
                text(command, "name")
                    .expect("a command has a name")
                    .to_owned(),
            );
        }
    }
    Model {
        system,
        views,
        commands,
    }
}

/// Every string the document gives as a `view:` or a `does:`, anywhere in it.
fn document_names(value: &Value, names: &mut BTreeSet<String>) {
    match value {
        Value::Mapping(map) => {
            for (key, value) in map {
                if let (Some("view" | "does"), Some(name)) = (key.as_str(), value.as_str()) {
                    names.insert(name.to_owned());
                }
                document_names(value, names);
            }
        }
        Value::Sequence(items) => items.iter().for_each(|item| document_names(item, names)),
        _ => {}
    }
}

/// The keys of every fixture row, per view: `{view: V, rows: [...]}` and
/// `{views: {V: {rows: [...]}}}` are the two shapes a fixture file takes.
fn fixture_keys() -> BTreeMap<String, BTreeSet<String>> {
    let mut keys: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut add = |view: &str, body: &Value| {
        let entry = keys.entry(view.to_owned()).or_default();
        for row in list(body, "rows") {
            let row = row.as_mapping().expect("a fixture row is a map");
            entry.extend(row.keys().filter_map(Value::as_str).map(str::to_owned));
        }
    };
    for file in yaml_files(&example().join("fixtures")) {
        let fixture = read_yaml(&file);
        if let Some(view) = text(&fixture, "view") {
            add(view, &fixture);
        }
        if let Some(views) = fixture.get("views").and_then(Value::as_mapping) {
            for (view, body) in views {
                if let Some(view) = view.as_str() {
                    add(view, body);
                }
            }
        }
    }
    keys
}

#[test]
fn model_invents_nothing() {
    let model = model();
    assert!(
        !model.views.is_empty() && !model.commands.is_empty(),
        "the model declares no view or no command, so there is nothing to hold to the document"
    );
    let prefix = format!("{}.", model.system);
    let local = |name: &str| name.strip_prefix(&prefix).unwrap_or(name).to_owned();

    let mut named = BTreeSet::new();
    document_names(&read_yaml(&example().join("library.ui.yaml")), &mut named);
    let invented: Vec<&String> = model
        .views
        .keys()
        .chain(&model.commands)
        .filter(|name| !named.contains(name.as_str()) && !named.contains(&local(name)))
        .collect();
    assert!(
        invented.is_empty(),
        "the model declares {invented:?}, which library.ui.yaml names nowhere (it names {named:?})"
    );

    let rows = fixture_keys();
    let mut unbacked = Vec::new();
    for (view, fields) in &model.views {
        let keys = rows.get(&local(view));
        for field in fields {
            if !keys.is_some_and(|keys| keys.contains(field)) {
                unbacked.push(format!("{view}.{field}"));
            }
        }
    }
    assert!(
        unbacked.is_empty(),
        "these view fields are a key of no fixture row of their view: {unbacked:?}"
    );
}
