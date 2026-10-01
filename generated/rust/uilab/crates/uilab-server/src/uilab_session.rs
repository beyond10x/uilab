// generated from uilab v1
// model digest d3dac30e4a3114e008b9d96d1e4eba874d61954f6a5319044185b6c608753f13
// contract digest 9e9941e5243af0824123a784b98dadddce493ba5c713cb7b55ce33bfaf77efa0
// do not edit: regenerate with `ess synthesize`

//! The `uilab-session` component of `uilab` v1, on the wire.
//!
//! The specification says this component's callers are not deployed with it, so its surface
//! exists on a wire. Which wire is derived rather than chosen: the one contract this model
//! projects for a command surface is the `OpenAPI` document, and an `OpenAPI` document is an
//! HTTP contract. The document is beside this file, served verbatim at `/openapi.json`.

use crate::{entry, http, json, wire};

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
    "{\"log\":\"ess/1\",\"event\":\"system.starting\",\"system\":\"uilab\",\"version\":\"v1\",\"model_digest\":\"d3dac30e4a3114e008b9d96d1e4eba874d61954f6a5319044185b6c608753f13\",\"contract_digest\":\"9e9941e5243af0824123a784b98dadddce493ba5c713cb7b55ce33bfaf77efa0\",\"components\":[\"uilab-session\"],\"capabilities\":{\"generated\":68,\"obligations\":5,\"refused\":2}",
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
///
/// Public so a caller can hand it a request it built itself: [`serve`] is this function behind a
/// socket, and nothing else.
pub fn dispatch<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, request: &http::Request) -> http::Response
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
            http::answer(run_uilab_session_documents(system))
        }
        "/session/views/pending" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_uilab_session_pending(system))
        }
        "/session/views/proposals" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_uilab_session_proposals(system))
        }
        other => http::Response::refusal(
            404,
            &format!("`{other}` is not a path this surface declares; `GET /openapi.json` publishes every one that is"),
        ),
    }
}

/// Runs one command or view of `uilab-session` by its qualified name, with no transport.
///
/// The same decoding, refusals and rendering the HTTP routes use — each route and this function call
/// one `run_*` function — so a conformance runner or an in-process caller drives the system
/// without a socket and without a dispatch table of its own. `Ok` is the declared outcome, as the
/// route's body renders it; a view ignores `input`, as its `GET` route ignores a body.
///
/// # Errors
///
/// [`entry::Refused::Unknown`] naming `name` when this surface declares no command or view
/// by it; [`entry::Refused::Input`] when `input` is not the command's declared input (the
/// route's `400`); [`entry::Refused::Unmet`] when the port reports an unmet obligation, and
/// [`entry::Refused::Undelivered`] when the command took effect and delivering what it published
/// failed (the route's `501`, with `committed` `false` and `true`).
pub fn handle<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, name: &str, input: json::Value) -> Result<json::Value, entry::Refused>
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let answered = match name {
        "uilab.session.AcceptProposal" => run_uilab_session_accept_proposal(system, &input),
        "uilab.session.Documents" => run_uilab_session_documents(system),
        "uilab.session.OpenDocument" => run_uilab_session_open_document(system, &input),
        "uilab.session.Pending" => run_uilab_session_pending(system),
        "uilab.session.Proposals" => run_uilab_session_proposals(system),
        "uilab.session.ProposePatch" => run_uilab_session_propose_patch(system, &input),
        "uilab.session.RejectProposal" => run_uilab_session_reject_proposal(system, &input),
        "uilab.session.SelectNode" => run_uilab_session_select_node(system, &input),
        "uilab.session.UndoProposal" => run_uilab_session_undo_proposal(system, &input),
        other => return Err(entry::Refused::Unknown(other.to_owned())),
    };
    let (_, body) = answered?;
    Ok(entry::read(&body))
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
    http::answer(run_uilab_session_accept_proposal(system, &value))
}

/// `uilab.session.AcceptProposal` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_uilab_session_accept_proposal<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let input = match wire::decode_command_uilab_session_accept_proposal(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.uilab_session.accept_proposal(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_uilab_session_accept_proposal(&outcome))
}

/// One declared outcome of `uilab.session.AcceptProposal`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_uilab_session_accept_proposal(outcome: &uilab_types::session::AcceptProposalOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::AcceptProposalOutcome::Accepted { proposal_accepted, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "accepted");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "uilab.session.ProposalAccepted");
            json::member(&mut body, "payload");
            wire::encode_event_uilab_session_proposal_accepted(proposal_accepted, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        uilab_types::session::AcceptProposalOutcome::Stale { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "stale");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.PatchRefused");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_patch_refused(error, &mut body);
            502
        }
        uilab_types::session::AcceptProposalOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_proposal_state_conflict(error, &mut body);
            409
        }
        uilab_types::session::AcceptProposalOutcome::WrongStateUnknownInstance => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push_str("[]");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            409
        }
    };
    body.push('}');
    (status, body)
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
    http::answer(run_uilab_session_open_document(system, &value))
}

/// `uilab.session.OpenDocument` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_uilab_session_open_document<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let input = match wire::decode_command_uilab_session_open_document(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.uilab_session.open_document(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_uilab_session_open_document(&outcome))
}

