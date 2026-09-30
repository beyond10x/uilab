//! The behaviour behind the generated `uilab-session` port.
//!
//! [`Behaviour`] satisfies the nine obligations of `uilab_types::session::obligations` — the six
//! command behaviours and the three view queries — over documents read and saved through a
//! [`DocumentSource`]. Every patch goes through [`uilab_doc::admit`], both when it is proposed and
//! again when it is accepted, against the document as it is at that moment.
//!
//! [`Handle`] shares one `Behaviour` between the port, which owns what it is given, and a server
//! that needs to read the document, the selection and the stored patches beside it.

#![warn(missing_docs)]

mod externals;
mod json;
mod source;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use indexmap::IndexMap;
use uilab_doc::{Child, Document, NodePath, Patch, Refusal, admit, resolve};
use uilab_types::obligation::UnmetObligation;
use uilab_types::primitives::Uuid;
use uilab_types::session::{self as s, obligations};

pub use externals::{Externals, Forced};
pub use json::{body_from_json, body_to_json};
pub use source::{DocumentSource, FileSource, MemorySource};

use externals::Command;

/// The `check` of a patch refused because its document is not open.
pub const DOCUMENT_NOT_OPEN: &str = "document_not_open";
/// The `check` of a patch refused because its body is missing or is not the node the op needs.
pub const NODE_SHAPE: &str = "node_shape";
/// The `check` of an undo refused because the document changed after the proposal was accepted.
pub const DOCUMENT_CHANGED: &str = "document_changed";
/// The `check` of an accept or undo refused because the document could not be saved.
pub const DOCUMENT_SAVED: &str = "document_saved";
/// The `check` of an outcome forced through [`Externals`].
pub const FORCED: &str = "forced";

struct DocumentRecord {
    data: s::DocumentData,
    doc: Document,
}

struct Applied {
    before: Document,
    after: Document,
    order: u64,
}

struct ProposalRecord {
    snapshot: s::ProposalSnapshot,
    /// `None` only for a proposal let through by [`Externals::admit_everything`].
    patch: Option<Patch>,
    /// Set when the proposal is accepted.
    applied: Option<Applied>,
}

/// The session's behaviour: open documents, their selections and every proposal made on them.
pub struct Behaviour<S> {
    source: S,
    externals: Option<Externals>,
    documents: IndexMap<String, DocumentRecord>,
    proposals: IndexMap<String, ProposalRecord>,
    accepts: u64,
}

fn fresh() -> Uuid {
    Uuid(uuid::Uuid::new_v4().to_string())
}

fn key(id: &Uuid) -> &str {
    &id.0
}

fn refused(check: &str) -> s::PatchRefused {
    s::PatchRefused {
        check: check.to_owned(),
    }
}

fn refusal(check: &str, message: impl Into<String>) -> Refusal {
    Refusal {
        check: check.to_owned(),
        message: message.into(),
    }
}

/// The `uilab_doc` patch a `ProposePatch` input describes.
///
/// Insert takes a [`Child`] as its body, Replace the replacement node, Remove no body; a JSON
/// `null` body counts as none.
pub fn build_patch(
    target: &s::NodePath,
    op: s::PatchOp,
    body: Option<&s::PatchBody>,
) -> Result<Patch, Refusal> {
    let target: NodePath = target
        .0
        .parse()
        .map_err(|e: uilab_doc::PathError| refusal("path_resolves", e.to_string()))?;
    let body = body.map(body_to_json).filter(|b| !b.is_null());
    match op {
        s::PatchOp::Insert => {
            let body =
                body.ok_or_else(|| refusal(NODE_SHAPE, "an insert carries the new child"))?;
            let child: Child = serde_json::from_value(body)
                .map_err(|e| refusal(NODE_SHAPE, format!("not a child: {e}")))?;
            Ok(Patch::Insert { target, child })
        }
        s::PatchOp::Replace => {
            let node = body.ok_or_else(|| refusal(NODE_SHAPE, "a replace carries the new node"))?;
            Ok(Patch::Replace { target, node })
        }
        s::PatchOp::Remove => match body {
            Some(_) => Err(refusal(NODE_SHAPE, "a remove carries no body")),
            None => Ok(Patch::Remove { target }),
        },
        s::PatchOp::Batch => {
            let body = body.ok_or_else(|| refusal(NODE_SHAPE, "a batch carries its patches"))?;
            let patches: Vec<Patch> = serde_json::from_value(body["patches"].clone())
                .map_err(|e| refusal(NODE_SHAPE, format!("not a list of patches: {e}")))?;
            Ok(Patch::Batch { target, patches })
        }
    }
}

