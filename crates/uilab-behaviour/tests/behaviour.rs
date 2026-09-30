//! The real, unforced path against the lending-library example.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use uilab_behaviour::{
    Behaviour, DOCUMENT_CHANGED, DOCUMENT_NOT_OPEN, FileSource, Handle, MemorySource,
    body_from_json,
};
use uilab_doc::{Document, resolve};
use uilab_session::{PublishedEvent, UilabSession};
use uilab_types::primitives::Uuid;
use uilab_types::session::{self as s, ProposalState};

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn library_text() -> String {
    std::fs::read_to_string(examples().join("library/library.ui.yaml")).unwrap()
}

fn library() -> Document {
    Document::from_yaml(&library_text()).unwrap()
}

/// A fresh copy of the library document on disk, in a directory of its own.
fn library_file() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("uilab-behaviour-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("library.ui.yaml");
    std::fs::write(&file, library_text()).unwrap();
    file
}

const FILE: &str = "library.ui.yaml";

/// Every obligation the port needs, in one bound.
trait All:
    s::obligations::OpenDocumentBehavior
    + s::obligations::SelectNodeBehavior
    + s::obligations::ProposePatchBehavior
    + s::obligations::AcceptProposalBehavior
    + s::obligations::RejectProposalBehavior
    + s::obligations::UndoProposalBehavior
    + s::obligations::DocumentsQuery
    + s::obligations::PendingQuery
    + s::obligations::ProposalsQuery
{
}

impl<T> All for T where
    T: s::obligations::OpenDocumentBehavior
        + s::obligations::SelectNodeBehavior
        + s::obligations::ProposePatchBehavior
        + s::obligations::AcceptProposalBehavior
        + s::obligations::RejectProposalBehavior
        + s::obligations::UndoProposalBehavior
        + s::obligations::DocumentsQuery
        + s::obligations::PendingQuery
        + s::obligations::ProposalsQuery
{
}

fn memory() -> UilabSession<Handle<MemorySource>> {
    UilabSession::new(Handle::new(Behaviour::new(
        MemorySource::new().with_file(FILE, library()),
    )))
}

fn open<B: All>(session: &mut UilabSession<B>, path: &str) -> s::DocumentId {
    match session
        .open_document(s::OpenDocument { path: path.into() })
        .unwrap()
    {
        s::OpenDocumentOutcome::Opened { document_opened } => document_opened.document_id,
        other => panic!("not opened: {other:?}"),
    }
}

fn insert(document_id: &s::DocumentId, target: &str, child: Value) -> s::ProposePatch {
    s::ProposePatch {
        document_id: document_id.clone(),
        target: s::NodePath(target.into()),
        op: s::PatchOp::Insert,
        utterance: "add a section".into(),
        body: Some(body_from_json(&child)),
    }
}

fn section(name: &str) -> Value {
    json!({"layer": "section", "name": name, "node": {"component": "record", "title": "Alerts"}})
}

macro_rules! proposed {
    ($outcome:expr) => {
        match $outcome.unwrap() {
            s::ProposePatchOutcome::Proposed { patch_proposed } => patch_proposed.proposal_id,
            other => panic!("not proposed: {other:?}"),
        }
    };
}

fn accept(id: &s::ProposalId) -> s::AcceptProposal {
    s::AcceptProposal {
        proposal_id: id.clone(),
    }
}

fn undo(id: &s::ProposalId) -> s::UndoProposal {
    s::UndoProposal {
        proposal_id: id.clone(),
    }
}

fn state_of<B: All>(session: &UilabSession<B>, id: &s::ProposalId) -> ProposalState {
    session
        .proposals()
        .unwrap()
        .into_iter()
        .find(|p| p.proposal_id == *id)
        .unwrap()
        .state
}

