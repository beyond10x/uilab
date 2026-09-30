// generated from uilab v1
// model digest 823a0dbdc48dcff7ae64379e3fd12563f56326f07f9fafab80645c14a42e2c24
// contract digest 1f7ac65ed8e829d046658cff8ae43891c3483669061f0d449be07d91d77ac6f9
// do not edit: regenerate with `ess synthesize`
//! Every generated declaration, as JSON, in the renderings the published wire contracts fix.
//!
//! Generated from the model beside the types it crosses, so a field renamed in the specification
//! is renamed here in the same regeneration. An absent optional field is omitted rather
//! than sent as `null`, which is what the `required` list of the published schema says.

use crate::json;

/// Writes `uilab.session.Document.State` as JSON.
pub fn encode_uilab_session_document_state(value: &uilab_types::session::DocumentState, out: &mut String) {
    match value {
        uilab_types::session::DocumentState::Open => json::push_text(out, "Open"),
    }
}

/// Reads `uilab.session.Document.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_session_document_state(value: &json::Value, at: &str) -> Result<uilab_types::session::DocumentState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Open`")? {
        "Open" => uilab_types::session::DocumentState::Open,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Open`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `uilab.session.DocumentId` as JSON.
pub fn encode_uilab_session_document_id(value: &uilab_types::session::DocumentId, out: &mut String) {
    json::push_text(out, &value.0.0);
}

/// Reads `uilab.session.DocumentId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_session_document_id(value: &json::Value, at: &str) -> Result<uilab_types::session::DocumentId, json::DecodeError> {
    Ok(uilab_types::session::DocumentId(uilab_types::primitives::Uuid(json::text_at(value, at, "a UUID")?.to_owned())))
}

/// Writes `uilab.session.NodePath` as JSON.
pub fn encode_uilab_session_node_path(value: &uilab_types::session::NodePath, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `uilab.session.NodePath` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_session_node_path(value: &json::Value, at: &str) -> Result<uilab_types::session::NodePath, json::DecodeError> {
    Ok(uilab_types::session::NodePath(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `uilab.session.PatchBody` as JSON.
pub fn encode_uilab_session_patch_body(value: &uilab_types::session::PatchBody, out: &mut String) {
    json::push_value(out, &value.0);
}

/// Reads `uilab.session.PatchBody` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_session_patch_body(value: &json::Value, at: &str) -> Result<uilab_types::session::PatchBody, json::DecodeError> {
    Ok(uilab_types::session::PatchBody(json::value_at(value, at)))
}

/// Writes `uilab.session.PatchOp` as JSON.
pub fn encode_uilab_session_patch_op(value: &uilab_types::session::PatchOp, out: &mut String) {
    match value {
        uilab_types::session::PatchOp::Insert => json::push_text(out, "Insert"),
        uilab_types::session::PatchOp::Replace => json::push_text(out, "Replace"),
        uilab_types::session::PatchOp::Remove => json::push_text(out, "Remove"),
        uilab_types::session::PatchOp::Batch => json::push_text(out, "Batch"),
    }
}

/// Reads `uilab.session.PatchOp` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_session_patch_op(value: &json::Value, at: &str) -> Result<uilab_types::session::PatchOp, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Insert`, `Replace`, `Remove`, `Batch`")? {
        "Insert" => uilab_types::session::PatchOp::Insert,
        "Replace" => uilab_types::session::PatchOp::Replace,
        "Remove" => uilab_types::session::PatchOp::Remove,
        "Batch" => uilab_types::session::PatchOp::Batch,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Insert`, `Replace`, `Remove`, `Batch`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `uilab.session.Proposal.State` as JSON.
pub fn encode_uilab_session_proposal_state(value: &uilab_types::session::ProposalState, out: &mut String) {
    match value {
        uilab_types::session::ProposalState::Accepted => json::push_text(out, "Accepted"),
        uilab_types::session::ProposalState::Proposed => json::push_text(out, "Proposed"),
        uilab_types::session::ProposalState::Rejected => json::push_text(out, "Rejected"),
        uilab_types::session::ProposalState::Undone => json::push_text(out, "Undone"),
    }
}

/// Reads `uilab.session.Proposal.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_session_proposal_state(value: &json::Value, at: &str) -> Result<uilab_types::session::ProposalState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Accepted`, `Proposed`, `Rejected`, `Undone`")? {
        "Accepted" => uilab_types::session::ProposalState::Accepted,
        "Proposed" => uilab_types::session::ProposalState::Proposed,
        "Rejected" => uilab_types::session::ProposalState::Rejected,
        "Undone" => uilab_types::session::ProposalState::Undone,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Accepted`, `Proposed`, `Rejected`, `Undone`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `uilab.session.ProposalId` as JSON.
pub fn encode_uilab_session_proposal_id(value: &uilab_types::session::ProposalId, out: &mut String) {
    json::push_text(out, &value.0.0);
}

/// Reads `uilab.session.ProposalId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_session_proposal_id(value: &json::Value, at: &str) -> Result<uilab_types::session::ProposalId, json::DecodeError> {
    Ok(uilab_types::session::ProposalId(uilab_types::primitives::Uuid(json::text_at(value, at, "a UUID")?.to_owned())))
}

/// Writes `uilab.wire.Changed` as JSON.
pub fn encode_uilab_wire_changed(value: &uilab_types::wire::Changed, out: &mut String) {
    out.push('{');
    json::member(out, "revision");
    json::push_integer(out, value.revision);
    json::member(out, "by");
    json::push_text(out, &value.by);
    json::member(out, "op");
    encode_uilab_session_patch_op(&value.op, out);
    json::member(out, "changed");
    encode_uilab_session_node_path(&value.changed, out);
    json::member(out, "parent");
    encode_uilab_session_node_path(&value.parent, out);
    if let Some(held0) = &value.node {
        json::member(out, "node");
        encode_uilab_wire_outline_node(&*held0, out);
    }
    json::member(out, "findings");
    out.push('[');
    for (index0, item0) in value.findings.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_uilab_wire_finding(&*item0, out);
    }
    out.push(']');
    out.push('}');
}

/// Reads `uilab.wire.Changed` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_changed(value: &json::Value, at: &str) -> Result<uilab_types::wire::Changed, json::DecodeError> {
    Ok(uilab_types::wire::Changed {
        revision: {
            let at0 = json::nested(at, "revision");
            let member0 = json::member_at(value, at, "revision")?;
            json::integer_at(member0, &at0, "an integer")?
        },
        by: {
            let at1 = json::nested(at, "by");
            let member1 = json::member_at(value, at, "by")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        op: {
            let at2 = json::nested(at, "op");
            let member2 = json::member_at(value, at, "op")?;
            decode_uilab_session_patch_op(member2, &at2)?
        },
        changed: {
            let at3 = json::nested(at, "changed");
            let member3 = json::member_at(value, at, "changed")?;
            decode_uilab_session_node_path(member3, &at3)?
        },
        parent: {
            let at4 = json::nested(at, "parent");
            let member4 = json::member_at(value, at, "parent")?;
            decode_uilab_session_node_path(member4, &at4)?
        },
        node: match value.member("node") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "node");
                Some(decode_uilab_wire_outline_node(member5, &at5)?)
            }
        },
        findings: {
            let at6 = json::nested(at, "findings");
            let member6 = json::member_at(value, at, "findings")?;
            {
                let mut items6 = Vec::new();
                for (index6, element6) in json::items_at(member6, &at6, "an array")?.iter().enumerate() {
                    let nested6 = json::nested(&at6, &index6.to_string());
                    items6.push(decode_uilab_wire_finding(element6, &nested6)?);
                }
                items6
            }
        },
    })
}

