//! Explicit personal expression, without inferred release or erasure authority.
use serde_json::{Value, json};
use std::collections::BTreeSet;
type R<T> = Result<T, String>;
fn need(ok: bool) -> R<()> {
    if ok {
        Ok(())
    } else {
        Err("invalid-expression".into())
    }
}
fn shape(v: &Value, keys: &str) -> R<()> {
    crate::shape(v, keys).map_err(|_| "invalid-expression".into())
}
fn canonical(v: &Value) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|_| "invalid-expression".into())
}
fn text(v: &Value, max: usize) -> bool {
    v.as_str().is_some_and(|s| !s.is_empty()) && canonical(v).is_ok_and(|s| s.len() <= max)
}
fn actor(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        !s.is_empty()
            && s.len() <= 64
            && s.as_bytes()[0].is_ascii_lowercase()
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
    })
}
pub fn state_id(v: &Value) -> R<String> {
    crate::canonical::identify("voluntary-expression-state", v)
        .map_err(|_| "invalid-expression".into())
}
fn records(v: &Value, people: &[Value]) -> R<()> {
    let rs = v.as_array().ok_or("invalid-records")?;
    need(rs.len() <= 64)?;
    let mut ids = BTreeSet::new();
    for r in rs {
        shape(r, "id type owner process regarding status")?;
        need(
            text(&r["id"], 1024)
                && ids.insert(r["id"].as_str().ok_or("invalid-id")?)
                && r["type"] == "Forgiveness"
                && people.contains(&r["owner"])
                && text(&r["process"], 1024)
                && text(&r["regarding"], 4096)
                && (r["status"] == "notYet" || r["status"] == "offered"),
        )?;
    }
    Ok(())
}
fn apply(rs: &Value, e: &Value) -> R<Value> {
    shape(e, "actor recordId basis")?;
    let mut result = rs.clone();
    let r = result
        .as_array_mut()
        .ok_or("invalid-records")?
        .iter_mut()
        .find(|r| r["id"] == e["recordId"])
        .ok_or("unknown-record")?;
    need(r["owner"] == e["actor"] && r["status"] == "notYet")?;
    r["status"] = json!("offered");
    Ok(result)
}
pub fn validate(s: &Value) -> R<Value> {
    let raw = canonical(s)?;
    shape(
        s,
        "kind instance participants policy initialRecords records entries sequence",
    )?;
    need(
        s["kind"] == "temple3.voluntary-expression-state/v0.1"
            && s["instance"].as_str().is_some_and(|v| {
                v.len() == 32
                    && v.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }),
    )?;
    let people = s["participants"].as_array().ok_or("invalid-participants")?;
    need(
        (1..=64).contains(&people.len())
            && people.iter().all(actor)
            && people.windows(2).all(|w| w[0].as_str() < w[1].as_str()),
    )?;
    need(
        s["policy"]
            == json!({"meaning":"personal-forgiveness-expression","participation":"optional-no-completion-obligation","authority":"record-owner-only","repeat":"reject-already-offered","withdrawal":"unavailable","consequence":"expression-only-no-judgment-release-or-data-erasure","disclosure":"participants-all-state","forgiveness":{"disposition":"refuse","scope":"origin-records-and-expression-history"},"retention":"retain-origin-and-all-expressions","amendment":"fixed-records-owners-text-process-and-policy","basis":"exact-state-identity","retry":"fresh-consent-after-stale-or-unknown-outcome","formation":"asserted-initial-snapshot-not-proof-of-source-truth-or-prior-consent"}),
    )?;
    records(&s["initialRecords"], people)?;
    records(&s["records"], people)?;
    let entries = s["entries"].as_array().ok_or("invalid-entries")?;
    let sequence = entries.len().to_string();
    need(entries.len() <= 64 && s["sequence"].as_str() == Some(sequence.as_str()))?;
    let mut current = s["initialRecords"].clone();
    for (i, e) in entries.iter().enumerate() {
        shape(e, "actor recordId basis")?;
        let mut prior = s.clone();
        prior["records"] = current.clone();
        prior["entries"] = json!(&entries[..i]);
        prior["sequence"] = json!(i.to_string());
        need(e["basis"] == state_id(&prior)?)?;
        current = apply(&current, e)?;
    }
    need(current == s["records"] && raw.len() <= 131072)?;
    Ok(s.clone())
}
pub fn formation(f: &Value) -> R<Value> {
    canonical(f)?;
    shape(f, "kind instance participants policy initialRecords")?;
    need(f["kind"] == "temple3.voluntary-expression-formation/v0.1")?;
    let mut s = f.clone();
    s["kind"] = json!("temple3.voluntary-expression-state/v0.1");
    s["records"] = f["initialRecords"].clone();
    s["entries"] = json!([]);
    s["sequence"] = json!("0");
    validate(&s)
}
pub fn next_state(s: &Value, actor: &Value, c: &Value) -> R<Value> {
    let mut result = validate(s)?;
    canonical(c)?;
    shape(c, "op args")?;
    need(c["op"] == "offer")?;
    shape(&c["args"], "recordId basis")?;
    need(c["args"]["basis"] == state_id(s)?)?;
    let e = json!({"actor":actor,"recordId":c["args"]["recordId"],"basis":c["args"]["basis"]});
    result["records"] = apply(&s["records"], &e)?;
    let entries = result["entries"].as_array_mut().ok_or("invalid-entries")?;
    entries.push(e);
    let count = entries.len();
    result["sequence"] = json!(count.to_string());
    validate(&result)
}
pub fn verify(i: &Value, e: &Value) -> bool {
    (|| -> R<bool> {
        shape(i, "before actor command after")?;
        shape(e, "")?;
        Ok(
            canonical(&next_state(&i["before"], &i["actor"], &i["command"])?)?
                == canonical(&i["after"])?,
        )
    })()
    .unwrap_or(false)
}