#[test]
fn propose_accept_saves_the_file_and_undo_restores_it() {
    let file = library_file();
    let path = file.to_str().unwrap().to_owned();
    let handle = Handle::new(Behaviour::new(FileSource));
    let mut session = UilabSession::new(handle.clone());
    let doc = open(&mut session, &path);
    assert_eq!(handle.file(&doc).as_deref(), Some(path.as_str()));
    assert_eq!(handle.selected(&doc).unwrap().to_string(), "/");

    let proposal =
        proposed!(session.propose_patch(insert(&doc, "page:overview", section("alerts"))));
    assert_eq!(session.pending().unwrap().len(), 1);
    assert!(handle.patch(&proposal).is_some());
    assert_eq!(
        handle.document(&doc).unwrap(),
        library(),
        "a proposal does not change the document"
    );

    let outcome = session.accept_proposal(accept(&proposal)).unwrap();
    assert!(
        matches!(outcome, s::AcceptProposalOutcome::Accepted { .. }),
        "{outcome:?}"
    );
    let saved = Document::from_yaml(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert!(
        resolve(&saved, &"page:overview/section:alerts".parse().unwrap()).is_ok(),
        "the file is saved"
    );
    assert_eq!(handle.document(&doc).unwrap(), saved);
    assert_eq!(handle.last_undoable(&doc), Some(proposal.clone()));
    assert_eq!(state_of(&session, &proposal), ProposalState::Accepted);
    assert!(session.pending().unwrap().is_empty());

    let outcome = session.undo_proposal(undo(&proposal)).unwrap();
    assert!(
        matches!(outcome, s::UndoProposalOutcome::Undone { .. }),
        "{outcome:?}"
    );
    let restored = Document::from_yaml(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(restored, library(), "the file is back to what it was");
    assert_eq!(handle.document(&doc).unwrap(), library());
    assert_eq!(handle.last_undoable(&doc), None);
    assert_eq!(state_of(&session, &proposal), ProposalState::Undone);

    let events: Vec<_> = session.drain_outbox();
    assert!(matches!(
        events.as_slice(),
        [
            PublishedEvent::DocumentOpened(_),
            PublishedEvent::PatchProposed(_),
            PublishedEvent::ProposalAccepted(_),
            PublishedEvent::ProposalUndone(_),
        ]
    ));
    std::fs::remove_dir_all(file.parent().unwrap()).unwrap();
}

#[test]
fn a_patch_that_fails_a_check_is_refused_and_nothing_is_recorded() {
    let mut session = memory();
    let doc = open(&mut session, FILE);
    session.drain_outbox();
    let refused = |outcome: Result<s::ProposePatchOutcome, _>| match outcome.unwrap() {
        s::ProposePatchOutcome::Refused { error } => error.check,
        other => panic!("not refused: {other:?}"),
    };

    assert_eq!(
        refused(session.propose_patch(insert(&doc, "page:loans", section("list")))),
        "name_unique"
    );
    assert_eq!(
        refused(session.propose_patch(insert(&doc, "page:loans", json!({"body": "body"})))),
        "node_shape"
    );
    let mut no_body = insert(&doc, "page:loans", Value::Null);
    no_body.body = None;
    assert_eq!(refused(session.propose_patch(no_body)), "node_shape");
    let remove_overview = s::ProposePatch {
        document_id: doc.clone(),
        target: s::NodePath("page:overview".into()),
        op: s::PatchOp::Remove,
        utterance: "drop the overview".into(),
        body: None,
    };
    assert_eq!(
        refused(session.propose_patch(remove_overview)),
        "nav_resolves"
    );
    assert_eq!(
        refused(session.propose_patch(insert(&doc, "page:nowhere", section("x")))),
        "path_resolves"
    );
    let unknown = s::DocumentId(Uuid(uuid::Uuid::new_v4().to_string()));
    assert_eq!(
        refused(session.propose_patch(insert(&unknown, "page:loans", section("x")))),
        DOCUMENT_NOT_OPEN
    );

    assert!(session.proposals().unwrap().is_empty());
    assert!(
        session.drain_outbox().is_empty(),
        "a refusal publishes nothing"
    );
}

#[test]
fn accepting_a_proposal_another_accept_made_inapplicable_is_stale() {
    let mut session = memory();
    let doc = open(&mut session, FILE);
    let first = proposed!(session.propose_patch(insert(&doc, "page:overview", section("alerts"))));
    let second = proposed!(session.propose_patch(insert(&doc, "page:overview", section("alerts"))));

    assert!(matches!(
        session.accept_proposal(accept(&first)).unwrap(),
        s::AcceptProposalOutcome::Accepted { .. }
    ));
    match session.accept_proposal(accept(&second)).unwrap() {
        s::AcceptProposalOutcome::Stale { error } => assert_eq!(error.check, "name_unique"),
        other => panic!("not stale: {other:?}"),
    }
    assert_eq!(
        state_of(&session, &second),
        ProposalState::Proposed,
        "a stale proposal stays waiting"
    );
    assert_eq!(session.pending().unwrap().len(), 1);

    session
        .reject_proposal(s::RejectProposal {
            proposal_id: second.clone(),
        })
        .unwrap();
    match session.accept_proposal(accept(&second)).unwrap() {
        s::AcceptProposalOutcome::WrongState { error } => {
            assert_eq!(error.state, ProposalState::Rejected)
        }
        other => panic!("not wrong-state: {other:?}"),
    }
    let unknown = s::ProposalId(Uuid(uuid::Uuid::new_v4().to_string()));
    assert_eq!(
        session.accept_proposal(accept(&unknown)).unwrap(),
        s::AcceptProposalOutcome::WrongStateUnknownInstance
    );
}

#[test]
fn undo_refuses_to_lose_a_later_change_and_undoes_newest_first() {
    let source = MemorySource::new().with_file(FILE, library());
    let handle = Handle::new(Behaviour::new(source.clone()));
    let mut session = UilabSession::new(handle.clone());
    let doc = open(&mut session, FILE);
    let a = proposed!(session.propose_patch(insert(&doc, "page:overview", section("alerts"))));
    let b = proposed!(session.propose_patch(insert(&doc, "page:overview", section("warnings"))));
    session.accept_proposal(accept(&a)).unwrap();
    session.accept_proposal(accept(&b)).unwrap();
    assert_eq!(source.saves(), 2);

    match session.undo_proposal(undo(&a)).unwrap() {
        s::UndoProposalOutcome::Stale { error } => assert_eq!(error.check, DOCUMENT_CHANGED),
        other => panic!("not stale: {other:?}"),
    }
    assert_eq!(handle.last_undoable(&doc), Some(b.clone()));
    assert!(matches!(
        session.undo_proposal(undo(&b)).unwrap(),
        s::UndoProposalOutcome::Undone { .. }
    ));
    assert_eq!(handle.last_undoable(&doc), Some(a.clone()));
    assert!(matches!(
        session.undo_proposal(undo(&a)).unwrap(),
        s::UndoProposalOutcome::Undone { .. }
    ));
    assert_eq!(source.file(FILE).unwrap(), library());
    assert_eq!(source.saves(), 4);

    match session.undo_proposal(undo(&a)).unwrap() {
        s::UndoProposalOutcome::WrongState { error } => {
            assert_eq!(error.state, ProposalState::Undone)
        }
        other => panic!("not wrong-state: {other:?}"),
    }
}

#[test]
fn select_resolves_the_path_in_the_open_document() {
    let handle = Handle::new(Behaviour::new(
        MemorySource::new().with_file(FILE, library()),
    ));
    let mut session = UilabSession::new(handle.clone());
    let doc = open(&mut session, FILE);
    let select = |path: &str| s::SelectNode {
        document_id: doc.clone(),
        path: s::NodePath(path.into()),
    };

    assert!(matches!(
        session
            .select_node(select("page:loans/section:list"))
            .unwrap(),
        s::SelectNodeOutcome::Selected { .. }
    ));
    assert_eq!(
        handle.selected(&doc).unwrap().to_string(),
        "page:loans/section:list"
    );
    assert!(matches!(
        session
            .select_node(select("page:loans/section:nope"))
            .unwrap(),
        s::SelectNodeOutcome::NotFound { .. }
    ));
    assert!(matches!(
        session.select_node(select("not a path")).unwrap(),
        s::SelectNodeOutcome::NotFound { .. }
    ));
    assert_eq!(
        handle.selected(&doc).unwrap().to_string(),
        "page:loans/section:list",
        "unchanged"
    );
    let documents = session.documents().unwrap();
    assert_eq!(documents[0].selected.0, "page:loans/section:list");

    let other = s::SelectNode {
        document_id: s::DocumentId(Uuid("0".into())),
        path: s::NodePath("/".into()),
    };
    assert!(matches!(
        session.select_node(other).unwrap(),
        s::SelectNodeOutcome::UnknownDocument { .. }
    ));
}

#[test]
fn a_file_that_is_not_a_document_is_unreadable() {
    let mut session = UilabSession::new(Behaviour::new(FileSource));
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no-such-file.ui.yaml");
    let outcome = session
        .open_document(s::OpenDocument {
            path: missing.to_str().unwrap().into(),
        })
        .unwrap();
    assert!(
        matches!(outcome, s::OpenDocumentOutcome::Unreadable { .. }),
        "{outcome:?}"
    );

    let mut session = memory();
    let outcome = session
        .open_document(s::OpenDocument {
            path: "elsewhere.ui.yaml".into(),
        })
        .unwrap();
    assert!(
        matches!(outcome, s::OpenDocumentOutcome::Unreadable { .. }),
        "{outcome:?}"
    );
    assert!(session.documents().unwrap().is_empty());
    assert!(session.drain_outbox().is_empty());
}
