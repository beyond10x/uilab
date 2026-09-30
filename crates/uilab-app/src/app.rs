//! The one session the server holds: the open document, its revision, the operators working on it,
//! the microphone and the proposal in flight.
//!
//! One task owns all of it and handles one message at a time. Browser connections and the operator
//! API send it what an operator did; speech and the agent run on blocking threads and send their
//! results back. Everything is broadcast to every connection, attributed to the operator it came
//! from.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use indexmap::IndexMap;
use serde_json::json;
use tokio::sync::{broadcast, mpsc, oneshot};
use uilab_behaviour::{FileSource, Handle, body_from_json};
use uilab_doc::{
    Document, Finding, Fixtures, Layer, NodePath, Patch, admit, check, field_findings, outline,
    outline_at, vocabulary, yaml_at,
};
use uilab_session::UilabSession;
use uilab_types::primitives::Uuid;
use uilab_types::session as s;

use crate::goal::{self, Goal, Next};
use crate::journal::Journal;
use crate::wire::{self, ChangedParts, Client, DocumentParts, OperatorParts, Server};

/// An operator the API registered and has not heard from for this long is gone.
pub const API_OPERATOR_TTL: Duration = Duration::from_secs(60);

/// What the session task is told.
pub enum Cmd {
    /// A browser connected; it is an anonymous human until it says hello.
    Connected {
        operator: String,
        /// This connection alone: the snapshot it starts from goes here, not to everybody.
        direct: mpsc::UnboundedSender<Server>,
    },
    /// A browser went away.
    Disconnected { operator: String },
    /// An operator did something.
    Client { by: String, message: Client },
    /// A frame of 16 kHz mono samples from an operator's microphone.
    Audio { by: String, samples: Vec<f32> },
    /// Speech came back.
    Heard {
        by: String,
        result: Result<uilab_stt::Transcript, String>,
        wav: Option<String>,
    },
    /// The agent came back.
    Proposed {
        by: String,
        target: NodePath,
        utterance: String,
        result: Box<Result<uilab_agent::Proposal, (String, String)>>,
        ms: u64,
        review: bool,
        /// The goal and step index the proposal carries out, when a goal asked for it.
        step: Option<(String, usize)>,
    },
    /// The agent came back with a goal's plan.
    Planned {
        goal_id: String,
        result: Result<uilab_agent::Plan, String>,
        ms: u64,
    },
    /// The latest goal message, if any goal was given.
    CurrentGoal {
        reply: oneshot::Sender<Option<Server>>,
    },
    /// The operator API registers an operator.
    Register {
        name: String,
        agent: bool,
        reply: oneshot::Sender<String>,
    },
    /// The operator API checks an operator exists, and keeps it present.
    Touch {
        operator: String,
        reply: oneshot::Sender<bool>,
    },
    /// The current document message.
    Snapshot { reply: oneshot::Sender<Server> },
    /// The document as YAML or as generated documentation, with the app name.
    Render {
        docs: bool,
        reply: oneshot::Sender<(String, String)>,
    },
    /// Expire operators the API stopped calling.
    Tick,
}

pub struct Config {
    pub doc: PathBuf,
    pub stt: Option<uilab_stt::TranscriberConfig>,
    pub proposer: uilab_agent::ProposerConfig,
    pub journal: PathBuf,
    /// Hold each proposal for accept or reject; otherwise apply it at once.
    pub review: bool,
}

struct Operator {
    name: String,
    agent: bool,
    last_seen: Instant,
    /// Registered through the API rather than a connection, so it expires.
    api: bool,
}

pub struct App {
    port: UilabSession<Handle<FileSource>>,
    handle: Handle<FileSource>,
    document_id: s::DocumentId,
    file: String,
    fixtures: Fixtures,
    fields: Vec<(String, Vec<String>)>,
    journal: Arc<Journal>,
    stt: Option<Arc<Mutex<uilab_stt::Transcriber>>>,
    /// The agent; `None` in a session opened without one, which asks nothing.
    proposer: Option<Arc<Mutex<uilab_agent::Proposer>>>,
    /// The proposal message of the proposal waiting in `pending`, for a browser that connects.
    shown: Option<Server>,
    operators: IndexMap<String, Operator>,
    selected_by: Option<String>,
    revision: u64,
    review: bool,
    next_api_operator: u64,
    pending: Option<String>,
    /// The document a proposal would make, by its id; what sample rows are built from while that
    /// proposal is the one in `pending`.
    preview: Option<(String, Document)>,
    audio: Option<(String, Vec<f32>)>,
    /// Where the microphone was opened; the transcription is proposed there.
    listening_in: uilab_agent::Workspace,
    busy: bool,
    /// The latest goal; kept after it ends so a browser that connects later still sees it.
    goal: Option<Goal>,
    next_goal: u64,
    out: broadcast::Sender<Server>,
    back: mpsc::Sender<Cmd>,
}

impl App {
    /// Opens the document and loads the models; fails before serving anything if one cannot load.
    pub fn start(
        config: Config,
        out: broadcast::Sender<Server>,
        back: mpsc::Sender<Cmd>,
    ) -> Result<Self, String> {
        let mut app = Self::open(&config.doc, &config.journal, config.review, out, back)?;
        println!("uilab: journal in {}", app.journal.dir().display());
        app.journal.write(
            "start",
            json!({
                "doc": app.file,
                "model": config.proposer.model,
                "base_url": config.proposer.base_url,
                "stt": config.stt.as_ref().map(|c| c.model.display().to_string()),
            }),
        );
        app.stt = match config.stt {
            Some(c) => {
                let mut stt =
                    uilab_stt::Transcriber::load(c).map_err(|e| format!("speech model: {e}"))?;
                // The first transcription in a process compiles the GPU shaders (29.7 s measured on
                // 2026-09-30); pay it here, before anybody speaks. Quiet noise, loud enough to pass
                // the silence gate; whatever whisper hears in it is thrown away.
                let started = Instant::now();
                let noise: Vec<f32> = (0..16_000)
                    .map(|i| ((i * 7919 % 97) as f32 - 48.0) * 0.004)
                    .collect();
                let _ = stt.transcribe(&noise, "");
                println!(
                    "uilab: speech warmed up in {} ms",
                    started.elapsed().as_millis()
                );
                Some(Arc::new(Mutex::new(stt)))
            }
            None => None,
        };
        let proposer =
            uilab_agent::Proposer::new(config.proposer).map_err(|e| format!("agent: {e}"))?;
        app.proposer = Some(Arc::new(Mutex::new(proposer)));
        Ok(app)
    }

    /// The session over one document, with neither speech nor an agent: nothing it is asked is
    /// sent anywhere, and the answers come as the [`Cmd::Planned`] and [`Cmd::Proposed`] the
    /// caller hands it. [`start`](Self::start) adds both.
    fn open(
        doc_path: &Path,
        journal: &Path,
        review: bool,
        out: broadcast::Sender<Server>,
        back: mpsc::Sender<Cmd>,
    ) -> Result<Self, String> {
        let file = doc_path.to_string_lossy().into_owned();
        let handle = Handle(Arc::new(Mutex::new(uilab_behaviour::Behaviour::new(
            FileSource,
        ))));
        let mut port = UilabSession::new(handle.clone());
        let document_id = match port
            .open_document(s::OpenDocument { path: file.clone() })
            .map_err(|e| e.to_string())?
        {
            s::OpenDocumentOutcome::Opened { document_opened } => document_opened.document_id,
            s::OpenDocumentOutcome::Unreadable { .. } => {
                return Err(format!("{file} is not a ui-spec/1 document"));
            }
        };
        let doc = handle
            .document(&document_id)
            .ok_or("the opened document is not held")?;
        let dir = doc_path.parent().unwrap_or(Path::new("."));
        let fixtures = Fixtures::load(&doc, dir).map_err(|e| e.to_string())?;
        let fields = fixtures.fields().into_iter().collect();
        let journal =
            Journal::open(journal).map_err(|e| format!("journal {}: {e}", journal.display()))?;
        Ok(App {
            port,
            handle,
            document_id,
            file,
            fixtures,
            fields,
            journal: Arc::new(journal),
            stt: None,
            proposer: None,
            shown: None,
            operators: IndexMap::new(),
            selected_by: None,
            revision: 0,
            review,
            next_api_operator: 0,
            pending: None,
            preview: None,
            audio: None,
            listening_in: uilab_agent::Workspace::App,
            busy: false,
            goal: None,
            next_goal: 0,
            out,
            back,
        })
    }

    pub async fn run(mut self, mut inbox: mpsc::Receiver<Cmd>) {
        while let Some(cmd) = inbox.recv().await {
            self.handle_cmd(cmd);
        }
    }

    fn send(&self, message: Server) {
        match &message {
            Server::Refused(r) => self.journal.write(
                "refused",
                json!({"check": r.check, "message": r.message, "by": message.by()}),
            ),
            Server::Failed(f) => self
                .journal
                .write("failed", json!({"message": f.message, "by": message.by()})),
            _ => {}
        }
        let _ = self.out.send(message);
    }

