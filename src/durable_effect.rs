//! Provider-bound effect intents; dispatch uncertainty is not terminal completion.
use serde_json::{Value, json};
use std::collections::BTreeSet;
type R<T> = Result<T, String>;
const KIND: &str = "temple3.durable-effect-state/v0.1";
fn need(ok: bool, c: &str) -> R<()> {
    if ok { Ok(()) } else { Err(c.into()) }
}
fn shape(v: &Value, keys: &str, c: &str) -> R<()> {
    crate::shape(v, keys).map_err(|_| c.into())
}
fn canonical(v: &Value, c: &str) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|_| c.into())
}
fn id(k: &str, v: &Value) -> R<String> {
    crate::canonical::identify(k, v).map_err(|e| e.code)
}
fn token(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        !s.is_empty()
            && s.len() <= 64
            && s.as_bytes()[0].is_ascii_lowercase()
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
    })
}
fn decimal(v: &Value, max: u32) -> bool {
    v.as_str().is_some_and(|s| {
        !s.is_empty()
            && s.len() <= 4
            && (s == "0" || s.as_bytes()[0] != b'0')
            && s.bytes().all(|b| b.is_ascii_digit())
            && s.parse::<u32>().is_ok_and(|n| n <= max)
    })
}
fn payload(v: &Value, c: &str) -> R<()> {
    shape(v, "marker", c)?;
    need(
        v["marker"]
            .as_str()
            .is_some_and(|s| !s.is_empty() && s.len() <= 128),
        c,
    )
}
pub fn intent_id(s: &Value, request: &Value, value: &Value) -> R<String> {
    id(
        "effect-intent",
        &json!({"instance":s["instance"],"provider":s["provider"],"requestId":request,"payload":value}),
    )
}
pub fn receipt_payload(s: &Value, i: &Value, outcome: &Value) -> R<Value> {
    need(
        *outcome == "applied" || *outcome == "rejected",
        "invalid-receipt",
    )?;
    let result = if *outcome == "applied" {
        json!({"markerId":id("provider-marker",&json!({"instance":s["instance"],"intentId":i["intentId"],"payload":i["payload"]}))?})
    } else {
        json!({"reason":"provider-declined"})
    };
    Ok(
        json!({"kind":"temple3.effect-receipt/v0.1","instance":s["instance"],"providerId":s["provider"]["id"],"requestId":i["requestId"],"intentId":i["intentId"],"outcome":outcome,"result":result}),
    )
}
pub fn receipt_valid(s: &Value, i: &Value, r: &Value) -> bool {
    crate::identity::verify_envelope(r, "attestation", &s["provider"]["publicKey"])
        && receipt_payload(s, i, &r["payload"]["outcome"])
            .is_ok_and(|expected| expected == r["payload"])
}
pub fn validate(s: &Value) -> R<()> {
    let c = "invalid-state";
    let raw = canonical(s, c)?;
    shape(
        s,
        "kind instance participants policy provider sequence intents",
        c,
    )?;
    need(
        s["kind"] == KIND
            && s["instance"].as_str().is_some_and(|v| {
                v.len() == 32
                    && v.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }),
        c,
    )?;
    let people = s["participants"].as_array().ok_or(c)?;
    need(
        !people.is_empty()
            && people.len() <= 8
            && people.iter().all(token)
            && people.windows(2).all(|w| w[0].as_str() < w[1].as_str()),
        c,
    )?;
    let p = &s["policy"];
    shape(
        p,
        "requester dispatcher forgiveness retention amendment cancellation completion",
        c,
    )?;
    need(
        people.contains(&p["requester"])
            && people.contains(&p["dispatcher"])
            && *p
                == json!({"requester":p["requester"],"dispatcher":p["dispatcher"],"forgiveness":{"disposition":"refuse","scope":"effect-intent-and-receipt-history"},"retention":"retain-all-transition-evidence","amendment":"fixed-policy","cancellation":"before-first-dispatch-only","completion":"authenticated-provider-terminal-receipt"}),
        c,
    )?;
    let provider = &s["provider"];
    shape(
        provider,
        "id publicKey operation idempotency reconciliation",
        c,
    )?;
    need(
        token(&provider["id"])
            && provider["operation"] == "append-marker/v1"
            && (provider["idempotency"] == "durable-intent-key"
                || provider["idempotency"] == "none")
            && provider["reconciliation"] == "lookup-intent/v1",
        c,
    )?;
    crate::identity::key_id(&provider["publicKey"]).map_err(|_| c)?;
    need(decimal(&s["sequence"], 1024), c)?;
    let intents = s["intents"].as_array().ok_or(c)?;
    need(intents.len() <= 8, "exhausted")?;
    let mut seen = BTreeSet::new();
    for i in intents {
        shape(i, "requestId intentId payload status attempts receipt", c)?;
        payload(&i["payload"], c)?;
        need(token(&i["requestId"]), c)?;
        need(seen.insert(i["requestId"].as_str().unwrap()), c)?;
        need(
            i["intentId"] == intent_id(s, &i["requestId"], &i["payload"])?
                && decimal(&i["attempts"], 8),
            c,
        )?;
        let attempts = i["attempts"].as_str().unwrap().parse::<u32>().unwrap();
        need(provider["idempotency"] != "none" || attempts <= 1, c)?;
        match i["status"].as_str() {
            Some("prepared" | "cancelled") => need(attempts == 0 && i["receipt"].is_null(), c)?,
            Some("unresolved") => need(attempts > 0 && i["receipt"].is_null(), c)?,
            Some("applied" | "rejected") => need(
                attempts > 0
                    && receipt_valid(s, i, &i["receipt"])
                    && i["receipt"]["payload"]["outcome"] == i["status"],
                c,
            )?,
            _ => return Err(c.into()),
        }
    }
    need(raw.len() <= 32768, "exhausted")
}
pub fn formation(f: &Value) -> R<Value> {
    canonical(f, "invalid-state")?;
    shape(
        f,
        "kind instance participants policy provider",
        "invalid-state",
    )?;
    need(
        f["kind"] == "temple3.durable-effect-formation/v0.1",
        "invalid-state",
    )?;
    let mut s = f.clone();
    s["kind"] = json!(KIND);
    s["sequence"] = json!("0");
    s["intents"] = json!([]);
    validate(&s)?;
    Ok(s)
}
pub fn next_state(s: &Value, actor: &Value, command: &Value) -> R<Value> {
    validate(s)?;
    canonical(command, "invalid-command")?;
    shape(command, "op args", "invalid-command")?;
    let op = command["op"].as_str().ok_or("invalid-command")?;
    let a = &command["args"];
    let fields = match op {
        "prepare" => "requestId payload",
        "dispatch" | "cancel" => "requestId",
        "resolve" => "requestId receipt",
        _ => return Err("invalid-command".into()),
    };
    shape(a, fields, "invalid-command")?;
    need(token(actor) && token(&a["requestId"]), "invalid-command")?;
    if op == "prepare" {
        payload(&a["payload"], "invalid-command")?;
    }
    let seq = s["sequence"].as_str().unwrap().parse::<u32>().unwrap();
    need(seq < 1024, "exhausted")?;
    need(
        s["participants"].as_array().unwrap().contains(actor),
        "unauthorized",
    )?;
    let mut n = s.clone();
    let intents = n["intents"].as_array_mut().unwrap();
    let index = intents
        .iter()
        .position(|i| i["requestId"] == a["requestId"]);
    if op == "prepare" {
        need(*actor == s["policy"]["requester"], "unauthorized")?;
        need(index.is_none(), "request-conflict")?;
        need(intents.len() < 8, "exhausted")?;
        intents.push(json!({"requestId":a["requestId"],"intentId":intent_id(s,&a["requestId"],&a["payload"])?,"payload":a["payload"],"status":"prepared","attempts":"0","receipt":null}));
    } else {
        need(
            *actor
                == s["policy"][if op == "cancel" {
                    "requester"
                } else {
                    "dispatcher"
                }],
            "unauthorized",
        )?;
        let i = &mut intents[index.ok_or("unknown-reference")?];
        match op {
            "cancel" => {
                need(i["status"] == "prepared", "not-cancellable")?;
                i["status"] = json!("cancelled");
            }
            "dispatch" => {
                need(
                    i["status"] == "prepared" || i["status"] == "unresolved",
                    "terminal-intent",
                )?;
                need(
                    i["status"] == "prepared"
                        || s["provider"]["idempotency"] == "durable-intent-key",
                    "retry-forbidden",
                )?;
                let attempts = i["attempts"].as_str().unwrap().parse::<u32>().unwrap();
                need(attempts < 8, "exhausted")?;
                i["attempts"] = json!((attempts + 1).to_string());
                i["status"] = json!("unresolved");
            }
            _ => {
                need(i["status"] == "unresolved", "not-unresolved")?;
                need(receipt_valid(s, i, &a["receipt"]), "invalid-receipt")?;
                i["receipt"] = a["receipt"].clone();
                i["status"] = a["receipt"]["payload"]["outcome"].clone();
            }
        }
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
