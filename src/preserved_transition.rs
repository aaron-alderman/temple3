//! Retained realization of a verified source transition; no provider truth claim.
use serde_json::{Value, json};
use std::collections::BTreeSet;
type R<T> = Result<T, String>;
fn need(ok: bool) -> R<()> {
    if ok {
        Ok(())
    } else {
        Err("invalid-transition".into())
    }
}
fn shape(v: &Value, s: &str) -> R<()> {
    crate::shape(v, s).map_err(|_| "invalid-transition".into())
}
fn canonical(v: &Value) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|_| "invalid-transition".into())
}
fn name(v: &Value) -> bool {
    v.as_str().is_some_and(|s| !s.is_empty()) && canonical(v).is_ok_and(|s| s.len() <= 1024)
}
fn hex(v: &Value, n: usize) -> bool {
    v.as_str().is_some_and(|s| {
        s.len() == n
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
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
fn source(s: &Value) -> R<&Value> {
    shape(s, "obligations inspection")?;
    let o = &s["obligations"];
    let i = &s["inspection"];
    shape(
        o,
        "kind sourceGeneration command root memberPredicate targetType statePredicate from to reviewer historyPredicate historyValue evidencePredicate evidenceMax",
    )?;
    shape(
        i,
        "kind obligationId artifactId executionId authorityId boundary",
    )?;
    need(
        o["kind"] == "temple3.transition-lowering-obligations/v0.1"
            && o.as_object().unwrap().values().all(name)
            && hex(&o["sourceGeneration"], 64)
            && o["from"] != o["to"],
    )?;
    let maximum = o["evidenceMax"].as_str().unwrap();
    need(
        maximum.len() <= 4
            && !maximum.starts_with('0')
            && maximum.bytes().all(|b| b.is_ascii_digit())
            && maximum
                .parse::<usize>()
                .is_ok_and(|n| (1..=4096).contains(&n)),
    )?;
    need(
        BTreeSet::from([
            o["statePredicate"].as_str().unwrap(),
            o["historyPredicate"].as_str().unwrap(),
            o["evidencePredicate"].as_str().unwrap(),
        ])
        .len()
            == 3,
    )?;
    for k in [
        "from",
        "to",
        "historyValue",
        "statePredicate",
        "historyPredicate",
        "evidencePredicate",
    ] {
        let v = o[k].as_str().unwrap();
        need(!v.contains('<') && !v.contains('$'))?;
    }
    let obligation_id =
        crate::canonical::identify("lowering-obligations", o).map_err(|_| "invalid-transition")?;
    need(
        i["kind"] == "temple3.transition-lowering-inspection/v0.1"
            && i["boundary"] == "declared-program-definition-only"
            && i["obligationId"] == obligation_id,
    )?;
    for (field, kind) in [
        ("artifactId", "lowered-fact-artifact"),
        ("executionId", "lowered-execution"),
        ("authorityId", "lowered-authority"),
    ] {
        need(
            i[field]
                .as_str()
                .and_then(|s| s.strip_prefix(&format!("{kind}:sha256:")))
                .is_some_and(|s| hex(&json!(s), 64)),
        )?;
    }
    Ok(o)
}
fn records(rs: &Value) -> R<()> {
    let records = rs.as_array().ok_or("invalid-transition")?;
    need((1..=8).contains(&records.len()))?;
    let mut ids = BTreeSet::new();
    for r in records {
        shape(r, "id root memberPredicate type fields")?;
        need(
            ["id", "root", "memberPredicate", "type"]
                .iter()
                .all(|k| name(&r[k]))
                && ids.insert(r["id"].as_str().unwrap()),
        )?;
        let fields = r["fields"].as_object().ok_or("invalid-transition")?;
        need((1..=16).contains(&fields.len()))?;
        for (k, v) in fields {
            need(name(&json!(k)) && v.is_string() && canonical(v)?.len() <= 4096)?;
        }
    }
    Ok(())
}
fn white(c: char) -> bool {
    matches!(c as u32,9..=13|32|160|5760|8192..=8202|8232|8233|8239|8287|12288|65279)
}
fn apply(rs: &Value, e: &Value, o: &Value, reviewer: &Value) -> R<Value> {
    shape(e, "recordId actor evidence")?;
    need(&e["actor"] == reviewer)?;
    let evidence = e["evidence"].as_str().ok_or("invalid-transition")?;
    need(
        name(&e["recordId"])
            && !evidence.is_empty()
            && evidence.encode_utf16().count()
                <= o["evidenceMax"].as_str().unwrap().parse::<usize>().unwrap()
            && evidence.chars().any(|c| !white(c)),
    )?;
    let mut result = rs.clone();
    let r = result
        .as_array_mut()
        .ok_or("invalid-transition")?
        .iter_mut()
        .find(|r| r["id"] == e["recordId"])
        .ok_or("invalid-transition")?;
    need(
        r["root"] == o["root"]
            && r["memberPredicate"] == o["memberPredicate"]
            && r["type"] == o["targetType"]
            && r["fields"][o["statePredicate"].as_str().unwrap()] == o["from"],
    )?;
    for (k, v) in [
        ("statePredicate", &o["to"]),
        ("historyPredicate", &o["historyValue"]),
        ("evidencePredicate", &e["evidence"]),
    ] {
        r["fields"][o[k].as_str().unwrap()] = v.clone();
    }
    records(&result)?;
    Ok(result)
}
pub fn validate(s: &Value) -> R<Value> {
    let raw = canonical(s)?;
    shape(
        s,
        "kind instance source participants policy initialRecords records entries sequence",
    )?;
    need(s["kind"] == "temple3.preserved-transition-state/v0.1" && hex(&s["instance"], 32))?;
    let o = source(&s["source"])?;
    let people = s["participants"].as_array().ok_or("invalid-transition")?;
    need(
        (1..=8).contains(&people.len())
            && people.iter().all(actor)
            && people.windows(2).all(|w| w[0].as_str() < w[1].as_str()),
    )?;
    let p = &s["policy"];
    shape(
        p,
        "sourceReviewer targetReviewer authority evidence evidenceLength forgiveness retention disclosure amendment retry formation",
    )?;
    need(
        p == &json!({"sourceReviewer":o["reviewer"],"targetReviewer":p["targetReviewer"],"authority":"mapped-reviewer-only","evidence":"reviewer-supplied-text-not-provider-confirmation","evidenceLength":"utf16-code-units","forgiveness":{"disposition":"refuse","scope":"transition-records-and-history"},"retention":"retain-origin-and-all-transitions","disclosure":"participants-all-state","amendment":"fixed-source-policy-and-record-set","retry":"reject-stale-basis","formation":"explicit-initial-record-snapshot"})
            && people.contains(&p["targetReviewer"]),
    )?;
    records(&s["initialRecords"])?;
    records(&s["records"])?;
    let entries = s["entries"].as_array().ok_or("invalid-transition")?;
    let expected_sequence = entries.len().to_string();
    need(entries.len() <= 8 && s["sequence"].as_str() == Some(expected_sequence.as_str()))?;
    let mut current = s["initialRecords"].clone();
    for e in entries {
        current = apply(&current, e, o, &p["targetReviewer"])?;
    }
    need(current == s["records"] && raw.len() <= 32768)?;
    Ok(s.clone())
}
pub fn formation(f: &Value, lowering: &Value, anchor: &Value) -> R<Value> {
    canonical(f)?;
    shape(f, "kind instance source participants policy initialRecords")?;
    need(f["kind"] == "temple3.preserved-transition-formation/v0.1")?;
    let inspection = crate::lowering_preservation::inspect(lowering, anchor)?;
    need(f["source"] == json!({"obligations":lowering["obligations"],"inspection":inspection}))?;
    let mut s = f.clone();
    s["kind"] = json!("temple3.preserved-transition-state/v0.1");
    s["records"] = s["initialRecords"].clone();
    s["entries"] = json!([]);
    s["sequence"] = json!("0");
    validate(&s)
}
pub fn next_state(s: &Value, actor: &Value, c: &Value) -> R<Value> {
    let mut s = validate(s)?;
    canonical(c)?;
    shape(c, "op args")?;
    need(c["op"] == "transition")?;
    shape(&c["args"], "recordId evidence")?;
    let e =
        json!({"recordId":c["args"]["recordId"],"evidence":c["args"]["evidence"],"actor":actor});
    let current = apply(
        &s["records"],
        &e,
        &s["source"]["obligations"],
        &s["policy"]["targetReviewer"],
    )?;
    s["records"] = current;
    s["entries"].as_array_mut().unwrap().push(e);
    s["sequence"] = json!(s["entries"].as_array().unwrap().len().to_string());
    validate(&s)
}
pub fn verify(i: &Value, e: &Value) -> bool {
    shape(i, "before actor command after").is_ok()
        && shape(e, "").is_ok()
        && next_state(&i["before"], &i["actor"], &i["command"])
            .is_ok_and(|v| canonical(&i["after"]).is_ok() && v == i["after"])
}
