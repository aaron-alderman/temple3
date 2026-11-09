//! Independent explicit claim lifecycle and retained-event reconstruction.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn need(ok: bool) -> R<()> {
    if ok {
        Ok(())
    } else {
        Err("invalid-claim-lifecycle".into())
    }
}
fn shape(v: &Value, fields: &str) -> R<()> {
    let o = v.as_object().ok_or("invalid-claim-lifecycle")?;
    need(
        o.len() == fields.split_whitespace().count()
            && fields.split_whitespace().all(|k| o.contains_key(k)),
    )
}
fn text(v: &Value) -> R<&str> {
    v.as_str().ok_or("invalid-claim-lifecycle".into())
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
    let n = s.parse::<usize>().map_err(|_| "invalid-claim-lifecycle")?;
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
    crate::canonical::canonical_json(f).map_err(|_| "invalid-claim-lifecycle")?;
    shape(f, "kind instance participants policy")?;
    let instance = text(&f["instance"])?;
    need(
        f["kind"] == "temple3.claim-lifecycle-formation/v0.1"
            && instance.len() == 32
            && instance
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
    )?;
    let people = f["participants"]
        .as_array()
        .ok_or("invalid-claim-lifecycle")?;
    need(
        (2..=64).contains(&people.len())
            && people.iter().all(actor)
            && people.windows(2).all(|w| w[0].as_str() < w[1].as_str()),
    )?;
    let p = &f["policy"];
    shape(
        p,
        "author reviewer money threshold comparison evidenceMeaning approvalBasis separation repeat readyEdits reversal amendment forgiveness retention disclosure retry maximumRecords maximumEvents estimatePolicy clearEstimate initialDocumentation initialMembership documentationMeaning documentationGuard documentationEdits membershipPolicy",
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
        p["approvalBasis"] == "estimate-and-evidence-revision"
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
    for (k, v) in [
        ("estimatePolicy", "explicit-missing-or-specified"),
        ("clearEstimate", "author-draft-revision"),
        ("initialDocumentation", "undocumented"),
        ("initialMembership", "active"),
        (
            "documentationMeaning",
            "author-marked-not-independent-verification",
        ),
        ("documentationEdits", "revision-relevant"),
        (
            "membershipPolicy",
            "author-draft-retire-reactivate-with-new-revision",
        ),
    ] {
        need(p[k] == v)?;
    }
    need(p["documentationGuard"] == "required" || p["documentationGuard"] == "not-required")?;
    bound(&p["maximumEvents"], 256)?;
    money(p, &p["threshold"])?;
    Ok(())
}
fn estimate(v: &Value, p: &Value) -> R<()> {
    if v["state"] == "missing" {
        shape(v, "state")?;
    } else {
        shape(v, "state money")?;
        need(v["state"] == "specified")?;
        money(p, &v["money"])?;
    }
    Ok(())
}
fn reconstruct(input: &Value, events: &Value) -> R<Value> {
    let consent = crate::claim_import_review::inspect(input)?;
    let target = crate::claim_import_target::inspect(&input["target"])?;
    let f = &input["target"]["formation"];
    formation(f)?;
    let p = &f["policy"];
    let events = events.as_array().ok_or("events")?;
    need(events.len() <= bound(&p["maximumEvents"], 256)?)?;
    let mut records = target["candidate"]["records"]
        .as_array()
        .ok_or("records")?
        .clone();
    let mut approvals: Vec<Value> = vec![];
    for review in consent["reviews"].as_array().ok_or("reviews")? {
        let r = records
            .iter_mut()
            .find(|r| r["id"] == review["targetId"])
            .ok_or("review-record")?;
        need(r["status"] == "ready")?;
        let mut basis = json!({"recordId":r["id"],"revision":r["revision"],"policyId":crate::canonical::identify("claim-lifecycle-policy",p).map_err(|e|format!("{e:?}"))?,"estimate":r["estimate"],"evidence":r["evidence"],"documentation":r["documentation"],"membership":r["membership"]});
        if p["approvalBasis"] == "all-claim-fields-revision" {
            basis["note"] = r["note"].clone();
        }
        let aid = format!("approval.{}", approvals.len() + 1);
        approvals.push(json!({"id":aid,"basis":basis,"reviewer":p["reviewer"],"origin":{"kind":"temple3.imported-ready-approval-origin/v0.1","reviewId":review["reviewId"],"candidateId":consent["candidateId"],"reviewPolicyId":consent["policyId"],"sourceId":review["sourceId"]}}));
        r["readyApprovalId"] = json!(aid);
    }

    for (i, e) in events.iter().enumerate() {
        shape(e, "actor command")?;
        let c = &e["command"];
        shape(c, "op args")?;
        let op = text(&c["op"])?;
        let a = &c["args"];
        shape(
            a,
            match op {
                "create" => "estimate evidence note",
                "revise" => "recordId estimate evidence note",
                "approve" | "reopen" | "retire" | "reactivate" => "recordId",
                "set-documentation" => "recordId documentation",
                "ready" => "recordId approvalId",
                _ => return Err("invalid-claim-lifecycle".into()),
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
            estimate(&a["estimate"], p)?;
        }
        if op == "create" {
            need(records.len() < bound(&p["maximumRecords"], 64)?)?;
            records.push(json!({"id":format!("claim.{}",records.len()+1),"revision":"1","estimate":a["estimate"],"evidence":a["evidence"],"note":a["note"],"documentation":p["initialDocumentation"],"membership":p["initialMembership"],"status":"draft","readyApprovalId":"none"}));
            continue;
        }
        need(reference(&a["recordId"], "claim."))?;
        let r = records
            .iter_mut()
            .find(|r| r["id"] == a["recordId"])
            .ok_or("invalid-claim-lifecycle")?;
        need(
            r["membership"]
                == if op == "reactivate" {
                    "retired"
                } else {
                    "active"
                },
        )?;
        if op == "revise" {
            need(r["status"] == "draft")?;
            let relevant = r["estimate"] != a["estimate"]
                || r["evidence"] != a["evidence"]
                || p["approvalBasis"] == "all-claim-fields-revision" && r["note"] != a["note"];
            need(relevant || r["note"] != a["note"])?;
            if relevant {
                r["revision"] = json!((bound(&r["revision"], 256)? + 1).to_string());
            }
            for k in ["estimate", "evidence", "note"] {
                r[k] = a[k].clone();
            }
        } else if op == "set-documentation" {
            need(
                (a["documentation"] == "undocumented" || a["documentation"] == "documented")
                    && r["status"] == "draft"
                    && r["documentation"] != a["documentation"],
            )?;
            r["documentation"] = a["documentation"].clone();
            r["revision"] = json!((bound(&r["revision"], 256)? + 1).to_string());
        } else if op == "retire" || op == "reactivate" {
            need(r["status"] == "draft")?;
            r["membership"] = json!(if op == "retire" { "retired" } else { "active" });
            r["revision"] = json!((bound(&r["revision"], 256)? + 1).to_string());
            r["readyApprovalId"] = json!("none");
        } else if op == "reopen" {
            need(r["status"] == "ready")?;
            r["status"] = json!("draft");
            r["readyApprovalId"] = json!("none");
            r["revision"] = json!((bound(&r["revision"], 256)? + 1).to_string());
        } else {
            need(r["status"] == "draft")?;
            need(r["estimate"]["state"] == "specified")?;
            let above = money(p, &r["estimate"]["money"])?["comparison"] == "above";
            let id = crate::canonical::identify("claim-lifecycle-policy", p)
                .map_err(|_| "invalid-claim-lifecycle")?;
            let mut basis = json!({"recordId":r["id"],"revision":r["revision"],"policyId":id,"estimate":r["estimate"],"evidence":r["evidence"],"documentation":r["documentation"],"membership":r["membership"]});
            if p["approvalBasis"] == "all-claim-fields-revision" {
                basis["note"] = r["note"].clone();
            }
            if op == "approve" {
                need(above && !approvals.iter().any(|v| v["basis"] == basis))?;
                approvals.push(json!({"id":format!("approval.{}",approvals.len()+1),"basis":basis,"reviewer":e["actor"],"event":(i+1).to_string()}));
            } else {
                need(
                    p["documentationGuard"] == "not-required" || r["documentation"] == "documented",
                )?;
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
    let s = json!({"kind":"temple3.claim-import-replay/v0.1","input":input,"origins":target["candidate"]["origins"],"events":events,"records":records,"approvals":approvals,"originAuthority":"not-verified","activation":"unavailable"});
    need(
        crate::canonical::canonical_json(&s)
            .map_err(|_| "invalid-claim-lifecycle")?
            .len()
            <= 131072,
    )?;
    Ok(s)
}
pub fn validate(s: &Value) -> R<Value> {
    crate::canonical::canonical_json(s).map_err(|_| "canonical")?;
    shape(
        s,
        "kind input origins events records approvals originAuthority activation",
    )?;
    need(s["kind"] == "temple3.claim-import-replay/v0.1")?;
    let rebuilt = reconstruct(&s["input"], &s["events"])?;
    need(rebuilt == *s)?;
    Ok(rebuilt)
}
pub fn run(c: &Value) -> R<Value> {
    match text(&c["op"])? {
        "formation" => {
            shape(c, "op input")?;
            reconstruct(&c["input"], &json!([]))
        }
        "validate" => {
            shape(c, "op input")?;
            validate(&c["input"])
        }
        "next" => {
            shape(c, "op input actor command")?;
            let s = validate(&c["input"])?;
            let mut events = s["events"].as_array().ok_or("events")?.clone();
            events.push(json!({"actor":c["actor"],"command":c["command"]}));
            reconstruct(&s["input"], &json!(events))
        }
        _ => Err("operation".into()),
    }
}
