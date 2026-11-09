//! Independent retained mapping journal audit; no append or activation.
use serde_json::{Value, json};
use std::collections::HashSet;
type R<T> = Result<T, String>;
fn need(v: bool) -> R<()> {
    if v {
        Ok(())
    } else {
        Err("invalid-mapping-custody".into())
    }
}
fn shape(v: &Value, fields: &str) -> R<()> {
    let o = v.as_object().ok_or("object-required")?;
    let f: Vec<_> = fields.split_whitespace().collect();
    need(o.len() == f.len() && f.iter().all(|k| o.contains_key(*k)))
}
fn size(v: &Value) -> R<u64> {
    crate::canonical::canonical_json(v)
        .map(|s| s.len() as u64)
        .map_err(|e| format!("{e:?}"))
}
fn id(k: &str, v: &Value) -> R<String> {
    crate::canonical::identify(k, v).map_err(|e| format!("{e:?}"))
}
fn token(v: &Value) -> R<()> {
    crate::identity::token(v).map_err(|e| format!("{e:?}"))
}
fn limit(v: &Value, max: u64) -> R<u64> {
    let s = v.as_str().ok_or("string-required")?;
    need(
        !s.is_empty()
            && s.len() <= 16
            && !s.starts_with('0')
            && s.bytes().all(|b| b.is_ascii_digit()),
    )?;
    let n = s.parse::<u64>().map_err(|_| "invalid-limit")?;
    need(n <= max)?;
    Ok(n)
}
pub fn config_id(c: &Value) -> R<String> {
    size(c)?;
    shape(
        c,
        "kind instance selectionPolicy expectedSelectionPolicyId sourceCustodyId sourceCustodyTip retention disclosure forgiveness retry maximumRequests maximumSubmissionBytes maximumRetainedBytes",
    )?;
    let instance = c["instance"].as_str().ok_or("instance-required")?;
    need(
        c["kind"] == "temple3.claim-mapping-custody/v0.1"
            && instance.len() == 32
            && instance
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
    )?;
    let p = &c["selectionPolicy"];
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
        token(&p[f])?;
    }
    limit(&p["maximumSignedBytes"], 4_194_304)?;
    need(c["expectedSelectionPolicyId"] == id("claim-mapping-selection-policy", p)?)?;
    token(&c["sourceCustodyId"])?;
    token(&c["sourceCustodyTip"])?;
    need(
        c["retention"] == "retain-formation-selections-consents-and-links"
            && c["disclosure"] == "trusted-custodian-only"
            && c["retry"] == "exact-submission-same-request-no-replacement"
            && c["forgiveness"]
                == json!({"disposition":"refuse","scope":"mapping-custody-records"}),
    )?;
    limit(&c["maximumRequests"], 9_007_199_254_740_991)?;
    limit(&c["maximumSubmissionBytes"], 4_194_304)?;
    limit(&c["maximumRetainedBytes"], 9_007_199_254_740_991)?;
    id("claim-mapping-custody", c)
}
pub fn audit(x: &Value) -> R<Value> {
    shape(
        x,
        "config expectedCustodyId expectedTip sourceCustody records",
    )?;
    size(x)?;
    let c = &x["config"];
    let cid = config_id(c)?;
    need(x["expectedCustodyId"] == cid && x["expectedTip"].is_string())?;
    let source =
        crate::source_capsule_custody::audit(&x["sourceCustody"]).map_err(|e| format!("{e:?}"))?;
    need(source["custodyId"] == c["sourceCustodyId"] && source["tip"] == c["sourceCustodyTip"])?;
    crate::identity::verify_records(&x["records"], &json!({"expectedTip":x["expectedTip"]}))
        .map_err(|e| format!("{e:?}"))?;
    let records = x["records"].as_array().ok_or("records-required")?;
    need(
        !records.is_empty()
            && records[0]["entry"]
                == json!({"kind":"temple3.claim-mapping-custody-formation/v0.1","config":c}),
    )?;
    let mut retained = 0u64;
    for r in records {
        retained = retained.checked_add(size(r)?).ok_or("size-overflow")?;
    }
    need(
        (records.len() - 1) as u64 <= limit(&c["maximumRequests"], 9_007_199_254_740_991)?
            && retained <= limit(&c["maximumRetainedBytes"], 9_007_199_254_740_991)?,
    )?;
    let mut seen = HashSet::new();
    let mut selections = vec![];
    for r in records.iter().skip(1) {
        let entry = &r["entry"];
        shape(entry, "kind submission link")?;
        need(entry["kind"] == "temple3.retained-claim-mapping/v0.1")?;
        let sub = &entry["submission"];
        shape(sub, "input consent")?;
        need(size(sub)? <= limit(&c["maximumSubmissionBytes"], 4_194_304)?)?;
        let input = &sub["input"];
        shape(input, "selection sourceDepositRecordId")?;
        token(&input["sourceDepositRecordId"])?;
        let selection = &input["selection"];
        need(
            selection["policy"] == c["selectionPolicy"]
                && selection["expectedPolicyId"] == c["expectedSelectionPolicyId"],
        )?;
        let consent = json!({"kind":"temple3.claim-mapping-retention-consent/v0.1","custodyId":cid,"inputId":id("claim-mapping-custody-input",input)?,"selectionId":id("signed",&selection["selection"])?});
        need(
            crate::identity::verify_envelope(
                &sub["consent"],
                "attestation",
                &c["selectionPolicy"]["authorizerKey"],
            ) && sub["consent"]["payload"] == consent,
        )?;
        let link = crate::claim_retained_mapping::inspect(
            &json!({"custody":x["sourceCustody"],"selection":selection,"expectedDepositRecordId":input["sourceDepositRecordId"]}),
        )?;
        let request = selection["selection"]["payload"]["requestId"]
            .as_str()
            .ok_or("request-required")?;
        need(link == entry["link"] && seen.insert(request))?;
        selections.push(
            json!({"requestId":request,"recordId":r["recordId"],"selectionId":link["selectionId"]}),
        );
    }
    Ok(
        json!({"kind":"temple3.claim-mapping-custody-audit/v0.1","custodyId":cid,"tip":x["expectedTip"],"retainedBytes":retained.to_string(),"selections":selections,"activation":"unavailable"}),
    )
}
