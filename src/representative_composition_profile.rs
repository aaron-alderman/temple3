//! Independently reconstructed representative-composition profile.
use crate::{Result, coordination, require, shape};
use serde_json::{Value, json};
fn read(path: &str) -> Value {
    json!({"op":"read","path":path.split('.').collect::<Vec<_>>()})
}
fn lit(v: Value) -> Value {
    json!({"op":"literal","value":v})
}
fn eq(a: Value, b: Value) -> Value {
    json!({"op":"eq","left":a,"right":b})
}
fn absent(path: &str) -> Value {
    eq(read(path), lit(Value::Null))
}
fn present(path: &str) -> Value {
    json!({"op":"not","value":absent(path)})
}
pub fn predicate() -> Value {
    json!({"op":"all","values":[eq(read("context.statement.relation"),lit(json!("representative-composition-command-v1"))),eq(read("context.statement.publicInputs.actor"),read("identity")),absent("delta.nextPredicate"),eq(read("delta.read"),read("delta.consume")),present("basis.cells.0"),eq(read("delta.consume.0"),read("basis.cells.0.id")),absent("basis.cells.1"),absent("delta.consume.1"),present("delta.create.0"),absent("delta.create.1"),eq(read("context.statement.publicInputs.before"),read("basis.cells.0.value")),eq(read("context.statement.publicInputs.after"),read("delta.create.0.value")),json!({"op":"verify","statement":read("context.statement"),"proof":read("context.proof")})]})
}
pub fn recognize(genesis: &Value, d: &Value, anchor: &Value) -> Result<Value> {
    require(
        anchor.as_str().is_some_and(|s| !s.is_empty()),
        "missing-representative-composition-trust-anchor",
    )?;
    crate::canonical::canonical_json(d)?;
    shape(d, "arkId formation identityProfile coordinationProfile")?;
    let initial = crate::typed_composition::representative_formation(&d["formation"])
        .map_err(|_| crate::err("invalid-representative-composition-profile"))?;
    require(
        d["arkId"].as_str().is_some_and(|s| !s.is_empty()),
        "invalid-representative-composition-profile",
    )?;
    let actors = d["identityProfile"]["actors"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-representative-composition-profile"))?;
    require(
        json!(
            actors
                .iter()
                .map(|a| a["actor"].clone())
                .collect::<Vec<_>>()
        ) == initial["participants"],
        "invalid-representative-composition-profile",
    )?;
    for (i, a) in actors.iter().enumerate() {
        require(
            !actors[..i].iter().any(|b| a["publicKey"] == b["publicKey"]),
            "invalid-representative-composition-profile",
        )?;
    }
    let expected = json!({"arkId":d["arkId"],"activePredicate":predicate(),"initialValues":[initial],"context":{"application":{"application":{"formation":d["formation"]},"coordinationProfile":d["coordinationProfile"]},"identityProfile":d["identityProfile"]}});
    require(
        genesis["signed"]["payload"]["genesisInput"] == expected,
        "invalid-representative-composition-profile",
    )?;
    coordination::initial(genesis, anchor)
}