/// Writes `uilab.wire.ClientMessage` as JSON.
pub fn encode_uilab_wire_client_message(value: &uilab_types::wire::ClientMessage, out: &mut String) {
    match value {
        uilab_types::wire::ClientMessage::Accept(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "accept");
            json::member(out, "value");
            encode_uilab_wire_decide(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::Goal(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "goal");
            json::member(out, "value");
            encode_uilab_wire_start_goal(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::Hello(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "hello");
            json::member(out, "value");
            encode_uilab_wire_hello(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::Mic(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "mic");
            json::member(out, "value");
            encode_uilab_wire_mic(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::Reject(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "reject");
            json::member(out, "value");
            encode_uilab_wire_decide(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::Resync(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "resync");
            json::member(out, "value");
            encode_uilab_wire_resync(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::Rows(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "rows");
            json::member(out, "value");
            encode_uilab_wire_read_rows(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::Say(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "say");
            json::member(out, "value");
            encode_uilab_wire_say(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::Select(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "select");
            json::member(out, "value");
            encode_uilab_wire_select(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::Settings(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "settings");
            json::member(out, "value");
            encode_uilab_wire_settings(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::StopGoal(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "stop_goal");
            json::member(out, "value");
            encode_uilab_wire_stop_goal(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ClientMessage::Undo(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "undo");
            json::member(out, "value");
            encode_uilab_wire_decide(&*held, out);
            out.push('}');
        }
    }
}

/// Reads `uilab.wire.ClientMessage` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_client_message(value: &json::Value, at: &str) -> Result<uilab_types::wire::ClientMessage, json::DecodeError> {
    let tag = json::member_at(value, at, "type")?;
    let at_tag = json::nested(at, "type");
    Ok(match json::text_at(tag, &at_tag, "one of `accept`, `goal`, `hello`, `mic`, `reject`, `resync`, `rows`, `say`, `select`, `settings`, `stop_goal`, `undo`")? {
        "accept" => uilab_types::wire::ClientMessage::Accept({
            let at0 = json::nested(at, "value");
            let member0 = json::member_at(value, at, "value")?;
            decode_uilab_wire_decide(member0, &at0)?
        }),
        "goal" => uilab_types::wire::ClientMessage::Goal({
            let at1 = json::nested(at, "value");
            let member1 = json::member_at(value, at, "value")?;
            decode_uilab_wire_start_goal(member1, &at1)?
        }),
        "hello" => uilab_types::wire::ClientMessage::Hello({
            let at2 = json::nested(at, "value");
            let member2 = json::member_at(value, at, "value")?;
            decode_uilab_wire_hello(member2, &at2)?
        }),
        "mic" => uilab_types::wire::ClientMessage::Mic({
            let at3 = json::nested(at, "value");
            let member3 = json::member_at(value, at, "value")?;
            decode_uilab_wire_mic(member3, &at3)?
        }),
        "reject" => uilab_types::wire::ClientMessage::Reject({
            let at4 = json::nested(at, "value");
            let member4 = json::member_at(value, at, "value")?;
            decode_uilab_wire_decide(member4, &at4)?
        }),
        "resync" => uilab_types::wire::ClientMessage::Resync({
            let at5 = json::nested(at, "value");
            let member5 = json::member_at(value, at, "value")?;
            decode_uilab_wire_resync(member5, &at5)?
        }),
        "rows" => uilab_types::wire::ClientMessage::Rows({
            let at6 = json::nested(at, "value");
            let member6 = json::member_at(value, at, "value")?;
            decode_uilab_wire_read_rows(member6, &at6)?
        }),
        "say" => uilab_types::wire::ClientMessage::Say({
            let at7 = json::nested(at, "value");
            let member7 = json::member_at(value, at, "value")?;
            decode_uilab_wire_say(member7, &at7)?
        }),
        "select" => uilab_types::wire::ClientMessage::Select({
            let at8 = json::nested(at, "value");
            let member8 = json::member_at(value, at, "value")?;
            decode_uilab_wire_select(member8, &at8)?
        }),
        "settings" => uilab_types::wire::ClientMessage::Settings({
            let at9 = json::nested(at, "value");
            let member9 = json::member_at(value, at, "value")?;
            decode_uilab_wire_settings(member9, &at9)?
        }),
        "stop_goal" => uilab_types::wire::ClientMessage::StopGoal({
            let at10 = json::nested(at, "value");
            let member10 = json::member_at(value, at, "value")?;
            decode_uilab_wire_stop_goal(member10, &at10)?
        }),
        "undo" => uilab_types::wire::ClientMessage::Undo({
            let at11 = json::nested(at, "value");
            let member11 = json::member_at(value, at, "value")?;
            decode_uilab_wire_decide(member11, &at11)?
        }),
        other => return Err(json::DecodeError { at: at_tag.clone(), expected: "one of `accept`, `goal`, `hello`, `mic`, `reject`, `resync`, `rows`, `say`, `select`, `settings`, `stop_goal`, `undo`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `uilab.wire.Decide` as JSON.
pub fn encode_uilab_wire_decide(value: &uilab_types::wire::Decide, out: &mut String) {
    out.push('{');
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    out.push('}');
}

/// Reads `uilab.wire.Decide` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_decide(value: &json::Value, at: &str) -> Result<uilab_types::wire::Decide, json::DecodeError> {
    Ok(uilab_types::wire::Decide {
        proposal_id: {
            let at0 = json::nested(at, "proposal_id");
            let member0 = json::member_at(value, at, "proposal_id")?;
            decode_uilab_session_proposal_id(member0, &at0)?
        },
    })
}

/// Writes `uilab.wire.DocumentState` as JSON.
pub fn encode_uilab_wire_document_state(value: &uilab_types::wire::DocumentState, out: &mut String) {
    out.push('{');
    json::member(out, "document_id");
    encode_uilab_session_document_id(&value.document_id, out);
    json::member(out, "file");
    json::push_text(out, &value.file);
    if let Some(held0) = &value.title {
        json::member(out, "title");
        json::push_text(out, &*held0);
    }
    json::member(out, "selected");
    encode_uilab_session_node_path(&value.selected, out);
    json::member(out, "outline");
    encode_uilab_wire_outline_node(&value.outline, out);
    json::member(out, "findings");
    out.push('[');
    for (index0, item0) in value.findings.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_uilab_wire_finding(&*item0, out);
    }
    out.push(']');
    if let Some(held0) = &value.undoable {
        json::member(out, "undoable");
        encode_uilab_session_proposal_id(&*held0, out);
    }
    json::member(out, "revision");
    json::push_integer(out, value.revision);
    json::member(out, "review");
    json::push_bool(out, value.review);
    out.push('}');
}

/// Reads `uilab.wire.DocumentState` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_document_state(value: &json::Value, at: &str) -> Result<uilab_types::wire::DocumentState, json::DecodeError> {
    Ok(uilab_types::wire::DocumentState {
        document_id: {
            let at0 = json::nested(at, "document_id");
            let member0 = json::member_at(value, at, "document_id")?;
            decode_uilab_session_document_id(member0, &at0)?
        },
        file: {
            let at1 = json::nested(at, "file");
            let member1 = json::member_at(value, at, "file")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        title: match value.member("title") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "title");
                Some(json::text_at(member2, &at2, "a string")?.to_owned())
            }
        },
        selected: {
            let at3 = json::nested(at, "selected");
            let member3 = json::member_at(value, at, "selected")?;
            decode_uilab_session_node_path(member3, &at3)?
        },
        outline: {
            let at4 = json::nested(at, "outline");
            let member4 = json::member_at(value, at, "outline")?;
            decode_uilab_wire_outline_node(member4, &at4)?
        },
        findings: {
            let at5 = json::nested(at, "findings");
            let member5 = json::member_at(value, at, "findings")?;
            {
                let mut items5 = Vec::new();
                for (index5, element5) in json::items_at(member5, &at5, "an array")?.iter().enumerate() {
                    let nested5 = json::nested(&at5, &index5.to_string());
                    items5.push(decode_uilab_wire_finding(element5, &nested5)?);
                }
                items5
            }
        },
        undoable: match value.member("undoable") {
            None | Some(json::Value::Null) => None,
            Some(member6) => {
                let at6 = json::nested(at, "undoable");
                Some(decode_uilab_session_proposal_id(member6, &at6)?)
            }
        },
        revision: {
            let at7 = json::nested(at, "revision");
            let member7 = json::member_at(value, at, "revision")?;
            json::integer_at(member7, &at7, "an integer")?
        },
        review: {
            let at8 = json::nested(at, "review");
            let member8 = json::member_at(value, at, "review")?;
            json::bool_at(member8, &at8, "a boolean")?
        },
    })
}

/// Writes `uilab.wire.Failed` as JSON.
pub fn encode_uilab_wire_failed(value: &uilab_types::wire::Failed, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.by {
        json::member(out, "by");
        json::push_text(out, &*held0);
    }
    json::member(out, "message");
    json::push_text(out, &value.message);
    out.push('}');
}

/// Reads `uilab.wire.Failed` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_failed(value: &json::Value, at: &str) -> Result<uilab_types::wire::Failed, json::DecodeError> {
    Ok(uilab_types::wire::Failed {
        by: match value.member("by") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "by");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        message: {
            let at1 = json::nested(at, "message");
            let member1 = json::member_at(value, at, "message")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
    })
}

/// Writes `uilab.wire.Finding` as JSON.
pub fn encode_uilab_wire_finding(value: &uilab_types::wire::Finding, out: &mut String) {
    out.push('{');
    json::member(out, "check");
    json::push_text(out, &value.check);
    json::member(out, "severity");
    encode_uilab_wire_severity(&value.severity, out);
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "message");
    json::push_text(out, &value.message);
    out.push('}');
}

/// Reads `uilab.wire.Finding` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_finding(value: &json::Value, at: &str) -> Result<uilab_types::wire::Finding, json::DecodeError> {
    Ok(uilab_types::wire::Finding {
        check: {
            let at0 = json::nested(at, "check");
            let member0 = json::member_at(value, at, "check")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        severity: {
            let at1 = json::nested(at, "severity");
            let member1 = json::member_at(value, at, "severity")?;
            decode_uilab_wire_severity(member1, &at1)?
        },
        path: {
            let at2 = json::nested(at, "path");
            let member2 = json::member_at(value, at, "path")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        message: {
            let at3 = json::nested(at, "message");
            let member3 = json::member_at(value, at, "message")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
    })
}

/// Writes `uilab.wire.Goal` as JSON.
pub fn encode_uilab_wire_goal(value: &uilab_types::wire::Goal, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id);
    json::member(out, "by");
    json::push_text(out, &value.by);
    json::member(out, "text");
    json::push_text(out, &value.text);
    json::member(out, "state");
    encode_uilab_wire_goal_state(&value.state, out);
    json::member(out, "steps");
    out.push('[');
    for (index0, item0) in value.steps.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_uilab_wire_goal_step(&*item0, out);
    }
    out.push(']');
    if let Some(held0) = &value.current {
        json::member(out, "current");
        json::push_integer(out, *held0);
    }
    if let Some(held0) = &value.message {
        json::member(out, "message");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads `uilab.wire.Goal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_goal(value: &json::Value, at: &str) -> Result<uilab_types::wire::Goal, json::DecodeError> {
    Ok(uilab_types::wire::Goal {
        goal_id: {
            let at0 = json::nested(at, "goal_id");
            let member0 = json::member_at(value, at, "goal_id")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        by: {
            let at1 = json::nested(at, "by");
            let member1 = json::member_at(value, at, "by")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        text: {
            let at2 = json::nested(at, "text");
            let member2 = json::member_at(value, at, "text")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        state: {
            let at3 = json::nested(at, "state");
            let member3 = json::member_at(value, at, "state")?;
            decode_uilab_wire_goal_state(member3, &at3)?
        },
        steps: {
            let at4 = json::nested(at, "steps");
            let member4 = json::member_at(value, at, "steps")?;
            {
                let mut items4 = Vec::new();
                for (index4, element4) in json::items_at(member4, &at4, "an array")?.iter().enumerate() {
                    let nested4 = json::nested(&at4, &index4.to_string());
                    items4.push(decode_uilab_wire_goal_step(element4, &nested4)?);
                }
                items4
            }
        },
        current: match value.member("current") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "current");
                Some(json::integer_at(member5, &at5, "an integer")?)
            }
        },
        message: match value.member("message") {
            None | Some(json::Value::Null) => None,
            Some(member6) => {
                let at6 = json::nested(at, "message");
                Some(json::text_at(member6, &at6, "a string")?.to_owned())
            }
        },
    })
}

/// Writes `uilab.wire.GoalState` as JSON.
pub fn encode_uilab_wire_goal_state(value: &uilab_types::wire::GoalState, out: &mut String) {
    match value {
        uilab_types::wire::GoalState::Planning => json::push_text(out, "planning"),
        uilab_types::wire::GoalState::Running => json::push_text(out, "running"),
        uilab_types::wire::GoalState::Done => json::push_text(out, "done"),
        uilab_types::wire::GoalState::Stopped => json::push_text(out, "stopped"),
        uilab_types::wire::GoalState::Failed => json::push_text(out, "failed"),
    }
}

/// Reads `uilab.wire.GoalState` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_goal_state(value: &json::Value, at: &str) -> Result<uilab_types::wire::GoalState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `planning`, `running`, `done`, `stopped`, `failed`")? {
        "planning" => uilab_types::wire::GoalState::Planning,
        "running" => uilab_types::wire::GoalState::Running,
        "done" => uilab_types::wire::GoalState::Done,
        "stopped" => uilab_types::wire::GoalState::Stopped,
        "failed" => uilab_types::wire::GoalState::Failed,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `planning`, `running`, `done`, `stopped`, `failed`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `uilab.wire.GoalStep` as JSON.
pub fn encode_uilab_wire_goal_step(value: &uilab_types::wire::GoalStep, out: &mut String) {
    out.push('{');
    json::member(out, "instruction");
    json::push_text(out, &value.instruction);
    json::member(out, "target");
    encode_uilab_session_node_path(&value.target, out);
    json::member(out, "why");
    json::push_text(out, &value.why);
    json::member(out, "status");
    encode_uilab_wire_step_status(&value.status, out);
    if let Some(held0) = &value.proposal_id {
        json::member(out, "proposal_id");
        encode_uilab_session_proposal_id(&*held0, out);
    }
    out.push('}');
}

/// Reads `uilab.wire.GoalStep` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_goal_step(value: &json::Value, at: &str) -> Result<uilab_types::wire::GoalStep, json::DecodeError> {
    Ok(uilab_types::wire::GoalStep {
        instruction: {
            let at0 = json::nested(at, "instruction");
            let member0 = json::member_at(value, at, "instruction")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        target: {
            let at1 = json::nested(at, "target");
            let member1 = json::member_at(value, at, "target")?;
            decode_uilab_session_node_path(member1, &at1)?
        },
        why: {
            let at2 = json::nested(at, "why");
            let member2 = json::member_at(value, at, "why")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        status: {
            let at3 = json::nested(at, "status");
            let member3 = json::member_at(value, at, "status")?;
            decode_uilab_wire_step_status(member3, &at3)?
        },
        proposal_id: match value.member("proposal_id") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "proposal_id");
                Some(decode_uilab_session_proposal_id(member4, &at4)?)
            }
        },
    })
}

/// Writes `uilab.wire.Hello` as JSON.
pub fn encode_uilab_wire_hello(value: &uilab_types::wire::Hello, out: &mut String) {
    out.push('{');
    json::member(out, "name");
    json::push_text(out, &value.name);
    json::member(out, "kind");
    encode_uilab_wire_operator_kind(&value.kind, out);
    out.push('}');
}

/// Reads `uilab.wire.Hello` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_hello(value: &json::Value, at: &str) -> Result<uilab_types::wire::Hello, json::DecodeError> {
    Ok(uilab_types::wire::Hello {
        name: {
            let at0 = json::nested(at, "name");
            let member0 = json::member_at(value, at, "name")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        kind: {
            let at1 = json::nested(at, "kind");
            let member1 = json::member_at(value, at, "kind")?;
            decode_uilab_wire_operator_kind(member1, &at1)?
        },
    })
}

/// Writes `uilab.wire.Mic` as JSON.
pub fn encode_uilab_wire_mic(value: &uilab_types::wire::Mic, out: &mut String) {
    out.push('{');
    json::member(out, "state");
    encode_uilab_wire_mic_state(&value.state, out);
    if let Some(held0) = &value.workspace {
        json::member(out, "workspace");
        encode_uilab_wire_workspace(&*held0, out);
    }
    out.push('}');
}

/// Reads `uilab.wire.Mic` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_mic(value: &json::Value, at: &str) -> Result<uilab_types::wire::Mic, json::DecodeError> {
    Ok(uilab_types::wire::Mic {
        state: {
            let at0 = json::nested(at, "state");
            let member0 = json::member_at(value, at, "state")?;
            decode_uilab_wire_mic_state(member0, &at0)?
        },
        workspace: match value.member("workspace") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "workspace");
                Some(decode_uilab_wire_workspace(member1, &at1)?)
            }
        },
    })
}

/// Writes `uilab.wire.MicState` as JSON.
pub fn encode_uilab_wire_mic_state(value: &uilab_types::wire::MicState, out: &mut String) {
    match value {
        uilab_types::wire::MicState::Open => json::push_text(out, "open"),
        uilab_types::wire::MicState::Closed => json::push_text(out, "closed"),
    }
}

/// Reads `uilab.wire.MicState` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_mic_state(value: &json::Value, at: &str) -> Result<uilab_types::wire::MicState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `open`, `closed`")? {
        "open" => uilab_types::wire::MicState::Open,
        "closed" => uilab_types::wire::MicState::Closed,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `open`, `closed`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `uilab.wire.Moved` as JSON.
pub fn encode_uilab_wire_moved(value: &uilab_types::wire::Moved, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.by {
        json::member(out, "by");
        json::push_text(out, &*held0);
    }
    json::member(out, "selected_by");
    json::push_text(out, &value.selected_by);
    json::member(out, "from");
    encode_uilab_session_node_path(&value.from, out);
    json::member(out, "to");
    encode_uilab_session_node_path(&value.to, out);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    json::member(out, "navigate_only");
    json::push_bool(out, value.navigate_only);
    json::member(out, "utterance");
    json::push_text(out, &value.utterance);
    out.push('}');
}

/// Reads `uilab.wire.Moved` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_moved(value: &json::Value, at: &str) -> Result<uilab_types::wire::Moved, json::DecodeError> {
    Ok(uilab_types::wire::Moved {
        by: match value.member("by") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "by");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        selected_by: {
            let at1 = json::nested(at, "selected_by");
            let member1 = json::member_at(value, at, "selected_by")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        from: {
            let at2 = json::nested(at, "from");
            let member2 = json::member_at(value, at, "from")?;
            decode_uilab_session_node_path(member2, &at2)?
        },
        to: {
            let at3 = json::nested(at, "to");
            let member3 = json::member_at(value, at, "to")?;
            decode_uilab_session_node_path(member3, &at3)?
        },
        reason: {
            let at4 = json::nested(at, "reason");
            let member4 = json::member_at(value, at, "reason")?;
            json::text_at(member4, &at4, "a string")?.to_owned()
        },
        navigate_only: {
            let at5 = json::nested(at, "navigate_only");
            let member5 = json::member_at(value, at, "navigate_only")?;
            json::bool_at(member5, &at5, "a boolean")?
        },
        utterance: {
            let at6 = json::nested(at, "utterance");
            let member6 = json::member_at(value, at, "utterance")?;
            json::text_at(member6, &at6, "a string")?.to_owned()
        },
    })
}

/// Writes `uilab.wire.Operator` as JSON.
pub fn encode_uilab_wire_operator(value: &uilab_types::wire::Operator, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    json::push_text(out, &value.id);
    json::member(out, "name");
    json::push_text(out, &value.name);
    json::member(out, "kind");
    encode_uilab_wire_operator_kind(&value.kind, out);
    json::member(out, "last_seen_ms");
    json::push_integer(out, value.last_seen_ms);
    out.push('}');
}

/// Reads `uilab.wire.Operator` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_operator(value: &json::Value, at: &str) -> Result<uilab_types::wire::Operator, json::DecodeError> {
    Ok(uilab_types::wire::Operator {
        id: {
            let at0 = json::nested(at, "id");
            let member0 = json::member_at(value, at, "id")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        name: {
            let at1 = json::nested(at, "name");
            let member1 = json::member_at(value, at, "name")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        kind: {
            let at2 = json::nested(at, "kind");
            let member2 = json::member_at(value, at, "kind")?;
            decode_uilab_wire_operator_kind(member2, &at2)?
        },
        last_seen_ms: {
            let at3 = json::nested(at, "last_seen_ms");
            let member3 = json::member_at(value, at, "last_seen_ms")?;
            json::integer_at(member3, &at3, "an integer")?
        },
    })
}

/// Writes `uilab.wire.OperatorKind` as JSON.
pub fn encode_uilab_wire_operator_kind(value: &uilab_types::wire::OperatorKind, out: &mut String) {
    match value {
        uilab_types::wire::OperatorKind::Human => json::push_text(out, "human"),
        uilab_types::wire::OperatorKind::Agent => json::push_text(out, "agent"),
    }
}

/// Reads `uilab.wire.OperatorKind` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_operator_kind(value: &json::Value, at: &str) -> Result<uilab_types::wire::OperatorKind, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `human`, `agent`")? {
        "human" => uilab_types::wire::OperatorKind::Human,
        "agent" => uilab_types::wire::OperatorKind::Agent,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `human`, `agent`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `uilab.wire.OutlineNode` as JSON.
pub fn encode_uilab_wire_outline_node(value: &uilab_types::wire::OutlineNode, out: &mut String) {
    out.push('{');
    json::member(out, "path");
    encode_uilab_session_node_path(&value.path, out);
    json::member(out, "layer");
    json::push_text(out, &value.layer);
    json::member(out, "name");
    json::push_text(out, &value.name);
    json::member(out, "kind");
    json::push_text(out, &value.kind);
    if let Some(held0) = &value.title {
        json::member(out, "title");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.view {
        json::member(out, "view");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.props {
        json::member(out, "props");
        json::push_value(out, &*held0);
    }
    json::member(out, "children");
    out.push('[');
    for (index0, item0) in value.children.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_uilab_wire_outline_node(&*item0, out);
    }
    out.push(']');
    out.push('}');
}

/// Reads `uilab.wire.OutlineNode` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_outline_node(value: &json::Value, at: &str) -> Result<uilab_types::wire::OutlineNode, json::DecodeError> {
    Ok(uilab_types::wire::OutlineNode {
        path: {
            let at0 = json::nested(at, "path");
            let member0 = json::member_at(value, at, "path")?;
            decode_uilab_session_node_path(member0, &at0)?
        },
        layer: {
            let at1 = json::nested(at, "layer");
            let member1 = json::member_at(value, at, "layer")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        name: {
            let at2 = json::nested(at, "name");
            let member2 = json::member_at(value, at, "name")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        kind: {
            let at3 = json::nested(at, "kind");
            let member3 = json::member_at(value, at, "kind")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
        title: match value.member("title") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "title");
                Some(json::text_at(member4, &at4, "a string")?.to_owned())
            }
        },
        view: match value.member("view") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "view");
                Some(json::text_at(member5, &at5, "a string")?.to_owned())
            }
        },
        props: match value.member("props") {
            None | Some(json::Value::Null) => None,
            Some(member6) => {
                let at6 = json::nested(at, "props");
                Some(json::value_at(member6, &at6))
            }
        },
        children: {
            let at7 = json::nested(at, "children");
            let member7 = json::member_at(value, at, "children")?;
            {
                let mut items7 = Vec::new();
                for (index7, element7) in json::items_at(member7, &at7, "an array")?.iter().enumerate() {
                    let nested7 = json::nested(&at7, &index7.to_string());
                    items7.push(decode_uilab_wire_outline_node(element7, &nested7)?);
                }
                items7
            }
        },
    })
}

/// Writes `uilab.wire.Presence` as JSON.
pub fn encode_uilab_wire_presence(value: &uilab_types::wire::Presence, out: &mut String) {
    out.push('{');
    json::member(out, "operators");
    out.push('[');
    for (index0, item0) in value.operators.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_uilab_wire_operator(&*item0, out);
    }
    out.push(']');
    if let Some(held0) = &value.selected_by {
        json::member(out, "selected_by");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads `uilab.wire.Presence` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_presence(value: &json::Value, at: &str) -> Result<uilab_types::wire::Presence, json::DecodeError> {
    Ok(uilab_types::wire::Presence {
        operators: {
            let at0 = json::nested(at, "operators");
            let member0 = json::member_at(value, at, "operators")?;
            {
                let mut items0 = Vec::new();
                for (index0, element0) in json::items_at(member0, &at0, "an array")?.iter().enumerate() {
                    let nested0 = json::nested(&at0, &index0.to_string());
                    items0.push(decode_uilab_wire_operator(element0, &nested0)?);
                }
                items0
            }
        },
        selected_by: match value.member("selected_by") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "selected_by");
                Some(json::text_at(member1, &at1, "a string")?.to_owned())
            }
        },
    })
}

/// Writes `uilab.wire.ProposalShown` as JSON.
pub fn encode_uilab_wire_proposal_shown(value: &uilab_types::wire::ProposalShown, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.by {
        json::member(out, "by");
        json::push_text(out, &*held0);
    }
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    json::member(out, "target");
    encode_uilab_session_node_path(&value.target, out);
    json::member(out, "changed");
    encode_uilab_session_node_path(&value.changed, out);
    json::member(out, "op");
    encode_uilab_session_patch_op(&value.op, out);
    json::member(out, "utterance");
    json::push_text(out, &value.utterance);
    json::member(out, "before");
    json::push_text(out, &value.before);
    json::member(out, "after");
    json::push_text(out, &value.after);
    json::member(out, "findings");
    out.push('[');
    for (index0, item0) in value.findings.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_uilab_wire_finding(&*item0, out);
    }
    out.push(']');
    json::member(out, "outline");
    encode_uilab_wire_outline_node(&value.outline, out);
    out.push('}');
}

/// Reads `uilab.wire.ProposalShown` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_proposal_shown(value: &json::Value, at: &str) -> Result<uilab_types::wire::ProposalShown, json::DecodeError> {
    Ok(uilab_types::wire::ProposalShown {
        by: match value.member("by") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "by");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        proposal_id: {
            let at1 = json::nested(at, "proposal_id");
            let member1 = json::member_at(value, at, "proposal_id")?;
            decode_uilab_session_proposal_id(member1, &at1)?
        },
        target: {
            let at2 = json::nested(at, "target");
            let member2 = json::member_at(value, at, "target")?;
            decode_uilab_session_node_path(member2, &at2)?
        },
        changed: {
            let at3 = json::nested(at, "changed");
            let member3 = json::member_at(value, at, "changed")?;
            decode_uilab_session_node_path(member3, &at3)?
        },
        op: {
            let at4 = json::nested(at, "op");
            let member4 = json::member_at(value, at, "op")?;
            decode_uilab_session_patch_op(member4, &at4)?
        },
        utterance: {
            let at5 = json::nested(at, "utterance");
            let member5 = json::member_at(value, at, "utterance")?;
            json::text_at(member5, &at5, "a string")?.to_owned()
        },
        before: {
            let at6 = json::nested(at, "before");
            let member6 = json::member_at(value, at, "before")?;
            json::text_at(member6, &at6, "a string")?.to_owned()
        },
        after: {
            let at7 = json::nested(at, "after");
            let member7 = json::member_at(value, at, "after")?;
            json::text_at(member7, &at7, "a string")?.to_owned()
        },
        findings: {
            let at8 = json::nested(at, "findings");
            let member8 = json::member_at(value, at, "findings")?;
            {
                let mut items8 = Vec::new();
                for (index8, element8) in json::items_at(member8, &at8, "an array")?.iter().enumerate() {
                    let nested8 = json::nested(&at8, &index8.to_string());
                    items8.push(decode_uilab_wire_finding(element8, &nested8)?);
                }
                items8
            }
        },
        outline: {
            let at9 = json::nested(at, "outline");
            let member9 = json::member_at(value, at, "outline")?;
            decode_uilab_wire_outline_node(member9, &at9)?
        },
    })
}

/// Writes `uilab.wire.ReadRows` as JSON.
pub fn encode_uilab_wire_read_rows(value: &uilab_types::wire::ReadRows, out: &mut String) {
    out.push('{');
    json::member(out, "view");
    json::push_text(out, &value.view);
    out.push('}');
}

/// Reads `uilab.wire.ReadRows` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_read_rows(value: &json::Value, at: &str) -> Result<uilab_types::wire::ReadRows, json::DecodeError> {
    Ok(uilab_types::wire::ReadRows {
        view: {
            let at0 = json::nested(at, "view");
            let member0 = json::member_at(value, at, "view")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
    })
}

/// Writes `uilab.wire.Refused` as JSON.
pub fn encode_uilab_wire_refused(value: &uilab_types::wire::Refused, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.by {
        json::member(out, "by");
        json::push_text(out, &*held0);
    }
    json::member(out, "check");
    json::push_text(out, &value.check);
    json::member(out, "message");
    json::push_text(out, &value.message);
    out.push('}');
}

/// Reads `uilab.wire.Refused` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_refused(value: &json::Value, at: &str) -> Result<uilab_types::wire::Refused, json::DecodeError> {
    Ok(uilab_types::wire::Refused {
        by: match value.member("by") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "by");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        check: {
            let at1 = json::nested(at, "check");
            let member1 = json::member_at(value, at, "check")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        message: {
            let at2 = json::nested(at, "message");
            let member2 = json::member_at(value, at, "message")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
    })
}

/// Writes `uilab.wire.Resync` as JSON.
pub fn encode_uilab_wire_resync(value: &uilab_types::wire::Resync, out: &mut String) {
    out.push('{');
    json::member(out, "revision");
    json::push_integer(out, value.revision);
    out.push('}');
}

/// Reads `uilab.wire.Resync` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_resync(value: &json::Value, at: &str) -> Result<uilab_types::wire::Resync, json::DecodeError> {
    Ok(uilab_types::wire::Resync {
        revision: {
            let at0 = json::nested(at, "revision");
            let member0 = json::member_at(value, at, "revision")?;
            json::integer_at(member0, &at0, "an integer")?
        },
    })
}

/// Writes `uilab.wire.Rows` as JSON.
pub fn encode_uilab_wire_rows(value: &uilab_types::wire::Rows, out: &mut String) {
    out.push('{');
    json::member(out, "view");
    json::push_text(out, &value.view);
    if let Some(held0) = &value.total {
        json::member(out, "total");
        json::push_integer(out, *held0);
    }
    json::member(out, "rows");
    out.push('[');
    for (index0, item0) in value.rows.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_value(out, &*item0);
    }
    out.push(']');
    out.push('}');
}

/// Reads `uilab.wire.Rows` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_rows(value: &json::Value, at: &str) -> Result<uilab_types::wire::Rows, json::DecodeError> {
    Ok(uilab_types::wire::Rows {
        view: {
            let at0 = json::nested(at, "view");
            let member0 = json::member_at(value, at, "view")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        total: match value.member("total") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "total");
                Some(json::integer_at(member1, &at1, "an integer")?)
            }
        },
        rows: {
            let at2 = json::nested(at, "rows");
            let member2 = json::member_at(value, at, "rows")?;
            {
                let mut items2 = Vec::new();
                for (index2, element2) in json::items_at(member2, &at2, "an array")?.iter().enumerate() {
                    let nested2 = json::nested(&at2, &index2.to_string());
                    items2.push(json::value_at(element2, &nested2));
                }
                items2
            }
        },
    })
}

/// Writes `uilab.wire.Say` as JSON.
pub fn encode_uilab_wire_say(value: &uilab_types::wire::Say, out: &mut String) {
    out.push('{');
    json::member(out, "text");
    json::push_text(out, &value.text);
    if let Some(held0) = &value.target {
        json::member(out, "target");
        encode_uilab_session_node_path(&*held0, out);
    }
    if let Some(held0) = &value.review {
        json::member(out, "review");
        json::push_bool(out, *held0);
    }
    if let Some(held0) = &value.workspace {
        json::member(out, "workspace");
        encode_uilab_wire_workspace(&*held0, out);
    }
    out.push('}');
}

/// Reads `uilab.wire.Say` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_say(value: &json::Value, at: &str) -> Result<uilab_types::wire::Say, json::DecodeError> {
    Ok(uilab_types::wire::Say {
        text: {
            let at0 = json::nested(at, "text");
            let member0 = json::member_at(value, at, "text")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        target: match value.member("target") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "target");
                Some(decode_uilab_session_node_path(member1, &at1)?)
            }
        },
        review: match value.member("review") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "review");
                Some(json::bool_at(member2, &at2, "a boolean")?)
            }
        },
        workspace: match value.member("workspace") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "workspace");
                Some(decode_uilab_wire_workspace(member3, &at3)?)
            }
        },
    })
}

/// Writes `uilab.wire.Select` as JSON.
pub fn encode_uilab_wire_select(value: &uilab_types::wire::Select, out: &mut String) {
    out.push('{');
    json::member(out, "path");
    encode_uilab_session_node_path(&value.path, out);
    out.push('}');
}

/// Reads `uilab.wire.Select` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_select(value: &json::Value, at: &str) -> Result<uilab_types::wire::Select, json::DecodeError> {
    Ok(uilab_types::wire::Select {
        path: {
            let at0 = json::nested(at, "path");
            let member0 = json::member_at(value, at, "path")?;
            decode_uilab_session_node_path(member0, &at0)?
        },
    })
}

/// Writes `uilab.wire.ServerMessage` as JSON.
pub fn encode_uilab_wire_server_message(value: &uilab_types::wire::ServerMessage, out: &mut String) {
    match value {
        uilab_types::wire::ServerMessage::Changed(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "changed");
            json::member(out, "value");
            encode_uilab_wire_changed(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ServerMessage::Document(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "document");
            json::member(out, "value");
            encode_uilab_wire_document_state(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ServerMessage::Failed(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "failed");
            json::member(out, "value");
            encode_uilab_wire_failed(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ServerMessage::Goal(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "goal");
            json::member(out, "value");
            encode_uilab_wire_goal(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ServerMessage::Moved(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "moved");
            json::member(out, "value");
            encode_uilab_wire_moved(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ServerMessage::Presence(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "presence");
            json::member(out, "value");
            encode_uilab_wire_presence(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ServerMessage::Proposal(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "proposal");
            json::member(out, "value");
            encode_uilab_wire_proposal_shown(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ServerMessage::Refused(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "refused");
            json::member(out, "value");
            encode_uilab_wire_refused(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ServerMessage::Rows(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "rows");
            json::member(out, "value");
            encode_uilab_wire_rows(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ServerMessage::Thinking(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "thinking");
            json::member(out, "value");
            encode_uilab_wire_thinking(&*held, out);
            out.push('}');
        }
        uilab_types::wire::ServerMessage::Transcript(held) => {
            out.push('{');
            json::member(out, "type");
            json::push_text(out, "transcript");
            json::member(out, "value");
            encode_uilab_wire_transcript(&*held, out);
            out.push('}');
        }
    }
}

/// Reads `uilab.wire.ServerMessage` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_server_message(value: &json::Value, at: &str) -> Result<uilab_types::wire::ServerMessage, json::DecodeError> {
    let tag = json::member_at(value, at, "type")?;
    let at_tag = json::nested(at, "type");
    Ok(match json::text_at(tag, &at_tag, "one of `changed`, `document`, `failed`, `goal`, `moved`, `presence`, `proposal`, `refused`, `rows`, `thinking`, `transcript`")? {
        "changed" => uilab_types::wire::ServerMessage::Changed({
            let at0 = json::nested(at, "value");
            let member0 = json::member_at(value, at, "value")?;
            decode_uilab_wire_changed(member0, &at0)?
        }),
        "document" => uilab_types::wire::ServerMessage::Document({
            let at1 = json::nested(at, "value");
            let member1 = json::member_at(value, at, "value")?;
            decode_uilab_wire_document_state(member1, &at1)?
        }),
        "failed" => uilab_types::wire::ServerMessage::Failed({
            let at2 = json::nested(at, "value");
            let member2 = json::member_at(value, at, "value")?;
            decode_uilab_wire_failed(member2, &at2)?
        }),
        "goal" => uilab_types::wire::ServerMessage::Goal({
            let at3 = json::nested(at, "value");
            let member3 = json::member_at(value, at, "value")?;
            decode_uilab_wire_goal(member3, &at3)?
        }),
        "moved" => uilab_types::wire::ServerMessage::Moved({
            let at4 = json::nested(at, "value");
            let member4 = json::member_at(value, at, "value")?;
            decode_uilab_wire_moved(member4, &at4)?
        }),
        "presence" => uilab_types::wire::ServerMessage::Presence({
            let at5 = json::nested(at, "value");
            let member5 = json::member_at(value, at, "value")?;
            decode_uilab_wire_presence(member5, &at5)?
        }),
        "proposal" => uilab_types::wire::ServerMessage::Proposal({
            let at6 = json::nested(at, "value");
            let member6 = json::member_at(value, at, "value")?;
            decode_uilab_wire_proposal_shown(member6, &at6)?
        }),
        "refused" => uilab_types::wire::ServerMessage::Refused({
            let at7 = json::nested(at, "value");
            let member7 = json::member_at(value, at, "value")?;
            decode_uilab_wire_refused(member7, &at7)?
        }),
        "rows" => uilab_types::wire::ServerMessage::Rows({
            let at8 = json::nested(at, "value");
            let member8 = json::member_at(value, at, "value")?;
            decode_uilab_wire_rows(member8, &at8)?
        }),
        "thinking" => uilab_types::wire::ServerMessage::Thinking({
            let at9 = json::nested(at, "value");
            let member9 = json::member_at(value, at, "value")?;
            decode_uilab_wire_thinking(member9, &at9)?
        }),
        "transcript" => uilab_types::wire::ServerMessage::Transcript({
            let at10 = json::nested(at, "value");
            let member10 = json::member_at(value, at, "value")?;
            decode_uilab_wire_transcript(member10, &at10)?
        }),
        other => return Err(json::DecodeError { at: at_tag.clone(), expected: "one of `changed`, `document`, `failed`, `goal`, `moved`, `presence`, `proposal`, `refused`, `rows`, `thinking`, `transcript`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `uilab.wire.Settings` as JSON.
pub fn encode_uilab_wire_settings(value: &uilab_types::wire::Settings, out: &mut String) {
    out.push('{');
    json::member(out, "review");
    json::push_bool(out, value.review);
    out.push('}');
}

/// Reads `uilab.wire.Settings` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_settings(value: &json::Value, at: &str) -> Result<uilab_types::wire::Settings, json::DecodeError> {
    Ok(uilab_types::wire::Settings {
        review: {
            let at0 = json::nested(at, "review");
            let member0 = json::member_at(value, at, "review")?;
            json::bool_at(member0, &at0, "a boolean")?
        },
    })
}

/// Writes `uilab.wire.Severity` as JSON.
pub fn encode_uilab_wire_severity(value: &uilab_types::wire::Severity, out: &mut String) {
    match value {
        uilab_types::wire::Severity::Error => json::push_text(out, "error"),
        uilab_types::wire::Severity::Warning => json::push_text(out, "warning"),
    }
}

/// Reads `uilab.wire.Severity` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_severity(value: &json::Value, at: &str) -> Result<uilab_types::wire::Severity, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `error`, `warning`")? {
        "error" => uilab_types::wire::Severity::Error,
        "warning" => uilab_types::wire::Severity::Warning,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `error`, `warning`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `uilab.wire.StartGoal` as JSON.
pub fn encode_uilab_wire_start_goal(value: &uilab_types::wire::StartGoal, out: &mut String) {
    out.push('{');
    json::member(out, "text");
    json::push_text(out, &value.text);
    if let Some(held0) = &value.target {
        json::member(out, "target");
        encode_uilab_session_node_path(&*held0, out);
    }
    if let Some(held0) = &value.max_steps {
        json::member(out, "max_steps");
        json::push_integer(out, *held0);
    }
    if let Some(held0) = &value.review {
        json::member(out, "review");
        json::push_bool(out, *held0);
    }
    if let Some(held0) = &value.workspace {
        json::member(out, "workspace");
        encode_uilab_wire_workspace(&*held0, out);
    }
    out.push('}');
}

/// Reads `uilab.wire.StartGoal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_start_goal(value: &json::Value, at: &str) -> Result<uilab_types::wire::StartGoal, json::DecodeError> {
    Ok(uilab_types::wire::StartGoal {
        text: {
            let at0 = json::nested(at, "text");
            let member0 = json::member_at(value, at, "text")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        target: match value.member("target") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "target");
                Some(decode_uilab_session_node_path(member1, &at1)?)
            }
        },
        max_steps: match value.member("max_steps") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "max_steps");
                Some(json::integer_at(member2, &at2, "an integer")?)
            }
        },
        review: match value.member("review") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "review");
                Some(json::bool_at(member3, &at3, "a boolean")?)
            }
        },
        workspace: match value.member("workspace") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "workspace");
                Some(decode_uilab_wire_workspace(member4, &at4)?)
            }
        },
    })
}

