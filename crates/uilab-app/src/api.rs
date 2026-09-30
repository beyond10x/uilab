//! The operator API: an operator that cannot hold a WebSocket open (an agent driving uilab through a
//! shell) registers, then acts one message at a time and gets back what that message caused.
//! Its actions go through the same session task as a browser's, so every browser sees them live.

use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::Shared;
use crate::app::Cmd;
use crate::wire::{self, Client, Server};

#[derive(Deserialize)]
pub struct Join {
    pub name: String,
    /// `agent` or `human`; `agent` when absent.
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Joined {
    pub operator_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Act {
    pub operator_id: String,
    pub message: Client,
    /// How long to wait for the action to settle; 90 s when absent.
    #[serde(default)]
    pub wait_ms: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct Acted {
    /// Whether the action produced its settling message before the wait ran out.
    pub settled: bool,
    /// Every message the action caused, in order: attributed to this operator, or unattributed
    /// (a document snapshot, rows). Presence is left out.
    pub messages: Vec<Server>,
}

pub async fn join(
    State(shared): State<Shared>,
    Json(join): Json<Join>,
) -> Result<Json<Joined>, (StatusCode, String)> {
    let (reply, answer) = oneshot::channel();
    let agent = join.kind.as_deref() != Some("human");
    shared
        .inbox
        .send(Cmd::Register {
            name: join.name,
            agent,
            reply,
        })
        .await
        .map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "the session is gone".to_owned(),
            )
        })?;
    let operator_id = answer.await.map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "the session is gone".to_owned(),
        )
    })?;
    Ok(Json(Joined { operator_id }))
}

pub async fn state(State(shared): State<Shared>) -> Result<Json<Server>, (StatusCode, String)> {
    let (reply, answer) = oneshot::channel();
    shared
        .inbox
        .send(Cmd::Snapshot { reply })
        .await
        .map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "the session is gone".to_owned(),
            )
        })?;
    answer.await.map(Json).map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "the session is gone".to_owned(),
        )
    })
}

pub async fn act(
    State(shared): State<Shared>,
    Json(act): Json<Act>,
) -> Result<Json<Acted>, (StatusCode, String)> {
    let gone = || {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "the session is gone".to_owned(),
        )
    };
    let (reply, answer) = oneshot::channel();
    shared
        .inbox
        .send(Cmd::Touch {
            operator: act.operator_id.clone(),
            reply,
        })
        .await
        .map_err(|_| gone())?;
    if !answer.await.map_err(|_| gone())? {
        return Err((
            StatusCode::NOT_FOUND,
            format!("no operator `{}`: join again", act.operator_id),
        ));
    }

    let me = act.operator_id.clone();
    let settles_immediately = matches!(&act.message, Client::Mic(mic) if wire::mic_open(mic))
        || matches!(&act.message, Client::Hello(_));
    let kind = Settle::of(&act.message);
    let mut out = shared.out.subscribe();
    shared
        .inbox
        .send(Cmd::Client {
            by: me.clone(),
            message: act.message,
        })
        .await
        .map_err(|_| gone())?;
    if settles_immediately {
        return Ok(Json(Acted {
            settled: true,
            messages: Vec::new(),
        }));
    }

    let wait = Duration::from_millis(act.wait_ms.unwrap_or(90_000));
    let mut messages = Vec::new();
    let collected = tokio::time::timeout(wait, async {
        loop {
            match out.recv().await {
                Ok(message) => {
                    let mine = message.by().is_none_or(|by| by == me);
                    if !mine || matches!(message, Server::Presence(_)) {
                        continue;
                    }
                    let done = kind.settled_by(&message);
                    messages.push(message);
                    if done {
                        return true;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return false,
            }
        }
    })
    .await;
    Ok(Json(Acted {
        settled: collected.unwrap_or(false),
        messages,
    }))
}

/// Which message ends an action.
enum Settle {
    /// An instruction: a proposal, or why there is none.
    Proposal,
    /// An accept: the change it made.
    Change,
    /// Anything that answers with a snapshot.
    Document,
    /// A rows request.
    Rows,
}

impl Settle {
    fn of(message: &Client) -> Self {
        match message {
            Client::Say(_) | Client::Mic(_) => Settle::Proposal,
            Client::Accept(_) => Settle::Change,
            Client::Rows(_) => Settle::Rows,
            Client::Select(_)
            | Client::Reject(_)
            | Client::Undo(_)
            | Client::Resync(_)
            | Client::Hello(_) => Settle::Document,
        }
    }

    fn settled_by(&self, message: &Server) -> bool {
        if matches!(message, Server::Refused(_) | Server::Failed(_)) {
            return true;
        }
        match self {
            Settle::Proposal => matches!(message, Server::Proposal(_)),
            Settle::Change => matches!(message, Server::Changed(_) | Server::Document(_)),
            Settle::Document => matches!(message, Server::Document(_)),
            Settle::Rows => matches!(message, Server::Rows(_)),
        }
    }
}
