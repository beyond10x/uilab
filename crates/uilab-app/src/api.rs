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
    /// How long to wait for the action to settle; 90 s when absent, 15 minutes for a goal.
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

/// `GET /api/goal`: the latest goal message, or `null` when no goal was given yet.
pub async fn goal(
    State(shared): State<Shared>,
) -> Result<Json<Option<Server>>, (StatusCode, String)> {
    let gone = || {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "the session is gone".to_owned(),
        )
    };
    let (reply, answer) = oneshot::channel();
    shared
        .inbox
        .send(Cmd::CurrentGoal { reply })
        .await
        .map_err(|_| gone())?;
    answer.await.map(Json).map_err(|_| gone())
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
    let mut kind = Settle::of(&act.message);
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

    let wait = act
        .wait_ms
        .map_or_else(|| kind.wait(), Duration::from_millis);
    let mut messages = Vec::new();
    let collected = tokio::time::timeout(wait, async {
        loop {
            match out.recv().await {
                Ok(message) => {
                    let mine = message.by().is_none_or(|by| by == me) || kind.takes_any(&message);
                    if !mine || matches!(message, Server::Presence(_)) {
                        continue;
                    }
                    let done = kind.settled_by(&message);
                    messages.push(message);
                    if done {
                        // Whatever the same step already sent goes too: with review off, the
                        // proposal and its automatic accept are sent back to back.
                        while let Ok(more) = out.try_recv() {
                            if more.by().is_none_or(|by| by == me)
                                && !matches!(more, Server::Presence(_))
                            {
                                messages.push(more);
                            }
                        }
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
#[derive(Clone)]
enum Settle {
    /// An instruction: a proposal, or why there is none.
    Proposal,
    /// An accept: the change it made.
    Change,
    /// Anything that answers with a snapshot.
    Document,
    /// A rows request.
    Rows,
    /// A goal: its end. `started` once a goal message came.
    Goal { started: bool },
    /// A stop: the end of the goal, whoever owns it.
    GoalEnd,
}

impl Settle {
    fn of(message: &Client) -> Self {
        match message {
            Client::Say(_) | Client::Mic(_) => Settle::Proposal,
            Client::Accept(_) => Settle::Change,
            Client::Rows(_) => Settle::Rows,
            Client::Select(_)
            | Client::Settings(_)
            | Client::Reject(_)
            | Client::Undo(_)
            | Client::Resync(_)
            | Client::Hello(_) => Settle::Document,
            Client::Goal(_) => Settle::Goal { started: false },
            Client::StopGoal(_) => Settle::GoalEnd,
        }
    }

    /// How long to wait when the request names no wait: a goal runs a plan and one proposal per
    /// step, each of which may wait on a person.
    fn wait(&self) -> Duration {
        match self {
            Settle::Goal { .. } => Duration::from_secs(15 * 60),
            _ => Duration::from_secs(90),
        }
    }

    /// Whether a message another operator caused belongs to this action: a stop is answered by
    /// the goal ending, and the goal may be somebody else's.
    fn takes_any(&self, message: &Server) -> bool {
        matches!(self, Settle::GoalEnd) && matches!(message, Server::Goal(_))
    }

    fn settled_by(&mut self, message: &Server) -> bool {
        if let Settle::Goal { started } = self {
            // Once the goal runs, a refused or failed step is part of it, not its end.
            return match message {
                Server::Goal(g) => {
                    *started = true;
                    wire::goal_ended(g)
                }
                Server::Refused(_) | Server::Failed(_) => !*started,
                _ => false,
            };
        }
        if matches!(message, Server::Refused(_) | Server::Failed(_)) {
            return true;
        }
        match self {
            Settle::Proposal => matches!(message, Server::Proposal(_)),
            Settle::Change => matches!(message, Server::Changed(_) | Server::Document(_)),
            Settle::Document => matches!(message, Server::Document(_)),
            Settle::Rows => matches!(message, Server::Rows(_)),
            Settle::GoalEnd => matches!(message, Server::Goal(g) if wire::goal_ended(g)),
            Settle::Goal { .. } => unreachable!("answered above"),
        }
    }
}

#[derive(Deserialize)]
pub struct Export {
    /// Present to download the file rather than show it.
    #[serde(default)]
    pub download: Option<String>,
}

async fn render(shared: &Shared, docs: bool) -> Result<(String, String), (StatusCode, String)> {
    let (reply, answer) = oneshot::channel();
    let gone = || {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "the session is gone".to_owned(),
        )
    };
    shared
        .inbox
        .send(Cmd::Render { docs, reply })
        .await
        .map_err(|_| gone())?;
    answer.await.map_err(|_| gone())
}

/// `GET /api/document.yaml`: the document as it stands; `?download` saves it as `<app>.ui.yaml`.
pub async fn document_yaml(
    State(shared): State<Shared>,
    axum::extract::Query(export): axum::extract::Query<Export>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    use axum::response::IntoResponse;
    let (app, yaml) = render(&shared, false).await?;
    let mut response = (
        [(axum::http::header::CONTENT_TYPE, "text/yaml; charset=utf-8")],
        yaml,
    )
        .into_response();
    if export.download.is_some() {
        let value = format!("attachment; filename=\"{app}.ui.yaml\"");
        if let Ok(value) = axum::http::HeaderValue::from_str(&value) {
            response
                .headers_mut()
                .insert(axum::http::header::CONTENT_DISPOSITION, value);
        }
    }
    Ok(response)
}

/// `GET /api/docs.md`: documentation generated from the document.
pub async fn docs_md(
    State(shared): State<Shared>,
) -> Result<([(axum::http::HeaderName, &'static str); 1], String), (StatusCode, String)> {
    let (_, docs) = render(&shared, true).await?;
    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            "text/markdown; charset=utf-8",
        )],
        docs,
    ))
}

/// `GET /api/help.md`: what uilab and its agent can do.
pub async fn help_md() -> ([(axum::http::HeaderName, &'static str); 1], String) {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/markdown; charset=utf-8",
        )],
        uilab_doc::help_markdown(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goal::Goal;

    fn goal_at(by: &str, f: impl FnOnce(&mut Goal)) -> Server {
        let mut g = Goal::new("goal-1", by, "t", "page:members".parse().unwrap(), 8, false);
        f(&mut g);
        wire::goal(&g)
    }

    fn step() -> uilab_agent::Step {
        uilab_agent::Step {
            instruction: "add a list".into(),
            target: "page:members".parse().unwrap(),
            why: "w".into(),
        }
    }

    fn start() -> Client {
        Client::from_text(r#"{"type":"goal","value":{"text":"t"}}"#).unwrap()
    }

    fn stop() -> Client {
        Client::from_text(r#"{"type":"stop_goal","value":{"goal_id":"goal-1"}}"#).unwrap()
    }

    #[test]
    fn a_goal_settles_when_it_ends_not_when_a_step_is_refused() {
        let mut settle = Settle::of(&start());
        assert!(!settle.settled_by(&goal_at("api-1", |_| {})), "planning");
        assert!(!settle.settled_by(&goal_at("api-1", |g| {
            g.planned(vec![step(), step()]);
        })));
        assert!(
            !settle.settled_by(&Server::refused("name_unique", "m", Some("api-1"))),
            "a refused step does not end the goal"
        );
        assert!(!settle.settled_by(&Server::failed("agent", Some("api-1"))));
        for end in [
            goal_at("api-1", |g| {
                g.planned(vec![]);
            }),
            goal_at("api-1", |g| {
                g.stop();
            }),
            goal_at("api-1", |g| {
                g.plan_failed("no");
            }),
        ] {
            assert!(settle.clone().settled_by(&end));
        }
    }

    #[test]
    fn a_goal_refused_before_it_starts_settles_at_once() {
        let mut settle = Settle::of(&start());
        assert!(settle.settled_by(&Server::refused("goal_running", "m", Some("api-1"))));
        let mut settle = Settle::of(&start());
        assert!(settle.settled_by(&Server::failed("still working", Some("api-1"))));
    }

    #[test]
    fn stopping_settles_on_the_goal_ending_whoever_owns_it() {
        let mut settle = Settle::of(&stop());
        let owned_elsewhere = goal_at("ws-3", |g| {
            g.stop();
        });
        assert!(settle.takes_any(&owned_elsewhere));
        assert!(!settle.takes_any(&Server::refused("c", "m", Some("ws-3"))));
        assert!(!settle.settled_by(&goal_at("ws-3", |_| {})));
        assert!(settle.settled_by(&owned_elsewhere));
        assert!(Settle::of(&stop()).settled_by(&Server::refused("wrong_state", "m", None)));
    }

    #[test]
    fn only_stopping_takes_other_operators_goal_messages() {
        let message = goal_at("ws-3", |g| {
            g.stop();
        });
        assert!(!Settle::of(&start()).takes_any(&message));
        assert!(
            !Settle::of(&Client::from_text(r#"{"type":"say","value":{"text":"t"}}"#).unwrap())
                .takes_any(&message)
        );
    }

    #[test]
    fn a_goal_waits_fifteen_minutes_by_default() {
        assert_eq!(Settle::of(&start()).wait(), Duration::from_secs(15 * 60));
        assert_eq!(Settle::of(&stop()).wait(), Duration::from_secs(90));
        let say = Client::from_text(r#"{"type":"say","value":{"text":"t"}}"#).unwrap();
        assert_eq!(Settle::of(&say).wait(), Duration::from_secs(90));
    }

    #[test]
    fn a_goal_does_not_settle_on_the_operators_earlier_goal() {
        // A browser that connects gets the latest goal re-broadcast to every subscriber
        // (app.rs, Cmd::Connected), attributed to its operator. When that is this operator's
        // previous goal, ended, and it reaches `act` before the new goal's planning message,
        // the new `op goal` must not settle on it.
        let earlier = goal_at("api-1", |g| {
            g.planned(vec![step()]);
            g.not_proposed(0, false);
        });
        assert!(
            matches!(&earlier, Server::Goal(g) if wire::goal_ended(g)),
            "precondition: the earlier goal is over"
        );
        let mut settle = Settle::of(&start());
        assert!(
            !settle.settled_by(&earlier),
            "settled on the previous goal before the new one was planned"
        );
    }
}