/// Writes `uilab.wire.StepStatus` as JSON.
pub fn encode_uilab_wire_step_status(value: &uilab_types::wire::StepStatus, out: &mut String) {
    match value {
        uilab_types::wire::StepStatus::Pending => json::push_text(out, "pending"),
        uilab_types::wire::StepStatus::Thinking => json::push_text(out, "thinking"),
        uilab_types::wire::StepStatus::Proposed => json::push_text(out, "proposed"),
        uilab_types::wire::StepStatus::Accepted => json::push_text(out, "accepted"),
        uilab_types::wire::StepStatus::Rejected => json::push_text(out, "rejected"),
        uilab_types::wire::StepStatus::Refused => json::push_text(out, "refused"),
        uilab_types::wire::StepStatus::Declined => json::push_text(out, "declined"),
    }
}

/// Reads `uilab.wire.StepStatus` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_step_status(value: &json::Value, at: &str) -> Result<uilab_types::wire::StepStatus, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `pending`, `thinking`, `proposed`, `accepted`, `rejected`, `refused`, `declined`")? {
        "pending" => uilab_types::wire::StepStatus::Pending,
        "thinking" => uilab_types::wire::StepStatus::Thinking,
        "proposed" => uilab_types::wire::StepStatus::Proposed,
        "accepted" => uilab_types::wire::StepStatus::Accepted,
        "rejected" => uilab_types::wire::StepStatus::Rejected,
        "refused" => uilab_types::wire::StepStatus::Refused,
        "declined" => uilab_types::wire::StepStatus::Declined,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `pending`, `thinking`, `proposed`, `accepted`, `rejected`, `refused`, `declined`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `uilab.wire.StopGoal` as JSON.
