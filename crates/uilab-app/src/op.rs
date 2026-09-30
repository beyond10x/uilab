//! `uilab op`: operate a running uilab from a shell, as a named operator the browser shows.
//!
//! Each command is one request to the operator API and prints what it caused. The operator id,
//! the server and the last proposal are kept in `~/.cache/uilab/operator-<name>.json`, so `accept`
//! and `reject` without an id act on the proposal the last `say` produced.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::{Act, Acted, Joined};
use crate::wire::{Client, Server};

#[derive(Args)]
pub struct Op {
    /// The operator acting.
    #[arg(long = "as", default_value = "Claude")]
    pub name: String,
    /// The server; defaults to the one this operator joined.
    #[arg(long)]
    pub server: Option<String>,
    /// Print the raw messages as JSON lines instead of a summary.
    #[arg(long)]
    pub json: bool,
    #[command(subcommand)]
    pub command: OpCommand,
}

#[derive(Subcommand)]
pub enum OpCommand {
    /// Join as this operator; the browser shows it from now on.
    Join {
        /// `agent` or `human`.
        #[arg(long, default_value = "agent")]
        kind: String,
    },
    /// Say an instruction at a node (or at the shared selection).
    Say {
        /// The node; the shared selection when absent.
        #[arg(long)]
        target: Option<String>,
        /// Hold this proposal for accept or reject, whatever the session setting.
        #[arg(long)]
        review: bool,
        text: String,
    },
    /// Give the agent a goal: it plans steps and proposes them one at a time. Waits until the
    /// goal is done, stopped or failed (up to 15 minutes) and prints each step with its status.
    Goal {
        /// The node the goal is planned at; the shared selection when absent.
        #[arg(long)]
        target: Option<String>,
        /// The most steps the plan may have; 8 when absent.
        #[arg(long)]
        max_steps: Option<u64>,
        /// Apply each step's proposal at once instead of waiting for accept or reject.
        #[arg(long)]
        auto: bool,
        text: String,
    },
    /// Stop the running goal; the one the server runs when no id is given.
    StopGoal { id: Option<String> },
    /// Select a node for everybody.
    Select { path: String },
    /// Accept a proposal; the last one this operator got when no id is given.
    Accept { id: Option<String> },
    /// Reject a proposal; the last one this operator got when no id is given.
    Reject { id: Option<String> },
    /// Undo an accepted proposal; the latest undoable one when no id is given.
    Undo { id: Option<String> },
    /// Print the document: revision, selection, findings and the tree.
    State,
    /// Run an instruction suite, judge each proposal, reject it, and write a report.
    Eval {
        /// The suite file.
        suite: PathBuf,
        /// Round number, part of the report name.
        #[arg(long, default_value_t = 1)]
        round: u32,
        /// Report directory.
        #[arg(long, default_value = "evals/reports")]
        out: PathBuf,
        /// The agent model the server runs, for the report.
        #[arg(long, default_value = uilab_agent::DEFAULT_MODEL)]
        model: String,
        /// Run only these case ids.
        #[arg(long)]
        only: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Default)]
struct Cached {
    server: String,
    operator_id: String,
    kind: String,
    last_proposal: Option<String>,
}

fn cache_path(name: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(format!(
        ".cache/uilab/operator-{}.json",
        name.to_lowercase()
    ))
}

fn load(name: &str) -> Option<Cached> {
    serde_json::from_str(&std::fs::read_to_string(cache_path(name)).ok()?).ok()
}

fn save(name: &str, cached: &Cached) -> Result<(), String> {
    let path = cache_path(name);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(
        &path,
        serde_json::to_string_pretty(cached).expect("cache serializes"),
    )
    .map_err(|e| e.to_string())
}

