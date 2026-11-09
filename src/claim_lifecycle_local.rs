//! Selected-cell lifecycle computation, independently implemented.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
const PREFIX: &str = "temple3.claim-lifecycle-partition/";
fn need(ok: bool) -> R<()> {
    if ok {
        Ok(())
    } else {
        Err("invalid-claim-lifecycle-local".into())
    }
}
fn shape(v: &Value, fields: &str) -> R<()> {
    let o = v.as_object().ok_or("invalid-claim-lifecycle-local")?;
    need(
        o.len() == fields.split_whitespace().count()
            && fields.split_whitespace().all(|k| o.contains_key(k)),
    )
}
fn text(v: &Value) -> R<&str> {
    v.as_str().ok_or("invalid-claim-lifecycle-local".into())
}
fn number(v: &Value, max: usize, zero: bool) -> R<usize> {
    let s = text(v)?;
    need(
        !s.is_empty()
            && s.bytes().all(|b| b.is_ascii_digit())
            && (s == "0" && zero || !s.starts_with('0')),
    )?;
    let n = s
        .parse::<usize>()
        .map_err(|_| "invalid-claim-lifecycle-local")?;
    need(n <= max)?;
    Ok(n)
}
fn size(v: &Value) -> R<usize> {
    Ok(crate::canonical::canonical_json(v)
        .map_err(|_| "invalid-claim-lifecycle-local")?
        .len())
}
fn id(kind: &str, v: &Value) -> R<String> {
    crate::canonical::identify(kind, v).map_err(|_| "invalid-claim-lifecycle-local".into())
}
fn cell(v: Value) -> R<Value> {
    Ok(json!({"id":id("cell",&v)?,"value":v}))
}
fn kind(v: &Value, category: &str) -> bool {
    v["value"]["kind"] == format!("{PREFIX}{category}/v0.1")
}
fn content(v: &Value, max: usize) -> R<()> {
    need(text(v)?.len() <= max)
}
fn money(p: &Value, v: &Value) -> R<Value> {
    crate::exact_money::classify(
        &json!({"policy":p["money"],"amount":v,"threshold":p["threshold"]}),
    )
}
fn reference(v: &Value, max: usize) -> R<()> {
    if v == "none" {
        return Ok(());
    }
    let suffix = text(v)?
        .strip_prefix("approval.")
        .ok_or("invalid-claim-lifecycle-local")?;
    number(&json!(suffix), max, false)?;
    Ok(())
}
fn estimate(p: &Value, v: &Value) -> R<()> {
    if v["state"] == "missing" {
        shape(v, "state")?;
    } else {
        shape(v, "state money")?;
        need(v["state"] == "specified")?;
        money(p, &v["money"])?;
    }
    Ok(())
}
fn wrap(category: &str, index: &str, value: Value, fid: &Value, current: Option<&str>) -> R<Value> {
    let mut w = json!({"kind":format!("{PREFIX}{category}/v0.1"),"formationId":fid,"index":index,"value":value});
    if let Some(a) = current {
        w["currentApprovalId"] = json!(a);
    }
    cell(w)
}
pub fn compute(x: &Value) -> R<Value> {
    size(x)?;
    shape(x, "actor command read")?;
    let actor = &x["actor"];
    let command = &x["command"];
    shape(command, "op args")?;
    let op = text(&command["op"])?;
    let a = &command["args"];
    shape(
        a,
        match op {
            "create" => "estimate evidence note",
            "revise" => "recordId estimate evidence note",
            "approve" | "reopen" | "retire" | "reactivate" => "recordId",
            "set-documentation" => "recordId documentation",
            "ready" => "recordId approvalId",
            _ => return Err("invalid-claim-lifecycle-local".into()),
        },
    )?;
    let read = x["read"]
        .as_array()
        .ok_or("invalid-claim-lifecycle-local")?;
    need((1..=3).contains(&read.len()))?;
    for (i, c) in read.iter().enumerate() {
        shape(c, "id value")?;
        need(
            cell(c["value"].clone())? == *c
                && (i == 0 || text(&read[i - 1]["id"])? < text(&c["id"])?),
        )?;
    }
    let heads: Vec<_> = read.iter().filter(|c| kind(c, "head")).collect();
    need(heads.len() == 1)?;
    let hc = heads[0];
    let h = &hc["value"];
    shape(
        h,
        "kind formationId formation eventCount recordCount approvalCount logicalBytes",
    )?;
    let initial = crate::claim_lifecycle::run(&json!({"op":"formation","input":h["formation"]}))?;
    let p = &h["formation"]["policy"];
    let fid = &h["formationId"];
    need(*fid == id("claim-lifecycle-formation", &h["formation"])?)?;
    let max_events = number(&p["maximumEvents"], 256, false)?;
    let max_records = number(&p["maximumRecords"], 64, false)?;
    let events = number(&h["eventCount"], max_events, true)?;
    let records = number(&h["recordCount"], max_records, true)?;
    let approvals = number(&h["approvalCount"], events, true)?;
    let bytes = number(&h["logicalBytes"], 131072, false)?;
    need(records <= events && bytes >= size(&initial)? && events < max_events)?;
    let role = if op == "approve"
        || op == "reopen" && p["reversal"] == "reviewer-reopen-invalidates-approval"
    {
        "reviewer"
    } else {
        "author"
    };
    need(*actor == p[role])?;
    let rc: Vec<_> = read.iter().filter(|c| kind(c, "record")).collect();
    let ac: Vec<_> = read.iter().filter(|c| kind(c, "approval")).collect();
    need(read.len() == 1 + rc.len() + ac.len() && rc.len() == if op == "create" { 0 } else { 1 })?;
    let mut previous = None;
    let mut r = Value::Null;
    let mut current = "none".to_string();
    let mut record_index = (records + 1).to_string();
    if !rc.is_empty() {
        let w = &rc[0]["value"];
        shape(w, "kind formationId index value currentApprovalId")?;
        need(w["formationId"] == *fid)?;
        number(&w["index"], records, false)?;
        record_index = text(&w["index"])?.to_string();
        r = w["value"].clone();
        shape(
            &r,
            "id revision estimate evidence note documentation membership status readyApprovalId",
        )?;
        need(r["id"] == format!("claim.{record_index}") && r["id"] == a["recordId"])?;
        number(&r["revision"], events, false)?;
        estimate(p, &r["estimate"])?;
        need(r["documentation"] == "documented" || r["documentation"] == "undocumented")?;
        need(
            r["membership"]
                == if op == "reactivate" {
                    "retired"
                } else {
                    "active"
                },
        )?;
        content(&r["evidence"], 2048)?;
        content(&r["note"], 128)?;
        need(r["status"] == "draft" || r["status"] == "ready")?;
        reference(&r["readyApprovalId"], approvals)?;
        reference(&w["currentApprovalId"], approvals)?;
        need(r["status"] != "draft" || r["readyApprovalId"] == "none")?;
        current = text(&w["currentApprovalId"])?.to_string();
        previous = Some(r.clone());
    }
    let requires = op == "ready" && a["approvalId"] != "none";
    need(ac.len() == usize::from(requires))?;
    if op == "create" || op == "revise" {
        estimate(p, &a["estimate"])?;
        content(&a["evidence"], 2048)?;
        content(&a["note"], 128)?;
    }
    let mut new_approval = None;
    if op == "create" {
        need(records < max_records)?;
        r = json!({"id":format!("claim.{}",records+1),"revision":"1","estimate":a["estimate"],"evidence":a["evidence"],"note":a["note"],"documentation":p["initialDocumentation"],"membership":p["initialMembership"],"status":"draft","readyApprovalId":"none"});
    } else if op == "revise" {
        need(r["status"] == "draft")?;
        let relevant = r["estimate"] != a["estimate"]
            || r["evidence"] != a["evidence"]
            || p["approvalBasis"] == "all-claim-fields-revision" && r["note"] != a["note"];
        need(relevant || r["note"] != a["note"])?;
        if relevant {
            r["revision"] = json!((number(&r["revision"], events, false)? + 1).to_string());
            current = "none".into();
        }
        for k in ["estimate", "evidence", "note"] {
            r[k] = a[k].clone();
        }
    } else if op == "set-documentation" {
        need(
            r["status"] == "draft"
                && (a["documentation"] == "documented" || a["documentation"] == "undocumented")
                && r["documentation"] != a["documentation"],
        )?;
        r["documentation"] = a["documentation"].clone();
        r["revision"] = json!((number(&r["revision"], events, false)? + 1).to_string());
        current = "none".into();
    } else if op == "retire" || op == "reactivate" {
        need(r["status"] == "draft")?;
        r["membership"] = json!(if op == "retire" { "retired" } else { "active" });
        r["revision"] = json!((number(&r["revision"], events, false)? + 1).to_string());
        r["readyApprovalId"] = json!("none");
        current = "none".into();
    } else if op == "reopen" {
        need(r["status"] == "ready")?;
        r["status"] = json!("draft");
        r["readyApprovalId"] = json!("none");
        r["revision"] = json!((number(&r["revision"], events, false)? + 1).to_string());
        current = "none".into();
    } else {
        need(r["status"] == "draft" && r["estimate"]["state"] == "specified")?;
        let above = money(p, &r["estimate"]["money"])?["comparison"] == "above";
        let mut basis = json!({"recordId":r["id"],"revision":r["revision"],"policyId":id("claim-lifecycle-policy",p)?,"estimate":r["estimate"],"evidence":r["evidence"],"documentation":r["documentation"],"membership":r["membership"]});
        if p["approvalBasis"] == "all-claim-fields-revision" {
            basis["note"] = r["note"].clone();
        }
        if op == "approve" {
            need(above && current == "none")?;
            current = format!("approval.{}", approvals + 1);
            new_approval = Some(
                json!({"id":current,"basis":basis,"reviewer":actor,"event":(events+1).to_string()}),
            );
        } else {
            need(p["documentationGuard"] == "not-required" || r["documentation"] == "documented")?;
            if above {
                need(requires && a["approvalId"] == current)?;
                let av = &ac[0]["value"];
                shape(av, "kind formationId index value")?;
                need(av["formationId"] == *fid)?;
                number(&av["index"], approvals, false)?;
                let avalue = &av["value"];
                shape(avalue, "id basis reviewer event")?;
                number(&avalue["event"], events, false)?;
                need(
                    avalue["id"] == format!("approval.{}", text(&av["index"])?)
                        && avalue["id"] == a["approvalId"]
                        && avalue["reviewer"] == p["reviewer"]
                        && avalue["basis"] == basis,
                )?;
            } else {
                need(a["approvalId"] == "none")?;
            }
            r["status"] = json!("ready");
            r["readyApprovalId"] = a["approvalId"].clone();
        }
    }
    let event = json!({"actor":actor,"command":command});
    let mut head = h.clone();
    head["eventCount"] = json!((events + 1).to_string());
    if op == "create" {
        head["recordCount"] = json!((records + 1).to_string());
    }
    if new_approval.is_some() {
        head["approvalCount"] = json!((approvals + 1).to_string());
    }
    let mut total = bytes as i64 + size(&event)? as i64 + i64::from(events > 0) + size(&r)? as i64;
    if let Some(old) = &previous {
        total -= size(old)? as i64;
    } else {
        total += i64::from(records > 0);
    }
    if let Some(approval) = &new_approval {
        total += size(approval)? as i64 + i64::from(approvals > 0);
    }
    need((1..=131072).contains(&total))?;
    head["logicalBytes"] = json!(total.to_string());
    let record = wrap("record", &record_index, r, fid, Some(&current))?;
    let mut create = vec![
        cell(head)?,
        wrap("event", &(events + 1).to_string(), event, fid, None)?,
    ];
    let mut consume = vec![text(&hc["id"])?.to_string()];
    if rc.is_empty() || record["id"] != rc[0]["id"] {
        create.push(record);
        if !rc.is_empty() {
            consume.push(text(&rc[0]["id"])?.to_string());
        }
    }
    if let Some(approval) = new_approval {
        create.push(wrap(
            "approval",
            &(approvals + 1).to_string(),
            approval,
            fid,
            None,
        )?);
    }
    consume.sort();
    create.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    Ok(json!({"consume":consume,"create":create}))
}