impl<S: DocumentSource> Behaviour<S> {
    /// A session with nothing open, reading and saving through `source`.
    pub fn new(source: S) -> Self {
        Self {
            source,
            externals: None,
            documents: IndexMap::new(),
            proposals: IndexMap::new(),
            accepts: 0,
        }
    }

    /// A session whose externally decided outcomes can be forced through `externals`.
    ///
    /// For conformance runs only; see [`Externals`].
    pub fn with_externals(source: S, externals: Externals) -> Self {
        Self {
            externals: Some(externals),
            ..Self::new(source)
        }
    }

    /// The source documents are read from and saved to.
    pub fn source(&self) -> &S {
        &self.source
    }

    /// The open document with this id, as it is now.
    pub fn document(&self, id: &s::DocumentId) -> Option<&Document> {
        self.documents.get(key(&id.0)).map(|r| &r.doc)
    }

    /// The path the document with this id was opened from and is saved to.
    pub fn file(&self, id: &s::DocumentId) -> Option<&str> {
        self.documents.get(key(&id.0)).map(|r| r.data.path.as_str())
    }

    /// The selected node of the document with this id; `None` also when the selection does not
    /// parse as a node path, which only [`Externals::admit_everything`] allows.
    pub fn selected(&self, id: &s::DocumentId) -> Option<NodePath> {
        self.documents
            .get(key(&id.0))
            .and_then(|r| r.data.selected.0.parse().ok())
    }

    /// The patch this proposal carries.
    pub fn patch(&self, id: &s::ProposalId) -> Option<&Patch> {
        self.proposals
            .get(key(&id.0))
            .and_then(|r| r.patch.as_ref())
    }

    /// The most recently accepted proposal on this document that undo would not refuse as stale:
    /// the document is still exactly what accepting it produced.
    pub fn last_undoable(&self, id: &s::DocumentId) -> Option<s::ProposalId> {
        let current = &self.documents.get(key(&id.0))?.doc;
        self.proposals
            .values()
            .filter(|r| {
                r.snapshot.state == s::ProposalState::Accepted && r.snapshot.data.document_id == *id
            })
            .filter_map(|r| r.applied.as_ref().map(|a| (a, r)))
            .filter(|(a, _)| a.after == *current)
            .max_by_key(|(a, _)| a.order)
            .map(|(_, r)| r.snapshot.data.proposal_id.clone())
    }

    fn forced(&self, command: Command) -> Option<Forced> {
        self.externals.as_ref().and_then(|e| e.take(command))
    }

    fn admits_everything(&self) -> bool {
        self.externals
            .as_ref()
            .is_some_and(Externals::admits_everything)
    }
}

impl<S: DocumentSource> obligations::OpenDocumentBehavior for Behaviour<S> {
    fn open_document(
        &mut self,
        input: s::OpenDocument,
    ) -> Result<s::OpenDocumentOutcome, UnmetObligation> {
        let unreadable = || s::OpenDocumentOutcome::Unreadable {
            error: s::DocumentUnreadable {
                path: input.path.clone(),
            },
        };
        if self.forced(Command::OpenDocument).is_some() {
            return Ok(unreadable());
        }
        let Ok(doc) = self.source.load(&input.path) else {
            return Ok(unreadable());
        };
        let document_id = s::DocumentId(fresh());
        let data = s::DocumentData {
            document_id: document_id.clone(),
            path: input.path.clone(),
            selected: s::NodePath(NodePath::root().to_string()),
        };
        let data = s::Document::new(data).into_data();
        self.documents
            .insert(document_id.0.0.clone(), DocumentRecord { data, doc });
        Ok(s::OpenDocumentOutcome::Opened {
            document_opened: s::DocumentOpened {
                document_id,
                path: input.path,
            },
        })
    }
}

impl<S: DocumentSource> obligations::SelectNodeBehavior for Behaviour<S> {
    fn select_node(
        &mut self,
        input: s::SelectNode,
    ) -> Result<s::SelectNodeOutcome, UnmetObligation> {
        let unknown = |input: s::SelectNode| s::SelectNodeOutcome::UnknownDocument {
            error: s::DocumentNotOpen {
                document_id: input.document_id,
            },
        };
        let not_found = |input: s::SelectNode| s::SelectNodeOutcome::NotFound {
            error: s::NodeNotFound { path: input.path },
        };
        match self.forced(Command::SelectNode) {
            Some(Forced::SelectNodeUnknownDocument) => return Ok(unknown(input)),
            Some(Forced::SelectNodeNotFound) => return Ok(not_found(input)),
            _ => {}
        }
        let admit_everything = self.admits_everything();
        let Some(record) = self.documents.get_mut(key(&input.document_id.0)) else {
            return Ok(unknown(input));
        };
        let resolves = input
            .path
            .0
            .parse::<NodePath>()
            .is_ok_and(|p| resolve(&record.doc, &p).is_ok());
        if !resolves && !admit_everything {
            return Ok(not_found(input));
        }
        record.data.selected = input.path.clone();
        Ok(s::SelectNodeOutcome::Selected {
            node_selected: s::NodeSelected {
                document_id: input.document_id,
                path: input.path,
            },
        })
    }
}

