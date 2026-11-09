//! Independent actor-request permission and current native judgment.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn need(ok: bool, code: &str) -> R<()> {
    if ok { Ok(()) } else { Err(code.into()) }
}
fn exact(v: &Value, fields: &str) -> R<()> {
    crate::shape(v, fields).map_err(|_| "invalid-decision-request".into())
}
pub fn prepare(x: &Value) -> R<Value> {
    let raw = crate::canonical::canonical_json(x).map_err(|e| e.code)?;
    exact(
        x,
        "genesis descriptor expectedTempleId certificates expectedFinalityId request",
    )?;
    let certificates = x["certificates"]
        .as_array()
        .ok_or("decision-request-bound")?;
    need(
        certificates.len() <= 1024 && raw.len() <= 4194304,
        "decision-request-bound",
    )?;
    let mut current = crate::basis_decision_profile::recognize(
        &x["genesis"],
        &x["descriptor"],
        &x["expectedTempleId"],
    )
    .map_err(|e| e.code)?;
    for c in certificates {
        current = crate::coordination::advance(&current, c).map_err(|e| e.code)?;
    }
    need(
        x["expectedFinalityId"].is_string() && current["finalityId"] == x["expectedFinalityId"],
        "request-head-mismatch",
    )?;
    let envelope = &x["request"];
    let p = &envelope["payload"];
    exact(
        p,
        "kind templeId actor requestId basisFinalityId policyId command execution",
    )?;
    let valid_id = p["requestId"].as_str().is_some_and(|s| {
        !s.is_empty()
            && s.len() <= 64
            && s.as_bytes()[0].is_ascii_lowercase()
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
    });
    need(
        p["kind"] == "temple3.decision-request/v0.1"
            && p["templeId"] == x["expectedTempleId"]
            && valid_id,
        "invalid-decision-request",
    )?;
    let actors = current["identityProfile"]["actors"]
        .as_array()
        .ok_or("invalid-request-signature")?;
    let actor = actors
        .iter()
        .find(|a| a["actor"] == p["actor"])
        .ok_or("invalid-request-signature")?;
    need(
        crate::identity::verify_envelope(envelope, "attestation", &actor["publicKey"]),
        "invalid-request-signature",
    )?;
    let execution = &p["execution"];
    exact(execution, "mode maxAdvances policyChange")?;
    need(
        (execution["mode"] == "exact-basis" || execution["mode"] == "re-evaluate-current")
            && execution["policyChange"] == "reject"
            && execution["maxAdvances"]
                .as_str()
                .is_some_and(|s| s.len() == 1 && (b'0'..=b'8').contains(&s.as_bytes()[0])),
        "invalid-request-execution",
    )?;
    need(
        execution["mode"] != "exact-basis" || execution["maxAdvances"] == "0",
        "invalid-request-execution",
    )?;
    let basis = crate::coordination::basis(&current, &p["basisFinalityId"])
        .map_err(|_| "unknown-request-basis")?;
    let before = &basis["temple"]["state"]["cells"][0]["value"];
    let value = &current["temple"]["state"]["cells"][0]["value"];
    let old = before["policies"]
        .as_array()
        .and_then(|p| p.last())
        .ok_or("request-policy-mismatch")?;
    let new = value["policies"]
        .as_array()
        .and_then(|p| p.last())
        .ok_or("request-policy-mismatch")?;
    need(old["id"] == p["policyId"], "request-policy-mismatch")?;
    crate::basis_decision::next_state(before, &p["actor"], &p["command"])?;
    let number = |v: &Value| -> R<u64> {
        v.as_str()
            .and_then(|s| s.parse().ok())
            .ok_or("invalid-request-execution".into())
    };
    let advances = number(&current["temple"]["sequence"])? - number(&basis["temple"]["sequence"])?;
    need(
        execution["mode"] != "exact-basis" || advances == 0,
        "stale-request-basis",
    )?;
    need(
        advances <= number(&execution["maxAdvances"])?,
        "request-horizon-exceeded",
    )?;
    need(old == new, "request-policy-changed")?;
    let after = crate::basis_decision::next_state(value, &p["actor"], &p["command"])?;
    Ok(
        json!({"actor":p["actor"],"requestId":p["requestId"],"originFinalityId":p["basisFinalityId"],"evaluatedFinalityId":current["finalityId"],"after":after}),
    )
}
pub fn outcome(x: &Value) -> bool {
    outcome_result(x).unwrap_or(false)
}
fn outcome_result(x: &Value) -> R<bool> {
    exact(x, "input certificate expectedOutcomeFinalityId")?;
    let source = &x["input"];
    let prepared = prepare(source)?;
    let mut current = crate::basis_decision_profile::recognize(
        &source["genesis"],
        &source["descriptor"],
        &source["expectedTempleId"],
    )
    .map_err(|e| e.code)?;
    for certificate in source["certificates"]
        .as_array()
        .ok_or("decision-request-bound")?
    {
        current = crate::coordination::advance(&current, certificate).map_err(|e| e.code)?;
    }
    let final_state =
        crate::coordination::advance(&current, &x["certificate"]).map_err(|e| e.code)?;
    let p = &x["certificate"]["proposal"]["payload"];
    let expected = json!({"before":current["temple"]["state"]["cells"][0]["value"],"actor":prepared["actor"],"command":source["request"]["payload"]["command"],"after":prepared["after"]});
    Ok(x["expectedOutcomeFinalityId"].is_string()
        && final_state["finalityId"] == x["expectedOutcomeFinalityId"]
        && p["actor"] == prepared["actor"]
        && p["requestId"] == prepared["requestId"]
        && p["context"]["statement"]["relation"] == "basis-decision-command-v1"
        && p["context"]["statement"]["publicInputs"] == expected)
}