pub fn encode_uilab_wire_stop_goal(value: &uilab_types::wire::StopGoal, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id);
    out.push('}');
}

/// Reads `uilab.wire.StopGoal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_stop_goal(value: &json::Value, at: &str) -> Result<uilab_types::wire::StopGoal, json::DecodeError> {
    Ok(uilab_types::wire::StopGoal {
        goal_id: {
            let at0 = json::nested(at, "goal_id");
            let member0 = json::member_at(value, at, "goal_id")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
    })
}

/// Writes `uilab.wire.Thinking` as JSON.
pub fn encode_uilab_wire_thinking(value: &uilab_types::wire::Thinking, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.by {
        json::member(out, "by");
        json::push_text(out, &*held0);
    }
    json::member(out, "target");
    encode_uilab_session_node_path(&value.target, out);
    out.push('}');
}

/// Reads `uilab.wire.Thinking` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_thinking(value: &json::Value, at: &str) -> Result<uilab_types::wire::Thinking, json::DecodeError> {
    Ok(uilab_types::wire::Thinking {
        by: match value.member("by") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "by");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        target: {
            let at1 = json::nested(at, "target");
            let member1 = json::member_at(value, at, "target")?;
            decode_uilab_session_node_path(member1, &at1)?
        },
    })
}

/// Writes `uilab.wire.Transcript` as JSON.
pub fn encode_uilab_wire_transcript(value: &uilab_types::wire::Transcript, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.by {
        json::member(out, "by");
        json::push_text(out, &*held0);
    }
    json::member(out, "text");
    json::push_text(out, &value.text);
    json::member(out, "audio_ms");
    json::push_integer(out, value.audio_ms);
    json::member(out, "took_ms");
    json::push_integer(out, value.took_ms);
    out.push('}');
}

