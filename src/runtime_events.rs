//! Independent bounded shadow-event validation. Signatures confer no source authority.
use crate::{Result, array, canonical, err, identity, require, string};
use serde_json::{Value, json};

fn shape(v: &Value, fields: &str) -> Result<()> {
    crate::shape(v, fields).map_err(|_| err("event-shape"))
}
pub fn projection(facts: &Value) -> Result<Value> {
    let rows = array(facts, "projection-size")?;
    require(rows.len() <= 256, "projection-size")?;
    let mut keyed = Vec::new();
    for f in rows {
        shape(f, "s p o pol")?;
        require(
            ["s", "p", "o"].iter().all(|k| {
                f[*k]
                    .as_str()
                    .is_some_and(|s| !s.is_empty() && s.len() <= 256)
            }) && f["pol"] == "is",
            "projection-fact",
        )?;
        keyed.push((canonical::canonical_json(f)?, f.clone()));
    }
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    require(
        keyed.windows(2).all(|w| w[0].0 != w[1].0),
        "projection-duplicate",
    )?;
    Ok(Value::Array(keyed.into_iter().map(|(_, f)| f).collect()))
}
fn theme(facts: &Value) -> Result<&str> {
    let rows: Vec<_> = array(facts, "projection-size")?
        .iter()
        .filter(|f| f["s"] == "console.session" && f["p"] == "theme")
        .collect();
    require(rows.len() == 1, "event-theme-unresolved")?;
    let v = string(&rows[0]["o"], "event-theme-unresolved")?;
    require(matches!(v, "green" | "amber"), "event-theme-unresolved")?;
    Ok(v)
}
pub fn initial_cursor(contract: &Value) -> Result<Value> {
    shape(
        contract,
        "kind streamId publicKey baselineHash command initialProjection contractId",
    )?;
    require(
        contract["kind"] == "temple3.sourcery-runtime-contract/v0.1"
            && contract["command"] == "console.move.theme",
        "contract-shape",
    )?;
    let stream = string(&contract["streamId"], "stream-id")?;
    require(
        !stream.is_empty()
            && stream.len() <= 64
            && stream
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
        "stream-id",
    )?;
    identity::key_id(&contract["publicKey"])?;
    let baseline = string(&contract["baselineHash"], "baseline-hash")?;
    require(
        baseline.len() == 64
            && baseline
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "baseline-hash",
    )?;
    let facts = projection(&contract["initialProjection"])?;
    theme(&facts)?;
    let body = crate::without(contract, "contractId")?;
    require(
        facts == contract["initialProjection"]
            && canonical::identify("sourcery-runtime-contract", &body)? == contract["contractId"],
        "contract-mismatch",
    )?;
    Ok(json!({"revision":"0","eventId":"empty","projection":facts,"requests":[]}))
}
/// The cursor must be obtained from verified replay, never from untrusted wire data.
pub fn reconcile(contract: &Value, cursor: &Value, envelope: &Value) -> Result<Value> {
    require(
        canonical::canonical_json(envelope)?.len() <= 65536,
        "event-size",
    )?;
    require(
        identity::verify_envelope(envelope, "attestation", &contract["publicKey"]),
        "event-signature",
    )?;
    let p = &envelope["payload"];
    shape(
        p,
        "kind contractId streamId baselineHash revision previousEventId requestId command args outcome beforeProjection afterProjection effects",
    )?;
    let id = identity::signed_id(envelope)?;
    let requests = array(&cursor["requests"], "event-cursor")?;
    if let Some(old) = requests.iter().find(|r| r["requestId"] == p["requestId"]) {
        require(old["eventId"] == id, "event-request-conflict")?;
        return Ok(json!({"status":"replay","cursor":cursor,"receipt":old["receipt"]}));
    }
    require(requests.len() < 64, "event-history-limit")?;
    require(
        p["kind"] == "temple3.sourcery-runtime-event/v0.1"
            && ["contractId", "streamId", "baselineHash", "command"]
                .iter()
                .all(|k| p[*k] == contract[*k])
            && p["outcome"] == "committed",
        "event-scope",
    )?;
    let args = array(&p["args"], "event-arguments")?;
    require(
        args.len() == 2 && args[0] == "console.session",
        "event-arguments",
    )?;
    let revision =
        (crate::decimal(&cursor["revision"])? + num_bigint::BigUint::from(1u8)).to_string();
    require(
        p["revision"] == revision && p["previousEventId"] == cursor["eventId"],
        "event-continuity",
    )?;
    let request = string(&p["requestId"], "event-request")?;
    require(
        !request.is_empty()
            && request.len() <= 128
            && request
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b":_-".contains(&b)),
        "event-request",
    )?;
    let next = string(&args[1], "event-value")?;
    require(matches!(next, "green" | "amber"), "event-value")?;
    let old = theme(&cursor["projection"])?;
    let effects = if old == next {
        json!([])
    } else {
        json!([
        {"s":"console.session","p":"theme","o":old,"pol":"is-not"},
        {"s":"console.session","p":"theme","o":next,"pol":"is"}])
    };
    let after: Vec<_> = array(&cursor["projection"], "projection-size")?
        .iter()
        .map(|f| {
            let mut f = f.clone();
            if f["s"] == "console.session" && f["p"] == "theme" {
                f["o"] = next.into();
            }
            f
        })
        .collect();
    let after = projection(&json!(after))?;
    let before_hash = canonical::identify("sourcery-runtime-projection", &cursor["projection"])?;
    let after_hash = canonical::identify("sourcery-runtime-projection", &after)?;
    require(
        p["effects"] == effects
            && p["beforeProjection"] == before_hash
            && p["afterProjection"] == after_hash,
        "event-effects-or-basis",
    )?;
    let receipt = json!({"kind":"temple3.sourcery-shadow-receipt/v0.1","contractId":contract["contractId"],"eventId":id,"requestId":request,"sourceRevision":revision,"beforeProjection":before_hash,"afterProjection":after_hash,"disposition":if old == next {"no-op-matched"} else {"replacement-matched"},"authority":"shadow-observation-only"});
    let mut retained = requests.clone();
    retained.push(json!({"requestId":request,"eventId":id,"receipt":receipt}));
    Ok(
        json!({"status":"recorded","receipt":receipt,"cursor":{"revision":revision,"eventId":id,"projection":after,"requests":retained}}),
    )
}
pub fn replay_events(contract: &Value, events: &Value) -> Result<Value> {
    let mut cursor = initial_cursor(contract)?;
    let mut outcomes = Vec::new();
    for envelope in array(events, "event-stream")? {
        let result = reconcile(contract, &cursor, envelope)?;
        cursor = result["cursor"].clone();
        outcomes.push(json!({"status":result["status"],"receipt":result["receipt"]}));
    }
    Ok(json!({"cursor":cursor,"outcomes":outcomes}))
}

