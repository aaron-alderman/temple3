use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn need(v: bool) -> R<()> {
    if v {
        Ok(())
    } else {
        Err("invalid-retained-mapping".into())
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
pub fn inspect(x: &Value) -> R<Value> {
    shape(x, "custody expectedDepositRecordId selection")?;
    canonical(x)?;
    crate::identity::token(&x["expectedDepositRecordId"]).map_err(|e| format!("{e:?}"))?;
    let custody =
        crate::source_capsule_custody::audit(&x["custody"]).map_err(|e| format!("{e:?}"))?;
    let selection = crate::claim_mapping_selection::inspect(&x["selection"])?;
    let deposit = custody["deposits"]
        .as_array()
        .ok_or("deposits-required")?
        .iter()
        .find(|d| d["recordId"] == x["expectedDepositRecordId"])
        .ok_or("missing-deposit")?;
    need(
        deposit["inspection"]["capsuleId"]
            == x["selection"]["reconciliation"]["source"]["capsuleId"],
    )?;
    Ok(
        json!({"kind":"temple3.retained-claim-mapping/v0.1","custodyId":custody["custodyId"],"custodyTip":custody["tip"],"depositRecordId":deposit["recordId"],"capsuleId":deposit["inspection"]["capsuleId"],"selectionId":selection["selectionId"],"reconciliationReportId":selection["reconciliationReportId"],"contentLink":"anchored-retained-capsule-reference","interpretation":"not-verified","sourceAuthority":"not-verified","targetRecognition":"not-verified","activation":"unavailable"}),
    )
}