/// Reads `uilab.wire.Transcript` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_transcript(value: &json::Value, at: &str) -> Result<uilab_types::wire::Transcript, json::DecodeError> {
    Ok(uilab_types::wire::Transcript {
        by: match value.member("by") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "by");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        text: {
            let at1 = json::nested(at, "text");
            let member1 = json::member_at(value, at, "text")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        audio_ms: {
            let at2 = json::nested(at, "audio_ms");
            let member2 = json::member_at(value, at, "audio_ms")?;
            json::integer_at(member2, &at2, "an integer")?
        },
        took_ms: {
            let at3 = json::nested(at, "took_ms");
            let member3 = json::member_at(value, at, "took_ms")?;
            json::integer_at(member3, &at3, "an integer")?
        },
    })
}

/// Writes `uilab.wire.Workspace` as JSON.
pub fn encode_uilab_wire_workspace(value: &uilab_types::wire::Workspace, out: &mut String) {
    match value {
        uilab_types::wire::Workspace::App => json::push_text(out, "app"),
        uilab_types::wire::Workspace::Components => json::push_text(out, "components"),
    }
}

/// Reads `uilab.wire.Workspace` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_uilab_wire_workspace(value: &json::Value, at: &str) -> Result<uilab_types::wire::Workspace, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `app`, `components`")? {
        "app" => uilab_types::wire::Workspace::App,
        "components" => uilab_types::wire::Workspace::Components,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `app`, `components`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes the event `uilab.session.DocumentOpened` as JSON.
