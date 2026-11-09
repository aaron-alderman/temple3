//! Consequence release with retained history and originating commitments.
use serde_json::{Value, json};
type R<T> = std::result::Result<T, String>;
const KIND: &str = "temple3.basis-consequence-state/v0.1";
fn need(ok: bool, c: &str) -> R<()> {
    if ok { Ok(()) } else { Err(c.into()) }
}
fn shape(v: &Value, k: &str, c: &str) -> R<()> {
    crate::shape(v, k).map_err(|_| c.into())
}
fn canonical(v: &Value, c: &str) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|_| c.into())
}
fn refusal() -> Value {
    json!({"disposition":"refuse","scope":"all-profile-records-and-decisions"})
}
fn release() -> Value {
    json!({"disposition":"release-consequence","scope":"named-use-restriction","authority":"current-steward","commitment":"originating-policy","retention":"retain-restriction-and-release-history"})
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
fn reference(v: &Value, p: &str) -> bool {
    v.as_str().and_then(|s| s.strip_prefix(p)).is_some_and(|s| {
        !s.is_empty() && s.as_bytes()[0] != b'0' && s.bytes().all(|b| b.is_ascii_digit())
    })
}
fn common(s: &Value) -> Value {
    let mut n = s.clone();
    let o = n.as_object_mut().unwrap();
    o.remove("restrictions");
    o.remove("releases");
    n["kind"] = json!("temple3.basis-decision-state/v0.1");
    for p in n["policies"].as_array_mut().unwrap() {
        p["forgiveness"] = refusal();
    }
    n
}
pub fn validate(s: &Value) -> R<()> {
    let c = "invalid-state";
    let bytes = canonical(s, c)?;
    shape(
        s,
        "kind instance participants sequence policies records decisions uses restrictions releases",
        c,
    )?;
    need(s["kind"] == KIND, c)?;
    let ps = s["policies"].as_array().ok_or(c)?;
    for p in ps {
        need(
            p.is_object() && (p["forgiveness"] == refusal() || p["forgiveness"] == release()),
            c,
        )?;
    }
    crate::basis_decision::validate(&common(s))?;
    for (k, prefix) in [("restrictions", "restriction"), ("releases", "release")] {
        let xs = s[k].as_array().ok_or(c)?;
        need(xs.len() <= 4, c)?;
        for (i, x) in xs.iter().enumerate() {
            need(x["id"] == format!("{prefix}.{}", i + 1), c)?;
        }
    }
    let restrictions = s["restrictions"].as_array().unwrap();
    let releases = s["releases"].as_array().unwrap();
    for r in restrictions {
        shape(
            r,
            "id recordId recordVersion policyId reviewer consequence",
            c,
        )?;
        let record = s["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == r["recordId"])
            .ok_or(c)?;
        let p = ps.iter().find(|x| x["id"] == r["policyId"]).ok_or(c)?;
        let v = r["recordVersion"].as_str().ok_or(c)?;
        let number = v.parse::<u32>().map_err(|_| c)?;
        need(
            number > 0
                && v.len() <= 4
                && number.to_string() == v
                && number <= record["version"].as_str().unwrap().parse::<u32>().unwrap()
                && r["reviewer"] == p["reviewer"]
                && r["consequence"] == "prevent-use",
            c,
        )?;
    }
    let mut seen = Vec::new();
    for r in releases {
        shape(
            r,
            "id restrictionId originatingPolicyId authorizingPolicyId steward consequence",
            c,
        )?;
        let restriction = restrictions
            .iter()
            .find(|x| x["id"] == r["restrictionId"])
            .ok_or(c)?;
        let oi = ps
            .iter()
            .position(|x| x["id"] == r["originatingPolicyId"])
            .ok_or(c)?;
        let ai = ps
            .iter()
            .position(|x| x["id"] == r["authorizingPolicyId"])
            .ok_or(c)?;
        need(
            restriction["policyId"] == ps[oi]["id"]
                && ps[oi]["forgiveness"] == release()
                && ai >= oi
                && r["steward"] == ps[ai]["steward"]
                && r["consequence"] == "prevent-use"
                && !seen.contains(&r["restrictionId"]),
            c,
        )?;
        seen.push(r["restrictionId"].clone());
    }
    let count: [&str; 6] = [
        "policies",
        "records",
        "decisions",
        "uses",
        "restrictions",
        "releases",
    ];
    need(
        s["sequence"].as_str().unwrap().parse::<usize>().unwrap() + 1
            >= count
                .iter()
                .map(|k| s[*k].as_array().unwrap().len())
                .sum::<usize>()
            && bytes.len() <= 16384,
        c,
    )
}
pub fn formation(f: &Value) -> R<Value> {
    canonical(f, "invalid-state")?;
    shape(f, "kind instance participants policy", "invalid-state")?;
    need(
        f["kind"] == "temple3.basis-consequence-formation/v0.1",
        "invalid-state",
    )?;
    let n = json!({"kind":KIND,"instance":f["instance"],"participants":f["participants"],"sequence":"0","policies":[f["policy"]],"records":[],"decisions":[],"uses":[],"restrictions":[],"releases":[]});
    validate(&n)?;
    Ok(n)
}
pub fn next_state(s: &Value, who: &Value, command: &Value) -> R<Value> {
    validate(s)?;
    canonical(command, "invalid-command")?;
    shape(command, "op args", "invalid-command")?;
    let op = command["op"].as_str().ok_or("invalid-command")?;
    let a = &command["args"];
    if op != "restrict" && op != "release" {
        let mut normalized = command.clone();
        if op == "amend" {
            shape(a, "policy", "invalid-command")?;
            need(
                a["policy"]["forgiveness"] == refusal() || a["policy"]["forgiveness"] == release(),
                "invalid-command",
            )?;
            normalized["args"]["policy"]["forgiveness"] = refusal();
        }
        let mut n = crate::basis_decision::next_state(&common(s), who, &normalized)?;
        if op == "use" {
            need(
                !s["restrictions"].as_array().unwrap().iter().any(|r| {
                    r["recordId"] == a["recordId"]
                        && !s["releases"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|x| x["restrictionId"] == r["id"])
                }),
                "restricted",
            )?;
        }
        n["kind"] = json!(KIND);
        n["policies"] = s["policies"].clone();
        if op == "amend" {
            n["policies"]
                .as_array_mut()
                .unwrap()
                .push(a["policy"].clone());
        }
        n["restrictions"] = s["restrictions"].clone();
        n["releases"] = s["releases"].clone();
        validate(&n)?;
        return Ok(n);
    }
    shape(
        a,
        if op == "restrict" {
            "recordId"
        } else {
            "restrictionId"
        },
        "invalid-command",
    )?;
    need(actor(who), "invalid-command")?;
    need(
        if op == "restrict" {
            reference(&a["recordId"], "record.")
        } else {
            reference(&a["restrictionId"], "restriction.")
        },
        "invalid-command",
    )?;
    let seq = s["sequence"].as_str().unwrap().parse::<u32>().unwrap();
    need(seq < 1024, "exhausted")?;
    let p = s["policies"].as_array().unwrap().last().unwrap();
    need(
        who == &p[if op == "restrict" {
            "reviewer"
        } else {
            "steward"
        }],
        "unauthorized",
    )?;
    let mut n = s.clone();
    if op == "restrict" {
        let r = s["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == a["recordId"])
            .ok_or("unknown-reference")?;
        let xs = n["restrictions"].as_array_mut().unwrap();
        need(xs.len() < 4, "exhausted")?;
        xs.push(json!({"id":format!("restriction.{}",xs.len()+1),"recordId":r["id"],"recordVersion":r["version"],"policyId":p["id"],"reviewer":who,"consequence":"prevent-use"}));
    } else {
        let r = s["restrictions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == a["restrictionId"])
            .ok_or("unknown-reference")?;
        let origin = s["policies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == r["policyId"])
            .unwrap();
        need(origin["forgiveness"] == release(), "forgiveness-refused")?;
        let xs = n["releases"].as_array_mut().unwrap();
        need(
            !xs.iter().any(|x| x["restrictionId"] == r["id"]),
            "already-released",
        )?;
        need(xs.len() < 4, "exhausted")?;
        xs.push(json!({"id":format!("release.{}",xs.len()+1),"restrictionId":r["id"],"originatingPolicyId":origin["id"],"authorizingPolicyId":p["id"],"steward":who,"consequence":"prevent-use"}));
    }
    n["sequence"] = json!((seq + 1).to_string());
    validate(&n)?;
    Ok(n)
}
pub fn verify(i: &Value, e: &Value) -> bool {
    (|| -> R<bool> {
        canonical(i, "invalid-command")?;
        canonical(e, "invalid-command")?;
        shape(i, "before actor command after", "invalid-command")?;
        shape(e, "", "invalid-command")?;
        Ok(next_state(&i["before"], &i["actor"], &i["command"])? == i["after"])
    })()
    .unwrap_or(false)
}