pub async fn run(op: Op) -> Result<(), String> {
    let http = reqwest::Client::new();
    let mut cached = load(&op.name).unwrap_or_default();
    if let Some(server) = &op.server {
        cached.server = server.trim_end_matches('/').to_owned();
    }
    if cached.server.is_empty() {
        cached.server = "http://127.0.0.1:8740".to_owned();
    }

    let message = match op.command {
        OpCommand::Join { kind } => {
            cached.kind = kind;
            join(&http, &op.name, &mut cached).await?;
            save(&op.name, &cached)?;
            println!(
                "joined {} as {} ({}), operator {}",
                cached.server, op.name, cached.kind, cached.operator_id
            );
            return Ok(());
        }
        OpCommand::State => {
            let document: Server = get(&http, &format!("{}/api/state", cached.server)).await?;
            print(&document, op.json);
            return Ok(());
        }
        OpCommand::Eval {
            suite,
            round,
            out,
            model,
            only,
        } => {
            let suite = crate::eval::load(&suite)?;
            if cached.operator_id.is_empty() {
                cached.kind = "agent".into();
                join(&http, &op.name, &mut cached).await?;
            }
            let mut outcomes = Vec::new();
            for case in suite
                .cases
                .iter()
                .filter(|c| only.is_empty() || only.contains(&c.id))
            {
                let say = client(
                    serde_json::json!({"type": "say", "value": {"text": case.say, "target": case.target, "review": true}}),
                )?;
                let started = std::time::Instant::now();
                let acted = match act(&http, &cached, &say).await {
                    Err(e) if e.starts_with("404") => {
                        join(&http, &op.name, &mut cached).await?;
                        act(&http, &cached, &say).await?
                    }
                    other => other?,
                };
                let ms = crate::eval::elapsed(started);
                let outcome = crate::eval::judge(case, &acted.messages, ms);
                println!(
                    "{} {:<20} {:>6} ms  {}{}",
                    if outcome.pass { "pass" } else { "FAIL" },
                    outcome.id,
                    outcome.ms,
                    outcome.got,
                    if outcome.pass {
                        String::new()
                    } else {
                        format!("  [{}]", outcome.reasons.join("; "))
                    },
                );
                for m in &acted.messages {
                    if let Server::Proposal(p) = m {
                        let reject = client(
                            serde_json::json!({"type": "reject", "value": {"proposal_id": p.proposal_id.0}}),
                        )?;
                        act(&http, &cached, &reject).await?;
                    }
                }
                outcomes.push(outcome);
            }
            let path = crate::eval::report(&out, &suite, round, &model, &outcomes)?;
            let passed = outcomes.iter().filter(|o| o.pass).count();
            println!(
                "{passed} of {} pass; report {}",
                outcomes.len(),
                path.display()
            );
            save(&op.name, &cached)?;
            return Ok(());
        }
        OpCommand::Say {
            target,
            text,
            review,
        } => client(serde_json::json!({
            "type": "say",
            "value": match (target, review) { (Some(t), true) => serde_json::json!({"text": text, "target": t, "review": true}), (Some(t), false) => serde_json::json!({"text": text, "target": t}), (None, true) => serde_json::json!({"text": text, "review": true}), (None, false) => serde_json::json!({"text": text}) },
        }))?,
        OpCommand::Goal {
            target,
            max_steps,
            auto,
            text,
        } => client(goal_message(text, target, max_steps, auto))?,
        OpCommand::StopGoal { id } => {
            let id = match id {
                Some(id) => id,
                None => {
                    let goal: Option<Server> =
                        get(&http, &format!("{}/api/goal", cached.server)).await?;
                    match goal {
                        Some(Server::Goal(g)) if !crate::wire::goal_ended(&g) => g.goal_id,
                        _ => return Err("no goal is running".to_owned()),
                    }
                }
            };
            client(serde_json::json!({"type": "stop_goal", "value": {"goal_id": id}}))?
        }
        OpCommand::Select { path } => {
            client(serde_json::json!({"type": "select", "value": {"path": path}}))?
        }
        OpCommand::Accept { id } => {
            let id = match id {
                Some(id) => Some(id),
                None => waiting_proposal(&http, &cached).await,
            };
            decide("accept", id)?
        }
        OpCommand::Reject { id } => {
            let id = match id {
                Some(id) => Some(id),
                None => waiting_proposal(&http, &cached).await,
            };
            decide("reject", id)?
        }
        OpCommand::Undo { id } => {
            let id = match id {
                Some(id) => Some(id),
                None => {
                    let document: Value =
                        get(&http, &format!("{}/api/state", cached.server)).await?;
                    document["value"]["undoable"].as_str().map(str::to_owned)
                }
            };
            decide("undo", id)?
        }
    };

    if cached.operator_id.is_empty() {
        cached.kind = "agent".into();
        join(&http, &op.name, &mut cached).await?;
    }
    let goal_run = matches!(message, Client::Goal(_)) && !op.json;
    let watched = match act_watching(&http, &cached, &message, goal_run).await {
        Err(e) if e.starts_with("404") => {
            join(&http, &op.name, &mut cached).await?;
            act_watching(&http, &cached, &message, goal_run).await?
        }
        other => other?,
    };
    let acted = watched.acted;
    let mut last_goal = watched.seen;
    for message in &acted.messages {
        if let Server::Proposal(p) = message {
            cached.last_proposal = Some(p.proposal_id.0.clone());
        }
        if goal_run {
            // The steps were printed as they moved; what is left is the goal's last word and
            // why steps were refused.
            match message {
                Server::Goal(g) if Some(&g.goal_id) != watched.prior.as_ref() => {
                    goal_progress(last_goal.as_ref(), g);
                    last_goal = Some(g.clone());
                }
                Server::Refused(_) | Server::Failed(_) => print(message, false),
                _ => {}
            }
        } else {
            print(message, op.json);
        }
    }
    if let (Some(g), true) = (&last_goal, goal_run) {
        goal_summary(g);
    }
    if !acted.settled {
        println!("(not settled before the wait ran out)");
    }
    save(&op.name, &cached)
}