pub fn encode_event_uilab_session_document_opened(value: &uilab_types::session::DocumentOpened, out: &mut String) {
    out.push('{');
    json::member(out, "document_id");
    encode_uilab_session_document_id(&value.document_id, out);
    json::member(out, "path");
    json::push_text(out, &value.path);
    out.push('}');
}

/// Writes the event `uilab.session.NodeSelected` as JSON.
pub fn encode_event_uilab_session_node_selected(value: &uilab_types::session::NodeSelected, out: &mut String) {
    out.push('{');
    json::member(out, "document_id");
    encode_uilab_session_document_id(&value.document_id, out);
    json::member(out, "path");
    encode_uilab_session_node_path(&value.path, out);
    out.push('}');
}

/// Writes the event `uilab.session.PatchProposed` as JSON.
pub fn encode_event_uilab_session_patch_proposed(value: &uilab_types::session::PatchProposed, out: &mut String) {
    out.push('{');
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    json::member(out, "document_id");
    encode_uilab_session_document_id(&value.document_id, out);
    json::member(out, "target");
    encode_uilab_session_node_path(&value.target, out);
    json::member(out, "op");
    encode_uilab_session_patch_op(&value.op, out);
    out.push('}');
}

/// Writes the event `uilab.session.ProposalAccepted` as JSON.
pub fn encode_event_uilab_session_proposal_accepted(value: &uilab_types::session::ProposalAccepted, out: &mut String) {
    out.push('{');
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    out.push('}');
}

