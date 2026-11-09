use serde_json::{Value, json};
use std::collections::BTreeSet;
type R<T> = Result<T, String>;
fn need(v: bool) -> R<()> {
    if v {
        Ok(())
    } else {
        Err("invalid-import-application".into())
    }
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
fn token(v: &Value) -> R<()> {
    crate::identity::token(v).map_err(|e| format!("{e:?}"))
}
pub fn config(c: &Value) -> R<Value> {
    canonical(c)?;
    shape(
        c,
        "kind formation actors migrator purpose forgiveness retention disclosure amendment maximumStateBytes",
    )?;
    need(
        c["kind"] == "temple3.claim-import-application-config/v0.1"
            && c["purpose"] == "hypothesis-rehearsal"
            && c["retention"] == "retain-config-origin-reviews-and-events"
            && c["disclosure"] == "all-configured-actors"
            && c["amendment"] == "fixed-config",
    )?;
    need(
        c["forgiveness"]
            == json!({"disposition":"refuse","scope":"config-origin-reviews-and-events"}),
    )?;
    crate::claim_lifecycle::run(&json!({"op":"formation","input":c["formation"]}))?;
    let migrator = c["migrator"].as_str().ok_or("migrator")?;
    need(
        !migrator.is_empty()
            && migrator.len() <= 64
            && migrator.as_bytes()[0].is_ascii_lowercase()
            && migrator
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
            && c["migrator"] != c["formation"]["policy"]["reviewer"],
    )?;
    let mut names: BTreeSet<&str> = c["formation"]["participants"]
        .as_array()
        .ok_or("participants")?
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect();
    names.insert(migrator);
    let actors = c["actors"].as_array().ok_or("actors")?;
    need(actors.len() == names.len())?;
    let mut keys = BTreeSet::new();
    for (a, name) in actors.iter().zip(names) {
        shape(a, "actor publicKey")?;
        need(a["actor"] == name)?;
        crate::identity::key_id(&a["publicKey"]).map_err(|e| format!("{e:?}"))?;
        need(keys.insert(a["publicKey"].as_str().ok_or("key")?))?;
    }
    let v = c["maximumStateBytes"].as_str().ok_or("limit")?;
    need(
        !v.is_empty()
            && v.len() <= 7
            && !v.starts_with('0')
            && v.bytes().all(|b| b.is_ascii_digit()),
    )?;
    need(v.parse::<usize>().map_err(|_| "limit")? <= 4194304)?;
    Ok(c.clone())
}
fn bounded(s: Value) -> R<Value> {
    let limit = s["config"]["maximumStateBytes"]
        .as_str()
        .ok_or("limit")?
        .parse::<usize>()
        .map_err(|_| "limit")?;
    need(canonical(&s)?.len() <= limit)?;
    Ok(s)
}
pub fn formation(c: &Value, pid: &Value) -> R<Value> {
    token(pid)?;
    bounded(
        json!({"kind":"temple3.claim-import-application/v0.1","config":config(c)?,"profileId":pid,"phase":"awaiting-origin","origin":null,"replay":null}),
    )
}
pub fn validate(s: &Value) -> R<Value> {
    canonical(s)?;
    shape(s, "kind config profileId phase origin replay")?;
    need(s["kind"] == "temple3.claim-import-application/v0.1")?;
    config(&s["config"])?;
    token(&s["profileId"])?;
    if s["phase"] == "awaiting-origin" {
        need(s["origin"].is_null() && s["replay"].is_null())?;
    } else {
        need(s["phase"] == "origin-admitted")?;
        let o = &s["origin"];
        shape(o, "actor profileId basisStateRoot")?;
        token(&o["basisStateRoot"])?;
        need(o["actor"] == s["config"]["migrator"] && o["profileId"] == s["profileId"])?;
        crate::claim_import_replay::validate(&s["replay"])?;
        let x = &s["replay"]["input"];
        let p = &x["policy"];
        need(
            x["target"]["formation"] == s["config"]["formation"]
                && p["targetProfileId"] == s["profileId"]
                && p["targetHeadId"] == o["basisStateRoot"]
                && x["expectedTargetHeadId"] == o["basisStateRoot"],
        )?;
        let actors = s["config"]["actors"].as_array().ok_or("actors")?;
        let key = |actor: &Value| {
            actors
                .iter()
                .find(|a| a["actor"] == *actor)
                .map(|a| &a["publicKey"])
                .ok_or("actor")
        };
        need(
            p["migratorKey"] == *key(&s["config"]["migrator"])?
                && p["reviewerKey"] == *key(&s["config"]["formation"]["policy"]["reviewer"])?,
        )?;
    }
    bounded(s.clone())
}
pub fn next(before: &Value, actor: &Value, command: &Value, root: &Value) -> R<Value> {
    let mut s = validate(before)?;
    token(root)?;
    shape(command, "op args")?;
    need(
        s["config"]["actors"]
            .as_array()
            .ok_or("actors")?
            .iter()
            .any(|a| a["actor"] == *actor),
    )?;
    if command["op"] == "admit-origin" {
        need(s["phase"] == "awaiting-origin" && *actor == s["config"]["migrator"])?;
        shape(&command["args"], "input")?;
        s["origin"] = json!({"actor":actor,"profileId":s["profileId"],"basisStateRoot":root});
        s["replay"] = crate::claim_import_replay::run(
            &json!({"op":"formation","input":command["args"]["input"]}),
        )?;
        s["phase"] = json!("origin-admitted");
    } else {
        need(command["op"] == "native-command" && s["phase"] == "origin-admitted")?;
        shape(&command["args"], "command")?;
        s["replay"] = crate::claim_import_replay::run(
            &json!({"op":"next","input":s["replay"],"actor":actor,"command":command["args"]["command"]}),
        )?;
    }
    validate(&s)
}
pub fn verify(x: &Value, e: &Value) -> bool {
    shape(x, "before actor command basisStateRoot after").is_ok()
        && shape(e, "").is_ok()
        && next(
            &x["before"],
            &x["actor"],
            &x["command"],
            &x["basisStateRoot"],
        )
        .is_ok_and(|v| v == x["after"])
}
pub fn run(x: &Value) -> R<Value> {
    match x["op"].as_str().ok_or("op")? {
        "formation" => {
            shape(x, "op config profileId")?;
            formation(&x["config"], &x["profileId"])
        }
        "validate" => {
            shape(x, "op input")?;
            validate(&x["input"])
        }
        "next" => {
            shape(x, "op before actor command basisStateRoot")?;
            next(
                &x["before"],
                &x["actor"],
                &x["command"],
                &x["basisStateRoot"],
            )
        }
        _ => Err("operation".into()),
    }
}
