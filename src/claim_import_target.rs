use serde_json::{Value, json};

type R<T> = Result<T, String>;
fn canonical(v: &Value) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|e| format!("{e:?}"))
}
fn id(kind: &str, v: &Value) -> R<String> {
    crate::canonical::identify(kind, v).map_err(|e| format!("{e:?}"))
}
pub fn inspect(x: &Value) -> R<Value> {
    let o = x.as_object().ok_or("shape")?;
    if o.len() != 4
        || ![
            "formation",
            "reconciliation",
            "expectedFormationId",
            "expectedReportId",
        ]
        .iter()
        .all(|k| o.contains_key(*k))
    {
        return Err("shape".into());
    }
    canonical(x)?;
    crate::claim_lifecycle::run(&json!({"op":"formation","input":x["formation"]}))?;
    let report = crate::claim_reconciliation::reconcile(&x["reconciliation"])?;
    if x["expectedFormationId"] != id("claim-lifecycle-formation", &x["formation"])?
        || x["expectedReportId"] != id("claim-reconciliation-report", &report)?
    {
        return Err("anchor".into());
    }
    let p = &x["formation"]["policy"];
    for k in ["money", "threshold", "documentationGuard"] {
        if x["reconciliation"]["policy"][k] != p[k] {
            return Err("policy".into());
        }
    }
    let entries = report["entries"].as_array().ok_or("entries")?;
    let mut diagnostics = Vec::new();
    let mut records = Vec::new();
    let mut origins = Vec::new();
    let limit = p["maximumRecords"]
        .as_str()
        .ok_or("limit")?
        .parse::<usize>()
        .map_err(|_| "limit")?;
    if entries.len() > limit {
        diagnostics
            .push(json!({"record":"all","field":"records","reason":"target-record-capacity"}));
    }
    for (i, e) in entries.iter().enumerate() {
        let mut add = |field: &str, reason: &str| {
            diagnostics.push(json!({"record":e["sourceId"],"field":field,"reason":reason}))
        };
        if e["targetId"] != format!("claim.{}", i + 1) {
            add("targetId", "target-allocation-mismatch");
        }
        for (field, bytes) in [("note", 128), ("evidence", 2048)] {
            if e["proposedFields"][field].as_str().ok_or("text")?.len() > bytes {
                add(field, &format!("target-{field}-byte-capacity"));
            }
        }
        for reason in e["unresolved"].as_array().ok_or("diagnostics")? {
            add("status", reason.as_str().ok_or("reason")?);
        }
        let mut record = e["proposedFields"].as_object().ok_or("fields")?.clone();
        record.insert("id".into(), e["targetId"].clone());
        record.insert("revision".into(), json!("1"));
        record.insert("readyApprovalId".into(), json!("none"));
        records.push(Value::Object(record));
        origins.push(json!({"targetId":e["targetId"],"sourceId":e["sourceId"],"originalRecord":x["reconciliation"]["source"]["records"][i],"historicalEvidence":e["historicalEvidence"],"readiness":e["readiness"]}));
    }
    let candidate = json!({"kind":"temple3.claim-import-initial-candidate/v0.1","formationId":x["expectedFormationId"],"reconciliationReportId":x["expectedReportId"],"records":records,"origins":origins,"events":[],"approvals":[],"activation":"unavailable"});
    if canonical(&candidate)?.len() > 131072 {
        diagnostics.push(
            json!({"record":"all","field":"candidate","reason":"target-candidate-byte-capacity"}),
        );
    }
    Ok(
        json!({"kind":"temple3.claim-import-target-inspection/v0.1","candidate":candidate,"representation":if diagnostics.is_empty(){"selected-fields-fit"}else{"unresolved"},"diagnostics":diagnostics,"sourceInterpretation":"not-verified","originAuthority":"not-verified","nativeReplay":"unavailable","activation":"unavailable"}),
    )
}
