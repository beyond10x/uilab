// generated from uilab v1
// model digest 3919335fd597a821e3c47c4dc4c8da6be9f962bf7f0876feaaf144e96873fc8e
// contract digest 797f8eeba742036695e9acbd13349f58b85ac02ab93c0a62197940b5d1883646
// do not edit: regenerate with `ess synthesize`

//! The `uilab-session` component of `uilab` v1, on the wire.
//!
//! The specification says this component's callers are not deployed with it, so its surface
//! exists on a wire. Which wire is derived rather than chosen: the one contract this model
//! projects for a command surface is the `OpenAPI` document, and an `OpenAPI` document is an
//! HTTP contract. The document is beside this file, served verbatim at `/openapi.json`.

use crate::{http, json, wire};

/// The contract this surface answers, byte for byte as `generated/` commits it.
///
/// Embedded rather than rebuilt at run time: a server that regenerated its own contract could
/// publish one the repository never reviewed.
pub const OPENAPI: &str = include_str!("uilab-session.openapi.json");

/// The prose the same model produced, byte for byte as the documentation projection wrote it.
pub const DOCS: &str = include_str!("uilab-session.docs.md");

/// Every route this surface answers, in path order.
///
/// The same set the `OpenAPI` document declares, plus the two documents about the surface
/// itself, which no specification construct names and nothing can therefore derive. A path
/// absent from this table is answered with `404`, including one the document declares and this
/// table forgot — which is the failure a table computed twice would hide.
pub const ROUTES: &[(&str, &str)] = &[
    ("GET", "/docs"),
    ("GET", "/openapi.json"),
    ("POST", "/session/commands/accept-proposal"),
    ("POST", "/session/commands/open-document"),
    ("POST", "/session/commands/propose-patch"),
    ("POST", "/session/commands/reject-proposal"),
    ("POST", "/session/commands/select-node"),
    ("POST", "/session/commands/undo-proposal"),
    ("GET", "/session/views/documents"),
    ("GET", "/session/views/pending"),
    ("GET", "/session/views/proposals"),
];

/// What this process says about itself as it starts, before it answers anything.
///
/// Three lines of JSON on standard output, in this order, every member of them derived from the
/// specification — except `runtime`, which is appended by the emitted code below and holds what
/// is true of *this process*: the language it was synthesised into, and the address it bound.
/// Everything outside `runtime` is the same in every language this plan is emitted into, and
/// `cargo xtask synth --check` starts both and compares them.
pub const STARTUP: &[&str] = &[
    "{\"log\":\"ess/1\",\"event\":\"system.starting\",\"system\":\"uilab\",\"version\":\"v1\",\"model_digest\":\"3919335fd597a821e3c47c4dc4c8da6be9f962bf7f0876feaaf144e96873fc8e\",\"contract_digest\":\"797f8eeba742036695e9acbd13349f58b85ac02ab93c0a62197940b5d1883646\",\"components\":[\"uilab-session\"],\"capabilities\":{\"generated\":56,\"obligations\":9,\"refused\":2}",
    "{\"log\":\"ess/1\",\"event\":\"surface.serving\",\"component\":\"uilab-session\",\"reached_by\":\"network\",\"transport\":\"http/1.1\",\"routes\":11,\"paths\":[{\"method\":\"GET\",\"path\":\"/docs\",\"serves\":\"documentation\",\"name\":\"docs\"},{\"method\":\"GET\",\"path\":\"/openapi.json\",\"serves\":\"contract\",\"name\":\"openapi\"},{\"method\":\"POST\",\"path\":\"/session/commands/accept-proposal\",\"serves\":\"command\",\"name\":\"uilab.session.AcceptProposal\"},{\"method\":\"POST\",\"path\":\"/session/commands/open-document\",\"serves\":\"command\",\"name\":\"uilab.session.OpenDocument\"},{\"method\":\"POST\",\"path\":\"/session/commands/propose-patch\",\"serves\":\"command\",\"name\":\"uilab.session.ProposePatch\"},{\"method\":\"POST\",\"path\":\"/session/commands/reject-proposal\",\"serves\":\"command\",\"name\":\"uilab.session.RejectProposal\"},{\"method\":\"POST\",\"path\":\"/session/commands/select-node\",\"serves\":\"command\",\"name\":\"uilab.session.SelectNode\"},{\"method\":\"POST\",\"path\":\"/session/commands/undo-proposal\",\"serves\":\"command\",\"name\":\"uilab.session.UndoProposal\"},{\"method\":\"GET\",\"path\":\"/session/views/documents\",\"serves\":\"view\",\"name\":\"uilab.session.Documents\"},{\"method\":\"GET\",\"path\":\"/session/views/pending\",\"serves\":\"view\",\"name\":\"uilab.session.Pending\"},{\"method\":\"GET\",\"path\":\"/session/views/proposals\",\"serves\":\"view\",\"name\":\"uilab.session.Proposals\"}]",
    "{\"log\":\"ess/1\",\"event\":\"system.ready\",\"system\":\"uilab\",\"surfaces\":1",
];

