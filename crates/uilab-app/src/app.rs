//! The one session the server holds: the open document, the microphone, the proposal in flight.
//!
//! One task owns all of it and handles one message at a time; WebSocket connections send it what
//! the browser said, and speech and the agent run on blocking threads and send their results back.
//! Everything the browser should see is broadcast to every connection.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::json;
use tokio::sync::{broadcast, mpsc};
use uilab_behaviour::{FileSource, Handle, body_from_json};
use uilab_doc::{
    Document, Finding, Fixtures, NodePath, Patch, admit, check, field_findings, outline,
    vocabulary, yaml_at,
};
use uilab_session::UilabSession;
use uilab_types::primitives::Uuid;
use uilab_types::session as s;

use crate::journal::Journal;
use crate::wire::{self, Client, DocumentParts, Server};

/// What the session task is told.
pub enum Cmd {
    /// A message from a browser.
    Client(Client),
    /// A frame of 16 kHz mono samples.
    Audio(Vec<f32>),
    /// Speech came back.
    Heard {
        result: Result<uilab_stt::Transcript, String>,
        wav: Option<String>,
    },
    /// The agent came back.
    Proposed {
        target: NodePath,
        utterance: String,
        result: Result<uilab_agent::Proposal, (String, String)>,
        ms: u64,
    },
    /// A browser connected and needs the current state.
    Hello,
}

pub struct Config {
    pub doc: PathBuf,
    pub stt: Option<uilab_stt::TranscriberConfig>,
    pub proposer: uilab_agent::ProposerConfig,
    pub journal: PathBuf,
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
    pending: Option<String>,
    audio: Option<Vec<f32>>,
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
            Server::Refused(r) => self
                .journal
                .write("refused", json!({"check": r.check, "message": r.message})),
            Server::Failed(f) => self.journal.write("failed", json!({"message": f.message})),
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

    fn send_document(&self) {
        let doc = self.doc();
        let tree = outline(&doc);
        let findings = self.findings(&doc);
        let selected = self.selected().to_string();
        let undoable = self
            .handle
            .last_undoable(&self.document_id)
            .map(|id| id.0.0);
        self.send(wire::document(DocumentParts {
            document_id: &self.document_id.0.0,
            file: &self.file,
            title: doc.title.clone(),
            selected: &selected,
            outline: &tree,
            findings: &findings,
            undoable,
        }));
    }

