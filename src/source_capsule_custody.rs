//! Independent read-only custody audit. No retention, disclosure or activation.
use crate::{Result, array, canonical, err, identity, require, shape};
use serde_json::{Value, json};
use std::collections::HashSet;
fn count(v: &Value, max: u64) -> Result<u64> {
    let s = v.as_str().ok_or_else(|| err("invalid-custody-limit"))?;
    require(
        !s.is_empty()
            && s.len() <= 16
            && !s.starts_with('0')
            && s.bytes().all(|b| b.is_ascii_digit()),
        "invalid-custody-limit",
    )?;
    let n = s.parse::<u64>().map_err(|_| err("invalid-custody-limit"))?;
    require(n <= max, "invalid-custody-limit")?;
    Ok(n)
}
fn size(v: &Value) -> Result<u64> {
    Ok(canonical::canonical_json(v)?.len() as u64)
}
pub fn config_id(c: &Value) -> Result<String> {
    canonical::canonical_json(c)?;
    shape(
        c,
        "kind instance depositorKey inspectorKeys capsulePolicy expectedPolicyId authority retention readAudit forgiveness maximumCapsules maximumRetainedBytes maximumRequestBytes maximumPending challengeLifetimeMs",
    )?;
    require(
        c["kind"] == "temple3.source-capsule-custody/v0.1"
            && c["instance"].as_str().is_some_and(|s| {
                s.len() == 32
                    && s.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }),
        "invalid-custody-config",
    )?;
    identity::key_id(&c["depositorKey"])?;
    let keys = array(&c["inspectorKeys"], "invalid-inspectors")?;
    require(!keys.is_empty() && keys.len() <= 64, "invalid-inspectors")?;
    for (i, key) in keys.iter().enumerate() {
        identity::key_id(key)?;
        require(
            i == 0 || keys[i - 1].as_str() < key.as_str(),
            "invalid-inspectors",
        )?;
    }
    require(
        c["authority"] == "depositor-write-listed-inspectors-read"
            && c["retention"] == "retain-formation-deposits-and-inspections"
            && c["readAudit"] == "not-retained"
            && c["forgiveness"]
                == json!({"disposition":"refuse","scope":"formation-deposits-and-inspections"}),
        "unsupported-custody-policy",
    )?;
    for field in [
        "maximumCapsules",
        "maximumRetainedBytes",
        "maximumRequestBytes",
        "maximumPending",
        "challengeLifetimeMs",
    ] {
        count(
            &c[field],
            if field == "maximumRequestBytes" {
                4194304
            } else {
                9007199254740991
            },
        )?;
    }
    let p = &c["capsulePolicy"];
    shape(
        p,
        "kind maximumRawBytes maximumEncodedBytes forgiveness retention disclosure activation",
    )?;
    require(
        p["kind"] == "temple3.source-observation-capsule-policy/v0.1"
            && p["forgiveness"] == json!({"disposition":"refuse","scope":"capsule-and-inspection"})
            && p["retention"] == "retain-exact-capsule"
            && p["disclosure"] == "trusted-inspector-only"
            && p["activation"] == "unavailable",
        "unsupported-capsule-policy",
    )?;
    count(&p["maximumRawBytes"], 9007199254740991)?;
    count(&p["maximumEncodedBytes"], 9007199254740991)?;
    require(
        canonical::identify("source-observation-capsule-policy", p)? == c["expectedPolicyId"],
        "wrong-capsule-policy",
    )?;
    canonical::identify("source-capsule-custody", c)
}
pub fn audit(x: &Value) -> Result<Value> {
    shape(x, "config expectedCustodyId expectedTip records")?;
    let c = &x["config"];
    let cid = config_id(c)?;
    require(
        cid == x["expectedCustodyId"] && x["expectedTip"].is_string(),
        "wrong-custody-anchor",
    )?;
    identity::verify_records(&x["records"], &json!({"expectedTip":x["expectedTip"]}))?;
    let records = array(&x["records"], "invalid-custody-records")?;
    require(!records.is_empty(), "missing-formation")?;
    require(
        records[0]["entry"]
            == json!({"kind":"temple3.source-capsule-custody-formation/v0.1","config":c}),
        "wrong-custody-formation",
    )?;
    let retained = records.iter().try_fold(0u64, |total, r| -> Result<u64> {
        total
            .checked_add(size(r)?)
            .ok_or_else(|| err("custody-byte-overflow"))
    })?;
    require(
        (records.len() - 1) as u64 <= count(&c["maximumCapsules"], 9007199254740991)?
            && retained <= count(&c["maximumRetainedBytes"], 9007199254740991)?,
        "custody-capacity",
    )?;
    let mut slots = HashSet::new();
    let mut capsules = HashSet::new();
    let mut deposits = Vec::new();
    for record in &records[1..] {
        let e = &record["entry"];
        shape(e, "kind deposit inspection")?;
        require(
            e["kind"] == "temple3.retained-source-capsule/v0.1",
            "invalid-custody-entry",
        )?;
        let d = &e["deposit"];
        require(
            size(d)? <= count(&c["maximumRequestBytes"], 4194304)?
                && identity::verify_envelope(d, "attestation", &c["depositorKey"]),
            "deposit-denied",
        )?;
        let p = &d["payload"];
        shape(p, "kind custodyId requestId input")?;
        identity::token(&p["requestId"])?;
        require(
            p["kind"] == "temple3.source-capsule-deposit/v0.1"
                && p["custodyId"] == cid
                && p["input"]["policy"] == c["capsulePolicy"]
                && p["input"]["expectedPolicyId"] == c["expectedPolicyId"],
            "deposit-denied",
        )?;
        let inspection = crate::source_observation_capsule::inspect(&p["input"])
            .map_err(|_| err("invalid-capsule"))?;
        require(inspection == e["inspection"], "invalid-retained-inspection")?;
        require(
            slots.insert(p["requestId"].as_str().unwrap().to_string())
                && capsules.insert(inspection["capsuleId"].as_str().unwrap().to_string()),
            "duplicate-capsule",
        )?;
        deposits.push(json!({"requestId":p["requestId"],"recordId":record["recordId"],"inspection":inspection}));
    }
    Ok(
        json!({"kind":"temple3.source-capsule-custody-audit/v0.1","custodyId":cid,"tip":x["expectedTip"],"retainedBytes":retained.to_string(),"deposits":deposits}),
    )
}