impl<S: DocumentSource> obligations::ProposePatchBehavior for Behaviour<S> {
    fn propose_patch(
        &mut self,
        input: s::ProposePatch,
    ) -> Result<s::ProposePatchOutcome, UnmetObligation> {
        let refuse = |check: &str| s::ProposePatchOutcome::Refused {
            error: refused(check),
        };
        if self.forced(Command::ProposePatch).is_some() {
            return Ok(refuse(FORCED));
        }
        let Some(record) = self.documents.get(key(&input.document_id.0)) else {
            return Ok(refuse(DOCUMENT_NOT_OPEN));
        };
        let admitted = build_patch(&input.target, input.op, input.body.as_ref())
            .and_then(|patch| admit(&record.doc, &patch).map(|_| patch));
        let patch = match admitted {
            Ok(patch) => Some(patch),
            Err(_) if self.admits_everything() => None,
            Err(refusal) => return Ok(refuse(&refusal.check)),
        };
        let proposal_id = s::ProposalId(fresh());
        let data = s::ProposalData {
            proposal_id: proposal_id.clone(),
            document_id: input.document_id.clone(),
            target: input.target.clone(),
            op: input.op,
            utterance: input.utterance,
        };
        let snapshot = s::AnyProposal::Proposed(s::Proposal::new(data)).snapshot();
        self.proposals.insert(
            proposal_id.0.0.clone(),
            ProposalRecord {
                snapshot,
                patch,
                applied: None,
            },
        );
        Ok(s::ProposePatchOutcome::Proposed {
            patch_proposed: s::PatchProposed {
                proposal_id,
                document_id: input.document_id,
                target: input.target,
                op: input.op,
            },
        })
    }
}

impl<S: DocumentSource> obligations::AcceptProposalBehavior for Behaviour<S> {
    fn accept_proposal(
        &mut self,
        input: s::AcceptProposal,
    ) -> Result<s::AcceptProposalOutcome, UnmetObligation> {
        let stale = |check: &str| s::AcceptProposalOutcome::Stale {
            error: refused(check),
        };
        if self.forced(Command::AcceptProposal).is_some() {
            return Ok(stale(FORCED));
        }
        let admit_everything = self.admits_everything();
        let Some(record) = self.proposals.get(key(&input.proposal_id.0)) else {
            return Ok(s::AcceptProposalOutcome::WrongStateUnknownInstance);
        };
        let proposed = match record.snapshot.clone().refine() {
            s::AnyProposal::Proposed(p) => p,
            other => {
                return Ok(s::AcceptProposalOutcome::WrongState {
                    error: s::ProposalStateConflict {
                        state: other.state(),
                    },
                });
            }
        };
        let doc_key = record.snapshot.data.document_id.0.0.clone();
        let Some(document) = self.documents.get(&doc_key) else {
            return Ok(stale(DOCUMENT_NOT_OPEN));
        };
        let before = document.doc.clone();
        let after = match record.patch.as_ref().map(|p| admit(&before, p)) {
            Some(Ok((next, _))) => next,
            Some(Err(_)) | None if admit_everything => before.clone(),
            Some(Err(refusal)) => return Ok(stale(&refusal.check)),
            None => return Ok(stale(NODE_SHAPE)),
        };
        if self.source.save(&document.data.path, &after).is_err() {
            return Ok(stale(DOCUMENT_SAVED));
        }
        self.accepts += 1;
        let order = self.accepts;
        if let Some(document) = self.documents.get_mut(&doc_key) {
            document.doc = after.clone();
        }
        if let Some(record) = self.proposals.get_mut(key(&input.proposal_id.0)) {
            record.snapshot = s::AnyProposal::Accepted(proposed.accept()).snapshot();
            record.applied = Some(Applied {
                before,
                after,
                order,
            });
        }
        Ok(s::AcceptProposalOutcome::Accepted {
            proposal_accepted: s::ProposalAccepted {
                proposal_id: input.proposal_id,
            },
        })
    }
}