    /// The document checks plus the fixture-field warnings.
    fn findings(&self, doc: &Document) -> Vec<Finding> {
        let mut findings = check(doc);
        findings.extend(field_findings(doc, &self.fixtures));
        findings
    }

    fn doc(&self) -> Document {
        self.handle
            .document(&self.document_id)
            .expect("the session's document is held")
    }

    fn selected(&self) -> NodePath {
        self.handle.selected(&self.document_id).unwrap_or_default()
    }

    fn document_message(&self) -> Server {
        let doc = self.doc();
        let tree = outline(&doc);
        let findings = self.findings(&doc);
        let selected = self.selected().to_string();
        let undoable = self
            .handle
            .last_undoable(&self.document_id)
            .map(|id| id.0.0);
        wire::document(DocumentParts {
            document_id: &self.document_id.0.0,
            file: &self.file,
            title: doc.title.clone(),
            selected: &selected,
            outline: &tree,
            findings: &findings,
            undoable,
            revision: self.revision,
            review: self.review,
        })
    }

    fn send_document(&self) {
        self.send(self.document_message());
    }

    fn send_presence(&self) {
        let now = Instant::now();
        let operators = self
            .operators
            .iter()
            .map(|(id, o)| OperatorParts {
                id: id.clone(),
                name: o.name.clone(),
                agent: o.agent,
                last_seen_ms: now.duration_since(o.last_seen).as_millis() as u64,
            })
            .collect();
        self.send(wire::presence(operators, self.selected_by.clone()));
    }

    fn seen(&mut self, by: &str) {
        if let Some(operator) = self.operators.get_mut(by) {
            operator.last_seen = Instant::now();
        }
    }

    fn name_of(&self, by: &str) -> String {
        self.operators
            .get(by)
            .map_or_else(|| by.to_owned(), |o| o.name.clone())
    }