/// One declared outcome of `uilab.session.OpenDocument`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_uilab_session_open_document(outcome: &uilab_types::session::OpenDocumentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::OpenDocumentOutcome::Opened { document_opened, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "opened");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "uilab.session.DocumentOpened");
            json::member(&mut body, "payload");
            wire::encode_event_uilab_session_document_opened(document_opened, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        uilab_types::session::OpenDocumentOutcome::Unreadable { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "unreadable");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.DocumentUnreadable");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_document_unreadable(error, &mut body);
            502
        }
    };
    body.push('}');
    (status, body)
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
    http::answer(run_uilab_session_propose_patch(system, &value))
}

/// `uilab.session.ProposePatch` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_uilab_session_propose_patch<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let input = match wire::decode_command_uilab_session_propose_patch(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.uilab_session.propose_patch(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_uilab_session_propose_patch(&outcome))
}

/// One declared outcome of `uilab.session.ProposePatch`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_uilab_session_propose_patch(outcome: &uilab_types::session::ProposePatchOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::ProposePatchOutcome::Proposed { patch_proposed, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "proposed");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "uilab.session.PatchProposed");
            json::member(&mut body, "payload");
            wire::encode_event_uilab_session_patch_proposed(patch_proposed, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        uilab_types::session::ProposePatchOutcome::Refused { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "refused");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.PatchRefused");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_patch_refused(error, &mut body);
            502
        }
    };
    body.push('}');
    (status, body)
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
    http::answer(run_uilab_session_reject_proposal(system, &value))
}

/// `uilab.session.RejectProposal` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_uilab_session_reject_proposal<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let input = match wire::decode_command_uilab_session_reject_proposal(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.uilab_session.reject_proposal(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_uilab_session_reject_proposal(&outcome))
}

/// One declared outcome of `uilab.session.RejectProposal`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_uilab_session_reject_proposal(outcome: &uilab_types::session::RejectProposalOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::RejectProposalOutcome::Rejected { proposal_rejected, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "rejected");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "uilab.session.ProposalRejected");
            json::member(&mut body, "payload");
            wire::encode_event_uilab_session_proposal_rejected(proposal_rejected, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        uilab_types::session::RejectProposalOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_proposal_state_conflict(error, &mut body);
            409
        }
        uilab_types::session::RejectProposalOutcome::WrongStateUnknownInstance => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push_str("[]");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            409
        }
    };
    body.push('}');
    (status, body)
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
    http::answer(run_uilab_session_select_node(system, &value))
}

/// `uilab.session.SelectNode` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_uilab_session_select_node<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let input = match wire::decode_command_uilab_session_select_node(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.uilab_session.select_node(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_uilab_session_select_node(&outcome))
}

/// One declared outcome of `uilab.session.SelectNode`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_uilab_session_select_node(outcome: &uilab_types::session::SelectNodeOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::SelectNodeOutcome::Selected { node_selected, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "selected");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "uilab.session.NodeSelected");
            json::member(&mut body, "payload");
            wire::encode_event_uilab_session_node_selected(node_selected, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        uilab_types::session::SelectNodeOutcome::UnknownDocument { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "unknown-document");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.DocumentNotOpen");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_document_not_open(error, &mut body);
            502
        }
        uilab_types::session::SelectNodeOutcome::NotFound { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.NodeNotFound");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_node_not_found(error, &mut body);
            502
        }
    };
    body.push('}');
    (status, body)
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
    http::answer(run_uilab_session_undo_proposal(system, &value))
}

/// `uilab.session.UndoProposal` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_uilab_session_undo_proposal<UilabSessionBehaviors>(system: &mut uilab_system::System<UilabSessionBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    UilabSessionBehaviors: uilab_types::session::obligations::AcceptProposalBehavior + uilab_types::session::obligations::OpenDocumentBehavior + uilab_types::session::obligations::ProposePatchBehavior + uilab_types::session::obligations::RejectProposalBehavior + uilab_types::session::obligations::SelectNodeBehavior + uilab_types::session::obligations::UndoProposalBehavior + uilab_types::session::obligations::DocumentsQuery + uilab_types::session::obligations::PendingQuery + uilab_types::session::obligations::ProposalsQuery,
{
    let input = match wire::decode_command_uilab_session_undo_proposal(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.uilab_session.undo_proposal(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_uilab_session_undo_proposal(&outcome))
}

/// One declared outcome of `uilab.session.UndoProposal`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_uilab_session_undo_proposal(outcome: &uilab_types::session::UndoProposalOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        uilab_types::session::UndoProposalOutcome::Undone { proposal_undone, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "undone");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "uilab.session.ProposalUndone");
            json::member(&mut body, "payload");
            wire::encode_event_uilab_session_proposal_undone(proposal_undone, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        uilab_types::session::UndoProposalOutcome::Stale { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "stale");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.PatchRefused");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_patch_refused(error, &mut body);
            502
        }
        uilab_types::session::UndoProposalOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_uilab_session_proposal_state_conflict(error, &mut body);
            409
        }
        uilab_types::session::UndoProposalOutcome::WrongStateUnknownInstance => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push_str("[]");
            json::member(&mut body, "error");
            json::push_text(&mut body, "uilab.session.ProposalStateConflict");
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `uilab.session.Documents` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_uilab_session_documents<UilabSessionBehaviors>(system: &uilab_system::System<UilabSessionBehaviors>) -> Result<(u16, String), entry::Refused>
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
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `uilab.session.Pending` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_uilab_session_pending<UilabSessionBehaviors>(system: &uilab_system::System<UilabSessionBehaviors>) -> Result<(u16, String), entry::Refused>
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
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `uilab.session.Proposals` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_uilab_session_proposals<UilabSessionBehaviors>(system: &uilab_system::System<UilabSessionBehaviors>) -> Result<(u16, String), entry::Refused>
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
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}