impl<S: DocumentSource> obligations::RejectProposalBehavior for Behaviour<S> {
    fn reject_proposal(
        &mut self,
        input: s::RejectProposal,
    ) -> Result<s::RejectProposalOutcome, UnmetObligation> {
        let Some(record) = self.proposals.get_mut(key(&input.proposal_id.0)) else {
            return Ok(s::RejectProposalOutcome::WrongStateUnknownInstance);
        };
        match record.snapshot.clone().refine() {
            s::AnyProposal::Proposed(p) => {
                record.snapshot = s::AnyProposal::Rejected(p.reject()).snapshot();
                Ok(s::RejectProposalOutcome::Rejected {
                    proposal_rejected: s::ProposalRejected {
                        proposal_id: input.proposal_id,
                    },
                })
            }
            other => Ok(s::RejectProposalOutcome::WrongState {
                error: s::ProposalStateConflict {
                    state: other.state(),
                },
            }),
        }
    }
}

impl<S: DocumentSource> obligations::UndoProposalBehavior for Behaviour<S> {
    fn undo_proposal(
        &mut self,
        input: s::UndoProposal,
    ) -> Result<s::UndoProposalOutcome, UnmetObligation> {
        let stale = |check: &str| s::UndoProposalOutcome::Stale {
            error: refused(check),
        };
        if self.forced(Command::UndoProposal).is_some() {
            return Ok(stale(FORCED));
        }
        let Some(record) = self.proposals.get(key(&input.proposal_id.0)) else {
            return Ok(s::UndoProposalOutcome::WrongStateUnknownInstance);
        };
        let accepted = match record.snapshot.clone().refine() {
            s::AnyProposal::Accepted(p) => p,
            other => {
                return Ok(s::UndoProposalOutcome::WrongState {
                    error: s::ProposalStateConflict {
                        state: other.state(),
                    },
                });
            }
        };
        let Some(applied) = record.applied.as_ref() else {
            return Ok(stale(DOCUMENT_CHANGED));
        };
        let doc_key = record.snapshot.data.document_id.0.0.clone();
        let Some(document) = self.documents.get(&doc_key) else {
            return Ok(stale(DOCUMENT_NOT_OPEN));
        };
        if document.doc != applied.after {
            return Ok(stale(DOCUMENT_CHANGED));
        }
        let before = applied.before.clone();
        if self.source.save(&document.data.path, &before).is_err() {
            return Ok(stale(DOCUMENT_SAVED));
        }
        if let Some(document) = self.documents.get_mut(&doc_key) {
            document.doc = before;
        }
        if let Some(record) = self.proposals.get_mut(key(&input.proposal_id.0)) {
            record.snapshot = s::AnyProposal::Undone(accepted.undo()).snapshot();
        }
        Ok(s::UndoProposalOutcome::Undone {
            proposal_undone: s::ProposalUndone {
                proposal_id: input.proposal_id,
            },
        })
    }
}

impl<S> obligations::DocumentsQuery for Behaviour<S> {
    fn documents(&self) -> Result<Vec<s::Documents>, UnmetObligation> {
        Ok(self
            .documents
            .values()
            .map(|r| s::Documents {
                document_id: r.data.document_id.clone(),
                path: r.data.path.clone(),
                selected: r.data.selected.clone(),
            })
            .collect())
    }
}

impl<S> obligations::PendingQuery for Behaviour<S> {
    fn pending(&self) -> Result<Vec<s::Pending>, UnmetObligation> {
        Ok(self
            .proposals
            .values()
            .filter(|r| r.snapshot.state == s::ProposalState::Proposed)
            .map(|r| s::Pending {
                proposal_id: r.snapshot.data.proposal_id.clone(),
                target: r.snapshot.data.target.clone(),
                op: r.snapshot.data.op,
            })
            .collect())
    }
}

impl<S> obligations::ProposalsQuery for Behaviour<S> {
    fn proposals(&self) -> Result<Vec<s::Proposals>, UnmetObligation> {
        Ok(self
            .proposals
            .values()
            .map(|r| {
                let d = &r.snapshot.data;
                s::Proposals {
                    proposal_id: d.proposal_id.clone(),
                    document_id: d.document_id.clone(),
                    target: d.target.clone(),
                    op: d.op,
                    utterance: d.utterance.clone(),
                    state: r.snapshot.state,
                }
            })
            .collect())
    }
}

/// One [`Behaviour`] shared between the port and whoever reads it beside the port.
///
/// The port takes a clone as its `B`; every obligation locks and delegates. Accessors return owned
/// values, since a borrow cannot outlive the lock.
pub struct Handle<S>(pub Arc<Mutex<Behaviour<S>>>);

