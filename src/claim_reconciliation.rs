use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
type R<T> = Result<T, String>;
fn need(v: bool) -> R<()> {
    if v {
        Ok(())
    } else {
        Err("invalid-reconciliation".into())
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
    let s = v.as_str().ok_or("text-required")?;
    need(!s.is_empty())?;
    Ok(s)
}
fn limit(v: &Value) -> R<u64> {
    let s = text(v)?;
    need(s.len() <= 16 && !s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit()))?;
    let n = s.parse::<u64>().map_err(|_| "invalid-limit")?;
    need(n <= 9_007_199_254_740_991)?;
    Ok(n)
}
fn money(v: &Value, p: &Value) -> R<Value> {
    crate::exact_money::classify(
        &json!({"policy":p["money"],"threshold":p["threshold"],"amount":v}),
    )
}
pub fn reconcile(x: &Value) -> R<Value> {
    shape(
        x,
        "source policy mappings expectedSourceId expectedPolicyId",
    )?;
    let encoded = canonical(x)?;
    let s = &x["source"];
    let p = &x["policy"];
    shape(s, "kind capsuleId interpretation records")?;
    need(
        s["kind"] == "temple3.claim-mapping-hypothesis/v0.1"
            && s["interpretation"] == "hypothesis-not-verified-source",
    )?;
    text(&s["capsuleId"])?;
    shape(
        p,
        "kind money threshold maximumRecords maximumInputBytes historicalApproval activation documentationGuard",
    )?;
    need(
        p["kind"] == "temple3.claim-reconciliation-policy/v0.1"
            && p["historicalApproval"] == "retain-as-non-operative-evidence"
            && p["activation"] == "unavailable",
    )?;
    need(p["documentationGuard"] == "required" || p["documentationGuard"] == "not-required")?;
    let maximum = limit(&p["maximumRecords"])?;
    let bytes = limit(&p["maximumInputBytes"])?;
    money(&p["threshold"], p)?;
    need(
        x["expectedSourceId"] == id("claim-mapping-hypothesis", s)?
            && x["expectedPolicyId"] == id("claim-reconciliation-policy", p)?,
    )?;
    need(encoded.len() as u64 <= bytes)?;
    let records = s["records"].as_array().ok_or("records-required")?;
    let maps = x["mappings"].as_array().ok_or("mappings-required")?;
    need(records.len() as u64 <= maximum && maps.len() == records.len())?;
    let mut sources = HashMap::new();
    for r in records {
        shape(
            r,
            "sourceId estimate evidence note documentation membership status historicalApproval",
        )?;
        let source = text(&r["sourceId"])?;
        need(sources.insert(source, r).is_none())?;
        need(
            r["evidence"].is_string()
                && r["note"].is_string()
                && r["historicalApproval"].is_boolean(),
        )?;
        need(r["documentation"] == "documented" || r["documentation"] == "undocumented")?;
        need(r["membership"] == "active" || r["membership"] == "retired")?;
        need(r["status"] == "draft" || r["status"] == "ready")?;
        if r["estimate"]["state"] == "missing" {
            shape(&r["estimate"], "state")?;
        } else {
            shape(&r["estimate"], "state money")?;
            need(r["estimate"]["state"] == "specified")?;
            money(&r["estimate"]["money"], p)?;
        }
    }
    let mut targets = HashSet::new();
    let mut entries = HashMap::new();
    for m in maps {
        shape(m, "sourceId targetId readiness reason")?;
        let source = text(&m["sourceId"])?;
        let target = text(&m["targetId"])?;
        text(&m["reason"])?;
        need(!entries.contains_key(source) && targets.insert(target))?;
        let r = *sources.get(source).ok_or("unknown-source")?;
        need(m["readiness"] == "preserve-state" || m["readiness"] == "reset-to-draft")?;
        let status = if m["readiness"] == "reset-to-draft" {
            "draft"
        } else {
            text(&r["status"])?
        };
        let mut unresolved = vec![];
        if status == "ready" {
            if r["estimate"]["state"] == "missing" {
                unresolved.push("ready-estimate-unavailable");
            }
            if r["membership"] == "retired" {
                unresolved.push("ready-membership-inactive");
            }
            if p["documentationGuard"] == "required" && r["documentation"] != "documented" {
                unresolved.push("ready-documentation-required");
            }
            if r["estimate"]["state"] == "specified"
                && money(&r["estimate"]["money"], p)?["comparison"] == "above"
            {
                unresolved.push("fresh-operative-review-required");
            }
        }
        let mut fields = r.as_object().ok_or("record-required")?.clone();
        fields.remove("sourceId");
        fields.remove("historicalApproval");
        fields.insert("status".into(), json!(status));
        entries.insert(source,json!({"sourceId":source,"targetId":target,"proposedFields":fields,"historicalEvidence":{"capsuleId":s["capsuleId"],"historicalApproval":r["historicalApproval"]},"readiness":{"selection":m["readiness"],"disposition":if r["status"]==status{"preserved"}else{"changed"},"reason":m["reason"]},"unresolved":unresolved,"activation":"unavailable"}));
    }
    let mut ordered = vec![];
    for r in records {
        ordered.push(
            entries
                .remove(text(&r["sourceId"])?)
                .ok_or("missing-mapping")?,
        );
    }
    Ok(
        json!({"kind":"temple3.claim-reconciliation/v0.1","sourceId":x["expectedSourceId"],"policyId":x["expectedPolicyId"],"entries":ordered,"coverage":{"input":records.len().to_string(),"mapped":maps.len().to_string(),"omitted":"0"},"authority":"not-verified","activation":"unavailable"}),
    )
}