/// Writes the event `uilab.session.ProposalRejected` as JSON.
pub fn encode_event_uilab_session_proposal_rejected(value: &uilab_types::session::ProposalRejected, out: &mut String) {
    out.push('{');
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    out.push('}');
}

/// Writes the event `uilab.session.ProposalUndone` as JSON.
pub fn encode_event_uilab_session_proposal_undone(value: &uilab_types::session::ProposalUndone, out: &mut String) {
    out.push('{');
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    out.push('}');
}

/// Writes the declared error `uilab.session.DocumentNotOpen` as JSON.
pub fn encode_error_uilab_session_document_not_open(value: &uilab_types::session::DocumentNotOpen, out: &mut String) {
    out.push('{');
    json::member(out, "document_id");
    encode_uilab_session_document_id(&value.document_id, out);
    out.push('}');
}

/// Writes the declared error `uilab.session.DocumentUnreadable` as JSON.
pub fn encode_error_uilab_session_document_unreadable(value: &uilab_types::session::DocumentUnreadable, out: &mut String) {
    out.push('{');
    json::member(out, "path");
    json::push_text(out, &value.path);
    out.push('}');
}

/// Writes the declared error `uilab.session.NodeNotFound` as JSON.
pub fn encode_error_uilab_session_node_not_found(value: &uilab_types::session::NodeNotFound, out: &mut String) {
    out.push('{');
    json::member(out, "path");
    encode_uilab_session_node_path(&value.path, out);
    out.push('}');
}

/// Writes the declared error `uilab.session.PatchRefused` as JSON.
pub fn encode_error_uilab_session_patch_refused(value: &uilab_types::session::PatchRefused, out: &mut String) {
    out.push('{');
    json::member(out, "check");
    json::push_text(out, &value.check);
    out.push('}');
}

/// Writes the declared error `uilab.session.ProposalStateConflict` as JSON.
pub fn encode_error_uilab_session_proposal_state_conflict(value: &uilab_types::session::ProposalStateConflict, out: &mut String) {
    out.push('{');
    json::member(out, "state");
    encode_uilab_session_proposal_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `uilab.session.Documents` as JSON.
pub fn encode_view_uilab_session_documents(value: &uilab_types::session::Documents, out: &mut String) {
    out.push('{');
    json::member(out, "document_id");
    encode_uilab_session_document_id(&value.document_id, out);
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "selected");
    encode_uilab_session_node_path(&value.selected, out);
    out.push('}');
}

/// Writes one row of the view `uilab.session.Pending` as JSON.
pub fn encode_view_uilab_session_pending(value: &uilab_types::session::Pending, out: &mut String) {
    out.push('{');
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    json::member(out, "target");
    encode_uilab_session_node_path(&value.target, out);
    json::member(out, "op");
    encode_uilab_session_patch_op(&value.op, out);
    out.push('}');
}

/// Writes one row of the view `uilab.session.Proposals` as JSON.
pub fn encode_view_uilab_session_proposals(value: &uilab_types::session::Proposals, out: &mut String) {
    out.push('{');
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    json::member(out, "document_id");
    encode_uilab_session_document_id(&value.document_id, out);
    json::member(out, "target");
    encode_uilab_session_node_path(&value.target, out);
    json::member(out, "op");
    encode_uilab_session_patch_op(&value.op, out);
    json::member(out, "utterance");
    json::push_text(out, &value.utterance);
    json::member(out, "state");
    encode_uilab_session_proposal_state(&value.state, out);
    out.push('}');
}

/// Writes the input of `uilab.session.AcceptProposal` as JSON.
pub fn encode_command_uilab_session_accept_proposal(value: &uilab_types::session::AcceptProposal, out: &mut String) {
    out.push('{');
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    out.push('}');
}

/// Reads the input of `uilab.session.AcceptProposal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_uilab_session_accept_proposal(value: &json::Value, at: &str) -> Result<uilab_types::session::AcceptProposal, json::DecodeError> {
    Ok(uilab_types::session::AcceptProposal {
        proposal_id: {
            let at0 = json::nested(at, "proposal_id");
            let member0 = json::member_at(value, at, "proposal_id")?;
            decode_uilab_session_proposal_id(member0, &at0)?
        },
    })
}

/// Writes the outcome of `uilab.session.AcceptProposal` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_uilab_session_accept_proposal(value: &uilab_types::session::AcceptProposalOutcome, out: &mut String) {
    out.push('{');
    match value {
        uilab_types::session::AcceptProposalOutcome::Accepted { proposal_accepted } => {
            json::member(out, "outcome");
            json::push_text(out, "accepted");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "uilab.session.ProposalAccepted");
            json::member(out, "payload");
            encode_event_uilab_session_proposal_accepted(proposal_accepted, out);
            out.push('}');
            out.push(']');
        }
        uilab_types::session::AcceptProposalOutcome::Stale { error } => {
            json::member(out, "outcome");
            json::push_text(out, "stale");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.PatchRefused");
            json::member(out, "payload");
            encode_error_uilab_session_patch_refused(error, out);
            out.push('}');
        }
        uilab_types::session::AcceptProposalOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.ProposalStateConflict");
            json::member(out, "payload");
            encode_error_uilab_session_proposal_state_conflict(error, out);
            out.push('}');
        }
        uilab_types::session::AcceptProposalOutcome::WrongStateUnknownInstance => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.ProposalStateConflict");
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `uilab.session.OpenDocument` as JSON.
pub fn encode_command_uilab_session_open_document(value: &uilab_types::session::OpenDocument, out: &mut String) {
    out.push('{');
    json::member(out, "path");
    json::push_text(out, &value.path);
    out.push('}');
}

/// Reads the input of `uilab.session.OpenDocument` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_uilab_session_open_document(value: &json::Value, at: &str) -> Result<uilab_types::session::OpenDocument, json::DecodeError> {
    Ok(uilab_types::session::OpenDocument {
        path: {
            let at0 = json::nested(at, "path");
            let member0 = json::member_at(value, at, "path")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
    })
}

