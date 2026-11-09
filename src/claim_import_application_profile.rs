use crate::{Result, coordination, require, shape};
use serde_json::{Value, json};
fn r(path: &str) -> Value {
    json!({"op":"read","path":path.split('.').collect::<Vec<_>>()})
}
fn l(v: Value) -> Value {
    json!({"op":"literal","value":v})
}
fn eq(a: Value, b: Value) -> Value {
    json!({"op":"eq","left":a,"right":b})
}
fn absent(path: &str) -> Value {
    eq(r(path), l(Value::Null))
}
fn present(path: &str) -> Value {
    json!({"op":"not","value":absent(path)})
}
pub fn predicate() -> Value {
    json!({"op":"all","values":[
     eq(r("context.statement.relation"),l(json!("claim-import-application-v1"))),
     eq(r("context.statement.publicInputs.actor"),r("identity")),
     eq(r("context.statement.publicInputs.basisStateRoot"),r("basis.stateRoot")),
     absent("delta.nextPredicate"),eq(r("delta.read"),r("delta.consume")),
     present("basis.cells.0"),eq(r("delta.consume.0"),r("basis.cells.0.id")),absent("basis.cells.1"),absent("delta.consume.1"),present("delta.create.0"),absent("delta.create.1"),
     eq(r("context.statement.publicInputs.before"),r("basis.cells.0.value")),eq(r("context.statement.publicInputs.after"),r("delta.create.0.value")),
     {"op":"verify","statement":r("context.statement"),"proof":r("context.proof")}
    ]})
}
pub fn recognize(genesis: &Value, d: &Value, anchor: &Value) -> Result<Value> {
    require(
        anchor.as_str().is_some_and(|s| !s.is_empty()),
        "missing-import-trust-anchor",
    )?;
    crate::canonical::canonical_json(d)?;
    shape(d, "arkId config identityProfile coordinationProfile")?;
    let c = crate::claim_import_application::config(&d["config"])
        .map_err(|_| crate::err("invalid-import-profile"))?;
    require(
        d["arkId"].as_str().is_some_and(|s| !s.is_empty())
            && d["identityProfile"]["actors"] == c["actors"],
        "invalid-import-profile",
    )?;
    let pid = crate::canonical::identify("claim-import-application-profile", d)?;
    let initial = crate::claim_import_application::formation(&c, &json!(pid))
        .map_err(|_| crate::err("invalid-import-profile"))?;
    let expected = json!({"arkId":d["arkId"],"activePredicate":predicate(),"initialValues":[initial],"context":{"application":{"application":{"config":c,"profileId":pid},"coordinationProfile":d["coordinationProfile"]},"identityProfile":d["identityProfile"]}});
    require(
        genesis["signed"]["payload"]["genesisInput"] == expected,
        "invalid-import-profile",
    )?;
    coordination::initial(genesis, anchor)
}
