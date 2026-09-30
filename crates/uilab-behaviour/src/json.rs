//! The one crossing between `serde_json` and the generated crate's own JSON value.

use uilab_types::json::Value;
use uilab_types::session::PatchBody;

/// A patch body from a `serde_json` value.
pub fn body_from_json(value: &serde_json::Value) -> PatchBody {
    PatchBody(from_serde(value))
}

/// A patch body as a `serde_json` value.
///
/// A number whose spelling `serde_json` does not read — only possible for a value built by hand —
/// comes out as a string of that spelling.
pub fn body_to_json(body: &PatchBody) -> serde_json::Value {
    to_serde(&body.0)
}

fn from_serde(value: &serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => Value::Number(n.to_string()),
        serde_json::Value::String(s) => Value::Text(s.clone()),
        serde_json::Value::Array(items) => Value::Array(items.iter().map(from_serde).collect()),
        serde_json::Value::Object(members) => Value::Object(
            members
                .iter()
                .map(|(k, v)| (k.clone(), from_serde(v)))
                .collect(),
        ),
    }
}

fn to_serde(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Number(spelling) => serde_json::from_str::<serde_json::Number>(spelling)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|_| serde_json::Value::String(spelling.clone())),
        Value::Text(s) => serde_json::Value::String(s.clone()),
        Value::Array(items) => serde_json::Value::Array(items.iter().map(to_serde).collect()),
        Value::Object(members) => serde_json::Value::Object(
            members
                .iter()
                .map(|(k, v)| (k.clone(), to_serde(v)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_in_order() {
        let original = serde_json::json!({
            "layer": "section",
            "name": "alerts",
            "node": {"component": "metric", "size": 5, "ratio": 1.5, "on": true, "none": null, "tags": ["a", "b"]}
        });
        let body = body_from_json(&original);
        assert_eq!(body_to_json(&body), original);
        let Value::Object(members) = &body.0 else {
            panic!("an object")
        };
        let keys: Vec<_> = members.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["layer", "name", "node"]);
    }
}
