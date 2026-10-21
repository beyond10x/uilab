// generated from uilab v1
// model digest d3dac30e4a3114e008b9d96d1e4eba874d61954f6a5319044185b6c608753f13
// contract digest 9e9941e5243af0824123a784b98dadddce493ba5c713cb7b55ce33bfaf77efa0
// do not edit: regenerate with `ess synthesize`

//! Ephemeral stores for the generated network entry points; no durability.

/// Ephemeral storage of `uilab.session.Document`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryDocumentStorage(std::rc::Rc<std::cell::RefCell<Vec<uilab_types::session::DocumentSnapshot>>>);
impl uilab_types::behaviour::DocumentStorage for MemoryDocumentStorage {
fn get(&self, identity: &uilab_types::session::DocumentId) -> Option<uilab_types::session::DocumentSnapshot> { let key = memory_key_1(identity); self.0.borrow().iter().find(|row| memory_key_1(&row.data.document_id) == key).cloned() }
fn put(&mut self, snapshot: uilab_types::session::DocumentSnapshot) { self.delete(&snapshot.data.document_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_1(&row.data.document_id).cmp(&memory_key_1(&other.data.document_id))); }
fn delete(&mut self, identity: &uilab_types::session::DocumentId) { let key = memory_key_1(identity); self.0.borrow_mut().retain(|row| memory_key_1(&row.data.document_id) != key); }
fn list(&self) -> Vec<uilab_types::session::DocumentSnapshot> { self.0.borrow().clone() }
}
impl uilab_types::behaviour::DocumentStorage for MemoryPorts {
fn get(&self, identity: &uilab_types::session::DocumentId) -> Option<uilab_types::session::DocumentSnapshot> { uilab_types::behaviour::DocumentStorage::get(&self.document_storage, identity) }
fn put(&mut self, snapshot: uilab_types::session::DocumentSnapshot) { uilab_types::behaviour::DocumentStorage::put(&mut self.document_storage, snapshot); }
fn delete(&mut self, identity: &uilab_types::session::DocumentId) { uilab_types::behaviour::DocumentStorage::delete(&mut self.document_storage, identity); }
fn list(&self) -> Vec<uilab_types::session::DocumentSnapshot> { uilab_types::behaviour::DocumentStorage::list(&self.document_storage) }
}

/// Ephemeral storage of `uilab.session.Proposal`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryProposalStorage(std::rc::Rc<std::cell::RefCell<Vec<uilab_types::session::ProposalSnapshot>>>);
impl uilab_types::behaviour::ProposalStorage for MemoryProposalStorage {
fn get(&self, identity: &uilab_types::session::ProposalId) -> Option<uilab_types::session::ProposalSnapshot> { let key = memory_key_6(identity); self.0.borrow().iter().find(|row| memory_key_6(&row.data.proposal_id) == key).cloned() }
fn put(&mut self, snapshot: uilab_types::session::ProposalSnapshot) { self.delete(&snapshot.data.proposal_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_6(&row.data.proposal_id).cmp(&memory_key_6(&other.data.proposal_id))); }
fn delete(&mut self, identity: &uilab_types::session::ProposalId) { let key = memory_key_6(identity); self.0.borrow_mut().retain(|row| memory_key_6(&row.data.proposal_id) != key); }
fn list(&self) -> Vec<uilab_types::session::ProposalSnapshot> { self.0.borrow().clone() }
}
impl uilab_types::behaviour::ProposalStorage for MemoryPorts {
fn get(&self, identity: &uilab_types::session::ProposalId) -> Option<uilab_types::session::ProposalSnapshot> { uilab_types::behaviour::ProposalStorage::get(&self.proposal_storage, identity) }
fn put(&mut self, snapshot: uilab_types::session::ProposalSnapshot) { uilab_types::behaviour::ProposalStorage::put(&mut self.proposal_storage, snapshot); }
fn delete(&mut self, identity: &uilab_types::session::ProposalId) { uilab_types::behaviour::ProposalStorage::delete(&mut self.proposal_storage, identity); }
fn list(&self) -> Vec<uilab_types::session::ProposalSnapshot> { uilab_types::behaviour::ProposalStorage::list(&self.proposal_storage) }
}

/// Ports shared by generated components. Values disappear when the process exits.
#[derive(Clone, Default)]
pub struct MemoryPorts {
/// Storage of `uilab.session.Document`.
pub document_storage: MemoryDocumentStorage,
/// Storage of `uilab.session.Proposal`.
pub proposal_storage: MemoryProposalStorage,
}

fn memory_key_1(value: &uilab_types::session::DocumentId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).0.clone()) }

fn memory_key_6(value: &uilab_types::session::ProposalId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).0.clone()) }

// A common structural order; individual specifications use only a subset of the variants.
#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MemoryKey {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(String),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<MemoryKey>),
    Map(Vec<(MemoryKey, MemoryKey)>),
    Optional(Option<Box<MemoryKey>>),
}

impl uilab_types::session::obligations::AcceptProposalBehavior for MemoryPorts {
fn accept_proposal(&mut self, input: uilab_types::session::AcceptProposal) -> Result<uilab_types::session::AcceptProposalOutcome, uilab_types::obligation::UnmetObligation> { uilab_types::session::obligations::AcceptProposalBehavior::accept_proposal(&mut uilab_types::session::obligations::Unimplemented, input) }
}
impl uilab_types::session::obligations::OpenDocumentBehavior for MemoryPorts {
fn open_document(&mut self, input: uilab_types::session::OpenDocument) -> Result<uilab_types::session::OpenDocumentOutcome, uilab_types::obligation::UnmetObligation> { uilab_types::session::obligations::OpenDocumentBehavior::open_document(&mut uilab_types::session::obligations::Unimplemented, input) }
}
impl uilab_types::session::obligations::ProposePatchBehavior for MemoryPorts {
fn propose_patch(&mut self, input: uilab_types::session::ProposePatch) -> Result<uilab_types::session::ProposePatchOutcome, uilab_types::obligation::UnmetObligation> { uilab_types::session::obligations::ProposePatchBehavior::propose_patch(&mut uilab_types::session::obligations::Unimplemented, input) }
}
impl uilab_types::session::obligations::SelectNodeBehavior for MemoryPorts {
fn select_node(&mut self, input: uilab_types::session::SelectNode) -> Result<uilab_types::session::SelectNodeOutcome, uilab_types::obligation::UnmetObligation> { uilab_types::session::obligations::SelectNodeBehavior::select_node(&mut uilab_types::session::obligations::Unimplemented, input) }
}
impl uilab_types::session::obligations::UndoProposalBehavior for MemoryPorts {
fn undo_proposal(&mut self, input: uilab_types::session::UndoProposal) -> Result<uilab_types::session::UndoProposalOutcome, uilab_types::obligation::UnmetObligation> { uilab_types::session::obligations::UndoProposalBehavior::undo_proposal(&mut uilab_types::session::obligations::Unimplemented, input) }
}