/// Writes the startup record, with this process's own facts closing each line.
fn announce(address: &std::net::SocketAddr) {
    for facts in STARTUP {
        let mut line = String::from(*facts);
        line.push_str(",\"runtime\":{\"address\":");
        json::push_text(&mut line, &address.to_string());
        line.push_str(",\"language\":\"rust\",\"port\":");
        json::push_integer(&mut line, i64::from(address.port()));
        line.push_str("}}");
        println!("{line}");
    }
}

/// Serves `uilab-session` at `address`, and does not return while it can answer.
///
/// `address` may name port `0`, which binds an ephemeral port; the startup record says which one
/// was taken, because a caller that cannot learn the port cannot make a request.
///
/// It chooses no realization. Every command reaches the port, and a port over unimplemented
/// obligations answers the typed refusal this surface reports as `501` — the honest empty
/// state rather than a server that pretends.
///
/// # Errors
///
/// Anything the listener refuses: the address is taken, the port is privileged, the socket
/// died.
pub fn serve<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, address: &str) -> std::io::Result<()>
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let listener = std::net::TcpListener::bind(address)?;
    announce(&listener.local_addr()?);
    for connection in listener.incoming() {
        let mut reader = std::io::BufReader::new(connection?);
        let answer = match http::read(&mut reader) {
            Ok(request) => dispatch(system, &request),
            Err(refusal) => refusal,
        };
        let mut stream = reader.into_inner();
        http::write(&mut stream, &answer)?;
    }
    Ok(())
}

/// Answers one request.
///
/// A path this table does not hold is a `404` naming where the whole table is published; a
/// path it holds under a different method is a `405` naming the one it answers. Neither is a
/// status the contract declares, and neither should be: both are facts about a transport rather
/// than about any command.
fn dispatch<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, request: &http::Request) -> http::Response
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    match request.path.as_str() {
        "/docs" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::Response::new(200, http::MARKDOWN, DOCS)
        }
        "/openapi.json" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::Response::new(200, http::JSON, OPENAPI)
        }
        "/session/commands/accept-proposal" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            serve_uilab_session_accept_proposal(system, &request.body)
        }
        "/session/commands/open-document" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            serve_uilab_session_open_document(system, &request.body)
        }
        "/session/commands/propose-patch" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            serve_uilab_session_propose_patch(system, &request.body)
        }
        "/session/commands/reject-proposal" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            serve_uilab_session_reject_proposal(system, &request.body)
        }
        "/session/commands/select-node" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            serve_uilab_session_select_node(system, &request.body)
        }
        "/session/commands/undo-proposal" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            serve_uilab_session_undo_proposal(system, &request.body)
        }
        "/session/views/documents" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            serve_uilab_session_documents(system)
        }
        "/session/views/pending" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            serve_uilab_session_pending(system)
        }
        "/session/views/proposals" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            serve_uilab_session_proposals(system)
        }
        other => http::Response::refusal(
            404,
            &format!("`{other}` is not a path this surface declares; `GET /openapi.json` publishes every one that is"),
        ),
    }
}

/// `POST` `uilab.session.AcceptProposal`: reads the declared input, runs the port, answers the declared outcome.
fn serve_uilab_session_accept_proposal<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, body: &[u8]) -> http::Response
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    let input = match wire::decode_command_uilab_session_accept_proposal(&value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return http::Response::refusal(400, &format!("{error}"));
        }
    };
    match system.uilab_session.accept_proposal(input) {
        Ok(outcome) => answer_uilab_session_accept_proposal(&outcome),
        Err(unmet) => http::Response::refusal(501, &format!("{unmet}")),
    }
}

