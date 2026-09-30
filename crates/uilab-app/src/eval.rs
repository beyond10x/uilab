//! `uilab op eval`: run an instruction suite through a live server as one operator, judge every
//! proposal against the case's expectations, reject it so the document stays as it was, and write
//! a report per round.

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::wire::Server;

#[derive(Deserialize)]
pub struct Suite {
    pub suite: String,
    #[serde(default)]
    pub document: Option<String>,
    pub cases: Vec<Case>,
}

#[derive(Deserialize, Clone)]
pub struct Case {
    pub id: String,
    pub target: String,
    pub say: String,
    pub expect: Expect,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Expect {
    /// `Insert`, `Replace` or `Remove`.
    #[serde(default)]
    pub op: Option<String>,
    /// Layer of the changed node.
    #[serde(default)]
    pub layer: Option<String>,
    /// Composite kind of the changed node.
    #[serde(default)]
    pub component: Option<String>,
    /// Longest acceptable time from instruction to proposal.
    #[serde(default)]
    pub max_ms: Option<u64>,
}

#[derive(Serialize)]
pub struct Outcome {
    pub id: String,
    pub target: String,
    pub say: String,
    pub pass: bool,
    /// Why it failed, one entry per unmet expectation.
    pub reasons: Vec<String>,
    pub ms: u64,
    /// `op target -> changed`, or the refusal.
    pub got: String,
    /// The proposed node as YAML.
    pub after: String,
    pub warnings: Vec<String>,
}

pub fn load(path: &Path) -> Result<Suite, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_yaml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Judges what one `say` caused against the case.
pub fn judge(case: &Case, messages: &[Server], ms: u64) -> Outcome {
    let mut reasons = Vec::new();
    let mut warnings = Vec::new();
    let got: String;
    let mut after = String::new();
    let proposal = messages.iter().find_map(|m| match m {
        Server::Proposal(p) => Some(p),
        _ => None,
    });
    match proposal {
        None => {
            let why = messages.iter().find_map(|m| match m {
                Server::Refused(r) => Some(format!("refused {}: {}", r.check, r.message)),
                Server::Failed(f) => Some(format!("failed: {}", f.message)),
                _ => None,
            });
            got = why.unwrap_or_else(|| "no answer before the wait ran out".into());
            reasons.push(got.clone());
        }
        Some(p) => {
            let op = serde_json::to_value(&p.op)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default();
            got = format!("{op} {} -> {}", p.target.0, p.changed.0);
            after = p.after.clone();
            // A batch may do what was asked plus the wiring it needs (a dialog and the row action
            // that opens it). It meets the case when its result holds a node of the expected
            // layer and kind; op, layer and component are then judged by that node.
            let node: Value = serde_yaml::from_str(&p.after).unwrap_or(Value::Null);
            let batch_judged = op == "Batch" && case.expect.op.as_deref() != Some("Batch");
            if batch_judged {
                match (&case.expect.layer, &case.expect.component) {
                    (Some(layer), component) if holds(&node, layer, component.as_deref()) => {}
                    (layer, component) => reasons.push(format!(
                        "batch holds no {} {}",
                        layer.as_deref().unwrap_or("node"),
                        component.as_deref().unwrap_or("")
                    )),
                }
            } else if let Some(want) = &case.expect.op
                && want != &op
            {
                reasons.push(format!("op {op}, expected {want}"));
            }
            let layer = p
                .changed
                .0
                .rsplit('/')
                .next()
                .and_then(|s| s.split(':').next())
                .unwrap_or("root")
                .to_owned();
            if !batch_judged
                && let Some(want) = &case.expect.layer
                && want != &layer
            {
                reasons.push(format!("layer {layer}, expected {want}"));
            }
            if !batch_judged && let Some(want) = &case.expect.component {
                let component = node["component"].as_str().unwrap_or("none");
                if want != component {
                    reasons.push(format!("component {component}, expected {want}"));
                }
            }
            let findings: Value = serde_json::to_value(&p.findings).unwrap_or_default();
            for f in findings.as_array().into_iter().flatten() {
                let line = format!(
                    "{} at {}: {}",
                    f["check"].as_str().unwrap_or(""),
                    f["path"].as_str().unwrap_or(""),
                    f["message"].as_str().unwrap_or("")
                );
                match (f["severity"].as_str(), f["check"].as_str()) {
                    (Some("error"), _) | (_, Some("column_fields")) => reasons.push(line),
                    _ => warnings.push(line),
                }
            }
        }
    }
    if let Some(max) = case.expect.max_ms
        && ms > max
    {
        reasons.push(format!("{ms} ms, over {max} ms"));
    }
    Outcome {
        id: case.id.clone(),
        target: case.target.clone(),
        say: case.say.clone(),
        pass: reasons.is_empty(),
        reasons,
        ms,
        got,
        after,
        warnings,
    }
}

/// Writes `<dir>/<suite>-round-<n>.json` and `.md`; returns the markdown path.
pub fn report(
    dir: &Path,
    suite: &Suite,
    round: u32,
    model: &str,
    outcomes: &[Outcome],
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let stem = format!("{}-round-{round}", suite.suite);
    let passed = outcomes.iter().filter(|o| o.pass).count();
    let json = serde_json::json!({
        "suite": suite.suite, "round": round, "model": model,
        "passed": passed, "cases": outcomes.len(), "outcomes": outcomes,
    });
    std::fs::write(
        dir.join(format!("{stem}.json")),
        serde_json::to_string_pretty(&json).expect("report serializes"),
    )
    .map_err(|e| e.to_string())?;
    let mut md = format!(
        "# {} round {round}\n\n{passed} of {} cases pass. Model: `{model}`. Document: `{}`. Median {} ms.\n\n| case | pass | ms | got | why |\n|---|---|---|---|---|\n",
        suite.suite,
        outcomes.len(),
        suite.document.as_deref().unwrap_or("-"),
        median(outcomes.iter().map(|o| o.ms).collect()),
    );
    for o in outcomes {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            o.id,
            if o.pass { "yes" } else { "no" },
            o.ms,
            o.got.replace('|', "\\|"),
            o.reasons.join("; ").replace('|', "\\|"),
        ));
    }
    let path = dir.join(format!("{stem}.md"));
    std::fs::write(&path, md).map_err(|e| e.to_string())?;
    Ok(path)
}

