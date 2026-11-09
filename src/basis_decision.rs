//! Standalone exact-basis decision semantics; no host or source-runtime authority.
use serde_json::{Value, json};
type R<T> = std::result::Result<T, String>;
fn need(ok: bool, code: &str) -> R<()> {
    if ok { Ok(()) } else { Err(code.into()) }
}
fn exact(v: &Value, keys: &str, c: &str) -> R<()> {
    crate::shape(v, keys).map_err(|_| c.into())
}
fn canonical(v: &Value, c: &str) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|_| c.into())
}
fn num(v: &Value, max: u32) -> Option<u32> {
    let s = v.as_str()?;
    let n = s.parse::<u32>().ok()?;
    (n <= max && n.to_string() == s).then_some(n)
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
fn unit(v: &Value) -> bool {
    v.as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 8 && s.bytes().all(|b| b.is_ascii_uppercase()))
}
fn reference(v: &Value, prefix: &str) -> bool {
    v.as_str()
        .and_then(|s| s.strip_prefix(prefix))
        .is_some_and(|s| {
            !s.is_empty() && s.as_bytes()[0] != b'0' && s.bytes().all(|b| b.is_ascii_digit())
        })
}
fn note(v: &Value) -> bool {
    v.as_str().is_some_and(|s| s.len() <= 128)
}
fn policy(p: &Value, participants: &[Value], c: &str) -> R<()> {
    exact(
        p,
        "id author reviewer steward threshold unit decisionValidity separation forgiveness retention disclosure amendment retry",
        c,
    )?;
    need(
        reference(&p["id"], "policy.")
            && ["author", "reviewer", "steward"]
                .iter()
                .all(|k| participants.contains(&p[*k]))
            && p["author"] != p["reviewer"],
        c,
    )?;
    need(
        num(&p["threshold"], 999999999).is_some() && unit(&p["unit"]),
        c,
    )?;
    for (k, v) in [
        ("decisionValidity", "exact-record-version-and-policy"),
        ("separation", "author-reviewer-distinct"),
        ("retention", "retain-all-profile-history"),
        ("disclosure", "participants-all-profile-state"),
        ("amendment", "current-steward-only"),
        ("retry", "reject-stale-basis"),
    ] {
        need(p[k] == v, c)?;
    }
    need(
        p["forgiveness"]
            == json!({"disposition":"refuse","scope":"all-profile-records-and-decisions"}),
        c,
    )
}
pub fn validate(s: &Value) -> R<()> {
    let c = "invalid-state";
    let bytes = canonical(s, c)?;
    exact(
        s,
        "kind instance participants sequence policies records decisions uses",
        c,
    )?;
    need(
        s["kind"] == "temple3.basis-decision-state/v0.1"
            && s["instance"].as_str().is_some_and(|s| {
                s.len() == 32
                    && s.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }),
        c,
    )?;
    let seq = num(&s["sequence"], 1024).ok_or(c)?;
    let people = s["participants"].as_array().ok_or(c)?;
    need(
        !people.is_empty()
            && people.len() <= 8
            && people.iter().all(actor)
            && people.windows(2).all(|w| w[0].as_str() < w[1].as_str()),
        c,
    )?;
    for (key, limit, prefix) in [
        ("policies", 4, "policy"),
        ("records", 4, "record"),
        ("decisions", 8, "decision"),
        ("uses", 8, "use"),
    ] {
        let entries = s[key].as_array().ok_or(c)?;
        need(entries.len() <= limit, c)?;
        for (i, v) in entries.iter().enumerate() {
            need(v["id"] == format!("{prefix}.{}", i + 1), c)?;
        }
    }
    let ps = s["policies"].as_array().ok_or(c)?;
    need(!ps.is_empty(), c)?;
    for p in ps {
        policy(p, people, c)?;
    }
    let rs = s["records"].as_array().ok_or(c)?;
    for r in rs {
        exact(r, "id version amount unit note", c)?;
        need(
            num(&r["version"], seq).is_some_and(|n| n > 0)
                && num(&r["amount"], 999999999).is_some()
                && unit(&r["unit"])
                && note(&r["note"]),
            c,
        )?;
    }
    let ds = s["decisions"].as_array().ok_or(c)?;
    for (field, keys) in [
        (
            "decisions",
            "id recordId recordVersion amount unit policyId reviewer",
        ),
        (
            "uses",
            "id recordId recordVersion amount unit policyId decisionId actor",
        ),
    ] {
        for h in s[field].as_array().ok_or(c)? {
            exact(h, keys, c)?;
            let r = rs.iter().find(|r| r["id"] == h["recordId"]).ok_or(c)?;
            let p = ps.iter().find(|p| p["id"] == h["policyId"]).ok_or(c)?;
            need(
                num(&h["recordVersion"], num(&r["version"], 1024).ok_or(c)?).is_some_and(|n| n > 0)
                    && num(&h["amount"], 999999999).is_some()
                    && h["unit"] == p["unit"],
                c,
            )?;
            if field == "decisions" {
                need(
                    h["reviewer"] == p["reviewer"]
                        && num(&h["amount"], 999999999) > num(&p["threshold"], 999999999),
                    c,
                )?;
            } else {
                need(h["actor"] == p["author"], c)?;
                if h["decisionId"] == "none" {
                    need(
                        num(&h["amount"], 999999999) <= num(&p["threshold"], 999999999),
                        c,
                    )?;
                } else {
                    let d = ds.iter().find(|d| d["id"] == h["decisionId"]).ok_or(c)?;
                    need(
                        ["recordId", "recordVersion", "amount", "unit", "policyId"]
                            .iter()
                            .all(|k| d[*k] == h[*k]),
                        c,
                    )?;
                }
            }
        }
    }
    need(
        seq as usize >= ps.len() + rs.len() + ds.len() + s["uses"].as_array().ok_or(c)?.len() - 1
            && bytes.len() <= 16384,
        c,
    )
}
pub fn formation(f: &Value) -> R<Value> {
    canonical(f, "invalid-state")?;
    exact(f, "kind instance participants policy", "invalid-state")?;
    need(
        f["kind"] == "temple3.basis-decision-formation/v0.1",
        "invalid-state",
    )?;
    let s = json!({"kind":"temple3.basis-decision-state/v0.1","instance":f["instance"],"participants":f["participants"],"sequence":"0","policies":[f["policy"]],"records":[],"decisions":[],"uses":[]});
    validate(&s)?;
    Ok(s)
}
pub fn next_state(before: &Value, who: &Value, command: &Value) -> R<Value> {
    validate(before)?;
    canonical(command, "invalid-command")?;
    exact(command, "op args", "invalid-command")?;
    need(actor(who), "invalid-command")?;
    let op = command["op"].as_str().ok_or("invalid-command")?;
    let a = &command["args"];
    let keys = match op {
        "create" => "amount note",
        "set-amount" => "recordId amount",
        "set-note" => "recordId note",
        "approve" => "recordId",
        "use" => "recordId decisionId",
        "amend" => "policy",
        _ => return Err("invalid-command".into()),
    };
    exact(a, keys, "invalid-command")?;
    if a.get("recordId").is_some() {
        need(reference(&a["recordId"], "record."), "invalid-command")?;
    }
    if a.get("amount").is_some() {
        need(num(&a["amount"], 999999999).is_some(), "invalid-command")?;
    }
    if a.get("note").is_some() {
        need(note(&a["note"]), "invalid-command")?;
    }
    if op == "use" {
        need(
            a["decisionId"] == "none" || reference(&a["decisionId"], "decision."),
            "invalid-command",
        )?;
    }
    if op == "amend" {
        policy(
            &a["policy"],
            before["participants"].as_array().unwrap(),
            "invalid-command",
        )?;
    }
    let seq = num(&before["sequence"], 1024).unwrap();
    need(seq < 1024, "exhausted")?;
    let p = before["policies"].as_array().unwrap().last().unwrap();
    let role = match op {
        "approve" => "reviewer",
        "amend" => "steward",
        _ => "author",
    };
    need(who == &p[role], "unauthorized")?;
    let index = before["records"]
        .as_array()
        .unwrap()
        .iter()
        .position(|r| r["id"] == a["recordId"]);
    if a.get("recordId").is_some() {
        need(index.is_some(), "unknown-reference")?;
    }
    let decision = before["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == a["decisionId"]);
    if op == "use" && a["decisionId"] != "none" {
        need(decision.is_some(), "unknown-reference")?;
    }
    let mut n = before.clone();
    match op {
        "create" => {
            let rs = n["records"].as_array_mut().unwrap();
            need(rs.len() < 4, "exhausted")?;
            rs.push(json!({"id":format!("record.{}",rs.len()+1),"version":"1","amount":a["amount"],"unit":p["unit"],"note":a["note"]}));
        }
        "set-amount" | "set-note" => {
            let r = &mut n["records"][index.unwrap()];
            let k = if op == "set-amount" { "amount" } else { "note" };
            need(r[k] != a[k], "unchanged-value")?;
            if k == "amount" {
                let v = num(&r["version"], 1024).unwrap();
                need(v < 1024, "exhausted")?;
                r["version"] = json!((v + 1).to_string());
            }
            r[k] = a[k].clone();
        }
        "amend" => {
            let ps = n["policies"].as_array_mut().unwrap();
            need(
                a["policy"]["id"] == format!("policy.{}", ps.len() + 1),
                "invalid-policy",
            )?;
            need(ps.len() < 4, "exhausted")?;
            ps.push(a["policy"].clone());
        }
        _ => {
            let r = &before["records"][index.unwrap()];
            need(r["unit"] == p["unit"], "unit-mismatch")?;
            let mut h = json!({"recordId":r["id"],"recordVersion":r["version"],"amount":r["amount"],"unit":r["unit"],"policyId":p["id"]});
            if op == "approve" {
                need(
                    num(&r["amount"], 999999999) > num(&p["threshold"], 999999999),
                    "approval-not-required",
                )?;
                let ds = n["decisions"].as_array_mut().unwrap();
                need(ds.len() < 8, "exhausted")?;
                h["id"] = json!(format!("decision.{}", ds.len() + 1));
                h["reviewer"] = who.clone();
                ds.push(h);
            } else {
                if num(&r["amount"], 999999999) <= num(&p["threshold"], 999999999) {
                    need(a["decisionId"] == "none", "approval-not-required")?;
                } else {
                    let d = decision.ok_or("decision-required")?;
                    need(
                        h.as_object().unwrap().iter().all(|(k, v)| &d[k] == v),
                        "stale-decision",
                    )?;
                }
                let us = n["uses"].as_array_mut().unwrap();
                need(us.len() < 8, "exhausted")?;
                h["id"] = json!(format!("use.{}", us.len() + 1));
                h["decisionId"] = a["decisionId"].clone();
                h["actor"] = who.clone();
                us.push(h);
            }
        }
    }
    n["sequence"] = json!((seq + 1).to_string());
    validate(&n)?;
    Ok(n)
}
pub fn verify(inputs: &Value, evidence: &Value) -> bool {
    (|| -> R<bool> {
        canonical(inputs, "invalid-command")?;
        exact(inputs, "before actor command after", "invalid-command")?;
        exact(evidence, "", "invalid-command")?;
        Ok(next_state(&inputs["before"], &inputs["actor"], &inputs["command"])? == inputs["after"])
    })()
    .unwrap_or(false)
}
