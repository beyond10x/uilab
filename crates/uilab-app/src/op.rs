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
        text: String,
    },
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
        OpCommand::Say { target, text } => client(serde_json::json!({
            "type": "say",
            "value": match target { Some(t) => serde_json::json!({"text": text, "target": t}), None => serde_json::json!({"text": text}) },
        }))?,
        OpCommand::Select { path } => {
            client(serde_json::json!({"type": "select", "value": {"path": path}}))?
        }
        OpCommand::Accept { id } => decide("accept", id.or(cached.last_proposal.clone()))?,
        OpCommand::Reject { id } => decide("reject", id.or(cached.last_proposal.clone()))?,
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
    let acted = match act(&http, &cached, &message).await {
        Err(e) if e.starts_with("404") => {
            join(&http, &op.name, &mut cached).await?;
            act(&http, &cached, &message).await?
        }
        other => other?,
    };
    for message in &acted.messages {
        if let Server::Proposal(p) = message {
            cached.last_proposal = Some(p.proposal_id.0.clone());
        }
        print(message, op.json);
    }
    if !acted.settled {
        println!("(not settled before the wait ran out)");
    }
    save(&op.name, &cached)
}

fn client(value: Value) -> Result<Client, String> {
    serde_json::from_value(value).map_err(|e| e.to_string())
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