pub fn replay_journal(contract: &Value, records: &Value, anchor: &Value) -> Result<Value> {
    let tip = identity::verify_records(records, anchor)?;
    let mut cursor = initial_cursor(contract)?;
    for record in array(records, "invalid-journal")? {
        let entry = &record["entry"];
        shape(entry, "kind event receipt")?;
        require(
            entry["kind"] == "sourcery-shadow-reconciliation",
            "receiver-record",
        )?;
        let next = reconcile(contract, &cursor, &entry["event"])?;
        require(
            next["status"] == "recorded" && next["receipt"] == entry["receipt"],
            "receiver-receipt",
        )?;
        cursor = next["cursor"].clone();
    }
    Ok(json!({"cursor":cursor,"tip":tip}))
}

/// Durable shadow receiver. The caller independently configures the contract and tip.
pub struct RuntimeReceiver {
    journal: crate::journal::FileJournal,
    contract: Value,
}
impl RuntimeReceiver {
    pub fn new(journal: crate::journal::FileJournal, contract: Value) -> Result<Self> {
        initial_cursor(&contract)?;
        Ok(Self { journal, contract })
    }
    pub fn read(&self, expected_tip: &Value) -> Result<Value> {
        require(expected_tip.is_string(), "receiver-tip-required")?;
        replay_journal(
            &self.contract,
            &json!(self.journal.read()?),
            &json!({"expectedTip":expected_tip}),
        )
    }
    pub fn receive(&self, envelope: &Value, expected_tip: &Value) -> Result<Value> {
        require(expected_tip.is_string(), "receiver-tip-required")?;
        self.journal.transaction(|journal| {
            let before = replay_journal(&self.contract, &json!(journal.read()?), &json!({"expectedTip":expected_tip}))?;
            let mut result = reconcile(&self.contract, &before["cursor"], envelope)?;
            if result["status"] == "replay" {result["tip"] = before["tip"].clone();}
            else {
                let record = journal.append(&json!({"kind":"sourcery-shadow-reconciliation","event":envelope,"receipt":result["receipt"]}), &before["tip"])?;
                result["tip"] = record["recordId"].clone();
            }
            Ok(result)
        })
    }
}