fn client(value: Value) -> Result<Client, String> {
    serde_json::from_value(value).map_err(|e| e.to_string())
}

/// An act, and for a goal what was printed of it while it ran.
struct Watched {
    acted: Acted,
    /// The goal the server held before the act; not this act's.
    prior: Option<String>,
    /// This act's goal as last printed.
    seen: Option<uilab_wire::UilabWireGoal>,
}

/// The goal the server holds, if any.
async fn current_goal(
    http: &reqwest::Client,
    cached: &Cached,
) -> Result<Option<uilab_wire::UilabWireGoal>, String> {
    let goal: Option<Server> = get(http, &format!("{}/api/goal", cached.server)).await?;
    Ok(match goal {
        Some(Server::Goal(g)) => Some(g),
        _ => None,
    })
}

/// The proposal the running goal waits on, else the last one this operator got.
async fn waiting_proposal(http: &reqwest::Client, cached: &Cached) -> Option<String> {
    let goal = current_goal(http, cached).await.ok().flatten();
    goal.as_ref()
        .and_then(waiting_on)
        .or_else(|| cached.last_proposal.clone())
}

/// The proposal a running goal's current step waits on for accept or reject.
fn waiting_on(goal: &uilab_wire::UilabWireGoal) -> Option<String> {
    if crate::wire::goal_ended(goal) {
        return None;
    }
    let step = goal
        .steps
        .get(usize::try_from(present(&goal.current)?.as_u64()?).ok()?)?;
    (name(&step.status) == "proposed")
        .then(|| present(&step.proposal_id).map(|id| id.0.clone()))
        .flatten()
}

/// Acts; with `watch`, polls the goal every 2 s meanwhile and prints each step as it moves,
/// with the proposal id a review waits on.
async fn act_watching(
    http: &reqwest::Client,
    cached: &Cached,
    message: &Client,
    watch: bool,
) -> Result<Watched, String> {
    if !watch {
        return Ok(Watched {
            acted: act(http, cached, message).await?,
            prior: None,
            seen: None,
        });
    }
    let prior = current_goal(http, cached).await?.map(|g| g.goal_id);
    let mut seen: Option<uilab_wire::UilabWireGoal> = None;
    let mut acting = std::pin::pin!(act(http, cached, message));
    let acted = loop {
        tokio::select! {
            acted = &mut acting => break acted?,
            () = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
                if let Ok(Some(g)) = current_goal(http, cached).await
                    && Some(&g.goal_id) != prior.as_ref()
                {
                    goal_progress(seen.as_ref(), &g);
                    seen = Some(g);
                }
            }
        }
    };
    Ok(Watched { acted, prior, seen })
}

fn goal_message(text: String, target: Option<String>, max_steps: Option<u64>, auto: bool) -> Value {
    let mut value = serde_json::json!({"text": text});
    if let Some(target) = target {
        value["target"] = target.into();
    }
    if let Some(max_steps) = max_steps {
        value["max_steps"] = max_steps.into();
    }
    if auto {
        value["review"] = false.into();
    }
    serde_json::json!({"type": "goal", "value": value})
}