/// One declared outcome of `uilab.session.AcceptProposal`, as the contract publishes it: the branch that was taken,
/// the declared error where there is one, and that error's own payload.
fn answer_uilab_session_accept_proposal(outcome: &uilab_types::session::AcceptProposalOutcome) -> http::Response {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::AcceptProposalOutcome::Accepted { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "accepted");
            202
        }
        uilab_types::session::AcceptProposalOutcome::Stale { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "stale");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.PatchRefused");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_patch_refused(error, &mut body);
            502
        }
        uilab_types::session::AcceptProposalOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_proposal_state_conflict(error, &mut body);
            409
        }
        uilab_types::session::AcceptProposalOutcome::WrongStateUnknownInstance => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            409
        }
    };
    body.push('}');
    http::Response::new(status, http::JSON, body)
}

/// `POST` `uilab.session.OpenDocument`: reads the declared input, runs the port, answers the declared outcome.
fn serve_uilab_session_open_document<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, body: &[u8]) -> http::Response
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    let input = match wire::decode_command_uilab_session_open_document(&value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return http::Response::refusal(400, &format!("{error}"));
        }
    };
    match system.uilab_session.open_document(input) {
        Ok(outcome) => answer_uilab_session_open_document(&outcome),
        Err(unmet) => http::Response::refusal(501, &format!("{unmet}")),
    }
}

/// One declared outcome of `uilab.session.OpenDocument`, as the contract publishes it: the branch that was taken,
/// the declared error where there is one, and that error's own payload.
fn answer_uilab_session_open_document(outcome: &uilab_types::session::OpenDocumentOutcome) -> http::Response {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::OpenDocumentOutcome::Opened { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "opened");
            202
        }
        uilab_types::session::OpenDocumentOutcome::Unreadable { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "unreadable");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.DocumentUnreadable");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_document_unreadable(error, &mut body);
            502
        }
    };
    body.push('}');
    http::Response::new(status, http::JSON, body)
}

/// `POST` `uilab.session.ProposePatch`: reads the declared input, runs the port, answers the declared outcome.
fn serve_uilab_session_propose_patch<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, body: &[u8]) -> http::Response
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    let input = match wire::decode_command_uilab_session_propose_patch(&value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return http::Response::refusal(400, &format!("{error}"));
        }
    };
    match system.uilab_session.propose_patch(input) {
        Ok(outcome) => answer_uilab_session_propose_patch(&outcome),
        Err(unmet) => http::Response::refusal(501, &format!("{unmet}")),
    }
}

/// One declared outcome of `uilab.session.ProposePatch`, as the contract publishes it: the branch that was taken,
/// the declared error where there is one, and that error's own payload.
fn answer_uilab_session_propose_patch(outcome: &uilab_types::session::ProposePatchOutcome) -> http::Response {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::ProposePatchOutcome::Proposed { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "proposed");
            202
        }
        uilab_types::session::ProposePatchOutcome::Refused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "refused");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.PatchRefused");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_patch_refused(error, &mut body);
            502
        }
    };
    body.push('}');
    http::Response::new(status, http::JSON, body)
}

/// `POST` `uilab.session.RejectProposal`: reads the declared input, runs the port, answers the declared outcome.
fn serve_uilab_session_reject_proposal<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, body: &[u8]) -> http::Response
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    let input = match wire::decode_command_uilab_session_reject_proposal(&value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return http::Response::refusal(400, &format!("{error}"));
        }
    };
    match system.uilab_session.reject_proposal(input) {
        Ok(outcome) => answer_uilab_session_reject_proposal(&outcome),
        Err(unmet) => http::Response::refusal(501, &format!("{unmet}")),
    }
}

/// One declared outcome of `uilab.session.RejectProposal`, as the contract publishes it: the branch that was taken,
/// the declared error where there is one, and that error's own payload.
fn answer_uilab_session_reject_proposal(outcome: &uilab_types::session::RejectProposalOutcome) -> http::Response {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::RejectProposalOutcome::Rejected { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "rejected");
            202
        }
        uilab_types::session::RejectProposalOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_proposal_state_conflict(error, &mut body);
            409
        }
        uilab_types::session::RejectProposalOutcome::WrongStateUnknownInstance => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            409
        }
    };
    body.push('}');
    http::Response::new(status, http::JSON, body)
}

