use crate::claim_reconciliation::reconcile;
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn need(v: bool) -> R<()> {
    if v {
        Ok(())
    } else {
        Err("invalid-mapping-selection".into())
    }
}
fn shape(v: &Value, fields: &str) -> R<()> {
    let o = v.as_object().ok_or("object-required")?;
    let f: Vec<_> = fields.split_whitespace().collect();
    need(o.len() == f.len() && f.iter().all(|k| o.contains_key(*k)))
}
fn canonical(v: &Value) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|e| format!("{e:?}"))
}
fn id(kind: &str, v: &Value) -> R<String> {
    crate::canonical::identify(kind, v).map_err(|e| format!("{e:?}"))
}
fn text(v: &Value) -> R<&str> {
    v.as_str().ok_or("text-required".into())
}
fn limit(v: &Value) -> R<u64> {
    let s = text(v)?;
    need(
        !s.is_empty()
            && s.len() <= 7
            && !s.starts_with('0')
            && s.bytes().all(|b| b.is_ascii_digit()),
    )?;
    s.parse().map_err(|_| "invalid-limit".into())
}
pub fn inspect(x: &Value) -> R<Value> {
    shape(x, "policy expectedPolicyId reconciliation selection")?;
    canonical(x)?;
    let p = &x["policy"];
    shape(
        p,
        "kind authorizerKey targetProfileId targetFormationId targetHeadId meaning maximumSignedBytes",
    )?;
    need(
        p["kind"] == "temple3.claim-mapping-selection-policy/v0.1"
            && p["meaning"] == "select-proposed-mapping-not-activation",
    )?;
    crate::identity::key_id(&p["authorizerKey"]).map_err(|e| format!("{e:?}"))?;
    for f in ["targetProfileId", "targetFormationId", "targetHeadId"] {
        crate::identity::token(&p[f]).map_err(|e| format!("{e:?}"))?;
    }
    let maximum = limit(&p["maximumSignedBytes"])?;
    need(text(&p["maximumSignedBytes"])?.len() <= 7 && maximum <= 4_194_304)?;
    need(x["expectedPolicyId"] == id("claim-mapping-selection-policy", p)?)?;
    let report = reconcile(&x["reconciliation"])?;
    let e = &x["selection"];
    need(
        canonical(e)?.len() as u64 <= maximum
            && crate::identity::verify_envelope(e, "attestation", &p["authorizerKey"]),
    )?;
    let s = &e["payload"];
    shape(
        s,
        "kind policyId requestId reconciliationInputId reconciliationReportId targetProfileId targetFormationId targetHeadId meaning",
    )?;
    crate::identity::token(&s["requestId"]).map_err(|e| format!("{e:?}"))?;
    need(
        *s == json!({"kind":"temple3.claim-mapping-selection/v0.1","policyId":x["expectedPolicyId"],"requestId":s["requestId"],"reconciliationInputId":id("claim-reconciliation-input",&x["reconciliation"] )?,"reconciliationReportId":id("claim-reconciliation-report",&report)?,"targetProfileId":p["targetProfileId"],"targetFormationId":p["targetFormationId"],"targetHeadId":p["targetHeadId"],"meaning":p["meaning"]}),
    )?;
    let unresolved: usize = report["entries"]
        .as_array()
        .ok_or("entries-required")?
        .iter()
        .map(|r| r["unresolved"].as_array().map_or(0, Vec::len))
        .sum();
    Ok(
        json!({"kind":"temple3.claim-mapping-selection-inspection/v0.1","selectionId":id("signed",e)?,"policyId":x["expectedPolicyId"],"requestId":s["requestId"],"reconciliationReportId":s["reconciliationReportId"],"targetProfileId":p["targetProfileId"],"targetFormationId":p["targetFormationId"],"targetHeadId":p["targetHeadId"],"unresolvedObligations":unresolved.to_string(),"meaning":p["meaning"],"sourceAuthority":"not-verified","activation":"unavailable"}),
    )
}