/// The spec name of a generated enum value.
fn name<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// What changed since the previous goal message: its state, and each step whose status moved.
fn goal_progress(before: Option<&uilab_wire::UilabWireGoal>, goal: &uilab_wire::UilabWireGoal) {
    let state = name(&goal.state);
    if before.is_none_or(|b| name(&b.state) != state) {
        match &goal.message {
            uilab_wire::EssPresence::Present(m) => println!("goal {} {state}: {m}", goal.goal_id),
            uilab_wire::EssPresence::Absent => println!("goal {} {state}", goal.goal_id),
        }
    }
    let n = goal.steps.len();
    for (i, step) in goal.steps.iter().enumerate() {
        let status = name(&step.status);
        let was = before.and_then(|b| b.steps.get(i)).map(|s| name(&s.status));
        if was.as_deref() != Some(status.as_str()) && status != "pending" {
            let proposal = match (status.as_str(), present(&step.proposal_id)) {
                ("proposed", Some(id)) => format!(" [proposal {}: accept or reject it]", id.0),
                _ => String::new(),
            };
            println!(
                "  step {}/{n} {status:<9} {}: {}{proposal}",
                i + 1,
                step.target.0,
                step.instruction
            );
        }
    }
}

fn present<T>(value: &uilab_wire::EssPresence<T>) -> Option<&T> {
    match value {
        uilab_wire::EssPresence::Present(v) => Some(v),
        uilab_wire::EssPresence::Absent => None,
    }
}

/// Every step of a goal with its status.
fn goal_summary(goal: &uilab_wire::UilabWireGoal) {
    println!(
        "goal {} {}: \"{}\"",
        goal.goal_id,
        name(&goal.state),
        goal.text
    );
    if let uilab_wire::EssPresence::Present(m) = &goal.message {
        println!("  {m}");
    }
    let n = goal.steps.len();
    for (i, step) in goal.steps.iter().enumerate() {
        let proposal = match &step.proposal_id {
            uilab_wire::EssPresence::Present(id) => format!(" ({})", id.0),
            uilab_wire::EssPresence::Absent => String::new(),
        };
        println!(
            "  {}/{n} {:<9} {} {}{proposal}",
            i + 1,
            name(&step.status),
            step.target.0,
            step.instruction
        );
    }
}

fn decide(kind: &str, id: Option<String>) -> Result<Client, String> {
    let id = id.ok_or_else(|| format!("no proposal to {kind}: give an id"))?;
    client(serde_json::json!({"type": kind, "value": {"proposal_id": id}}))
}

async fn join(http: &reqwest::Client, name: &str, cached: &mut Cached) -> Result<(), String> {
    let kind = if cached.kind.is_empty() {
        "agent"
    } else {
        cached.kind.as_str()
    };
    let response = http
        .post(format!("{}/api/operators", cached.server))
        .json(&serde_json::json!({"name": name, "kind": kind}))
        .send()
        .await
        .map_err(|e| format!("{}: {e}", cached.server))?;
    let joined: Joined = decode(response).await?;
    cached.operator_id = joined.operator_id;
    Ok(())
}

async fn act(http: &reqwest::Client, cached: &Cached, message: &Client) -> Result<Acted, String> {
    let body = Act {
        operator_id: cached.operator_id.clone(),
        message: message.clone(),
        wait_ms: None,
    };
    let response = http
        .post(format!("{}/api/act", cached.server))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("{}: {e}", cached.server))?;
    decode(response).await
}

async fn get<T: serde::de::DeserializeOwned>(
    http: &reqwest::Client,
    url: &str,
) -> Result<T, String> {
    let response = http
        .get(url)
        .send()
        .await
        .map_err(|e| format!("{url}: {e}"))?;
    decode(response).await
}

async fn decode<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> Result<T, String> {
    let status = response.status();
    let text = response.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("{} {text}", status.as_u16()));
    }
    serde_json::from_str(&text).map_err(|e| format!("{e}: {text}"))
}

