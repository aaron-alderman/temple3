//! Explicit mutable disclosure grants; computation does not establish delivery authority.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn need(ok: bool, code: &str) -> R<()> {
    if ok { Ok(()) } else { Err(code.into()) }
}
fn shape(v: &Value, fields: &str) -> R<()> {
    crate::shape(v, fields).map_err(|_| "invalid-disclosure-shape".into())
}
fn encoded(v: &Value) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|_| "noncanonical-value".into())
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
fn revision(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        !s.is_empty()
            && s.len() <= 4
            && (s == "0" || s.as_bytes()[0] != b'0')
            && s.bytes().all(|b| b.is_ascii_digit())
            && s.parse::<u32>().is_ok_and(|n| n <= 1024)
    })
}
fn number(v: &Value) -> u32 {
    v.as_str().unwrap().parse().unwrap()
}
pub fn validate(s: &Value) -> R<Value> {
    let raw = encoded(s)?;
    shape(
        s,
        "kind instance participants policy resources grants sequence",
    )?;
    need(
        s["kind"] == "temple3.disclosure-grant-state/v0.1"
            && s["instance"].as_str().is_some_and(|v| {
                v.len() == 32
                    && v.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
            && revision(&s["sequence"]),
        "invalid-disclosure-state",
    )?;
    let people = s["participants"]
        .as_array()
        .ok_or("invalid-disclosure-state")?;
    need(
        (1..=8).contains(&people.len())
            && people.iter().all(token)
            && people.windows(2).all(|w| w[0].as_str() < w[1].as_str()),
        "invalid-disclosure-state",
    )?;
    let p = &s["policy"];
    shape(
        p,
        "steward authority disclosure revocation restoration content forgiveness retention amendment retry",
    )?;
    need(
        people.contains(&p["steward"])
            && p["authority"] == "steward-changes-grants-no-implied-read"
            && p["disclosure"] == "explicit-resource-actor-grants"
            && p["revocation"] == "advance-grant-revision-deny-future-delivery"
            && p["restoration"] == "advance-grant-revision-fresh-authorization"
            && p["content"] == "immutable-formation-resources"
            && p["retention"] == "retain-all-transition-evidence"
            && p["amendment"] == "fixed-policy"
            && p["retry"] == "reject-stale-basis",
        "invalid-disclosure-policy",
    )?;
    need(
        p["forgiveness"] == json!({"disposition":"refuse","scope":"disclosure-grant-history"}),
        "invalid-disclosure-policy",
    )?;
    let resources = s["resources"]
        .as_array()
        .ok_or("invalid-disclosure-state")?;
    need(
        (1..=8).contains(&resources.len()),
        "invalid-disclosure-state",
    )?;
    for (i, r) in resources.iter().enumerate() {
        shape(r, "id value")?;
        need(
            r["id"] == format!("resource.{}", i + 1) && encoded(&r["value"])?.len() <= 4096,
            "invalid-disclosure-resource",
        )?;
    }
    let grants = s["grants"]
        .as_array()
        .ok_or("incomplete-disclosure-grants")?;
    need(
        grants.len() == resources.len() * people.len(),
        "incomplete-disclosure-grants",
    )?;
    let mut total = 0;
    for (i, g) in grants.iter().enumerate() {
        shape(g, "resourceId actor allowed revision")?;
        need(
            g["resourceId"] == resources[i / people.len()]["id"]
                && g["actor"] == people[i % people.len()]
                && g["allowed"].is_boolean()
                && revision(&g["revision"]),
            "invalid-disclosure-grant",
        )?;
        total += number(&g["revision"]);
    }
    need(
        total == number(&s["sequence"]),
        "disclosure-revision-mismatch",
    )?;
    need(raw.len() <= 65536, "disclosure-state-bound")?;
    Ok(s.clone())
}
pub fn formation(f: &Value) -> R<Value> {
    encoded(f)?;
    shape(f, "kind instance participants policy resources grants")?;
    need(
        f["kind"] == "temple3.disclosure-grant-formation/v0.1",
        "invalid-disclosure-formation",
    )?;
    let mut s = f.clone();
    s["kind"] = json!("temple3.disclosure-grant-state/v0.1");
    s["sequence"] = json!("0");
    validate(&s)
}
pub fn next_state(before: &Value, actor: &Value, c: &Value) -> R<Value> {
    let mut s = validate(before)?;
    encoded(c)?;
    shape(c, "op args")?;
    need(c["op"] == "set-grant", "unsupported-disclosure-command")?;
    let a = &c["args"];
    shape(a, "resourceId actor allowed expectedRevision")?;
    need(
        token(&a["actor"])
            && a["resourceId"].is_string()
            && a["allowed"].is_boolean()
            && revision(&a["expectedRevision"]),
        "invalid-disclosure-command",
    )?;
    need(actor == &s["policy"]["steward"], "unauthorized")?;
    let index = s["grants"]
        .as_array()
        .unwrap()
        .iter()
        .position(|g| g["resourceId"] == a["resourceId"] && g["actor"] == a["actor"])
        .ok_or("unknown-disclosure-grant")?;
    let g = &s["grants"][index];
    need(
        g["revision"] == a["expectedRevision"],
        "stale-disclosure-grant",
    )?;
    need(g["allowed"] != a["allowed"], "unchanged-disclosure-grant")?;
    need(number(&s["sequence"]) < 1024, "disclosure-exhausted")?;
    s["grants"][index]["revision"] = json!((number(&g["revision"]) + 1).to_string());
    s["grants"][index]["allowed"] = a["allowed"].clone();
    s["sequence"] = json!((number(&s["sequence"]) + 1).to_string());
    validate(&s)
}
pub fn binding(s: &Value, actor: &Value, resource_id: &Value) -> R<Value> {
    validate(s)?;
    let g = s["grants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| &g["actor"] == actor && &g["resourceId"] == resource_id && g["allowed"] == true)
        .ok_or("disclosure-denied")?;
    let r = s["resources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| &r["id"] == resource_id)
        .unwrap();
    let hash =
        crate::canonical::identify("disclosure-resource", r).map_err(|_| "noncanonical-value")?;
    Ok(
        json!({"kind":"temple3.disclosure-binding/v0.1","instance":s["instance"],"actor":actor,"resourceId":resource_id,"revision":g["revision"],"resourceHash":hash}),
    )
}
pub fn verify_binding(s: &Value, b: &Value) -> bool {
    binding(s, &b["actor"], &b["resourceId"]).is_ok_and(|v| {
        encoded(&v)
            .ok()
            .zip(encoded(b).ok())
            .is_some_and(|(a, b)| a == b)
    })
}
pub fn verify(i: &Value, e: &Value) -> bool {
    shape(i, "before actor command after").is_ok()
        && shape(e, "").is_ok()
        && next_state(&i["before"], &i["actor"], &i["command"]).is_ok_and(|v| {
            encoded(&v)
                .ok()
                .zip(encoded(&i["after"]).ok())
                .is_some_and(|(a, b)| a == b)
        })
}
