//! Exact internal inventory conservation, independently reconstructed per account.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
const KIND: &str = "temple3.inventory-state/v0.1";
fn need(b: bool, c: &str) -> R<()> {
    if b { Ok(()) } else { Err(c.into()) }
}
fn shape(v: &Value, k: &str, c: &str) -> R<()> {
    crate::shape(v, k).map_err(|_| c.into())
}
fn canonical(v: &Value, c: &str) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|_| c.into())
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
fn positive(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        !s.is_empty()
            && s.len() <= 9
            && s.as_bytes()[0] != b'0'
            && s.bytes().all(|b| b.is_ascii_digit())
    })
}
fn signed(v: &Value) -> bool {
    v == "0"
        || v.as_str()
            .is_some_and(|s| positive(&json!(s.strip_prefix('-').unwrap_or(s))))
}
fn sequence(v: &Value) -> bool {
    (v == "0" || positive(v))
        && v.as_str()
            .is_some_and(|s| s.len() <= 4 && s.parse::<u32>().is_ok_and(|n| n <= 1024))
}
fn unit(v: &Value) -> bool {
    v.as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 8 && s.bytes().all(|b| b.is_ascii_uppercase()))
}
fn reference(v: &Value, p: &str) -> bool {
    v.as_str().and_then(|s| s.strip_prefix(p)).is_some_and(|s| {
        !s.is_empty() && s.as_bytes()[0] != b'0' && s.bytes().all(|b| b.is_ascii_digit())
    })
}
fn integer(v: &Value) -> i64 {
    v.as_str().unwrap().parse().unwrap()
}
fn balances(s: &Value) -> R<Value> {
    // At most eight bounded transfers and openings: all intermediate sums fit i64.
    let accounts = s["accounts"].as_array().unwrap();
    let mut result = Vec::new();
    let mut total = 0i64;
    let mut opening_total = 0i64;
    for a in accounts {
        let mut q = integer(&a["opening"]);
        opening_total += q;
        for t in s["transfers"].as_array().unwrap() {
            if t["status"] == "posted" {
                if t["source"] == a["id"] {
                    q -= integer(&t["quantity"]);
                }
                if t["destination"] == a["id"] {
                    q += integer(&t["quantity"]);
                }
            }
        }
        total += q;
        result.push(json!({"accountId":a["id"],"quantity":q.to_string()}));
    }
    need(
        result.iter().all(|b| signed(&b["quantity"])),
        "balance-overflow",
    )?;
    need(
        s["policy"]["stock"] != "nonnegative"
            || result.iter().all(|b| integer(&b["quantity"]) >= 0),
        "insufficient-stock",
    )?;
    need(total == opening_total, "conservation-failed")?;
    Ok(json!(result))
}
pub fn validate(s: &Value) -> R<()> {
    let c = "invalid-state";
    let raw = canonical(s, c)?;
    shape(
        s,
        "kind instance participants policy accounts sequence transfers balances",
        c,
    )?;
    need(
        s["kind"] == KIND
            && s["instance"].as_str().is_some_and(|v| {
                v.len() == 32
                    && v.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
            && sequence(&s["sequence"]),
        c,
    )?;
    let people = s["participants"].as_array().ok_or(c)?;
    need(
        !people.is_empty()
            && people.len() <= 8
            && people.iter().all(actor)
            && people.windows(2).all(|w| w[0].as_str() < w[1].as_str()),
        c,
    )?;
    let p = &s["policy"];
    shape(
        p,
        "unit reviewer stock authority correction postedCorrection forgiveness retention amendment retry",
        c,
    )?;
    need(
        unit(&p["unit"])
            && people.contains(&p["reviewer"])
            && (p["stock"] == "nonnegative" || p["stock"] == "allow-negative"),
        c,
    )?;
    need(
        *p == json!({"unit":p["unit"],"reviewer":p["reviewer"],"stock":p["stock"],"authority":"source-owner-drafts-reviewer-posts-roles-may-overlap","correction":"draft-only","postedCorrection":"new-independent-transfer","forgiveness":{"disposition":"refuse","scope":"inventory-transfer-history"},"retention":"retain-all-transition-evidence","amendment":"fixed-policy","retry":"reject-stale-basis"}),
        c,
    )?;
    let accounts = s["accounts"].as_array().ok_or(c)?;
    need(accounts.len() >= 2 && accounts.len() <= 8, c)?;
    for (index, a) in accounts.iter().enumerate() {
        shape(a, "id owner opening", c)?;
        need(
            a["id"] == format!("account.{}", index + 1)
                && people.contains(&a["owner"])
                && signed(&a["opening"]),
            c,
        )?;
        need(
            p["stock"] != "nonnegative" || integer(&a["opening"]) >= 0,
            c,
        )?;
    }
    let transfers = s["transfers"].as_array().ok_or(c)?;
    need(transfers.len() <= 8, "exhausted")?;
    for (index, t) in transfers.iter().enumerate() {
        shape(
            t,
            "id source destination quantity unit status author postedBy",
            c,
        )?;
        let source = accounts.iter().find(|a| a["id"] == t["source"]);
        let destination = accounts.iter().find(|a| a["id"] == t["destination"]);
        need(
            t["id"] == format!("transfer.{}", index + 1)
                && source.is_some()
                && destination.is_some()
                && t["source"] != t["destination"]
                && positive(&t["quantity"])
                && t["unit"] == p["unit"],
            c,
        )?;
        need(
            t["author"] == source.unwrap()["owner"]
                && matches!(t["status"].as_str(), Some("draft" | "posted" | "cancelled")),
            c,
        )?;
        need(
            t["postedBy"]
                == if t["status"] == "posted" {
                    p["reviewer"].clone()
                } else {
                    Value::Null
                },
            c,
        )?;
    }
    need(
        integer(&s["sequence"])
            >= (transfers.len() + transfers.iter().filter(|t| t["status"] != "draft").count())
                as i64,
        c,
    )?;
    let supplied = s["balances"].as_array().ok_or(c)?;
    need(supplied.len() == accounts.len(), c)?;
    for (i, b) in supplied.iter().enumerate() {
        shape(b, "accountId quantity", c)?;
        need(
            b["accountId"] == accounts[i]["id"] && signed(&b["quantity"]),
            c,
        )?;
    }
    need(s["balances"] == balances(s)?, "balance-mismatch")?;
    need(raw.len() <= 16384, "exhausted")
}
pub fn formation(f: &Value) -> R<Value> {
    canonical(f, "invalid-state")?;
    shape(
        f,
        "kind instance participants policy accounts",
        "invalid-state",
    )?;
    need(
        f["kind"] == "temple3.inventory-formation/v0.1",
        "invalid-state",
    )?;
    let accounts = f["accounts"].as_array().ok_or("invalid-state")?;
    let mut s = f.clone();
    s["kind"] = json!(KIND);
    s["sequence"] = json!("0");
    s["transfers"] = json!([]);
    s["balances"] = json!(
        accounts
            .iter()
            .map(|a| json!({"accountId":a["id"],"quantity":a["opening"]}))
            .collect::<Vec<_>>()
    );
    validate(&s)?;
    Ok(s)
}
pub fn next_state(s: &Value, who: &Value, command: &Value) -> R<Value> {
    validate(s)?;
    canonical(command, "invalid-command")?;
    shape(command, "op args", "invalid-command")?;
    let op = command["op"].as_str().ok_or("invalid-command")?;
    let a = &command["args"];
    let fields = match op {
        "draft" => "source destination quantity unit",
        "correct" => "transferId quantity",
        "cancel" | "post" => "transferId",
        _ => return Err("invalid-command".into()),
    };
    shape(a, fields, "invalid-command")?;
    need(actor(who), "invalid-command")?;
    if op == "draft" {
        need(
            reference(&a["source"], "account.")
                && reference(&a["destination"], "account.")
                && positive(&a["quantity"])
                && unit(&a["unit"]),
            "invalid-command",
        )?;
    } else {
        need(reference(&a["transferId"], "transfer."), "invalid-command")?;
        if op == "correct" {
            need(positive(&a["quantity"]), "invalid-command")?;
        }
    }
    need(integer(&s["sequence"]) < 1024, "exhausted")?;
    need(
        s["participants"].as_array().unwrap().contains(who),
        "unauthorized",
    )?;
    let mut n = s.clone();
    if op == "draft" {
        let accounts = s["accounts"].as_array().unwrap();
        let source = accounts.iter().find(|x| x["id"] == a["source"]);
        let destination = accounts.iter().find(|x| x["id"] == a["destination"]);
        need(
            source.is_some() && destination.is_some(),
            "unknown-reference",
        )?;
        need(a["source"] != a["destination"], "same-account")?;
        need(*who == source.unwrap()["owner"], "unauthorized")?;
        need(a["unit"] == s["policy"]["unit"], "unit-mismatch")?;
        let ts = n["transfers"].as_array_mut().unwrap();
        need(ts.len() < 8, "exhausted")?;
        ts.push(json!({"id":format!("transfer.{}",ts.len()+1),"source":a["source"],"destination":a["destination"],"quantity":a["quantity"],"unit":a["unit"],"status":"draft","author":who,"postedBy":null}));
    } else {
        let t = n["transfers"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|t| t["id"] == a["transferId"])
            .ok_or("unknown-reference")?;
        need(
            *who == if op == "post" {
                s["policy"]["reviewer"].clone()
            } else {
                t["author"].clone()
            },
            "unauthorized",
        )?;
        need(t["status"] == "draft", "not-draft")?;
        match op {
            "correct" => {
                need(t["quantity"] != a["quantity"], "unchanged-value")?;
                t["quantity"] = a["quantity"].clone();
            }
            "cancel" => t["status"] = json!("cancelled"),
            _ => {
                t["status"] = json!("posted");
                t["postedBy"] = who.clone();
            }
        }
    }
    n["sequence"] = json!((integer(&s["sequence"]) + 1).to_string());
    n["balances"] = balances(&n)?;
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