pub fn initial_head(f: &Value) -> R<Value> {
    let initial = crate::claim_lifecycle::run(&json!({"op":"formation","input":f}))?;
    Ok(
        json!({"kind":format!("{PREFIX}head/v0.1"),"formationId":id("claim-lifecycle-formation",f)?,"formation":f,"eventCount":"0","recordCount":"0","approvalCount":"0","logicalBytes":size(&initial)?.to_string()}),
    )
}
pub fn verify_bound(i: &Value, e: &Value) -> bool {
    if shape(i, "actor command read readIds consume create").is_err() || shape(e, "").is_err() {
        return false;
    }
    let Some(read) = i["read"].as_array() else {
        return false;
    };
    if i["readIds"] != json!(read.iter().map(|c| c["id"].clone()).collect::<Vec<_>>()) {
        return false;
    }
    compute(&json!({"actor":i["actor"],"command":i["command"],"read":i["read"]}))
        .is_ok_and(|v| v == json!({"consume":i["consume"],"create":i["create"]}))
}
pub fn verify_compact(i: &Value, e: &Value) -> bool {
    if shape(i, "actor command read readIds consume createIds").is_err() || shape(e, "").is_err() {
        return false;
    }
    let Some(read) = i["read"].as_array() else {
        return false;
    };
    if i["readIds"] != json!(read.iter().map(|c| c["id"].clone()).collect::<Vec<_>>()) {
        return false;
    }
    compute(&json!({"actor":i["actor"],"command":i["command"],"read":i["read"]})).is_ok_and(|v| {
        v["consume"] == i["consume"]
            && v["create"].as_array().is_some_and(|cells| {
                i["createIds"] == json!(cells.iter().map(|c| c["id"].clone()).collect::<Vec<_>>())
            })
    })
}
