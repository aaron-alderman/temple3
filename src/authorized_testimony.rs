//! Independent person-attribution evidence, not provider acknowledgement.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn need(ok: bool, code: &str) -> R<()> {
    if ok { Ok(()) } else { Err(code.into()) }
}
fn exact(v: &Value, fields: &str) -> R<()> {
    crate::shape(v, fields).map_err(|_| "invalid-testimony-shape".into())
}
fn canonical(v: &Value) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|e| e.code)
}
fn id(kind: &str, v: &Value) -> R<String> {
    crate::canonical::identify(kind, v).map_err(|e| e.code)
}
fn token(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        !s.is_empty()
            && s.len() <= 64
            && s.as_bytes()[0].is_ascii_lowercase()
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
    })
}
pub fn policy(p: &Value) -> R<()> {
    canonical(p)?;
    exact(
        p,
        "kind instance participants attestor subject assertion evidenceClass effect retry forgiveness retention disclosure amendment",
    )?;
    need(
        p["kind"] == "temple3.testimony-policy/v0.1"
            && p["instance"].as_str().is_some_and(|s| {
                s.len() == 32
                    && s.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }),
        "invalid-testimony-policy",
    )?;
    let actors = p["participants"]
        .as_array()
        .ok_or("invalid-testimony-policy")?;
    need(
        !actors.is_empty() && actors.len() <= 8,
        "invalid-testimony-policy",
    )?;
    for (i, a) in actors.iter().enumerate() {
        exact(a, "actor publicKey")?;
        need(
            token(&a["actor"]) && (i == 0 || actors[i - 1]["actor"].as_str() < a["actor"].as_str()),
            "invalid-testimony-policy",
        )?;
        crate::identity::key_id(&a["publicKey"]).map_err(|e| e.code)?;
        need(
            !actors[..i].iter().any(|b| b["publicKey"] == a["publicKey"]),
            "shared-testimony-key",
        )?;
    }
    need(
        actors.iter().any(|a| a["actor"] == p["attestor"]),
        "invalid-testimony-policy",
    )?;
    for (field, selected) in [
        ("subject", "exact-request-and-basis"),
        ("assertion", "reported-completion"),
        ("evidenceClass", "authorized-person-testimony"),
        ("effect", "record-attribution-only"),
        ("retry", "same-slot-same-content"),
        ("retention", "retain-all-testimony-history"),
        ("disclosure", "participants-only"),
        ("amendment", "fixed-policy"),
    ] {
        need(p[field] == selected, "invalid-testimony-policy")?;
    }
    need(
        p["forgiveness"] == json!({"disposition":"refuse","scope":"testimony-history"}),
        "invalid-testimony-policy",
    )
}
pub fn inspect(x: &Value) -> R<Value> {
    let raw = canonical(x)?;
    exact(
        x,
        "policy expectedPolicyId requestId subject basis testimony",
    )?;
    need(raw.len() <= 65536, "testimony-bundle-bound")?;
    let p = &x["policy"];
    policy(p)?;
    let pid = id("testimony-policy", p)?;
    need(
        x["expectedPolicyId"].as_str() == Some(pid.as_str()),
        "testimony-policy-anchor-mismatch",
    )?;
    let actor = p["participants"]
        .as_array()
        .and_then(|a| a.iter().find(|a| a["actor"] == p["attestor"]))
        .ok_or("invalid-testimony-policy")?;
    need(
        crate::identity::verify_envelope(&x["testimony"], "attestation", &actor["publicKey"]),
        "invalid-testimony-signature",
    )?;
    let payload = &x["testimony"]["payload"];
    exact(
        payload,
        "kind policyId instance attestor requestId subjectId basisId assertion text",
    )?;
    need(token(&x["requestId"]), "invalid-testimony-request")?;
    need(
        canonical(&x["subject"])?.len() <= 16384 && canonical(&x["basis"])?.len() <= 16384,
        "testimony-subject-bound",
    )?;
    let text = payload["text"].as_str().ok_or("invalid-testimony-text")?;
    need(!text.is_empty()&&text.len()<=512&&text.chars().any(|c|!matches!(c as u32,9..=13|32|160|5760|8192..=8202|8232|8233|8239|8287|12288|65279)),"invalid-testimony-text")?;
    let sid = id("testimony-subject", &x["subject"])?;
    let bid = id("testimony-basis", &x["basis"])?;
    let expected = json!({"kind":"temple3.person-testimony/v0.1","policyId":pid,"instance":p["instance"],"attestor":p["attestor"],"requestId":x["requestId"],"subjectId":sid,"basisId":bid,"assertion":"reported-completion","text":text});
    need(payload == &expected, "testimony-binding-mismatch")?;
    Ok(
        json!({"status":"attested","evidenceClass":p["evidenceClass"],"policyId":pid,"testimonyId":id("person-testimony",&x["testimony"])? ,"attestor":p["attestor"],"requestId":x["requestId"],"subjectId":sid,"basisId":bid}),
    )
}
fn entry_view(p: &Value, pid: &Value, e: &Value) -> R<Value> {
    exact(e, "requestId subject basis testimony")?;
    inspect(
        &json!({"policy":p,"expectedPolicyId":pid,"requestId":e["requestId"],"subject":e["subject"],"basis":e["basis"],"testimony":e["testimony"]}),
    )
}
pub fn state(s: &Value) -> R<Value> {
    let raw = canonical(s)?;
    exact(s, "kind policy policyId sequence entries")?;
    need(
        s["kind"] == "temple3.testimony-state/v0.1",
        "invalid-testimony-state",
    )?;
    policy(&s["policy"])?;
    need(
        s["policyId"].as_str() == Some(id("testimony-policy", &s["policy"])?.as_str()),
        "testimony-policy-anchor-mismatch",
    )?;
    let entries = s["entries"].as_array().ok_or("invalid-testimony-state")?;
    need(
        entries.len() <= 8 && s["sequence"].as_str() == Some(entries.len().to_string().as_str()),
        "invalid-testimony-state",
    )?;
    for (i, e) in entries.iter().enumerate() {
        entry_view(&s["policy"], &s["policyId"], e)?;
        need(
            !entries[..i]
                .iter()
                .any(|p| p["requestId"] == e["requestId"]),
            "duplicate-testimony-slot",
        )?;
    }
    need(raw.len() <= 65536, "testimony-state-bound")?;
    Ok(s.clone())
}
pub fn formation(f: &Value) -> R<Value> {
    canonical(f)?;
    exact(f, "kind policy")?;
    need(
        f["kind"] == "temple3.testimony-formation/v0.1",
        "invalid-testimony-formation",
    )?;
    policy(&f["policy"])?;
    state(
        &json!({"kind":"temple3.testimony-state/v0.1","policy":f["policy"],"policyId":id("testimony-policy",&f["policy"])?,"sequence":"0","entries":[]}),
    )
}
pub fn next_state(before: &Value, actor: &Value, command: &Value) -> R<Value> {
    let mut result = state(before)?;
    canonical(command)?;
    exact(command, "op args")?;
    need(command["op"] == "record", "unsupported-testimony-command")?;
    let e = &command["args"];
    exact(e, "requestId subject basis testimony")?;
    need(actor == &before["policy"]["attestor"], "unauthorized")?;
    entry_view(&before["policy"], &before["policyId"], e)?;
    let entries = before["entries"]
        .as_array()
        .ok_or("invalid-testimony-state")?;
    if let Some(prior) = entries.iter().find(|p| p["requestId"] == e["requestId"]) {
        need(prior == e, "testimony-slot-conflict")?;
        return Ok(result);
    }
    need(entries.len() < 8, "testimony-capacity")?;
    let mut appended = entries.clone();
    appended.push(e.clone());
    result["entries"] = json!(appended);
    result["sequence"] = json!((entries.len() + 1).to_string());
    state(&result)
}
pub fn command_valid(inputs: &Value, evidence: &Value) -> bool {
    if exact(inputs, "before actor command after").is_err() || exact(evidence, "").is_err() {
        return false;
    }
    next_state(&inputs["before"], &inputs["actor"], &inputs["command"])
        .is_ok_and(|after| after != inputs["before"] && after == inputs["after"])
}
pub fn run(c: &Value) -> R<Value> {
    match c["kind"].as_str() {
        Some("formation") => formation(&c["input"]),
        Some("state") => state(&c["input"]),
        Some("next") => next_state(&c["before"], &c["actor"], &c["command"]),
        Some("command") => Ok(json!(command_valid(&c["inputs"], &c["evidence"]))),
        _ => inspect(&c["input"]),
    }
}
