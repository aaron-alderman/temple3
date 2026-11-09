//! Independent explicit claim lifecycle and retained-event reconstruction.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn need(ok: bool) -> R<()> {
    if ok {
        Ok(())
    } else {
        Err("invalid-claim-readiness".into())
    }
}
fn shape(v: &Value, fields: &str) -> R<()> {
    let o = v.as_object().ok_or("invalid-claim-readiness")?;
    need(
        o.len() == fields.split_whitespace().count()
            && fields.split_whitespace().all(|k| o.contains_key(k)),
    )
}
fn text(v: &Value) -> R<&str> {
    v.as_str().ok_or("invalid-claim-readiness".into())
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
fn positive(s: &str) -> bool {
    !s.is_empty() && !s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit())
}
fn bound(v: &Value, max: usize) -> R<usize> {
    let s = text(v)?;
    need(positive(s))?;
    let n = s.parse::<usize>().map_err(|_| "invalid-claim-readiness")?;
    need(n <= max)?;
    Ok(n)
}
fn reference(v: &Value, prefix: &str) -> bool {
    v.as_str()
        .and_then(|s| s.strip_prefix(prefix))
        .is_some_and(positive)
}
fn money(p: &Value, amount: &Value) -> R<Value> {
    crate::exact_money::classify(
        &json!({"policy":p["money"],"amount":amount,"threshold":p["threshold"]}),
    )
}
fn formation(f: &Value) -> R<()> {
    crate::canonical::canonical_json(f).map_err(|_| "invalid-claim-readiness")?;
    shape(f, "kind instance participants policy")?;
    let instance = text(&f["instance"])?;
    need(
        f["kind"] == "temple3.claim-readiness-formation/v0.1"
            && instance.len() == 32
            && instance
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
    )?;
    let people = f["participants"]
        .as_array()
        .ok_or("invalid-claim-readiness")?;
    need(
        (2..=64).contains(&people.len())
            && people.iter().all(actor)
            && people.windows(2).all(|w| w[0].as_str() < w[1].as_str()),
    )?;
    let p = &f["policy"];
    shape(
        p,
        "author reviewer money threshold comparison evidenceMeaning approvalBasis separation repeat readyEdits reversal amendment forgiveness retention disclosure retry maximumRecords maximumEvents",
    )?;
    need(
        people.contains(&p["author"])
            && people.contains(&p["reviewer"])
            && p["author"] != p["reviewer"],
    )?;
    for (k, v) in [
        ("comparison", "strictly-above"),
        (
            "evidenceMeaning",
            "author-asserted-text-not-independent-verification",
        ),
        ("separation", "author-reviewer-distinct"),
        ("repeat", "reject-already-ready"),
        ("readyEdits", "reject-until-reopened"),
        ("amendment", "fixed-formation"),
        ("retention", "retain-all-events-and-approvals"),
        ("disclosure", "participants-all-state"),
        ("retry", "reject-stale-basis"),
    ] {
        need(p[k] == v)?;
    }
    need(
        p["approvalBasis"] == "amount-and-evidence-revision"
            || p["approvalBasis"] == "all-claim-fields-revision",
    )?;
    need(
        p["reversal"] == "author-reopen-invalidates-approval"
            || p["reversal"] == "reviewer-reopen-invalidates-approval",
    )?;
    need(
        p["forgiveness"]
            == json!({"disposition":"refuse","scope":"all-claims-events-and-approvals"}),
    )?;
    bound(&p["maximumRecords"], 64)?;
    bound(&p["maximumEvents"], 256)?;
    money(p, &p["threshold"])?;
    Ok(())
}
fn reconstruct(f: &Value, events: &Value) -> R<Value> {
    let p = &f["policy"];
    let events = events.as_array().ok_or("invalid-claim-readiness")?;
    need(events.len() <= bound(&p["maximumEvents"], 256)?)?;
    let mut records: Vec<Value> = vec![];
    let mut approvals: Vec<Value> = vec![];
    for (i, e) in events.iter().enumerate() {
        shape(e, "actor command")?;
        let c = &e["command"];
        shape(c, "op args")?;
        let op = text(&c["op"])?;
        let a = &c["args"];
        shape(
            a,
            match op {
                "create" => "amount evidence note",
                "revise" => "recordId amount evidence note",
                "approve" | "reopen" => "recordId",
                "ready" => "recordId approvalId",
                _ => return Err("invalid-claim-readiness".into()),
            },
        )?;
        need(actor(&e["actor"]))?;
        let role = if op == "approve"
            || op == "reopen" && p["reversal"] == "reviewer-reopen-invalidates-approval"
        {
            "reviewer"
        } else {
            "author"
        };
        need(e["actor"] == p[role])?;
        if op == "create" || op == "revise" {
            need(text(&a["evidence"])?.len() <= 2048 && text(&a["note"])?.len() <= 128)?;
            money(p, &a["amount"])?;
        }
        if op == "create" {
            need(records.len() < bound(&p["maximumRecords"], 64)?)?;
            records.push(json!({"id":format!("claim.{}",records.len()+1),"revision":"1","amount":a["amount"],"evidence":a["evidence"],"note":a["note"],"status":"draft","readyApprovalId":"none"}));
            continue;
        }
        need(reference(&a["recordId"], "claim."))?;
        let r = records
            .iter_mut()
            .find(|r| r["id"] == a["recordId"])
            .ok_or("invalid-claim-readiness")?;
        if op == "revise" {
            need(r["status"] == "draft")?;
            let relevant = r["amount"] != a["amount"]
                || r["evidence"] != a["evidence"]
                || p["approvalBasis"] == "all-claim-fields-revision" && r["note"] != a["note"];
            need(relevant || r["note"] != a["note"])?;
            if relevant {
                r["revision"] = json!((bound(&r["revision"], 256)? + 1).to_string());
            }
            for k in ["amount", "evidence", "note"] {
                r[k] = a[k].clone();
            }
        } else if op == "reopen" {
            need(r["status"] == "ready")?;
            r["status"] = json!("draft");
            r["readyApprovalId"] = json!("none");
            r["revision"] = json!((bound(&r["revision"], 256)? + 1).to_string());
        } else {
            need(r["status"] == "draft")?;
            let above = money(p, &r["amount"])?["comparison"] == "above";
            let id = crate::canonical::identify("claim-readiness-policy", p)
                .map_err(|_| "invalid-claim-readiness")?;
            let mut basis = json!({"recordId":r["id"],"revision":r["revision"],"policyId":id,"amount":r["amount"],"evidence":r["evidence"]});
            if p["approvalBasis"] == "all-claim-fields-revision" {
                basis["note"] = r["note"].clone();
            }
            if op == "approve" {
                need(above && !approvals.iter().any(|v| v["basis"] == basis))?;
                approvals.push(json!({"id":format!("approval.{}",approvals.len()+1),"basis":basis,"reviewer":e["actor"],"event":(i+1).to_string()}));
            } else {
                need(a["approvalId"] == "none" || reference(&a["approvalId"], "approval."))?;
                if above {
                    need(
                        approvals
                            .iter()
                            .any(|v| v["id"] == a["approvalId"] && v["basis"] == basis),
                    )?;
                } else {
                    need(a["approvalId"] == "none")?;
                }
                r["status"] = json!("ready");
                r["readyApprovalId"] = a["approvalId"].clone();
            }
        }
    }
    let s = json!({"kind":"temple3.claim-readiness-state/v0.1","formation":f,"events":events,"records":records,"approvals":approvals});
    need(
        crate::canonical::canonical_json(&s)
            .map_err(|_| "invalid-claim-readiness")?
            .len()
            <= 131072,
    )?;
    Ok(s)
}
pub fn validate(s: &Value) -> R<Value> {
    crate::canonical::canonical_json(s).map_err(|_| "invalid-claim-readiness")?;
    shape(s, "kind formation events records approvals")?;
    formation(&s["formation"])?;
    let rebuilt = reconstruct(&s["formation"], &s["events"])?;
    need(rebuilt == *s)?;
    Ok(rebuilt)
}
pub fn run(c: &Value) -> R<Value> {
    match text(&c["op"])? {
        "formation" => {
            formation(&c["input"])?;
            reconstruct(&c["input"], &json!([]))
        }
        "validate" => validate(&c["input"]),
        "next" => {
            let s = validate(&c["input"])?;
            crate::canonical::canonical_json(&c["command"])
                .map_err(|_| "invalid-claim-readiness")?;
            let mut events = s["events"]
                .as_array()
                .ok_or("invalid-claim-readiness")?
                .clone();
            events.push(json!({"actor":c["actor"],"command":c["command"]}));
            reconstruct(&s["formation"], &json!(events))
        }
        _ => Err("invalid-claim-readiness".into()),
    }
}

pub fn verify(i: &Value, e: &Value) -> bool {
    shape(i, "before actor command after").is_ok()
        && shape(e, "").is_ok()
        && run(&json!({"op":"next","input":i["before"],"actor":i["actor"],"command":i["command"]}))
            .is_ok_and(|v| v == i["after"])
}