    fn handle_cmd(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::Hello => self.send_document(),
            Cmd::Client(message) => self.client(message),
            Cmd::Audio(samples) => {
                if let Some(buffer) = &mut self.audio {
                    buffer.extend(samples);
                }
            }
            Cmd::Heard { result: Ok(t), wav } => {
                self.journal.write(
                    "heard",
                    json!({"text": t.text, "audio_ms": t.audio_ms, "took_ms": t.took_ms, "wav": wav}),
                );
                self.send(wire::transcript(&t.text, t.audio_ms, t.took_ms));
                self.busy = false;
                if t.text.trim().is_empty() {
                    self.send(Server::refused("speech", "nothing was heard"));
                } else {
                    self.propose(t.text);
                }
            }
            Cmd::Heard {
                result: Err(e),
                wav,
            } => {
                self.journal.write("heard", json!({"error": e, "wav": wav}));
                self.busy = false;
                self.send(Server::failed(format!("speech: {e}")));
            }
            Cmd::Proposed {
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
                                "utterance": utterance,
                                "target": target.to_string(),
                                "ms": ms,
                                "attempts": proposal.attempts,
                                "turns": proposal.turns,
                                "patch": proposal.patch,
                            }),
                        );
                        self.record(&target, utterance, proposal.patch)
                    }
                    Err((check, message)) => {
                        self.journal.write(
                            "not_proposed",
                            json!({
                                "utterance": utterance,
                                "target": target.to_string(),
                                "ms": ms,
                                "check": check,
                                "message": message,
                            }),
                        );
                        self.send(Server::refused(check, message))
                    }
                }
            }
        }
    }

    fn client(&mut self, message: Client) {
        if !matches!(message, Client::Rows(_)) {
            self.journal.write("operator", json!({"message": message}));
        }
        match message {
            Client::Select(select) => {
                let outcome = self.port.select_node(s::SelectNode {
                    document_id: self.document_id.clone(),
                    path: s::NodePath(select.path.0.clone()),
                });
                match outcome {
                    Ok(s::SelectNodeOutcome::Selected { .. }) => self.send_document(),
                    Ok(_) => self.send(Server::refused(
                        "path_resolves",
                        format!("no node at `{}`", select.path.0),
                    )),
                    Err(e) => self.send(Server::failed(e.to_string())),
                }
            }
            Client::Mic(mic) if wire::mic_open(&mic) => {
                if self.stt.is_none() {
                    self.send(Server::failed(
                        "speech is off: start uilab with a speech model, or type the instruction",
                    ));
                } else {
                    self.audio = Some(Vec::new());
                }
            }
            Client::Mic(_) => {
                if let Some(samples) = self.audio.take() {
                    self.transcribe(samples);
                }
            }
            Client::Say(say) => self.propose(say.text),
            Client::Accept(d) => {
                let outcome = self.port.accept_proposal(s::AcceptProposal {
                    proposal_id: proposal(&d.proposal_id.0),
                });
                self.pending = None;
                match outcome {
                    Ok(s::AcceptProposalOutcome::Accepted { .. }) => {
                        self.journal
                            .write("accepted", json!({"proposal_id": d.proposal_id.0}));
                        self.send_document()
                    }
                    Ok(s::AcceptProposalOutcome::Stale { error }) => self.send(Server::refused(
                        error.check,
                        "the document changed since the proposal",
                    )),
                    Ok(_) => self.send(Server::refused(
                        "wrong_state",
                        "that proposal is not waiting",
                    )),
                    Err(e) => self.send(Server::failed(e.to_string())),
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
                        self.journal
                            .write("undone", json!({"proposal_id": d.proposal_id.0}));
                        self.send_document()
                    }
                    Ok(s::UndoProposalOutcome::Stale { error }) => {
                        self.send(Server::refused(error.check, "a later change would be lost"))
                    }
                    Ok(_) => self.send(Server::refused(
                        "wrong_state",
                        "that proposal was not applied",
                    )),
                    Err(e) => self.send(Server::failed(e.to_string())),
                }
            }
            Client::Rows(read) => {
                let rows = self.fixtures.rows(&read.view);
                self.send(wire::rows(&read.view, rows.total, rows.rows));
            }
        }
    }

    fn transcribe(&mut self, samples: Vec<f32>) {
        let Some(stt) = self.stt.clone() else { return };
        if self.busy {
            self.send(Server::failed("still working on the last instruction"));
            return;
        }
        self.busy = true;
        let prompt = vocabulary(&self.doc(), &self.selected()).join(", ");
        let wav = self.journal.audio(&samples);
        let back = self.back.clone();
        tokio::task::spawn_blocking(move || {
            let result = stt
                .lock()
                .expect("the transcriber lock")
                .transcribe(&samples, &prompt)
                .map_err(|e| e.to_string());
            let _ = back.blocking_send(Cmd::Heard { result, wav });
        });
    }

    fn propose(&mut self, utterance: String) {
        if self.busy {
            self.send(Server::failed("still working on the last instruction"));
            return;
        }
        if let Some(pending) = self.pending.take() {
            let _ = self.port.reject_proposal(s::RejectProposal {
                proposal_id: proposal(&pending),
            });
        }
        self.busy = true;
        let target = self.selected();
        self.send(wire::thinking(&target.to_string()));
        let doc = self.doc();
        let proposer = self.proposer.clone();
        let fields = self.fields.clone();
        let back = self.back.clone();
        tokio::task::spawn_blocking(move || {
            let started = std::time::Instant::now();
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
                target,
                utterance,
                result,
                ms,
            });
        });
    }

    /// Records the agent's patch as a proposal and shows it.
    fn record(&mut self, target: &NodePath, utterance: String, patch: Patch) {
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
                ));
                return;
            }
            Err(e) => {
                self.send(Server::failed(e.to_string()));
                return;
            }
        };
        let before = self.doc();
        let Ok((after, _)) = admit(&before, &patch) else {
            self.send(Server::failed("the admitted patch no longer applies"));
            return;
        };
        let findings = self.findings(&after);
        let changed = patch.changed_path();
        self.pending = Some(proposal_id.0.0.clone());
        let _ = target;
        self.send(Server::Proposal(uilab_wire::UilabWireProposalShown {
            after: yaml_at(&after, &changed).unwrap_or_default(),
            before: yaml_at(&before, &changed).unwrap_or_default(),
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
