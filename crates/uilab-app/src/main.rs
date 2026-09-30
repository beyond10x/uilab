//! `uilab serve`: the browser app, one WebSocket, and the session behind it.

mod app;
mod journal;
mod wire;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use clap::{Parser, Subcommand};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc};
use tower_http::services::ServeDir;

use crate::app::{App, Cmd, Config};
use crate::wire::{Client, Server};

#[derive(Parser)]
#[command(
    name = "uilab",
    about = "Edit a ui-spec/1 document by voice, in the browser"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the editor over one document.
    Serve(Serve),
}

#[derive(clap::Args)]
struct Serve {
    /// The ui-spec/1 document to edit; accepted patches are written back to it.
    #[arg(long)]
    doc: PathBuf,
    /// Address to listen on.
    #[arg(long, default_value = "127.0.0.1:8740")]
    listen: SocketAddr,
    /// The built browser app.
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../../widget/dist"))]
    assets: PathBuf,
    /// Whisper model (ggml). `task model` downloads the default.
    #[arg(long, default_value_t = default_model())]
    stt_model: String,
    /// Spoken language, e.g. `en` or `de`; detected when absent.
    #[arg(long)]
    language: Option<String>,
    /// Run speech on the CPU.
    #[arg(long)]
    stt_cpu: bool,
    /// No speech: instructions are typed.
    #[arg(long)]
    no_stt: bool,
    /// Model identifier the agent uses.
    #[arg(long)]
    model: Option<String>,
    /// Messages endpoint the agent uses (origin plus `/v1`).
    #[arg(long)]
    base_url: Option<String>,
    /// Where each run keeps its journal: events.jsonl and one WAV per utterance.
    #[arg(long, default_value_t = default_journal())]
    journal: String,
}

fn default_journal() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{home}/.cache/uilab/sessions")
}

fn default_model() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{home}/.cache/uilab/models/ggml-large-v3-turbo.bin")
}

#[derive(Clone)]
struct Shared {
    inbox: mpsc::Sender<Cmd>,
    out: broadcast::Sender<Server>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let Command::Serve(serve) = Cli::parse().command;
    if let Err(e) = run(serve).await {
        eprintln!("uilab: {e}");
        std::process::exit(1);
    }
}

async fn run(serve: Serve) -> Result<(), String> {
    if !serve.assets.join("index.html").is_file() {
        return Err(format!(
            "no browser app at {}: run `task widget` first",
            serve.assets.display()
        ));
    }
    let mut proposer = uilab_agent::ProposerConfig::default();
    if let Some(model) = serve.model {
        proposer.model = model;
    }
    if let Some(base_url) = serve.base_url {
        proposer.base_url = base_url;
    }
    let stt = (!serve.no_stt).then(|| uilab_stt::TranscriberConfig {
        model: PathBuf::from(&serve.stt_model),
        language: serve.language.clone(),
        gpu: !serve.stt_cpu,
    });
    let config = Config {
        doc: serve.doc.clone(),
        stt,
        proposer,
        journal: PathBuf::from(&serve.journal),
    };

    let (out, _) = broadcast::channel(256);
    let (inbox, rx) = mpsc::channel(1024);
    let app = {
        let out = out.clone();
        let back = inbox.clone();
        tokio::task::spawn_blocking(move || App::start(config, out, back))
            .await
            .map_err(|e| e.to_string())??
    };
    tokio::spawn(app.run(rx));

    let shared = Shared { inbox, out };
    let router = Router::new()
        .route("/ws", get(socket))
        .fallback_service(ServeDir::new(&serve.assets))
        .with_state(shared);
    let listener = tokio::net::TcpListener::bind(serve.listen)
        .await
        .map_err(|e| format!("{}: {e}", serve.listen))?;
    println!(
        "uilab: http://{} editing {}",
        serve.listen,
        serve.doc.display()
    );
    axum::serve(listener, router)
        .await
        .map_err(|e| e.to_string())
}

async fn socket(ws: WebSocketUpgrade, State(shared): State<Shared>) -> Response {
    ws.max_message_size(4 << 20)
        .on_upgrade(move |socket| connection(socket, shared))
}

async fn connection(socket: WebSocket, shared: Shared) {
    let (mut sink, mut stream) = socket.split();
    let mut out = shared.out.subscribe();
    let _ = shared.inbox.send(Cmd::Hello).await;
    let writer = tokio::spawn(async move {
        loop {
            match out.recv().await {
                Ok(message) => {
                    if sink
                        .send(Message::Text(message.to_text().into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
    while let Some(Ok(message)) = stream.next().await {
        let cmd = match message {
            Message::Text(text) => match Client::from_text(&text) {
                Ok(client) => Cmd::Client(client),
                Err(e) => {
                    let _ = shared
                        .out
                        .send(Server::failed(format!("unreadable message: {e}")));
                    continue;
                }
            },
            Message::Binary(bytes) => Cmd::Audio(
                bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|b| f32::from_le_bytes(*b))
                    .collect(),
            ),
            Message::Close(_) => break,
            _ => continue,
        };
        if shared.inbox.send(cmd).await.is_err() {
            break;
        }
    }
    writer.abort();
}