impl<S> Clone for Handle<S> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<S: DocumentSource> Handle<S> {
    /// Shares `behaviour`.
    pub fn new(behaviour: Behaviour<S>) -> Self {
        Self(Arc::new(Mutex::new(behaviour)))
    }

    /// The behaviour, locked. A poisoned lock is taken over: every command leaves the records
    /// consistent before it can panic.
    pub fn lock(&self) -> MutexGuard<'_, Behaviour<S>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// See [`Behaviour::document`].
    pub fn document(&self, id: &s::DocumentId) -> Option<Document> {
        self.lock().document(id).cloned()
    }

    /// See [`Behaviour::file`].
    pub fn file(&self, id: &s::DocumentId) -> Option<String> {
        self.lock().file(id).map(str::to_owned)
    }

    /// See [`Behaviour::selected`].
    pub fn selected(&self, id: &s::DocumentId) -> Option<NodePath> {
        self.lock().selected(id)
    }

    /// See [`Behaviour::patch`].
    pub fn patch(&self, id: &s::ProposalId) -> Option<Patch> {
        self.lock().patch(id).cloned()
    }

    /// See [`Behaviour::last_undoable`].
    pub fn last_undoable(&self, id: &s::DocumentId) -> Option<s::ProposalId> {
        self.lock().last_undoable(id)
    }
}

macro_rules! delegate {
    ($trait:ident, $method:ident, $input:ty, $output:ty) => {
        impl<S: DocumentSource> obligations::$trait for Handle<S> {
            fn $method(&mut self, input: $input) -> Result<$output, UnmetObligation> {
                self.lock().$method(input)
            }
        }
    };
    ($trait:ident, $method:ident, $row:ty) => {
        impl<S: DocumentSource> obligations::$trait for Handle<S> {
            fn $method(&self) -> Result<Vec<$row>, UnmetObligation> {
                self.lock().$method()
            }
        }
    };
}

delegate!(
    OpenDocumentBehavior,
    open_document,
    s::OpenDocument,
    s::OpenDocumentOutcome
);
delegate!(
    SelectNodeBehavior,
    select_node,
    s::SelectNode,
    s::SelectNodeOutcome
);
delegate!(
    ProposePatchBehavior,
    propose_patch,
    s::ProposePatch,
    s::ProposePatchOutcome
);
delegate!(
    AcceptProposalBehavior,
    accept_proposal,
    s::AcceptProposal,
    s::AcceptProposalOutcome
);
delegate!(
    RejectProposalBehavior,
    reject_proposal,
    s::RejectProposal,
    s::RejectProposalOutcome
);
delegate!(
    UndoProposalBehavior,
    undo_proposal,
    s::UndoProposal,
    s::UndoProposalOutcome
);
delegate!(DocumentsQuery, documents, s::Documents);
delegate!(PendingQuery, pending, s::Pending);
delegate!(ProposalsQuery, proposals, s::Proposals);

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn a_handle_over_either_source_can_move_to_another_task() {
        assert_send_sync::<Handle<FileSource>>();
        assert_send_sync::<Handle<MemorySource>>();
    }

    #[test]
    fn build_patch_needs_the_body_its_op_takes() {
        let target = s::NodePath("page:loans".into());
        let check = |op, body: Option<serde_json::Value>| {
            build_patch(&target, op, body.as_ref().map(body_from_json).as_ref())
                .map_err(|r| r.check)
        };
        assert_eq!(check(s::PatchOp::Insert, None), Err(NODE_SHAPE.to_owned()));
        assert_eq!(
            check(
                s::PatchOp::Insert,
                Some(serde_json::json!({"body": "body"}))
            ),
            Err(NODE_SHAPE.to_owned())
        );
        assert_eq!(
            check(s::PatchOp::Replace, Some(serde_json::Value::Null)),
            Err(NODE_SHAPE.to_owned())
        );
        assert_eq!(
            check(s::PatchOp::Remove, Some(serde_json::json!({}))),
            Err(NODE_SHAPE.to_owned())
        );
        assert!(check(s::PatchOp::Remove, None).is_ok());
        assert!(
            check(
                s::PatchOp::Insert,
                Some(serde_json::json!({"layer": "section", "name": "extra", "node": {"component": "record"}}))
            )
            .is_ok()
        );
        assert_eq!(
            build_patch(&s::NodePath("nowhere".into()), s::PatchOp::Remove, None)
                .map_err(|r| r.check),
            Err("path_resolves".to_owned())
        );
    }
}
