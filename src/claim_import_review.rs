use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

use crate::claim_import_target as target;
type R<T> = Result<T, String>;
fn need(v: bool) -> R<()> {
    if v { Ok(()) } else { Err("review".into()) }
}
fn shape(v: &Value, keys: &str) -> R<()> {
    let o = v.as_object().ok_or("shape")?;
    need(
        o.len() == keys.split_whitespace().count()
            && keys.split_whitespace().all(|k| o.contains_key(k)),
    )
}
fn canonical(v: &Value) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|e| format!("{e:?}"))
}
fn id(k: &str, v: &Value) -> R<String> {
    crate::canonical::identify(k, v).map_err(|e| format!("{e:?}"))
}
fn limit(v: &Value) -> R<usize> {
    let s = v.as_str().ok_or("limit")?;
    need(
        !s.is_empty()
            && s.len() <= 7
            && !s.starts_with('0')
            && s.bytes().all(|b| b.is_ascii_digit()),
    )?;
    let n = s.parse::<usize>().map_err(|_| "limit")?;
    need(n <= 4194304)?;
    Ok(n)
}
pub fn inspect(x: &Value) -> R<Value> {
    shape(
        x,
        "target policy expectedPolicyId expectedTargetHeadId reviews",
    )?;
    canonical(x)?;
    let p = &x["policy"];
    shape(
        p,
        "kind reviewerActor reviewerKey migratorKey targetProfileId targetFormationId targetHeadId meaning maximumReviewBytes maximumInputBytes",
    )?;
    need(
        p["kind"] == "temple3.claim-import-review-policy/v0.1"
            && p["meaning"] == "fresh-review-of-exact-imported-ready-basis-not-activation",
    )?;
    for k in [
        "reviewerActor",
        "targetProfileId",
        "targetFormationId",
        "targetHeadId",
    ] {
        crate::identity::token(&p[k]).map_err(|e| format!("{e:?}"))?;
    }
    for k in ["reviewerKey", "migratorKey"] {
        crate::identity::key_id(&p[k]).map_err(|e| format!("{e:?}"))?;
    }
    need(p["reviewerKey"] != p["migratorKey"])?;
    let max_review = limit(&p["maximumReviewBytes"])?;
    let max_input = limit(&p["maximumInputBytes"])?;
    let pid = id("claim-import-review-policy", p)?;
    let t = target::inspect(&x["target"])?;
    need(
        x["expectedPolicyId"] == pid
            && p["targetFormationId"] == x["target"]["expectedFormationId"]
            && p["targetHeadId"] == x["expectedTargetHeadId"]
            && p["reviewerActor"] == x["target"]["formation"]["policy"]["reviewer"],
    )?;
    let diagnostics = t["diagnostics"].as_array().ok_or("diagnostics")?;
    need(
        diagnostics
            .iter()
            .all(|d| d["reason"] == "fresh-operative-review-required"),
    )?;
    let required: HashSet<_> = diagnostics
        .iter()
        .map(|d| d["record"].as_str().unwrap())
        .collect();
    let cid = id("claim-import-candidate", &t["candidate"])?;
    let mut payloads = Vec::new();
    for o in t["candidate"]["origins"].as_array().ok_or("origins")? {
        if required.contains(o["sourceId"].as_str().ok_or("source")?) {
            payloads.push(json!({"kind":"temple3.claim-import-ready-review/v0.1","policyId":pid,"targetInputId":id("claim-import-target-input",&x["target"] )?,"candidateId":cid,"sourceId":o["sourceId"],"targetId":o["targetId"],"targetHeadId":p["targetHeadId"],"meaning":p["meaning"]}));
        }
    }
    let reviews = x["reviews"].as_array().ok_or("reviews")?;
    need(canonical(x)?.len() <= max_input && reviews.len() == payloads.len())?;
    let mut checked = HashMap::new();
    for r in reviews {
        need(
            canonical(r)?.len() <= max_review
                && crate::identity::verify_envelope(r, "attestation", &p["reviewerKey"]),
        )?;
        let expected = payloads
            .iter()
            .find(|e| e["sourceId"] == r["payload"]["sourceId"])
            .ok_or("basis")?;
        let source = expected["sourceId"].as_str().ok_or("source")?;
        need(!checked.contains_key(source) && *expected == r["payload"])?;
        checked.insert(source.to_owned(), id("signed", r)?);
    }
    let receipts:Vec<_>=payloads.iter().map(|e|json!({"sourceId":e["sourceId"],"targetId":e["targetId"],"reviewId":checked[e["sourceId"].as_str().unwrap()]})).collect();
    Ok(
        json!({"kind":"temple3.claim-import-review-inspection/v0.1","policyId":pid,"candidateId":cid,"reviews":receipts,"review":"fresh-exact-basis-consent","sourceAuthority":"not-verified","targetProfileRecognition":"not-verified","liveHead":"not-verified","activation":"unavailable"}),
    )
}