/// Writes the outcome of `uilab.session.OpenDocument` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_uilab_session_open_document(value: &uilab_types::session::OpenDocumentOutcome, out: &mut String) {
    out.push('{');
    match value {
        uilab_types::session::OpenDocumentOutcome::Opened { document_opened } => {
            json::member(out, "outcome");
            json::push_text(out, "opened");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "uilab.session.DocumentOpened");
            json::member(out, "payload");
            encode_event_uilab_session_document_opened(document_opened, out);
            out.push('}');
            out.push(']');
        }
        uilab_types::session::OpenDocumentOutcome::Unreadable { error } => {
            json::member(out, "outcome");
            json::push_text(out, "unreadable");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.DocumentUnreadable");
            json::member(out, "payload");
            encode_error_uilab_session_document_unreadable(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `uilab.session.ProposePatch` as JSON.
pub fn encode_command_uilab_session_propose_patch(value: &uilab_types::session::ProposePatch, out: &mut String) {
    out.push('{');
    json::member(out, "document_id");
    encode_uilab_session_document_id(&value.document_id, out);
    json::member(out, "target");
    encode_uilab_session_node_path(&value.target, out);
    json::member(out, "op");
    encode_uilab_session_patch_op(&value.op, out);
    json::member(out, "utterance");
    json::push_text(out, &value.utterance);
    if let Some(held0) = &value.body {
        json::member(out, "body");
        encode_uilab_session_patch_body(&*held0, out);
    }
    out.push('}');
}

/// Reads the input of `uilab.session.ProposePatch` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_uilab_session_propose_patch(value: &json::Value, at: &str) -> Result<uilab_types::session::ProposePatch, json::DecodeError> {
    Ok(uilab_types::session::ProposePatch {
        document_id: {
            let at0 = json::nested(at, "document_id");
            let member0 = json::member_at(value, at, "document_id")?;
            decode_uilab_session_document_id(member0, &at0)?
        },
        target: {
            let at1 = json::nested(at, "target");
            let member1 = json::member_at(value, at, "target")?;
            decode_uilab_session_node_path(member1, &at1)?
        },
        op: {
            let at2 = json::nested(at, "op");
            let member2 = json::member_at(value, at, "op")?;
            decode_uilab_session_patch_op(member2, &at2)?
        },
        utterance: {
            let at3 = json::nested(at, "utterance");
            let member3 = json::member_at(value, at, "utterance")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
        body: match value.member("body") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "body");
                Some(decode_uilab_session_patch_body(member4, &at4)?)
            }
        },
    })
}

/// Writes the outcome of `uilab.session.ProposePatch` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_uilab_session_propose_patch(value: &uilab_types::session::ProposePatchOutcome, out: &mut String) {
    out.push('{');
    match value {
        uilab_types::session::ProposePatchOutcome::Proposed { patch_proposed } => {
            json::member(out, "outcome");
            json::push_text(out, "proposed");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "uilab.session.PatchProposed");
            json::member(out, "payload");
            encode_event_uilab_session_patch_proposed(patch_proposed, out);
            out.push('}');
            out.push(']');
        }
        uilab_types::session::ProposePatchOutcome::Refused { error } => {
            json::member(out, "outcome");
            json::push_text(out, "refused");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.PatchRefused");
            json::member(out, "payload");
            encode_error_uilab_session_patch_refused(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `uilab.session.RejectProposal` as JSON.
pub fn encode_command_uilab_session_reject_proposal(value: &uilab_types::session::RejectProposal, out: &mut String) {
    out.push('{');
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    out.push('}');
}

/// Reads the input of `uilab.session.RejectProposal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_uilab_session_reject_proposal(value: &json::Value, at: &str) -> Result<uilab_types::session::RejectProposal, json::DecodeError> {
    Ok(uilab_types::session::RejectProposal {
        proposal_id: {
            let at0 = json::nested(at, "proposal_id");
            let member0 = json::member_at(value, at, "proposal_id")?;
            decode_uilab_session_proposal_id(member0, &at0)?
        },
    })
}

/// Writes the outcome of `uilab.session.RejectProposal` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_uilab_session_reject_proposal(value: &uilab_types::session::RejectProposalOutcome, out: &mut String) {
    out.push('{');
    match value {
        uilab_types::session::RejectProposalOutcome::Rejected { proposal_rejected } => {
            json::member(out, "outcome");
            json::push_text(out, "rejected");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "uilab.session.ProposalRejected");
            json::member(out, "payload");
            encode_event_uilab_session_proposal_rejected(proposal_rejected, out);
            out.push('}');
            out.push(']');
        }
        uilab_types::session::RejectProposalOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.ProposalStateConflict");
            json::member(out, "payload");
            encode_error_uilab_session_proposal_state_conflict(error, out);
            out.push('}');
        }
        uilab_types::session::RejectProposalOutcome::WrongStateUnknownInstance => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.ProposalStateConflict");
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `uilab.session.SelectNode` as JSON.
pub fn encode_command_uilab_session_select_node(value: &uilab_types::session::SelectNode, out: &mut String) {
    out.push('{');
    json::member(out, "document_id");
    encode_uilab_session_document_id(&value.document_id, out);
    json::member(out, "path");
    encode_uilab_session_node_path(&value.path, out);
    out.push('}');
}

/// Reads the input of `uilab.session.SelectNode` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_uilab_session_select_node(value: &json::Value, at: &str) -> Result<uilab_types::session::SelectNode, json::DecodeError> {
    Ok(uilab_types::session::SelectNode {
        document_id: {
            let at0 = json::nested(at, "document_id");
            let member0 = json::member_at(value, at, "document_id")?;
            decode_uilab_session_document_id(member0, &at0)?
        },
        path: {
            let at1 = json::nested(at, "path");
            let member1 = json::member_at(value, at, "path")?;
            decode_uilab_session_node_path(member1, &at1)?
        },
    })
}

/// Writes the outcome of `uilab.session.SelectNode` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_uilab_session_select_node(value: &uilab_types::session::SelectNodeOutcome, out: &mut String) {
    out.push('{');
    match value {
        uilab_types::session::SelectNodeOutcome::Selected { node_selected } => {
            json::member(out, "outcome");
            json::push_text(out, "selected");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "uilab.session.NodeSelected");
            json::member(out, "payload");
            encode_event_uilab_session_node_selected(node_selected, out);
            out.push('}');
            out.push(']');
        }
        uilab_types::session::SelectNodeOutcome::UnknownDocument { error } => {
            json::member(out, "outcome");
            json::push_text(out, "unknown-document");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.DocumentNotOpen");
            json::member(out, "payload");
            encode_error_uilab_session_document_not_open(error, out);
            out.push('}');
        }
        uilab_types::session::SelectNodeOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.NodeNotFound");
            json::member(out, "payload");
            encode_error_uilab_session_node_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `uilab.session.UndoProposal` as JSON.
pub fn encode_command_uilab_session_undo_proposal(value: &uilab_types::session::UndoProposal, out: &mut String) {
    out.push('{');
    json::member(out, "proposal_id");
    encode_uilab_session_proposal_id(&value.proposal_id, out);
    out.push('}');
}

/// Reads the input of `uilab.session.UndoProposal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_uilab_session_undo_proposal(value: &json::Value, at: &str) -> Result<uilab_types::session::UndoProposal, json::DecodeError> {
    Ok(uilab_types::session::UndoProposal {
        proposal_id: {
            let at0 = json::nested(at, "proposal_id");
            let member0 = json::member_at(value, at, "proposal_id")?;
            decode_uilab_session_proposal_id(member0, &at0)?
        },
    })
}

/// Writes the outcome of `uilab.session.UndoProposal` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_uilab_session_undo_proposal(value: &uilab_types::session::UndoProposalOutcome, out: &mut String) {
    out.push('{');
    match value {
        uilab_types::session::UndoProposalOutcome::Undone { proposal_undone } => {
            json::member(out, "outcome");
            json::push_text(out, "undone");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "uilab.session.ProposalUndone");
            json::member(out, "payload");
            encode_event_uilab_session_proposal_undone(proposal_undone, out);
            out.push('}');
            out.push(']');
        }
        uilab_types::session::UndoProposalOutcome::Stale { error } => {
            json::member(out, "outcome");
            json::push_text(out, "stale");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.PatchRefused");
            json::member(out, "payload");
            encode_error_uilab_session_patch_refused(error, out);
            out.push('}');
        }
        uilab_types::session::UndoProposalOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.ProposalStateConflict");
            json::member(out, "payload");
            encode_error_uilab_session_proposal_state_conflict(error, out);
            out.push('}');
        }
        uilab_types::session::UndoProposalOutcome::WrongStateUnknownInstance => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "uilab.session.ProposalStateConflict");
            out.push('}');
        }
    }
    out.push('}');
}
