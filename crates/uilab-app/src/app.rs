//! The one session the server holds: the open document, its revision, the operators working on it,
//! the microphone and the proposal in flight.
//!
//! One task owns all of it and handles one message at a time. Browser connections and the operator
//! API send it what an operator did; speech and the agent run on blocking threads and send their
//! results back. Everything is broadcast to every connection, attributed to the operator it came
//! from.

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

use crate::journal::Journal;
use crate::wire::{self, ChangedParts, Client, DocumentParts, OperatorParts, Server};

/// An operator the API registered and has not heard from for this long is gone.
pub const API_OPERATOR_TTL: Duration = Duration::from_secs(60);

/// What the session task is told.
pub enum Cmd {
    /// A browser connected; it is an anonymous human until it says hello.
    Connected { operator: String },
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
        result: Result<uilab_agent::Proposal, (String, String)>,
        ms: u64,
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
    /// Expire operators the API stopped calling.
    Tick,
}

pub struct Config {
    pub doc: PathBuf,
    pub stt: Option<uilab_stt::TranscriberConfig>,
    pub proposer: uilab_agent::ProposerConfig,
    pub journal: PathBuf,
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
    proposer: Arc<Mutex<uilab_agent::Proposer>>,
    operators: IndexMap<String, Operator>,
    selected_by: Option<String>,
    revision: u64,
    next_api_operator: u64,
    pending: Option<String>,
    audio: Option<(String, Vec<f32>)>,
    busy: bool,
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
        let file = config.doc.to_string_lossy().into_owned();
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
        let dir = config.doc.parent().unwrap_or(Path::new("."));
        let fixtures = Fixtures::load(&doc, dir).map_err(|e| e.to_string())?;
        let fields = fixtures.fields().into_iter().collect();
        let journal = Journal::open(&config.journal)
            .map_err(|e| format!("journal {}: {e}", config.journal.display()))?;
        println!("uilab: journal in {}", journal.dir().display());
        journal.write(
            "start",
            json!({
                "doc": file,
                "model": config.proposer.model,
                "base_url": config.proposer.base_url,
                "stt": config.stt.as_ref().map(|c| c.model.display().to_string()),
            }),
        );
        let stt = match config.stt {
            Some(c) => Some(Arc::new(Mutex::new(
                uilab_stt::Transcriber::load(c).map_err(|e| format!("speech model: {e}"))?,
            ))),
            None => None,
        };
        let proposer =
            uilab_agent::Proposer::new(config.proposer).map_err(|e| format!("agent: {e}"))?;
        Ok(App {
            port,
            handle,
            document_id,
            file,
            fixtures,
            fields,
            journal: Arc::new(journal),
            stt,
            proposer: Arc::new(Mutex::new(proposer)),
            operators: IndexMap::new(),
            selected_by: None,
            revision: 0,
            next_api_operator: 0,
            pending: None,
            audio: None,
            busy: false,
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
            Cmd::Connected { operator } => {
                self.operators.insert(
                    operator,
                    Operator {
                        name: "anonymous".into(),
                        agent: false,
                        last_seen: Instant::now(),
                        api: false,
                    },
                );
                self.send_document();
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
            Cmd::Tick => {
                let before = self.operators.len();
                self.operators
                    .retain(|_, o| !o.api || o.last_seen.elapsed() < API_OPERATOR_TTL);
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
                    self.propose(&by, t.text, None);
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
            } => {
                self.busy = false;
                match result {
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
                            }),
                        );
                        self.record(&by, utterance, proposal.patch)
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
                            }),
                        );
                        self.send(Server::refused(check, message, Some(&by)))
                    }
                }
            }
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
                if self.stt.is_none() {
                    self.send(Server::failed(
                        "speech is off: start uilab with a speech model, or type the instruction",
                        Some(by),
                    ));
                } else {
                    self.audio = Some((by.to_owned(), Vec::new()));
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
                self.propose(by, say.text, target)
            }
            Client::Accept(d) => {
                let id = proposal(&d.proposal_id.0);
                let outcome = self.port.accept_proposal(s::AcceptProposal {
                    proposal_id: id.clone(),
                });
                self.pending = None;
                match outcome {
                    Ok(s::AcceptProposalOutcome::Accepted { .. }) => {
                        self.revision += 1;
                        self.journal.write(
                            "accepted",
                            json!({"by": by, "proposal_id": d.proposal_id.0, "revision": self.revision}),
                        );
                        self.announce_change(by, &id);
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
            }
            Client::Reject(d) => {
                let _ = self.port.reject_proposal(s::RejectProposal {
                    proposal_id: proposal(&d.proposal_id.0),
                });
                self.pending = None;
                self.send_document();
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
                let rows = self.fixtures.rows(&read.view);
                self.send(wire::rows(&read.view, rows.total, rows.rows));
            }
        }
    }

    /// Broadcasts an accepted proposal as a delta. A page or a menu section changes the menu as
    /// well, which a delta at one path cannot carry, so those go out as a full snapshot.
    fn announce_change(&self, by: &str, id: &s::ProposalId) {
        let Some(patch) = self.handle.patch(id) else {
            self.send_document();
            return;
        };
        let changed = patch.changed_path();
        if matches!(
            changed.layer(),
            Layer::Page | Layer::NavSection | Layer::Shell
        ) {
            self.send_document();
            return;
        }
        let doc = self.doc();
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

    fn propose(&mut self, by: &str, utterance: String, target: Option<NodePath>) {
        if self.busy {
            self.send(Server::failed(
                "still working on the last instruction",
                Some(by),
            ));
            return;
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
        let proposer = self.proposer.clone();
        let fields = self.fields.clone();
        let back = self.back.clone();
        let by = by.to_owned();
        tokio::task::spawn_blocking(move || {
            let started = Instant::now();
            let result = proposer
                .lock()
                .expect("the proposer lock")
                .propose_with(&doc, &target, &utterance, &fields)
                .map_err(|e| match e {
                    uilab_agent::ProposeError::Refused { check, message } => (check, message),
                    other => ("agent".to_owned(), other.to_string()),
                });
            let ms = started.elapsed().as_millis() as u64;
            let _ = back.blocking_send(Cmd::Proposed {
                by,
                target,
                utterance,
                result,
                ms,
            });
        });
    }

    /// Records the agent's patch as a proposal and shows it.
    fn record(&mut self, by: &str, utterance: String, patch: Patch) {
        let body = match &patch {
            Patch::Insert { child, .. } => Some(body_from_json(
                &serde_json::to_value(child).expect("a child serializes"),
            )),
            Patch::Replace { node, .. } => Some(body_from_json(node)),
            Patch::Remove { .. } => None,
        };
        let op = match &patch {
            Patch::Insert { .. } => s::PatchOp::Insert,
            Patch::Replace { .. } => s::PatchOp::Replace,
            Patch::Remove { .. } => s::PatchOp::Remove,
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
                self.send(Server::refused(
                    error.check,
                    "the proposed patch fails a document check",
                    Some(by),
                ));
                return;
            }
            Err(e) => {
                self.send(Server::failed(e.to_string(), Some(by)));
                return;
            }
        };
        let before = self.doc();
        let Ok((after, _)) = admit(&before, &patch) else {
            self.send(Server::failed(
                "the admitted patch no longer applies",
                Some(by),
            ));
            return;
        };
        let findings = self.findings(&after);
        let changed = patch.changed_path();
        self.pending = Some(proposal_id.0.0.clone());
        let name = self.name_of(by);
        self.journal.write(
            "shown",
            json!({"by": by, "name": name, "proposal_id": proposal_id.0.0, "changed": changed.to_string()}),
        );
        self.send(Server::Proposal(uilab_wire::UilabWireProposalShown {
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
        }));
    }
}

fn proposal(id: &str) -> s::ProposalId {
    s::ProposalId(Uuid(id.to_owned()))
}