    fn handle_cmd(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::Connected { operator, direct } => {
                self.operators.insert(
                    operator,
                    Operator {
                        name: "anonymous".into(),
                        agent: false,
                        last_seen: Instant::now(),
                        api: false,
                    },
                );
                // The snapshot goes to the new connection alone; everybody else already has it,
                // and a re-send would read as a new event there. Presence did change for all.
                let _ = direct.send(self.document_message());
                if let Some(goal) = &self.goal {
                    let _ = direct.send(wire::goal(goal));
                }
                // The card of a proposal still waiting, which a goal step may be blocked on.
                if let Some(Server::Proposal(p)) = &self.shown
                    && self.pending.as_deref() == Some(p.proposal_id.0.as_str())
                {
                    let _ = direct.send(Server::Proposal(p.clone()));
                }
                self.send_presence();
            }
            Cmd::Disconnected { operator } => {
                if let Some((owner, _)) = &self.audio
                    && owner == &operator
                {
                    self.audio = None;
                }
                self.operators.shift_remove(&operator);
                self.send_presence();
            }
            Cmd::Register { name, agent, reply } => {
                self.next_api_operator += 1;
                let id = format!("api-{}", self.next_api_operator);
                self.journal
                    .write("joined", json!({"by": id, "name": name, "agent": agent}));
                self.operators.insert(
                    id.clone(),
                    Operator {
                        name,
                        agent,
                        last_seen: Instant::now(),
                        api: true,
                    },
                );
                let _ = reply.send(id);
                self.send_presence();
            }
            Cmd::Touch { operator, reply } => {
                let known = self.operators.contains_key(&operator);
                self.seen(&operator);
                let _ = reply.send(known);
            }
            Cmd::Snapshot { reply } => {
                let _ = reply.send(self.document_message());
            }
            Cmd::Render { docs, reply } => {
                let doc = self.doc();
                let text = if docs {
                    uilab_doc::docs_markdown(&doc, &self.fixtures, &self.findings(&doc))
                } else {
                    doc.to_yaml().unwrap_or_default()
                };
                let _ = reply.send((doc.app.clone(), text));
            }
            Cmd::CurrentGoal { reply } => {
                let _ = reply.send(self.goal.as_ref().map(wire::goal));
            }
            Cmd::Tick => {
                let before = self.operators.len();
                // An API operator waiting on its goal makes no calls while the goal runs.
                let running = self
                    .goal
                    .as_ref()
                    .filter(|g| g.is_active())
                    .map(|g| g.by.clone());
                self.operators.retain(|id, o| {
                    !o.api
                        || o.last_seen.elapsed() < API_OPERATOR_TTL
                        || running.as_deref() == Some(id.as_str())
                });
                if self.operators.len() != before {
                    self.send_presence();
                }
            }
            Cmd::Client { by, message } => {
                self.seen(&by);
                self.client(&by, message);
            }
            Cmd::Audio { by, samples } => {
                if let Some((owner, buffer)) = &mut self.audio
                    && owner == &by
                {
                    buffer.extend(samples);
                }
            }
            Cmd::Heard {
                by,
                result: Ok(t),
                wav,
            } => {
                self.journal.write(
                    "heard",
                    json!({"by": by, "text": t.text, "audio_ms": t.audio_ms, "took_ms": t.took_ms, "wav": wav}),
                );
                self.send(wire::transcript(&t.text, t.audio_ms, t.took_ms, Some(&by)));
                self.busy = false;
                if t.text.trim().is_empty() {
                    self.send(Server::refused("speech", "nothing was heard", Some(&by)));
                } else {
                    self.propose(&by, t.text, None, None, self.listening_in);
                }
            }
            Cmd::Heard {
                by,
                result: Err(e),
                wav,
            } => {
                self.journal
                    .write("heard", json!({"by": by, "error": e, "wav": wav}));
                self.busy = false;
                self.send(Server::failed(format!("speech: {e}"), Some(&by)));
            }
            Cmd::Proposed {
                by,
                target,
                utterance,
                result,
                ms,
                review,
                step,
            } => {
                self.busy = false;
                let goal_id = step.as_ref().map(|(id, _)| id.clone());
                let index = step.as_ref().map(|(_, i)| *i);
                if let Some((id, index)) = &step
                    && !self
                        .goal
                        .as_ref()
                        .is_some_and(|g| &g.id == id && g.awaits(*index))
                {
                    self.journal.write(
                        "discarded",
                        json!({"by": by, "goal_id": id, "step": index, "ms": ms}),
                    );
                    return;
                }
                match *result {
                    Ok(proposal) => {
                        self.journal.write(
                            "proposed",
                            json!({
                                "by": by,
                                "utterance": utterance,
                                "target": target.to_string(),
                                "ms": ms,
                                "attempts": proposal.attempts,
                                "turns": proposal.turns,
                                "patch": proposal.patch,
                                "goal_id": goal_id,
                                "step": index,
                            }),
                        );
                        let recorded = self.record(&by, utterance, proposal.patch);
                        match (recorded, index) {
                            (Ok(id), Some(index)) => {
                                self.goal_next(|g| g.proposed(index, id.clone()));
                                if !review {
                                    self.accept(&by, &id, true);
                                }
                            }
                            (Ok(id), None) => {
                                if !review {
                                    self.accept(&by, &id, true);
                                }
                            }
                            (Err(_), Some(index)) => {
                                self.goal_next(|g| g.not_proposed(index, false));
                            }
                            (Err(_), None) => {}
                        }
                    }
                    Err((check, message)) => {
                        self.journal.write(
                            "not_proposed",
                            json!({
                                "by": by,
                                "utterance": utterance,
                                "target": target.to_string(),
                                "ms": ms,
                                "check": check,
                                "message": message,
                                "goal_id": goal_id,
                                "step": index,
                            }),
                        );
                        let declined = check == "declined";
                        self.send(Server::refused(check, message, Some(&by)));
                        if let Some(index) = index {
                            self.goal_next(|g| g.not_proposed(index, declined));
                        }
                    }
                }
            }
            Cmd::Planned {
                goal_id,
                result,
                ms,
            } => {
                match &result {
                    Ok(plan) => self.journal.write(
                        "planned",
                        json!({"goal_id": goal_id, "ms": ms, "turns": plan.turns, "cost_micro_usd": plan.cost_micro_usd, "steps": plan.steps}),
                    ),
                    Err(e) => self.journal.write(
                        "not_planned",
                        json!({"goal_id": goal_id, "ms": ms, "message": e}),
                    ),
                }
                if !self.goal.as_ref().is_some_and(|g| g.id == goal_id) {
                    return;
                }
                match result {
                    Ok(plan) => self.goal_next(|g| g.planned(plan.steps)),
                    Err(message) => self.goal_next(|g| g.plan_failed(message)),
                }
            }
        }
    }

    /// Applies one transition to the goal and does what follows: broadcast the goal and journal
    /// it, then propose the next step, if there is one.
    fn goal_next(&mut self, transition: impl FnOnce(&mut Goal) -> Next) {
        let Some(goal) = self.goal.as_mut() else {
            return;
        };
        let next = transition(goal);
        if next == Next::Ignored {
            return;
        }
        self.send_goal();
        if let Next::Propose {
            index,
            instruction,
            target,
        } = next
        {
            self.run_step(index, instruction, target);
        }
    }

    /// Broadcasts the goal whole and journals it: every transition goes through here.
    fn send_goal(&self) {
        let Some(goal) = &self.goal else { return };
        let message = wire::goal(goal);
        self.journal.write(
            "goal",
            serde_json::to_value(&message)
                .map(|v| v["value"].clone())
                .unwrap_or_default(),
        );
        self.send(message);
    }

    /// Proposes one step of the running goal, as the goal's operator, at the step's target. A
    /// target that does not resolve on the document the earlier steps left (an earlier step that
    /// would have created it was refused or rejected) is refused here, without a model call.
    fn run_step(&mut self, index: usize, instruction: String, target: NodePath) {
        let Some(goal) = &self.goal else { return };
        let (id, by, review, workspace) = (
            goal.id.clone(),
            goal.by.clone(),
            goal.review,
            goal.workspace,
        );
        if uilab_doc::resolve(&self.doc(), &target).is_err() {
            self.send(Server::refused(
                "path_resolves",
                format!("step {}: no node at `{target}`", index + 1),
                Some(&by),
            ));
            self.goal_next(|g| g.not_proposed(index, false));
            return;
        }
        let step = Some((id, index));
        if !self.start_propose(&by, instruction, Some(target), review, step, workspace) {
            self.goal_next(|g| g.not_proposed(index, false));
        }
    }

    fn start_goal(&mut self, by: &str, start: uilab_wire::UilabWireStartGoal) {
        if self.goal.as_ref().is_some_and(Goal::is_active) {
            self.send(Server::refused(
                "goal_running",
                "a goal is running: stop it or wait for it to end",
                Some(by),
            ));
            return;
        }
        // A plan must not land inside a transcription or an instruction still being worked on.
        let busy = if self.busy {
            Some("still working on the last instruction")
        } else if self.audio.is_some() {
            Some("an operator holds the microphone")
        } else {
            None
        };
        if let Some(message) = busy {
            self.send(Server::refused("goal_busy", message, Some(by)));
            return;
        }
        let target = match &start.target {
            uilab_wire::EssPresence::Present(path) => match path.0.parse::<NodePath>() {
                Ok(path) => path,
                Err(e) => {
                    self.send(Server::refused("path_resolves", e.to_string(), Some(by)));
                    return;
                }
            },
            uilab_wire::EssPresence::Absent => self.selected(),
        };
        let workspace = wire::workspace(&start.workspace);
        let target = placed(target, workspace);
        let max_steps = match &start.max_steps {
            uilab_wire::EssPresence::Present(n) => {
                match n.as_u64().and_then(|n| usize::try_from(n).ok()) {
                    Some(n) if n > 0 => n,
                    _ => {
                        self.send(Server::refused(
                            "max_steps",
                            format!("max_steps is {n}; give a whole number of at least 1"),
                            Some(by),
                        ));
                        return;
                    }
                }
            }
            uilab_wire::EssPresence::Absent => goal::DEFAULT_MAX_STEPS,
        };
        let review = match start.review {
            uilab_wire::EssPresence::Present(review) => review,
            uilab_wire::EssPresence::Absent => self.review,
        };
        if let Some(pending) = self.pending.take() {
            let _ = self.port.reject_proposal(s::RejectProposal {
                proposal_id: proposal(&pending),
            });
            // As a reject does: every browser's card of it closes.
            self.send_document();
        }
        self.next_goal += 1;
        let goal_id = format!("goal-{}", self.next_goal);
        let mut goal = Goal::new(
            goal_id.clone(),
            by,
            start.text.clone(),
            target.clone(),
            max_steps,
            review,
        );
        goal.workspace = workspace;
        self.goal = Some(goal);
        self.send_goal();

        let doc = self.doc();
        let Some(proposer) = self.proposer.clone() else {
            return;
        };
        let fields = self.fields.clone();
        let back = self.back.clone();
        let text = start.text;
        tokio::task::spawn_blocking(move || {
            let started = Instant::now();
            let result = proposer
                .lock()
                .expect("the proposer lock")
                .plan_goal_in(&doc, &target, &text, max_steps, &fields, workspace)
                .map_err(|e| e.to_string());
            let ms = started.elapsed().as_millis() as u64;
            let _ = back.blocking_send(Cmd::Planned {
                goal_id,
                result,
                ms,
            });
        });
    }

    fn stop_goal(&mut self, by: &str, goal_id: &str) {
        let Some(goal) = self
            .goal
            .as_mut()
            .filter(|g| g.id == goal_id && g.is_active())
        else {
            self.send(Server::refused(
                "wrong_state",
                format!("no goal `{goal_id}` is running"),
                Some(by),
            ));
            return;
        };
        let thinking = goal.current.filter(|&i| goal.awaits(i));
        let owner = goal.by.clone();
        let (next, waiting) = goal.stop();
        self.journal
            .write("goal_stopped", json!({"by": by, "goal_id": goal_id}));
        if next == Next::Ignored {
            return;
        }
        self.send_goal();
        if let Some(index) = thinking {
            // The step's answer will be thrown away when it comes; end the step for its
            // operator now, so its browser stops showing it at work.
            self.send(Server::refused(
                "goal_stopped",
                format!(
                    "the goal was stopped before step {} was proposed",
                    index + 1
                ),
                Some(&owner),
            ));
        }
        if let Some(waiting) = waiting {
            let _ = self.port.reject_proposal(s::RejectProposal {
                proposal_id: proposal(&waiting),
            });
            self.settle_pending(&waiting);
            self.send_document();
        }
    }

    fn client(&mut self, by: &str, message: Client) {
        if !matches!(message, Client::Rows(_)) {
            self.journal
                .write("operator", json!({"by": by, "message": message}));
        }
        match message {
            Client::Hello(hello) => {
                let agent = wire::is_agent(&hello.kind);
                if let Some(operator) = self.operators.get_mut(by) {
                    operator.name = hello.name;
                    operator.agent = agent;
                }
                self.send_presence();
            }
            Client::Resync(_) => self.send_document(),
            Client::Select(select) => {
                let outcome = self.port.select_node(s::SelectNode {
                    document_id: self.document_id.clone(),
                    path: s::NodePath(select.path.0.clone()),
                });
                match outcome {
                    Ok(s::SelectNodeOutcome::Selected { .. }) => {
                        self.selected_by = Some(by.to_owned());
                        self.send_document();
                        self.send_presence();
                    }
                    Ok(_) => self.send(Server::refused(
                        "path_resolves",
                        format!("no node at `{}`", select.path.0),
                        Some(by),
                    )),
                    Err(e) => self.send(Server::failed(e.to_string(), Some(by))),
                }
            }
            Client::Mic(mic) if wire::mic_open(&mic) => {
                if self.goal.as_ref().is_some_and(Goal::is_active) {
                    // Failed rather than refused: the browser lets go of the microphone on it.
                    self.send(Server::failed(
                        "a goal is running: stop it or wait for it to end",
                        Some(by),
                    ));
                } else if self.stt.is_none() {
                    self.send(Server::failed(
                        "speech is off: start uilab with a speech model, or type the instruction",
                        Some(by),
                    ));
                } else {
                    self.audio = Some((by.to_owned(), Vec::new()));
                    self.listening_in = wire::workspace(&mic.workspace);
                }
            }
            Client::Mic(_) => {
                if let Some((owner, samples)) = self.audio.take() {
                    if owner == by {
                        self.transcribe(by, samples);
                    } else {
                        self.audio = Some((owner, samples));
                    }
                }
            }
            Client::Say(say) => {
                let target = match &say.target {
                    uilab_wire::EssPresence::Present(path) => match path.0.parse::<NodePath>() {
                        Ok(path) => Some(path),
                        Err(e) => {
                            self.send(Server::refused("path_resolves", e.to_string(), Some(by)));
                            return;
                        }
                    },
                    uilab_wire::EssPresence::Absent => None,
                };
                let review = match say.review {
                    uilab_wire::EssPresence::Present(review) => Some(review),
                    uilab_wire::EssPresence::Absent => None,
                };
                let workspace = wire::workspace(&say.workspace);
                self.propose(by, say.text, target, review, workspace)
            }
            Client::Accept(d) => self.accept(by, &d.proposal_id.0, false),
            Client::Settings(settings) => {
                self.review = settings.review;
                self.journal
                    .write("settings", json!({"by": by, "review": self.review}));
                self.send_document();
            }
            Client::Reject(d) => {
                let _ = self.port.reject_proposal(s::RejectProposal {
                    proposal_id: proposal(&d.proposal_id.0),
                });
                self.settle_pending(&d.proposal_id.0);
                self.send_document();
                self.goal_next(|g| g.decided(&d.proposal_id.0, false));
            }
            Client::Undo(d) => {
                match self.port.undo_proposal(s::UndoProposal {
                    proposal_id: proposal(&d.proposal_id.0),
                }) {
                    Ok(s::UndoProposalOutcome::Undone { .. }) => {
                        self.revision += 1;
                        self.journal.write(
                            "undone",
                            json!({"by": by, "proposal_id": d.proposal_id.0, "revision": self.revision}),
                        );
                        self.send_document()
                    }
                    Ok(s::UndoProposalOutcome::Stale { error }) => self.send(Server::refused(
                        error.check,
                        "a later change would be lost",
                        Some(by),
                    )),
                    Ok(_) => self.send(Server::refused(
                        "wrong_state",
                        "that proposal was not applied",
                        Some(by),
                    )),
                    Err(e) => self.send(Server::failed(e.to_string(), Some(by))),
                }
            }
            Client::Rows(read) => {
                let rows = if self.fixtures.has(&read.view) {
                    self.fixtures.rows(&read.view)
                } else {
                    // A waiting proposal is what the canvas shows, so its composites shape the rows.
                    let rows = match (&self.preview, &self.pending) {
                        (Some((id, after)), Some(pending)) if id == pending => {
                            uilab_doc::sample_rows(after, &read.view)
                        }
                        _ => uilab_doc::sample_rows(&self.doc(), &read.view),
                    };
                    uilab_doc::ViewRows { total: None, rows }
                };
                self.send(wire::rows(&read.view, rows.total, rows.rows));
            }
            Client::Goal(start) => self.start_goal(by, start),
            Client::StopGoal(stop) => self.stop_goal(by, &stop.goal_id),
        }
    }

    /// Broadcasts an accepted proposal as a delta. A page or a menu section changes the menu as
    /// well, which a delta at one path cannot carry, and a change that adds or drops a widget
    /// instance changes the use sites on component nodes elsewhere in the tree: those go out as a
    /// full snapshot.
    fn announce_change(&self, by: &str, id: &s::ProposalId, before: &Document) {
        let Some(patch) = self.handle.patch(id) else {
            self.send_document();
            return;
        };
        let changed = patch.changed_path();
        let doc = self.doc();
        if matches!(patch, Patch::Batch { .. })
            || matches!(
                changed.layer(),
                Layer::Page | Layer::NavSection | Layer::Shell
            )
            || component_uses(before) != component_uses(&doc)
        {
            self.send_document();
            return;
        }
        let node = match patch {
            Patch::Remove { .. } => None,
            _ => outline_at(&doc, &changed),
        };
        let parent = changed.parent().unwrap_or_default().to_string();
        let findings = self.findings(&doc);
        self.send(wire::changed(ChangedParts {
            revision: self.revision,
            by,
            op: patch.op_name(),
            changed: &changed.to_string(),
            parent: &parent,
            node: node.as_ref(),
            findings: &findings,
        }));
    }

    fn transcribe(&mut self, by: &str, samples: Vec<f32>) {
        let Some(stt) = self.stt.clone() else { return };
        if self.busy {
            self.send(Server::failed(
                "still working on the last instruction",
                Some(by),
            ));
            return;
        }
        self.busy = true;
        let prompt = vocabulary(&self.doc(), &self.selected()).join(", ");
        let wav = self.journal.audio(&samples);
        let back = self.back.clone();
        let by = by.to_owned();
        tokio::task::spawn_blocking(move || {
            let result = stt
                .lock()
                .expect("the transcriber lock")
                .transcribe(&samples, &prompt)
                .map_err(|e| e.to_string());
            let _ = back.blocking_send(Cmd::Heard { by, result, wav });
        });
    }

    /// A decision on `proposal_id` ends the wait only when it is the proposal waiting; a stale
    /// card or command deciding an earlier one leaves it.
    fn settle_pending(&mut self, proposal_id: &str) {
        if self.pending.as_deref() == Some(proposal_id) {
            self.pending = None;
        }
    }

    /// Applies a waiting proposal. `automatic` when review is off and nobody pressed accept.
    fn accept(&mut self, by: &str, proposal_id: &str, automatic: bool) {
        let id = proposal(proposal_id);
        let before = self.doc();
        let outcome = self.port.accept_proposal(s::AcceptProposal {
            proposal_id: id.clone(),
        });
        self.settle_pending(proposal_id);
        let applied = matches!(outcome, Ok(s::AcceptProposalOutcome::Accepted { .. }));
        match outcome {
            Ok(s::AcceptProposalOutcome::Accepted { .. }) => {
                self.revision += 1;
                self.journal.write(
                    "accepted",
                    json!({"by": by, "proposal_id": proposal_id, "revision": self.revision, "automatic": automatic}),
                );
                self.announce_change(by, &id, &before);
            }
            Ok(s::AcceptProposalOutcome::Stale { error }) => self.send(Server::refused(
                error.check,
                "the document changed since the proposal",
                Some(by),
            )),
            Ok(_) => self.send(Server::refused(
                "wrong_state",
                "that proposal is not waiting",
                Some(by),
            )),
            Err(e) => self.send(Server::failed(e.to_string(), Some(by))),
        }
        // A step whose proposal could not be applied counts as rejected; the run goes on.
        self.goal_next(|g| g.decided(proposal_id, applied));
    }

    /// An instruction from an operator: refused while a goal runs.
    fn propose(
        &mut self,
        by: &str,
        utterance: String,
        target: Option<NodePath>,
        review: Option<bool>,
        workspace: uilab_agent::Workspace,
    ) {
        if self.goal.as_ref().is_some_and(Goal::is_active) {
            self.send(Server::refused(
                "goal_running",
                "a goal is running: stop it or wait for it to end",
                Some(by),
            ));
            return;
        }
        let review = review.unwrap_or(self.review);
        let target = placed(target.unwrap_or_else(|| self.selected()), workspace);
        self.start_propose(by, utterance, Some(target), review, None, workspace);
    }

    /// Asks the agent for one patch off the runtime; the answer comes back as [`Cmd::Proposed`].
    /// `false` when the last instruction is still being worked on, and nothing was started.
    fn start_propose(
        &mut self,
        by: &str,
        utterance: String,
        target: Option<NodePath>,
        review: bool,
        step: Option<(String, usize)>,
        workspace: uilab_agent::Workspace,
    ) -> bool {
        if self.busy {
            self.send(Server::failed(
                "still working on the last instruction",
                Some(by),
            ));
            return false;
        }
        if let Some(pending) = self.pending.take() {
            let _ = self.port.reject_proposal(s::RejectProposal {
                proposal_id: proposal(&pending),
            });
        }
        self.busy = true;
        let target = target.unwrap_or_else(|| self.selected());
        self.send(wire::thinking(&target.to_string(), Some(by)));
        let doc = self.doc();
        let Some(proposer) = self.proposer.clone() else {
            return true;
        };
        let fields = self.fields.clone();
        let back = self.back.clone();
        let by = by.to_owned();
        tokio::task::spawn_blocking(move || {
            let started = Instant::now();
            let result = proposer
                .lock()
                .expect("the proposer lock")
                .propose_in(&doc, &target, &utterance, &fields, workspace)
                .map_err(|e| match e {
                    uilab_agent::ProposeError::Refused { check, message } => (check, message),
                    uilab_agent::ProposeError::Declined(reason) => ("declined".to_owned(), reason),
                    other => ("agent".to_owned(), other.to_string()),
                });
            let ms = started.elapsed().as_millis() as u64;
            let _ = back.blocking_send(Cmd::Proposed {
                by,
                target,
                utterance,
                result: Box::new(result),
                ms,
                review,
                step,
            });
        });
        true
    }

    /// Records the agent's patch as a proposal and shows it. The proposal id, or the check that
    /// kept it from being recorded (already sent to the operator).
    fn record(&mut self, by: &str, utterance: String, patch: Patch) -> Result<String, String> {
        let body = match &patch {
            Patch::Insert { child, .. } => Some(body_from_json(
                &serde_json::to_value(child).expect("a child serializes"),
            )),
            Patch::Replace { node, .. } => Some(body_from_json(node)),
            Patch::Remove { .. } => None,
            Patch::Batch { patches, .. } => {
                Some(body_from_json(&serde_json::json!({"patches": patches})))
            }
        };
        let op = match &patch {
            Patch::Insert { .. } => s::PatchOp::Insert,
            Patch::Replace { .. } => s::PatchOp::Replace,
            Patch::Remove { .. } => s::PatchOp::Remove,
            Patch::Batch { .. } => s::PatchOp::Batch,
        };
        let outcome = self.port.propose_patch(s::ProposePatch {
            document_id: self.document_id.clone(),
            target: s::NodePath(patch.target().to_string()),
            op,
            utterance: utterance.clone(),
            body,
        });
        let proposal_id = match outcome {
            Ok(s::ProposePatchOutcome::Proposed { patch_proposed }) => patch_proposed.proposal_id,
            Ok(s::ProposePatchOutcome::Refused { error }) => {
                let check = error.check.clone();
                self.send(Server::refused(
                    error.check,
                    "the proposed patch fails a document check",
                    Some(by),
                ));
                return Err(check);
            }
            Err(e) => {
                self.send(Server::failed(e.to_string(), Some(by)));
                return Err("failed".to_owned());
            }
        };
        let before = self.doc();
        let Ok((after, admitted)) = admit(&before, &patch) else {
            self.send(Server::failed(
                "the admitted patch no longer applies",
                Some(by),
            ));
            return Err("failed".to_owned());
        };
        // The card names what this proposal brings: a finding the document already had is in the
        // document's own findings, not repeated on every card. Counted, so a second equal one is new.
        let mut had: HashMap<Finding, usize> = HashMap::new();
        for f in self.findings(&before) {
            *had.entry(f).or_default() += 1;
        }
        let mut findings: Vec<Finding> = self
            .findings(&after)
            .into_iter()
            .filter(|f| match had.get_mut(f) {
                Some(n) if *n > 0 => {
                    *n -= 1;
                    false
                }
                _ => true,
            })
            .collect();
        findings.extend(admitted.into_iter().filter(|f| f.check == "replace_drops"));
        let changed = patch.changed_path();
        self.pending = Some(proposal_id.0.0.clone());
        let name = self.name_of(by);
        self.journal.write(
            "shown",
            json!({"by": by, "name": name, "proposal_id": proposal_id.0.0, "changed": changed.to_string()}),
        );
        let shown = Server::Proposal(uilab_wire::UilabWireProposalShown {
            after: yaml_at(&after, &changed).unwrap_or_default(),
            before: yaml_at(&before, &changed).unwrap_or_default(),
            by: uilab_wire::EssPresence::Present(by.to_owned()),
            changed: wire::node_path(&changed.to_string()),
            findings: wire::findings(&findings),
            op: wire::op(patch.op_name()),
            outline: wire::outline(&outline(&after)),
            proposal_id: wire::proposal_id(&proposal_id.0.0),
            target: wire::node_path(&patch.target().to_string()),
            utterance,
        });
        self.shown = Some(shown.clone());
        self.preview = Some((proposal_id.0.0.clone(), after));
        self.send(shown);
        Ok(proposal_id.0.0)
    }
}