fn median(mut values: Vec<u64>) -> u64 {
    values.sort_unstable();
    values.get(values.len() / 2).copied().unwrap_or(0)
}

/// Milliseconds since `started`.
pub fn elapsed(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

/// Whether `node` (a YAML subtree read into JSON) holds, at any depth, a node of `layer` whose
/// component is `component` (any component when `None`).
fn holds(node: &Value, layer: &str, component: Option<&str>) -> bool {
    let key = match layer {
        "section" => "sections",
        "overlay" => "overlays",
        "widget" => "widgets",
        "item" => "item",
        "page" => "pages",
        "region" => "regions",
        _ => return false,
    };
    let Value::Object(map) = node else {
        return match node {
            Value::Array(items) => items.iter().any(|v| holds(v, layer, component)),
            _ => false,
        };
    };
    let here = map
        .get(key)
        .and_then(Value::as_object)
        .is_some_and(|children| {
            children
                .values()
                .any(|c| component.is_none_or(|want| c["component"].as_str() == Some(want)))
        });
    here || map.values().any(|v| holds(v, layer, component))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case(expect: Expect) -> Case {
        Case {
            id: "c".into(),
            target: "page:loans".into(),
            say: "s".into(),
            expect,
        }
    }

    fn proposal(after: &str, changed: &str, op: &str) -> Server {
        serde_json::from_value(serde_json::json!({
            "type": "proposal",
            "value": {
                "proposal_id": "p", "target": "page:loans", "changed": changed, "op": op,
                "utterance": "u", "before": "", "after": after, "findings": [], "by": "api-1",
                "outline": {"path": "/", "layer": "root", "name": "", "kind": "document", "children": []},
            }
        }))
        .unwrap()
    }

    #[test]
    fn a_batch_passes_when_it_holds_the_expected_node() {
        let expect = Expect {
            op: Some("Insert".into()),
            layer: Some("overlay".into()),
            component: Some("confirm".into()),
            max_ms: None,
        };
        let page = "kind: list_page\nsections:\n  list: {component: collection}\noverlays:\n  cancel: {kind: dialog, component: confirm}\n";
        let out = judge(
            &case(expect.clone()),
            &[proposal(page, "page:loans", "Batch")],
            10,
        );
        assert!(out.pass, "{:?}", out.reasons);
        let without = "kind: list_page\nsections:\n  list: {component: collection}\n";
        let out = judge(
            &case(expect),
            &[proposal(without, "page:loans", "Batch")],
            10,
        );
        assert_eq!(out.reasons, ["batch holds no overlay confirm"]);
    }

    #[test]
    fn a_matching_proposal_passes() {
        let expect = Expect {
            op: Some("Insert".into()),
            layer: Some("section".into()),
            component: Some("collection".into()),
            max_ms: None,
        };
        let out = judge(
            &case(expect),
            &[proposal(
                "component: collection\n",
                "page:loans/section:overdue",
                "Insert",
            )],
            10,
        );
        assert!(out.pass, "{:?}", out.reasons);
    }

    #[test]
    fn wrong_kind_layer_and_refusals_fail_with_reasons() {
        let expect = Expect {
            op: Some("Insert".into()),
            layer: Some("overlay".into()),
            component: Some("form".into()),
            max_ms: Some(5),
        };
        let out = judge(
            &case(expect.clone()),
            &[proposal(
                "component: record\n",
                "page:loans/section:x",
                "Replace",
            )],
            10,
        );
        assert_eq!(out.reasons.len(), 4, "{:?}", out.reasons);
        let refused = judge(
            &case(expect),
            &[Server::refused("opens_resolves", "m", Some("api-1"))],
            1,
        );
        assert!(!refused.pass);
        assert!(refused.reasons[0].starts_with("refused opens_resolves"));
    }
}
