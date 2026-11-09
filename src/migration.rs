//! The closed, lexical positive-property-assertion migration slice.
use crate::{
    Result, array, bind,
    canonical::{canonical_json, identify},
    commitment, err, kernel,
    predicate::{eq, literal, read},
    require, shape, string,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn manifest(pins: &Value) -> Result<Value> {
    bind(
        "migration-manifest",
        "manifestId",
        json!({"kind":"temple3.migration-manifest/v0.1","sourcePins":pins,"sourceId":identify("legacy-source",pins)?,"actor":"actor:migration","contract":{"command":"action.migration.theme.command","effect":"positive-property-assertion","root":"settings","membership":"session","memberType":"Session","target":"session.one","property":"theme","allowed":["amber","green"]}}),
    )
}
fn check_manifest(m: &Value) -> Result<()> {
    shape(m, "kind sourcePins sourceId actor contract manifestId")
        .map_err(|_| err("invalid-migration-input"))?;
    require(
        *m == manifest(&m["sourcePins"])?,
        "unsupported-migration-profile",
    )
}
fn token(v: &Value) -> Result<()> {
    require(
        v.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 256),
        "legacy-nonlexical-value",
    )?;
    canonical_json(v)?;
    Ok(())
}
pub fn fact_value(m: &Value, fact: &Value) -> Value {
    json!({"kind":"temple3.legacy-fact/v0.1","sourceId":m["sourceId"],"fact":fact})
}
pub fn active_facts(rows: &Value) -> Result<Vec<Value>> {
    let rows = array(rows, "legacy-snapshot-too-large")?;
    require(rows.len() <= 128, "legacy-snapshot-too-large")?;
    let mut active = BTreeMap::new();
    for r in rows {
        shape(r, "s p o pol").map_err(|_| err("invalid-migration-input"))?;
        for f in ["s", "p", "o"] {
            token(&r[f])?;
        }
        let key = (
            string(&r["s"], "legacy-nonlexical-value")?.to_owned(),
            string(&r["p"], "legacy-nonlexical-value")?.to_owned(),
            string(&r["o"], "legacy-nonlexical-value")?.to_owned(),
        );
        match r["pol"].as_str() {
            Some("is") => {
                active.insert(key, json!({"s":r["s"],"p":r["p"],"o":r["o"]}));
            }
            Some("is-not") => {
                active.remove(&key);
            }
            _ => return Err(err("unsupported-legacy-polarity")),
        }
    }
    Ok(active.into_values().collect())
}
fn has(facts: &[Value], s: &Value, p: &Value, o: &Value) -> bool {
    facts.contains(&json!({"s":s,"p":p,"o":o}))
}
fn eligible(facts: &[Value], c: &Value) -> bool {
    has(facts, &c["root"], &c["membership"], &c["target"])
        && has(facts, &c["target"], &json!("isa"), &c["memberType"])
        && facts
            .iter()
            .filter(|f| f["s"] == c["target"] && f["p"] == c["property"])
            .count()
            == 1
}
pub fn covenant(m: &Value, values: &[Value]) -> Result<Value> {
    check_manifest(m)?;
    let c = &m["contract"];
    let facts: Vec<_> = values.iter().map(|v| v["fact"].clone()).collect();
    if !eligible(&facts, c) {
        return Ok(literal(json!(false)));
    }
    let mut ids = values
        .iter()
        .map(|v| identify("cell", v))
        .collect::<Result<Vec<_>>>()?;
    ids.sort();
    let alternatives = array(&c["allowed"], "unsupported-migration-profile")?
        .iter()
        .map(|v| {
            Ok(eq(
                read(&["delta", "create"]),
                literal(json!([commitment::cell_for(&fact_value(
                    m,
                    &json!({"s":c["target"],"p":c["property"],"o":v})
                ))?])),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(
        json!({"op":"all","values":[eq(read(&["identity"]),literal(m["actor"].clone())),eq(read(&["context"]),literal(json!({"migrationManifestId":m["manifestId"]}))),eq(read(&["delta","read"]),literal(json!(ids))),eq(read(&["delta","consume"]),literal(json!([]))),eq(read(&["delta","nextPredicate"]),literal(Value::Null)),{"op":"any","values":alternatives}]}),
    )
}
pub fn construct(m: &Value, t: &Value, command: &Value, args: &Value) -> Result<Value> {
    check_manifest(m)?;
    require(
        canonical_json(&json!({"manifest":m,"temple":t}))?.len() <= 131072,
        "migration-input-too-large",
    )?;
    let c = &m["contract"];
    require(*command == c["command"], "unsupported-legacy-command")?;
    let args = array(args, "legacy-nonlexical-argument")?;
    require(
        args.len() == 2 && args.iter().all(Value::is_string),
        "legacy-nonlexical-argument",
    )?;
    for a in args {
        token(a)?;
    }
    let cells = array(&t["state"]["cells"], "invalid-migration-basis")?;
    require(cells.len() <= 128, "invalid-migration-basis")?;
    let mut facts = vec![];
    for cell in cells {
        for (v, s) in [
            (cell, "id value"),
            (&cell["value"], "kind sourceId fact"),
            (&cell["value"]["fact"], "s p o"),
        ] {
            shape(v, s).map_err(|_| err("invalid-migration-input"))?;
        }
        for f in ["s", "p", "o"] {
            token(&cell["value"]["fact"][f])?;
        }
        require(
            cell["value"]["kind"] == "temple3.legacy-fact/v0.1"
                && cell["value"]["sourceId"] == m["sourceId"]
                && cell["id"] == identify("cell", &cell["value"])?,
            "invalid-migration-basis",
        )?;
        facts.push(cell["value"]["fact"].clone());
    }
    let target = &args[0];
    let value = &args[1];
    require(
        *target == c["target"]
            && has(&facts, &c["root"], &c["membership"], target)
            && has(&facts, target, &json!("isa"), &c["memberType"]),
        "legacy-target-denied",
    )?;
    require(
        array(&c["allowed"], "unsupported-migration-profile")?.contains(value),
        "legacy-value-denied",
    )?;
    let current: Vec<_> = facts
        .iter()
        .filter(|f| f["s"] == *target && f["p"] == c["property"])
        .collect();
    require(current.len() == 1, "legacy-property-unresolved")?;
    require(current[0]["o"] != *value, "legacy-no-op")?;
    let cell = commitment::cell_for(&fact_value(
        m,
        &json!({"s":target,"p":c["property"],"o":value}),
    ))?;
    let delta = kernel::build_delta(
        t,
        &json!({"read":cells.iter().map(|c|&c["id"]).collect::<Vec<_>>(),"create":[cell]}),
    )?;
    Ok(
        json!({"identity":m["actor"],"delta":delta,"context":{"migrationManifestId":m["manifestId"]}}),
    )
}