fn proposal(id: &str) -> s::ProposalId {
    s::ProposalId(Uuid(id.to_owned()))
}

/// Where an instruction lands: in the Components workspace, a selection outside the widgets (a
/// page left selected in the app canvas) gives way to the root, where widgets are declared.
fn placed(target: NodePath, workspace: uilab_agent::Workspace) -> NodePath {
    let in_widgets = target
        .0
        .first()
        .is_some_and(|segment| segment.layer == Layer::Component);
    match workspace {
        uilab_agent::Workspace::Components if !in_widgets => NodePath::root(),
        _ => target,
    }
}

/// The use sites each component node of the outline carries, in outline order.
fn component_uses(doc: &Document) -> Vec<Option<serde_json::Value>> {
    outline(doc)
        .children
        .into_iter()
        .filter(|c| c.layer == Layer::Component)
        .map(|c| c.props.and_then(|p| p.get("uses").cloned()))
        .collect()
}

/// The session actor without a model: `App::open` asks for nothing, so each test answers for the
/// agent by handing the actor the `Planned` and `Proposed` it would have got back.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::goal::{GoalState, StepStatus};
    use std::sync::atomic::{AtomicU64, Ordering};

    static RIGS: AtomicU64 = AtomicU64::new(0);

    struct Rig {
        app: App,
        rx: broadcast::Receiver<Server>,
        /// What the actor sent to the connections the rig connected, and to nobody else.
        direct: mpsc::UnboundedReceiver<Server>,
        direct_tx: mpsc::UnboundedSender<Server>,
        _back: mpsc::Receiver<Cmd>,
        root: PathBuf,
    }

    impl Drop for Rig {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn copy_dir(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_dir(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), &target).unwrap();
            }
        }
    }

    /// A session over a copy of the library example, with the API operator `api-1` joined.
    fn rig(review: bool) -> Rig {
        rig_over(review, |text| text)
    }

    /// A session over a copy of the library example as `edit` rewrites it.
    fn rig_over(review: bool, edit: impl FnOnce(String) -> String) -> Rig {
        let root = std::env::temp_dir().join(format!(
            "uilab-app-test-{}-{}",
            std::process::id(),
            RIGS.fetch_add(1, Ordering::Relaxed)
        ));
        let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library");
        copy_dir(&example, &root.join("library"));
        let file = root.join("library/library.ui.yaml");
        let text = std::fs::read_to_string(&file).unwrap();
        std::fs::write(&file, edit(text)).unwrap();
        let (out, rx) = broadcast::channel(1024);
        let (back, back_rx) = mpsc::channel(16);
        let app = App::open(
            &root.join("library/library.ui.yaml"),
            &root.join("journal"),
            review,
            out,
            back,
        )
        .unwrap();
        let (direct_tx, direct) = mpsc::unbounded_channel();
        let mut rig = Rig {
            app,
            rx,
            direct,
            direct_tx,
            _back: back_rx,
            root,
        };
        let (reply, answer) = oneshot::channel();
        rig.app.handle_cmd(Cmd::Register {
            name: "Bot".into(),
            agent: true,
            reply,
        });
        assert_eq!(answer.blocking_recv().unwrap(), "api-1");
        rig.drain();
        rig
    }

    impl Rig {
        fn client(&mut self, by: &str, text: &str) {
            self.app.handle_cmd(Cmd::Client {
                by: by.into(),
                message: Client::from_text(text).unwrap(),
            });
        }

        /// Everything sent since the last drain: broadcast, then direct.
        fn drain(&mut self) -> Vec<Server> {
            let mut messages = self.drain_broadcast();
            messages.extend(self.drain_direct());
            messages
        }

        fn drain_broadcast(&mut self) -> Vec<Server> {
            let mut messages = Vec::new();
            while let Ok(m) = self.rx.try_recv() {
                messages.push(m);
            }
            messages
        }

        fn drain_direct(&mut self) -> Vec<Server> {
            let mut messages = Vec::new();
            while let Ok(m) = self.direct.try_recv() {
                messages.push(m);
            }
            messages
        }

        /// A browser connects on its own connection.
        fn connect(&mut self, operator: &str) {
            self.app.handle_cmd(Cmd::Connected {
                operator: operator.into(),
                direct: self.direct_tx.clone(),
            });
        }

        fn goal(&self) -> &Goal {
            self.app.goal.as_ref().expect("a goal was given")
        }

        fn statuses(&self) -> Vec<StepStatus> {
            self.goal().steps.iter().map(|s| s.status).collect()
        }

        fn start(&mut self, review: bool) {
            self.client(
                "api-1",
                &format!(
                    r#"{{"type":"goal","value":{{"text":"build out the member area","target":"page:members","review":{review}}}}}"#
                ),
            );
        }

        fn planned(&mut self, targets: &[&str]) {
            let id = self.goal().id.clone();
            self.app.handle_cmd(Cmd::Planned {
                goal_id: id,
                result: Ok(uilab_agent::Plan {
                    steps: targets
                        .iter()
                        .enumerate()
                        .map(|(i, t)| uilab_agent::Step {
                            instruction: format!("step {i}"),
                            target: t.parse().unwrap(),
                            why: "w".into(),
                        })
                        .collect(),
                    turns: 1,
                    cost_micro_usd: None,
                }),
                ms: 1,
            });
        }

        /// The agent answers step `index` with the removal of the member list.
        fn answer(&mut self, index: usize) {
            let (id, review) = (self.goal().id.clone(), self.goal().review);
            self.app.handle_cmd(Cmd::Proposed {
                by: "api-1".into(),
                target: "page:members".parse().unwrap(),
                utterance: format!("step {index}"),
                result: Box::new(Ok(remove_list())),
                ms: 1,
                review,
                step: Some((id, index)),
            });
        }
    }

    fn remove_list() -> uilab_agent::Proposal {
        uilab_agent::Proposal {
            patch: Patch::Remove {
                target: "page:members/section:list".parse().unwrap(),
            },
            turns: 1,
            cost_micro_usd: None,
            attempts: 1,
        }
    }

    fn goal_states(messages: &[Server]) -> Vec<String> {
        messages
            .iter()
            .filter_map(|m| match m {
                Server::Goal(g) => serde_json::to_value(&g.state)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned)),
                _ => None,
            })
            .collect()
    }

    fn refusals(messages: &[Server]) -> Vec<(String, Option<String>)> {
        messages
            .iter()
            .filter_map(|m| match m {
                Server::Refused(r) => Some((r.check.clone(), m.by().map(str::to_owned))),
                _ => None,
            })
            .collect()
    }

    fn proposals(messages: &[Server]) -> Vec<String> {
        messages
            .iter()
            .filter_map(|m| match m {
                Server::Proposal(p) => Some(p.proposal_id.0.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_goal_plans_then_runs_its_first_step_as_its_operator() {
        let mut rig = rig(true);
        rig.start(true);
        assert_eq!(rig.goal().state, GoalState::Planning);
        assert_eq!(goal_states(&rig.drain()), ["planning"]);
        rig.planned(&["page:members", "page:members"]);
        assert_eq!(rig.goal().state, GoalState::Running);
        assert_eq!(rig.statuses(), [StepStatus::Thinking, StepStatus::Pending]);
        assert!(rig.app.busy);
        let messages = rig.drain();
        assert_eq!(goal_states(&messages), ["running"]);
        assert!(
            messages
                .iter()
                .any(|m| matches!(m, Server::Thinking(_)) && m.by() == Some("api-1"))
        );
    }

    #[test]
    fn a_step_whose_target_is_gone_is_refused_without_asking_the_agent() {
        let mut rig = rig(true);
        rig.start(true);
        rig.drain();
        rig.planned(&["page:members/section:nothing", "page:members"]);
        assert_eq!(rig.statuses(), [StepStatus::Refused, StepStatus::Thinking]);
        let messages = rig.drain();
        assert_eq!(
            refusals(&messages),
            [("path_resolves".to_owned(), Some("api-1".to_owned()))]
        );
        assert_eq!(goal_states(&messages), ["running", "running"]);
    }

    #[test]
    fn without_review_each_step_is_accepted_and_the_next_one_starts() {
        let mut rig = rig(true);
        rig.start(false);
        rig.planned(&["page:members", "page:members"]);
        rig.drain();
        rig.answer(0);
        assert_eq!(rig.statuses(), [StepStatus::Accepted, StepStatus::Thinking]);
        assert_eq!(rig.app.revision, 1);
        assert_eq!(rig.app.pending, None);
        let messages = rig.drain();
        assert_eq!(proposals(&messages).len(), 1);
        assert_eq!(goal_states(&messages), ["running", "running"]);
    }

    #[test]
    fn with_review_a_step_waits_and_a_reject_from_anybody_moves_on() {
        let mut rig = rig(false);
        rig.start(true);
        rig.planned(&["page:members", "page:members"]);
        rig.answer(0);
        assert_eq!(rig.statuses(), [StepStatus::Proposed, StepStatus::Pending]);
        let id = rig.goal().steps[0].proposal_id.clone().unwrap();
        assert_eq!(rig.app.pending.as_deref(), Some(id.as_str()));
        rig.client(
            "ws-7",
            &format!(r#"{{"type":"reject","value":{{"proposal_id":"{id}"}}}}"#),
        );
        assert_eq!(rig.statuses(), [StepStatus::Rejected, StepStatus::Thinking]);
        assert_eq!(rig.app.revision, 0);
    }

    #[test]
    fn stopping_while_planning_stops_and_the_late_plan_is_ignored() {
        let mut rig = rig(true);
        rig.start(true);
        rig.drain();
        rig.client(
            "ws-7",
            r#"{"type":"stop_goal","value":{"goal_id":"goal-1"}}"#,
        );
        assert_eq!(rig.goal().state, GoalState::Stopped);
        assert_eq!(goal_states(&rig.drain()), ["stopped"]);
        rig.planned(&["page:members"]);
        assert_eq!(rig.goal().state, GoalState::Stopped);
        assert!(rig.goal().steps.is_empty());
        assert!(goal_states(&rig.drain()).is_empty());
    }

    #[test]
    fn stopping_while_a_step_thinks_ends_the_step_for_its_operator_and_drops_the_answer() {
        let mut rig = rig(true);
        rig.start(true);
        rig.planned(&["page:members", "page:members"]);
        rig.drain();
        rig.client(
            "ws-7",
            r#"{"type":"stop_goal","value":{"goal_id":"goal-1"}}"#,
        );
        let messages = rig.drain();
        assert_eq!(goal_states(&messages), ["stopped"]);
        assert_eq!(
            refusals(&messages),
            [("goal_stopped".to_owned(), Some("api-1".to_owned()))],
            "the owner's thinking step ends with a message attributed to it"
        );
        rig.answer(0);
        let messages = rig.drain();
        assert!(
            proposals(&messages).is_empty(),
            "the late answer is not shown"
        );
        assert!(!rig.app.busy);
        assert_eq!(rig.app.pending, None);
        assert_eq!(rig.statuses(), [StepStatus::Pending, StepStatus::Pending]);
    }

    #[test]
    fn stopping_with_a_proposal_waiting_rejects_it_and_sends_the_document() {
        let mut rig = rig(true);
        rig.start(true);
        rig.planned(&["page:members"]);
        rig.answer(0);
        rig.drain();
        rig.client(
            "ws-7",
            r#"{"type":"stop_goal","value":{"goal_id":"goal-1"}}"#,
        );
        assert_eq!(rig.statuses(), [StepStatus::Rejected]);
        assert_eq!(rig.app.pending, None);
        let messages = rig.drain();
        assert_eq!(goal_states(&messages), ["stopped"]);
        assert!(messages.iter().any(|m| matches!(m, Server::Document(_))));
        rig.client(
            "ws-7",
            r#"{"type":"stop_goal","value":{"goal_id":"goal-1"}}"#,
        );
        assert_eq!(
            refusals(&rig.drain()),
            [("wrong_state".to_owned(), Some("ws-7".to_owned()))]
        );
    }

    #[test]
    fn an_answer_for_a_step_the_run_no_longer_waits_for_is_discarded() {
        let mut rig = rig(true);
        rig.start(false);
        rig.planned(&["page:members/section:nothing", "page:members"]);
        rig.drain();
        rig.answer(0);
        assert!(proposals(&rig.drain()).is_empty());
        assert_eq!(rig.statuses(), [StepStatus::Refused, StepStatus::Thinking]);
        assert_eq!(rig.app.revision, 0);
    }

    #[test]
    fn the_goals_api_operator_stays_present_while_the_goal_runs() {
        let mut rig = rig(true);
        let long_ago = Instant::now()
            .checked_sub(API_OPERATOR_TTL * 2)
            .expect("the clock is past two TTLs");
        rig.start(true);
        rig.app.operators.get_mut("api-1").unwrap().last_seen = long_ago;
        rig.app.handle_cmd(Cmd::Tick);
        assert!(rig.app.operators.contains_key("api-1"));
        rig.client(
            "ws-7",
            r#"{"type":"stop_goal","value":{"goal_id":"goal-1"}}"#,
        );
        rig.app.operators.get_mut("api-1").unwrap().last_seen = long_ago;
        rig.app.handle_cmd(Cmd::Tick);
        assert!(!rig.app.operators.contains_key("api-1"));
    }

    #[test]
    fn while_a_goal_runs_say_and_another_goal_are_refused() {
        let mut rig = rig(true);
        rig.start(true);
        rig.drain();
        rig.client("ws-7", r#"{"type":"say","value":{"text":"add a page"}}"#);
        rig.client("ws-7", r#"{"type":"goal","value":{"text":"another"}}"#);
        assert_eq!(
            refusals(&rig.drain()),
            [
                ("goal_running".to_owned(), Some("ws-7".to_owned())),
                ("goal_running".to_owned(), Some("ws-7".to_owned()))
            ]
        );
        assert_eq!(rig.goal().id, "goal-1");
    }

    #[test]
    fn a_goal_is_refused_while_the_agent_is_busy_or_a_microphone_is_open() {
        let mut rig = rig(true);
        rig.app.audio = Some(("ws-7".into(), Vec::new()));
        rig.start(true);
        rig.app.audio = None;
        rig.app.busy = true;
        rig.start(true);
        assert_eq!(
            refusals(&rig.drain()),
            [
                ("goal_busy".to_owned(), Some("api-1".to_owned())),
                ("goal_busy".to_owned(), Some("api-1".to_owned()))
            ]
        );
        assert!(rig.app.goal.is_none());
    }

    #[test]
    fn a_new_goal_rejects_a_waiting_proposal_and_sends_the_document() {
        let mut rig = rig(true);
        rig.app.handle_cmd(Cmd::Proposed {
            by: "ws-7".into(),
            target: "page:members".parse().unwrap(),
            utterance: "remove the list".into(),
            result: Box::new(Ok(remove_list())),
            ms: 1,
            review: true,
            step: None,
        });
        assert!(rig.app.pending.is_some());
        rig.drain();
        rig.start(true);
        assert_eq!(rig.app.pending, None);
        let messages = rig.drain();
        assert!(messages.iter().any(|m| matches!(m, Server::Document(_))));
        assert_eq!(goal_states(&messages), ["planning"]);
    }

    #[test]
    fn a_browser_that_connects_gets_the_goal_and_the_waiting_proposal() {
        let mut rig = rig(true);
        rig.start(true);
        rig.planned(&["page:members"]);
        rig.answer(0);
        let id = rig.goal().steps[0].proposal_id.clone().unwrap();
        rig.drain();
        rig.connect("ws-9");
        let messages = rig.drain();
        assert_eq!(goal_states(&messages), ["running"]);
        assert_eq!(proposals(&messages), [id]);
    }

    #[test]
    fn the_connect_snapshot_goes_to_the_connecting_browser_and_nobody_else() {
        let mut rig = rig(true);
        rig.start(true);
        rig.planned(&["page:members"]);
        rig.answer(0);
        let id = rig.goal().steps[0].proposal_id.clone().unwrap();
        rig.drain();
        rig.connect("ws-9");
        let everybody = rig.drain_broadcast();
        assert!(
            everybody.iter().all(|m| matches!(m, Server::Presence(_))),
            "browsers already connected get only the new presence: {everybody:?}"
        );
        assert!(!everybody.is_empty());
        let direct = rig.drain_direct();
        let kinds: Vec<&str> = direct
            .iter()
            .map(|m| match m {
                Server::Document(_) => "document",
                Server::Goal(_) => "goal",
                Server::Proposal(_) => "proposal",
                _ => "other",
            })
            .collect();
        assert_eq!(kinds, ["document", "goal", "proposal"]);
        assert_eq!(proposals(&direct), [id]);
    }

    const LOAN_CARD: &str = "widgets:
  loan_card:
    summary: A loan as a card.
    params:
      loan: {type: Loan, required: true}
      compact: {type: boolean, default: false}
    body:
      - {name: title, primitive: text, text: args.loan.title, style: heading}
      - {name: due, primitive: badge, text: args.loan.due}
pages:
";

    /// The library with the widget `loan_card`, used once as a section of the overview.
    fn with_loan_card(text: String) -> String {
        assert!(text.contains("\npages:\n") && text.contains("    sections:\n      on_loan:\n"));
        text.replacen("\npages:\n", &format!("\n{LOAN_CARD}"), 1).replacen(
            "    sections:\n      on_loan:\n",
            "    sections:\n      latest: {component: loan_card, args: {loan: rows.first}}\n      on_loan:\n",
            1,
        )
    }

    /// The outline of the last document message, as the browser receives it.
    fn last_outline(messages: &[Server]) -> serde_json::Value {
        let doc = messages
            .iter()
            .rev()
            .find_map(|m| match m {
                Server::Document(d) => Some(d),
                _ => None,
            })
            .expect("a document was sent");
        serde_json::to_value(&doc.outline).unwrap()
    }

    fn child<'v>(node: &'v serde_json::Value, path: &str) -> &'v serde_json::Value {
        node["children"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["path"] == path)
            .unwrap_or_else(|| panic!("no child `{path}` in {}", node["path"]))
    }

    #[test]
    fn the_document_a_browser_gets_carries_widgets_their_body_and_their_instances() {
        let mut rig = rig_over(false, with_loan_card);
        rig.connect("ws-9");
        let root = last_outline(&rig.drain_direct());
        let card = child(&root, "component:loan_card");
        assert_eq!(card["layer"], "component");
        assert_eq!(card["name"], "loan_card");
        assert_eq!(card["kind"], "widget");
        assert_eq!(card["title"], "A loan as a card.");
        assert_eq!(
            card["props"]["params"],
            serde_json::json!({
                "loan": {"type": "Loan", "required": true},
                "compact": {"type": "boolean", "default": false},
            })
        );
        assert_eq!(card["props"]["arrange"], "column");
        let body: Vec<(&str, &str, &str)> = card["children"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| {
                (
                    n["path"].as_str().unwrap(),
                    n["layer"].as_str().unwrap(),
                    n["kind"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            body,
            [
                ("component:loan_card/node:title", "node", "text"),
                ("component:loan_card/node:due", "node", "badge"),
            ]
        );
        assert_eq!(
            child(card, "component:loan_card/node:title")["props"],
            serde_json::json!({"text": "args.loan.title", "style": "heading"})
        );
        let latest = child(
            child(&root, "page:overview"),
            "page:overview/section:latest",
        );
        assert_eq!(latest["kind"], "loan_card");
        assert_eq!(
            latest["props"]["args"],
            serde_json::json!({"loan": "rows.first"})
        );
    }

    /// The Components tab lists a widget's use sites from the outline, so the outline carries
    /// every one the docs list, a page header's included.
    #[test]
    fn the_outline_a_browser_gets_carries_each_widget_s_use_sites() {
        let mut rig = rig_over(false, |text| {
            let text = with_loan_card(text);
            assert!(text.contains("    title: Overview\n    sections:\n"));
            text.replacen(
                "    title: Overview\n    sections:\n",
                "    title: Overview\n    header: {metrics: [{name: due, component: loan_card, args: {loan: rows.first}}]}\n    sections:\n",
                1,
            )
        });
        rig.connect("ws-9");
        let root = last_outline(&rig.drain_direct());
        let card = child(&root, "component:loan_card");
        assert_eq!(
            card["props"]["uses"],
            serde_json::json!([
                {"path": "page:overview/section:latest"},
                {"path": "page:overview", "trail": "header/metrics/due"},
            ])
        );
    }

    #[test]
    fn selecting_a_widget_or_one_of_its_body_nodes_lands_there() {
        let mut rig = rig_over(false, with_loan_card);
        rig.connect("ws-9");
        rig.drain();
        for path in ["component:loan_card", "component:loan_card/node:due"] {
            rig.client(
                "ws-9",
                &format!(r#"{{"type":"select","value":{{"path":"{path}"}}}}"#),
            );
            let messages = rig.drain();
            assert_eq!(refusals(&messages), []);
            let selected = messages
                .iter()
                .rev()
                .find_map(|m| match m {
                    Server::Document(d) => Some(d.selected.0.clone()),
                    _ => None,
                })
                .expect("a document was sent");
            assert_eq!(selected, path);
        }
        rig.client(
            "ws-9",
            r#"{"type":"select","value":{"path":"component:loan_card/node:ghost"}}"#,
        );
        assert_eq!(
            refusals(&rig.drain())
                .into_iter()
                .map(|(check, _)| check)
                .collect::<Vec<_>>(),
            ["path_resolves"]
        );
    }

    #[test]
    fn a_reject_of_another_proposal_keeps_the_waiting_one() {
        let mut rig = rig(true);
        rig.start(true);
        rig.planned(&["page:members"]);
        rig.answer(0);
        let id = rig.goal().steps[0].proposal_id.clone().unwrap();
        rig.client(
            "ws-7",
            r#"{"type":"reject","value":{"proposal_id":"an-earlier-proposal"}}"#,
        );
        assert_eq!(rig.app.pending.as_deref(), Some(id.as_str()));
        assert_eq!(rig.statuses(), [StepStatus::Proposed]);
    }

    #[test]
    fn a_browser_that_connects_gets_no_proposal_once_it_is_decided() {
        let mut rig = rig(true);
        rig.start(true);
        rig.planned(&["page:members"]);
        rig.answer(0);
        let id = rig.goal().steps[0].proposal_id.clone().unwrap();
        rig.client(
            "ws-7",
            &format!(r#"{{"type":"reject","value":{{"proposal_id":"{id}"}}}}"#),
        );
        rig.drain();
        rig.connect("ws-9");
        assert!(proposals(&rig.drain()).is_empty());
    }

    #[test]
    fn a_stale_decision_on_another_proposal_keeps_the_step_proposal_for_a_browser_that_connects() {
        let mut rig = rig(true);
        rig.start(true);
        rig.planned(&["page:members"]);
        rig.answer(0);
        let id = rig.goal().steps[0].proposal_id.clone().unwrap();
        // A card or a command left over from an earlier proposal decides that one, not the step's.
        rig.client(
            "ws-7",
            r#"{"type":"accept","value":{"proposal_id":"an-earlier-proposal"}}"#,
        );
        assert_eq!(
            rig.statuses(),
            [StepStatus::Proposed],
            "precondition: the step still waits on its proposal"
        );
        rig.drain();
        rig.connect("ws-9");
        assert_eq!(
            proposals(&rig.drain()),
            [id],
            "the step waits on a proposal no browser that connects is shown"
        );
    }

    #[test]
    fn a_replace_that_drops_columns_says_so_on_the_proposal_card() {
        let mut rig = rig(true);
        rig.app.handle_cmd(Cmd::Proposed {
            by: "api-1".into(),
            target: "page:members/section:list".parse().unwrap(),
            utterance: "only names".into(),
            result: Box::new(Ok(uilab_agent::Proposal {
                patch: Patch::Replace {
                    target: "page:members/section:list".parse().unwrap(),
                    node: json!({
                        "component": "collection",
                        "reads": {"view": "members.All"},
                        "columns": [{"field": "name"}],
                    }),
                },
                turns: 1,
                cost_micro_usd: None,
                attempts: 1,
            })),
            ms: 1,
            review: true,
            step: None,
        });
        let proposal = rig
            .drain()
            .into_iter()
            .find_map(|m| match m {
                Server::Proposal(p) => Some(p),
                _ => None,
            })
            .expect("a proposal is shown");
        let findings = serde_json::to_value(&proposal.findings).unwrap();
        let drops: Vec<&str> = findings
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["check"] == "replace_drops")
            .filter_map(|f| f["message"].as_str())
            .collect();
        assert_eq!(
            drops,
            ["replace at page:members/section:list drops columns joined, loans, standing"]
        );
    }

    /// Accepts `patch` as the API operator's, with review off.
    fn accept_patch(rig: &mut Rig, patch: Patch) -> Vec<Server> {
        rig.app.handle_cmd(Cmd::Proposed {
            by: "api-1".into(),
            target: patch.target().clone(),
            utterance: "u".into(),
            result: Box::new(Ok(uilab_agent::Proposal {
                patch,
                turns: 1,
                cost_micro_usd: None,
                attempts: 1,
            })),
            ms: 1,
            review: false,
            step: None,
        });
        rig.drain_broadcast()
    }

    fn loan_card_uses(outline: &serde_json::Value) -> Vec<String> {
        child(outline, "component:loan_card")["props"]["uses"]
            .as_array()
            .unwrap()
            .iter()
            .map(|u| u["path"].as_str().unwrap().to_owned())
            .collect()
    }

    /// The use sites sit on component nodes, which a delta at the changed path does not re-send:
    /// a change that adds or drops an instance goes out as a full document, and one that does
    /// not stays a delta.
    #[test]
    fn a_change_that_adds_or_drops_a_widget_instance_sends_the_whole_outline() {
        let mut rig = rig_over(false, with_loan_card);
        let sent = accept_patch(
            &mut rig,
            Patch::Insert {
                target: "page:overview".parse().unwrap(),
                child: uilab_doc::Child {
                    layer: Layer::Section,
                    name: "more".into(),
                    node: json!({"component": "loan_card", "args": {"loan": "rows.first"}}),
                    nav_section: None,
                },
            },
        );
        assert!(!sent.iter().any(|m| matches!(m, Server::Changed(_))));
        assert_eq!(
            loan_card_uses(&last_outline(&sent)),
            ["page:overview/section:latest", "page:overview/section:more"]
        );

        let sent = accept_patch(
            &mut rig,
            Patch::Remove {
                target: "page:overview/section:latest".parse().unwrap(),
            },
        );
        assert_eq!(
            loan_card_uses(&last_outline(&sent)),
            ["page:overview/section:more"]
        );

        let sent = accept_patch(
            &mut rig,
            Patch::Remove {
                target: "page:overview/section:recent".parse().unwrap(),
            },
        );
        assert!(sent.iter().any(|m| matches!(m, Server::Changed(_))));
        assert!(!sent.iter().any(|m| matches!(m, Server::Document(_))));
    }

    /// Where the agent was asked to work, from the `thinking` message.
    fn thinking_at(messages: &[Server]) -> String {
        messages
            .iter()
            .find_map(|m| match m {
                Server::Thinking(t) => Some(t.target.0.clone()),
                _ => None,
            })
            .expect("the agent was asked")
    }

    /// Found on 2026-09-30: "put some basic set of components" given on the Components tab with
    /// `page:overview` still selected became three sections of that page.
    #[test]
    fn an_instruction_from_the_components_workspace_lands_among_the_widgets() {
        let mut rig = rig_over(false, with_loan_card);
        rig.client(
            "api-1",
            r#"{"type":"select","value":{"path":"page:overview"}}"#,
        );
        rig.drain();
        rig.client(
            "api-1",
            r#"{"type":"say","value":{"text":"put some basic set of components now","workspace":"components"}}"#,
        );
        assert_eq!(thinking_at(&rig.drain()), "/");
        rig.app.busy = false;

        rig.client(
            "api-1",
            r#"{"type":"select","value":{"path":"component:loan_card/node:title"}}"#,
        );
        rig.drain();
        rig.client(
            "api-1",
            r#"{"type":"say","value":{"text":"make the title smaller","workspace":"components"}}"#,
        );
        assert_eq!(
            thinking_at(&rig.drain()),
            "component:loan_card/node:title",
            "a selection inside the widgets stays"
        );
        rig.app.busy = false;

        rig.client(
            "api-1",
            r#"{"type":"select","value":{"path":"page:overview"}}"#,
        );
        rig.drain();
        rig.client("api-1", r#"{"type":"say","value":{"text":"add a chart"}}"#);
        assert_eq!(
            thinking_at(&rig.drain()),
            "page:overview",
            "the app canvas keeps the selection"
        );
    }

    #[test]
    fn a_goal_from_the_components_workspace_is_planned_among_the_widgets() {
        let mut rig = rig_over(false, with_loan_card);
        rig.client(
            "api-1",
            r#"{"type":"select","value":{"path":"page:overview"}}"#,
        );
        rig.client(
            "api-1",
            r#"{"type":"goal","value":{"text":"build a basic component set","workspace":"components"}}"#,
        );
        assert_eq!(rig.goal().target.to_string(), "/");
        assert_eq!(rig.goal().workspace, uilab_agent::Workspace::Components);
    }

    /// Found on 2026-09-30: every card repeated the document's five `draft_read` warnings.
    #[test]
    fn a_proposal_card_names_only_the_findings_the_proposal_brings() {
        let mut rig = rig_over(true, |text| {
            assert!(text.contains("    sections:\n      on_loan:\n"));
            text.replacen(
                "    sections:\n      on_loan:\n",
                "    sections:\n      trend: {component: chart, reads: {view: draft.LoansPerMonth}}\n      on_loan:\n",
                1,
            )
        });
        let sent = accept_patch_reviewed(
            &mut rig,
            Patch::Insert {
                target: "page:overview".parse().unwrap(),
                child: uilab_doc::Child {
                    layer: Layer::Section,
                    name: "by_state".into(),
                    node: json!({"component": "chart", "reads": {"view": "draft.LoansByState"}}),
                    nav_section: None,
                },
            },
        );
        let proposal = sent
            .iter()
            .find_map(|m| match m {
                Server::Proposal(p) => Some(p),
                _ => None,
            })
            .expect("a proposal is shown");
        let findings = serde_json::to_value(&proposal.findings).unwrap();
        let paths: Vec<&str> = findings
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|f| f["path"].as_str())
            .collect();
        assert_eq!(paths, ["page:overview/section:by_state"]);
    }

    /// Shows `patch` as the API operator's proposal, waiting for review.
    fn accept_patch_reviewed(rig: &mut Rig, patch: Patch) -> Vec<Server> {
        rig.app.handle_cmd(Cmd::Proposed {
            by: "api-1".into(),
            target: patch.target().clone(),
            utterance: "u".into(),
            result: Box::new(Ok(uilab_agent::Proposal {
                patch,
                turns: 1,
                cost_micro_usd: None,
                attempts: 1,
            })),
            ms: 1,
            review: true,
            step: None,
        });
        rig.drain()
    }

    /// The rows the session answers a `rows` request for `view` with.
    fn rows_of(rig: &mut Rig, view: &str) -> Vec<serde_json::Value> {
        rig.drain();
        rig.client(
            "api-1",
            &format!(r#"{{"type":"rows","value":{{"view":"{view}"}}}}"#),
        );
        rig.drain()
            .into_iter()
            .find_map(|m| match m {
                Server::Rows(r) if r.view == view => Some(r.rows),
                _ => None,
            })
            .expect("the rows are answered")
    }

    /// A proposal preview reads a draft view the document does not: its sample rows carry the
    /// fields the preview's composites name while the proposal waits, and the document's again
    /// once it is rejected.
    #[test]
    fn sample_rows_follow_the_waiting_proposal_and_not_the_document_it_would_change() {
        let mut rig = rig(true);
        let view = "draft.MembersWithOverdue";
        let sent = accept_patch_reviewed(
            &mut rig,
            Patch::Replace {
                target: "page:overview/section:on_loan".parse().unwrap(),
                node: json!({
                    "component": "metric",
                    "title": "Members with overdue loans",
                    "reads": {"view": view},
                    "from": "members",
                }),
            },
        );
        let id = proposals(&sent)
            .pop()
            .expect("the replace waits as a proposal");
        let previewed = rows_of(&mut rig, view);
        assert!(!previewed.is_empty());
        assert!(
            previewed.iter().all(|r| r["members"].is_i64()),
            "the preview's metric reads `members`: {previewed:?}"
        );

        rig.client(
            "api-1",
            &format!(r#"{{"type":"reject","value":{{"proposal_id":"{id}"}}}}"#),
        );
        let after = rows_of(&mut rig, view);
        assert!(
            after.iter().all(|r| r.get("members").is_none()),
            "nothing in the document reads the view once the proposal is rejected: {after:?}"
        );
    }
}