fn print(message: &Server, json: bool) {
    if json {
        println!("{}", message.to_text());
        return;
    }
    match message {
        Server::Document(d) => {
            println!("document revision {} selected {}", d.revision, d.selected.0);
            let findings: Value = serde_json::to_value(&d.findings).unwrap_or_default();
            for f in findings.as_array().into_iter().flatten() {
                println!(
                    "  finding {} {} at {}: {}",
                    f["severity"].as_str().unwrap_or(""),
                    f["check"].as_str().unwrap_or(""),
                    f["path"].as_str().unwrap_or(""),
                    f["message"].as_str().unwrap_or("")
                );
            }
            tree(&serde_json::to_value(&d.outline).unwrap_or_default(), 1);
        }
        Server::Transcript(t) => println!(
            "heard \"{}\" ({} ms of audio in {} ms)",
            t.text, t.audio_ms, t.took_ms
        ),
        Server::Thinking(t) => println!("thinking at {}", t.target.0),
        Server::Proposal(p) => {
            let op: Value = serde_json::to_value(&p.op).unwrap_or_default();
            println!(
                "proposal {} {} {} -> {}",
                p.proposal_id.0,
                op.as_str().unwrap_or(""),
                p.target.0,
                p.changed.0
            );
            for line in p.after.lines() {
                println!("  + {line}");
            }
            let findings: Value = serde_json::to_value(&p.findings).unwrap_or_default();
            for f in findings.as_array().into_iter().flatten() {
                println!(
                    "  finding {} {}: {}",
                    f["severity"].as_str().unwrap_or(""),
                    f["check"].as_str().unwrap_or(""),
                    f["message"].as_str().unwrap_or("")
                );
            }
        }
        Server::Changed(c) => {
            let op: Value = serde_json::to_value(&c.op).unwrap_or_default();
            println!(
                "changed revision {} {} {}",
                c.revision,
                op.as_str().unwrap_or(""),
                c.changed.0
            );
        }
        Server::Refused(r) => println!("refused {}: {}", r.check, r.message),
        Server::Failed(f) => println!("failed: {}", f.message),
        Server::Rows(r) => println!("rows {}: {}", r.view, r.rows.len()),
        Server::Presence(p) => println!("presence: {} operators", p.operators.len()),
        Server::Goal(g) => goal_summary(g),
    }
}

fn tree(node: &Value, depth: usize) {
    let label = match node["layer"].as_str() {
        Some("root") => "/".to_owned(),
        Some("nav") => "nav".to_owned(),
        Some(layer) => format!("{layer}:{}", node["name"].as_str().unwrap_or("")),
        None => return,
    };
    let mut line = format!(
        "{}{label} ({})",
        "  ".repeat(depth),
        node["kind"].as_str().unwrap_or("")
    );
    if let Some(title) = node["title"].as_str() {
        line.push_str(&format!(" \"{title}\""));
    }
    if let Some(view) = node["view"].as_str() {
        line.push_str(&format!(" reads {view}"));
    }
    println!("{line}");
    for child in node["children"].as_array().into_iter().flatten() {
        tree(child, depth + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goal::Goal;

    fn wire_goal(f: impl FnOnce(&mut Goal)) -> uilab_wire::UilabWireGoal {
        let mut g = Goal::new(
            "goal-1",
            "api-1",
            "t",
            "page:members".parse().unwrap(),
            8,
            true,
        );
        f(&mut g);
        match crate::wire::goal(&g) {
            Server::Goal(g) => g,
            _ => unreachable!(),
        }
    }

    fn steps(n: usize) -> Vec<uilab_agent::Step> {
        (0..n)
            .map(|i| uilab_agent::Step {
                instruction: format!("step {i}"),
                target: "page:members".parse().unwrap(),
                why: "w".into(),
            })
            .collect()
    }

    #[test]
    fn a_bare_accept_or_reject_acts_on_the_proposal_the_goal_waits_on() {
        let waiting = wire_goal(|g| {
            g.planned(steps(2));
            g.proposed(0, "p1");
        });
        assert_eq!(waiting_on(&waiting).as_deref(), Some("p1"));
    }

    #[test]
    fn nothing_is_waited_on_while_planning_thinking_after_a_decision_or_once_over() {
        let cases = [
            wire_goal(|_| {}),
            wire_goal(|g| {
                g.planned(steps(2));
            }),
            wire_goal(|g| {
                g.planned(steps(2));
                g.proposed(0, "p1");
                g.decided("p1", true);
            }),
            wire_goal(|g| {
                g.planned(steps(1));
                g.proposed(0, "p1");
                g.stop();
            }),
        ];
        for goal in &cases {
            assert_eq!(waiting_on(goal), None, "{}", crate::wire::goal_state(goal));
        }
    }
}
