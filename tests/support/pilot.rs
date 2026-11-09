//! Test-only replacement covenant for the durable journal suite.
use temple3::{
    Error, Result, commitment, identity,
    journal::FileJournal,
    kernel,
    predicate::{eq, literal, read},
    replica::Replica,
};
use serde_json::{Value, json};
pub const SEED: [u8; 32] = [7; 32]; // PUBLIC local pilot fixture, never an operational key.
pub const CONTEXT: &str = "temple3.property-edit-pilot/v0.1";
fn body(value: &str, parent: &str) -> Value {
    json!({"kind":CONTEXT,"subject":"session.one","property":"theme","value":value,"parentStateRoot":parent})
}
pub fn policy() -> Value {
    let r = |p: &[&str]| read(p);
    let l = |v: Value| literal(v);
    let mut checks = vec![
        eq(r(&["identity"]), l(json!("editor"))),
        eq(r(&["context"]), l(json!({"application":CONTEXT}))),
        eq(r(&["delta", "nextPredicate"]), l(Value::Null)),
        eq(r(&["delta", "read"]), r(&["delta", "consume"])),
        eq(
            r(&["delta", "consume", "0"]),
            r(&["basis", "cells", "0", "id"]),
        ),
        eq(r(&["delta", "consume", "1"]), l(Value::Null)),
        eq(r(&["basis", "cells", "1"]), l(Value::Null)),
        eq(r(&["delta", "create", "1"]), l(Value::Null)),
        eq(
            r(&["delta", "create", "0", "value", "parentStateRoot"]),
            r(&["basis", "stateRoot"]),
        ),
        json!({"op":"not","value":eq(r(&["basis","cells","0","value","value"]),r(&["delta","create","0","value","value"]))}),
    ];
    for prefix in [
        vec!["basis", "cells", "0", "value"],
        vec!["delta", "create", "0", "value"],
    ] {
        for (field, value) in [
            ("kind", CONTEXT),
            ("subject", "session.one"),
            ("property", "theme"),
        ] {
            let mut p = prefix.clone();
            p.push(field);
            checks.push(eq(r(&p), l(json!(value))));
        }
        let mut p = prefix;
        p.push("value");
        checks.push(
            json!({"op":"any","values":[eq(r(&p),l(json!("green"))),eq(r(&p),l(json!("amber")))]}),
        );
    }
    json!({"op":"all","values":checks})
}
pub fn genesis_with_seed(seed: &[u8; 32]) -> Result<Value> {
    let public = hex::encode(
        ed25519_dalek::SigningKey::from_bytes(seed)
            .verifying_key()
            .as_bytes(),
    );
    identity::signed_genesis(
        &json!({"activePredicate":policy(),"initialValues":[body("green","genesis"),{"kind":"pilot-note","text":"Preserve this unrelated cell"}],"context":{
            "identityProfile":{"kind":"temple3.identity-profile/v0.1","actors":[{"actor":"editor","publicKey":public}],"finalizer":public},
            "application":{"coordinationProfile":{"kind":"temple3.coordination-profile/v0.1","mode":"single-instance","members":[{"voter":"pilot-voter","publicKey":public}],"quorum":"1","faultBound":"0"},"application":{"profile":CONTEXT}}
        }}),
        seed,
    )
}
pub fn open(directory: &std::path::Path, tip: Option<&Value>) -> Result<Replica> {
    open_with_seed(directory, tip, &SEED)
}
pub fn open_with_seed(
    directory: &std::path::Path,
    tip: Option<&Value>,
    seed: &[u8; 32],
) -> Result<Replica> {
    let g = genesis_with_seed(seed)?;
    Replica::open(
        FileJournal::new(directory),
        g["signed"]["payload"]["templeId"].clone(),
        "pilot-voter".into(),
        *seed,
        tip,
    )
}
pub fn initialize(directory: &std::path::Path) -> Result<Replica> {
    initialize_with_seed(directory, &SEED)
}
pub fn initialize_with_seed(directory: &std::path::Path, seed: &[u8; 32]) -> Result<Replica> {
    let g = genesis_with_seed(seed)?;
    Replica::create(
        FileJournal::new(directory),
        &g,
        g["signed"]["payload"]["templeId"].clone(),
        "pilot-voter".into(),
        *seed,
    )
}
pub fn current_cell(state: &Value) -> Result<&Value> {
    let cells = array(&state["temple"]["state"]["cells"], "invalid-state")?;
    let mut matches = cells.iter().filter(|c| {
        c["value"]["kind"] == CONTEXT
            && c["value"]["subject"] == "session.one"
            && c["value"]["property"] == "theme"
    });
    let first = matches
        .next()
        .ok_or_else(|| err("pilot-property-unresolved"))?;
    require(matches.next().is_none(), "pilot-property-unresolved")?;
    Ok(first)
}
pub fn propose(state: &Value, input: &Value) -> Result<Value> {
    propose_with_seed(state, input, &SEED)
}
pub fn propose_with_seed(state: &Value, input: &Value, seed: &[u8; 32]) -> Result<Value> {
    shape(input, "value basisStateRoot requestId")?;
    let value = string(&input["value"], "pilot-value-denied")?;
    require(["green", "amber"].contains(&value), "pilot-value-denied")?;
    identity::token(&input["requestId"])?;
    let old = current_cell(state)?;
    require(old["value"]["value"] != value, "pilot-no-op")?;
    let parent = string(&state["temple"]["stateRoot"], "invalid-state")?;
    let cell = commitment::cell_for(&body(value, parent))?;
    let mut delta = kernel::build_delta(
        &state["temple"],
        &json!({"read":[old["id"]],"consume":[old["id"]],"create":[cell]}),
    )?;
    require(input["basisStateRoot"].is_string(), "invalid-delta")?;
    delta["basisStateRoot"] = input["basisStateRoot"].clone();
    identity::sign_envelope(
        "proposal",
        &json!({"templeId":state["temple"]["templeId"],"actor":"editor","requestId":input["requestId"],"delta":delta,"context":{"application":CONTEXT}}),
        seed,
    )
}
pub fn preview(state: &Value, p: &Value) -> Result<Value> {
    identity::authenticate(&state["identityProfile"], &state["temple"]["templeId"], p)?;
    let r = kernel::evaluate_transition(
        &state["temple"],
        &json!({"identity":p["payload"]["actor"],"delta":p["payload"]["delta"],"context":p["payload"]["context"]}),
    )?;
    Ok(
        json!({"status":if r.accepted{"ready"}else{"rejected"},"reason":r.witness.get("reason"),"proposal":p,"proposalId":identity::signed_id(p)?,"witness":r.witness}),
    )
}
pub fn commit(replica: &Replica, p: &Value) -> Result<Value> {
    let state = replica.state(None)?;
    let pid = identity::signed_id(p)?;
    if array(&state["requests"], "invalid-state")?
        .iter()
        .any(|r| r["proposalId"] == pid)
    {
        return Ok(json!({"status":"replay"}));
    }
    require(
        array(&state["certificates"], "invalid-state")?.len() < 64,
        "pilot-history-limit",
    )?;
    let preview = preview(&state, p)?;
    if preview["status"] == "rejected" {
        return Ok(preview);
    }
    replica.cast_vote(p)?;
    replica.resolve()
}
fn err(code: &str) -> Error {
    Error { code: code.into() }
}

fn require(ok: bool, code: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(err(code)) }
}

fn array<'a>(value: &'a Value, code: &str) -> Result<&'a [Value]> {
    value.as_array().map(Vec::as_slice).ok_or_else(|| err(code))
}

fn string<'a>(value: &'a Value, code: &str) -> Result<&'a str> {
    value.as_str().ok_or_else(|| err(code))
}

fn shape(value: &Value, fields: &str) -> Result<()> {
    let fields: Vec<_> = fields.split_whitespace().collect();
    require(value.as_object().is_some_and(|object|
        object.len() == fields.len() && fields.iter().all(|key| object.contains_key(*key))),
        "invalid-envelope")
}