/// `POST` `uilab.session.SelectNode`: reads the declared input, runs the port, answers the declared outcome.
fn serve_uilab_session_select_node<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, body: &[u8]) -> http::Response
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    let input = match wire::decode_command_uilab_session_select_node(&value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return http::Response::refusal(400, &format!("{error}"));
        }
    };
    match system.uilab_session.select_node(input) {
        Ok(outcome) => answer_uilab_session_select_node(&outcome),
        Err(unmet) => http::Response::refusal(501, &format!("{unmet}")),
    }
}

/// One declared outcome of `uilab.session.SelectNode`, as the contract publishes it: the branch that was taken,
/// the declared error where there is one, and that error's own payload.
fn answer_uilab_session_select_node(outcome: &uilab_types::session::SelectNodeOutcome) -> http::Response {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::SelectNodeOutcome::Selected { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "selected");
            202
        }
        uilab_types::session::SelectNodeOutcome::UnknownDocument { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "unknown-document");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.DocumentNotOpen");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_document_not_open(error, &mut body);
            502
        }
        uilab_types::session::SelectNodeOutcome::NotFound { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.NodeNotFound");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_node_not_found(error, &mut body);
            502
        }
    };
    body.push('}');
    http::Response::new(status, http::JSON, body)
}

/// `POST` `uilab.session.UndoProposal`: reads the declared input, runs the port, answers the declared outcome.
fn serve_uilab_session_undo_proposal<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, body: &[u8]) -> http::Response
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    let input = match wire::decode_command_uilab_session_undo_proposal(&value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return http::Response::refusal(400, &format!("{error}"));
        }
    };
    match system.uilab_session.undo_proposal(input) {
        Ok(outcome) => answer_uilab_session_undo_proposal(&outcome),
        Err(unmet) => http::Response::refusal(501, &format!("{unmet}")),
    }
}

/// One declared outcome of `uilab.session.UndoProposal`, as the contract publishes it: the branch that was taken,
/// the declared error where there is one, and that error's own payload.
fn answer_uilab_session_undo_proposal(outcome: &uilab_types::session::UndoProposalOutcome) -> http::Response {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::UndoProposalOutcome::Undone { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "undone");
            202
        }
        uilab_types::session::UndoProposalOutcome::Stale { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "stale");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.PatchRefused");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_patch_refused(error, &mut body);
            502
        }
        uilab_types::session::UndoProposalOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_proposal_state_conflict(error, &mut body);
            409
        }
        uilab_types::session::UndoProposalOutcome::WrongStateUnknownInstance => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            409
        }
    };
    body.push('}');
    http::Response::new(status, http::JSON, body)
}

/// `GET` `uilab.session.Documents` at `read_your_writes` consistency: every row the owed projection holds.
fn serve_uilab_session_documents<UilabSessionBehaviors>(system: &uilab_system::System<UilabSessionBehaviors>) -> http::Response
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    match system.uilab_session.documents() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_uilab_session_documents(row, &mut body);
            }
            body.push(']');
            body.push('}');
            http::Response::new(200, http::JSON, body)
        }
        Err(unmet) => http::Response::refusal(501, &format!("{unmet}")),
    }
}

/// `GET` `uilab.session.Pending` at `read_your_writes` consistency: every row the owed projection holds.
fn serve_uilab_session_pending<UilabSessionBehaviors>(system: &uilab_system::System<UilabSessionBehaviors>) -> http::Response
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    match system.uilab_session.pending() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_uilab_session_pending(row, &mut body);
            }
            body.push(']');
            body.push('}');
            http::Response::new(200, http::JSON, body)
        }
        Err(unmet) => http::Response::refusal(501, &format!("{unmet}")),
    }
}

/// `GET` `uilab.session.Proposals` at `read_your_writes` consistency: every row the owed projection holds.
fn serve_uilab_session_proposals<UilabSessionBehaviors>(system: &uilab_system::System<UilabSessionBehaviors>) -> http::Response
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    match system.uilab_session.proposals() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_uilab_session_proposals(row, &mut body);
            }
            body.push(']');
            body.push('}');
            http::Response::new(200, http::JSON, body)
        }
        Err(unmet) => http::Response::refusal(501, &format!("{unmet}")),
    }
}
