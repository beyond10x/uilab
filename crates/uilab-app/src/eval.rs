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
    /// The words are not an instruction: the case passes only when the agent declines.
    #[serde(default)]
    pub declined: bool,
    /// The agent moves the target to this path before it proposes (or instead of proposing).
    #[serde(default)]
    pub moved_to: Option<String>,
    /// The agent must not move the target.
    #[serde(default)]
    pub stays: bool,
    /// The instruction only navigates: the case passes only when nothing is proposed.
    #[serde(default)]
    pub navigate_only: bool,
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

/// Judges what one `say` caused against the case: the move the agent made, if any, and the
/// proposal, refusal or navigation that ended it.
pub fn judge(case: &Case, messages: &[Server], ms: u64) -> Outcome {
    let mut reasons = Vec::new();
    let mut warnings = Vec::new();
    let got: String;
    let mut after = String::new();
    let proposal = messages.iter().find_map(|m| match m {
        Server::Proposal(p) => Some(p),
        _ => None,
    });
    let moved = messages.iter().find_map(|m| match m {
        Server::Moved(m) => Some(m),
        _ => None,
    });
    match (&case.expect.moved_to, moved) {
        (Some(want), None) => reasons.push(format!("no move, expected a move to {want}")),
        (Some(want), Some(m)) if &m.to.0 != want => {
            reasons.push(format!("moved to {}, expected a move to {want}", m.to.0));
        }
        _ => {}
    }
    if case.expect.stays
        && let Some(m) = moved
    {
        reasons.push(format!("moved to {}, expected no move", m.to.0));
    }
    match proposal {
        None if case.expect.navigate_only && moved.is_some_and(|m| m.navigate_only) => {
            got = format!("moved to {} (navigate only)", moved.map_or("", |m| &m.to.0));
        }
        Some(p) if case.expect.navigate_only => {
            got = format!("proposed at {}", p.changed.0);
            after = p.after.clone();
            reasons.push(format!("{got}, expected no proposal after a navigation"));
        }
        None if moved.is_some_and(|m| m.navigate_only) => {
            got = format!("moved to {} (navigate only)", moved.map_or("", |m| &m.to.0));
            let wanted = if case.expect.declined {
                "a decline"
            } else {
                "a proposal"
            };
            reasons.push(format!("{got}, expected {wanted}"));
        }
        None => {
            let why = messages.iter().find_map(|m| match m {
                Server::Refused(r) => Some(format!("refused {}: {}", r.check, r.message)),
                Server::Failed(f) => Some(format!("failed: {}", f.message)),
                _ => None,
            });
            got = why.unwrap_or_else(|| "no answer before the wait ran out".into());
            if !(case.expect.declined && got.starts_with("refused declined")) {
                reasons.push(got.clone());
            }
        }
        Some(p) if case.expect.declined => {
            got = format!(
                "proposed at {} for words that are no instruction",
                p.changed.0
            );
            after = p.after.clone();
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
                    (None, None) => {}
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
    let got = match moved {
        Some(m) if proposal.is_some() => format!("moved to {}; {got}", m.to.0),
        _ => got,
    };
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
    // Children are a map (sections, overlays, widgets) or, for items, a list of named nodes.
    let matches = |c: &Value| component.is_none_or(|want| c["component"].as_str() == Some(want));
    let here = match map.get(key) {
        Some(Value::Object(children)) => children.values().any(matches),
        Some(Value::Array(children)) => children.iter().any(matches),
        _ => false,
    };
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
            declined: false,
            ..Expect::default()
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
            declined: false,
            ..Expect::default()
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
            declined: false,
            ..Expect::default()
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

    fn moved(to: &str, navigate_only: bool) -> Server {
        serde_json::from_value(serde_json::json!({
            "type": "moved",
            "value": {
                "by": "api-1", "selected_by": "api-1", "from": "page:loans/section:list",
                "to": to, "reason": "r", "navigate_only": navigate_only, "utterance": "u",
            }
        }))
        .unwrap()
    }

    #[test]
    fn a_case_expecting_a_move_passes_only_on_a_move_there() {
        let expect = Expect {
            moved_to: Some("/".into()),
            op: Some("Insert".into()),
            layer: Some("page".into()),
            ..Expect::default()
        };
        let page = proposal("kind: list_page\n", "page:overdue", "Insert");
        let out = judge(
            &case(expect.clone()),
            &[moved("/", false), page.clone()],
            10,
        );
        assert!(out.pass, "{:?}", out.reasons);
        assert!(out.got.contains("moved to /"), "{}", out.got);

        let out = judge(&case(expect.clone()), std::slice::from_ref(&page), 10);
        assert_eq!(out.reasons, ["no move, expected a move to /"]);
        let out = judge(&case(expect), &[moved("nav", false), page], 10);
        assert_eq!(out.reasons, ["moved to nav, expected a move to /"]);
    }

    #[test]
    fn a_navigation_case_passes_on_a_move_and_no_proposal() {
        let expect = Expect {
            moved_to: Some("page:members".into()),
            navigate_only: true,
            ..Expect::default()
        };
        let out = judge(&case(expect.clone()), &[moved("page:members", true)], 10);
        assert!(out.pass, "{:?}", out.reasons);
        let out = judge(
            &case(expect.clone()),
            &[
                moved("page:members", false),
                proposal("component: record\n", "page:members/section:x", "Insert"),
            ],
            10,
        );
        assert!(!out.pass);
        assert!(
            out.reasons
                .iter()
                .any(|r| r.contains("expected no proposal")),
            "{:?}",
            out.reasons
        );
    }

    #[test]
    fn a_case_that_stays_fails_on_a_move() {
        let expect = Expect {
            stays: true,
            ..Expect::default()
        };
        let list = proposal(
            "component: collection\n",
            "page:loans/section:list",
            "Replace",
        );
        assert!(judge(&case(expect.clone()), std::slice::from_ref(&list), 10).pass);
        let out = judge(&case(expect), &[moved("/", false), list], 10);
        assert_eq!(out.reasons, ["moved to /, expected no move"]);
    }

    #[test]
    fn the_library_suite_loads_with_its_move_cases() {
        let suite = load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../evals/library.yaml"),
        )
        .unwrap();
        let by_id = |id: &str| {
            suite
                .cases
                .iter()
                .find(|c| c.id == id)
                .unwrap_or_else(|| panic!("case {id}"))
        };
        let new_page = by_id("retarget-new-page");
        assert_eq!(new_page.target, "page:loans/section:list");
        assert_eq!(new_page.expect.moved_to.as_deref(), Some("/"));
        assert_eq!(new_page.expect.layer.as_deref(), Some("page"));
        let navigate = by_id("retarget-navigate");
        assert!(navigate.expect.navigate_only);
        assert_eq!(navigate.expect.moved_to.as_deref(), Some("page:members"));
        assert!(by_id("retarget-none").expect.stays);
    }

    /// Adversary (story:agent-retarget): a navigation-only move settles the act (api `Settle`), so
    /// a case that did not expect one fails on the move, not on a wait that never ran out.
    #[test]
    fn adversary_an_unexpected_navigation_is_reported_as_the_move_it_was() {
        let out = judge(&case(Expect::default()), &[moved("page:members", true)], 10);
        assert!(!out.pass);
        assert!(
            out.got.contains("moved to page:members"),
            "got: {}; reasons: {:?}",
            out.got,
            out.reasons
        );
        assert!(
            !out.reasons.iter().any(|r| r.contains("wait ran out")),
            "{:?}",
            out.reasons
        );
    }

    /// Coordinator decision (round 2): a navigation-only move the case did not expect is
    /// reported as "moved to X (navigate only)", and the reason says what the case wanted.
    #[test]
    fn an_unexpected_navigation_names_the_move_and_what_the_case_wanted() {
        let navigated = [moved("page:members", true)];
        let wants_a_patch = Expect {
            op: Some("Insert".into()),
            ..Expect::default()
        };
        let out = judge(&case(wants_a_patch), &navigated, 10);
        assert_eq!(out.got, "moved to page:members (navigate only)");
        assert_eq!(
            out.reasons,
            ["moved to page:members (navigate only), expected a proposal"]
        );

        let wants_a_decline = Expect {
            declined: true,
            ..Expect::default()
        };
        let out = judge(&case(wants_a_decline), &navigated, 10);
        assert_eq!(out.got, "moved to page:members (navigate only)");
        assert_eq!(
            out.reasons,
            ["moved to page:members (navigate only), expected a decline"]
        );

        let stays = Expect {
            stays: true,
            ..Expect::default()
        };
        let out = judge(&case(stays), &navigated, 10);
        assert_eq!(out.got, "moved to page:members (navigate only)");
        assert_eq!(
            out.reasons,
            [
                "moved to page:members, expected no move",
                "moved to page:members (navigate only), expected a proposal"
            ]
        );

        let expected_move_then_patch = Expect {
            moved_to: Some("page:members".into()),
            ..Expect::default()
        };
        let out = judge(&case(expected_move_then_patch), &navigated, 10);
        assert_eq!(
            out.reasons,
            ["moved to page:members (navigate only), expected a proposal"]
        );
    }
}
