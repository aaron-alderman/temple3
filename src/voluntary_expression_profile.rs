//! Independently reconstructed fixed covenant and exact genesis recognition.
use crate::{Result, coordination, require, shape};
use serde_json::{Value, json};

fn read(path: &str) -> Value {
    json!({"op":"read","path":path.split('.').collect::<Vec<_>>()})
}
fn literal(v: Value) -> Value {
    json!({"op":"literal","value":v})
}
fn eq(left: Value, right: Value) -> Value {
    json!({"op":"eq","left":left,"right":right})
}
fn absent(path: &str) -> Value {
    eq(read(path), literal(Value::Null))
}
fn present(path: &str) -> Value {
    json!({"op":"not","value":absent(path)})
}
pub fn predicate() -> Value {
    json!({"op":"all","values":[
        eq(read("context.statement.relation"),literal(json!("voluntary-expression-command-v1"))),
        eq(read("context.statement.publicInputs.actor"),read("identity")),
        absent("delta.nextPredicate"),eq(read("delta.read"),read("delta.consume")),
        present("basis.cells.0"),eq(read("delta.consume.0"),read("basis.cells.0.id")),
        absent("basis.cells.1"),absent("delta.consume.1"),
        present("delta.create.0"),absent("delta.create.1"),
        eq(read("context.statement.publicInputs.before"),read("basis.cells.0.value")),
        eq(read("context.statement.publicInputs.after"),read("delta.create.0.value")),
        {"op":"verify","statement":read("context.statement"),"proof":read("context.proof")}
    ]})
}
pub fn recognize(genesis: &Value, d: &Value, anchor: &Value) -> Result<Value> {
    require(
        anchor.as_str().is_some_and(|s| !s.is_empty()),
        "missing-basis-trust-anchor",
    )?;
    crate::canonical::canonical_json(d)?;
    shape(d, "arkId formation identityProfile coordinationProfile")?;
    crate::voluntary_expression_capacity::require_capacity(&d["formation"])
        .map_err(|_| crate::err("insufficient-expression-capacity"))?;
    let initial = crate::voluntary_expression::formation(&d["formation"])
        .map_err(|_| crate::err("invalid-basis-profile"))?;
    require(
        d["arkId"].as_str().is_some_and(|s| !s.is_empty()),
        "invalid-basis-profile",
    )?;
    let actors = d["identityProfile"]["actors"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-basis-profile"))?;
    require(
        json!(
            actors
                .iter()
                .map(|a| a["actor"].clone())
                .collect::<Vec<_>>()
        ) == initial["participants"],
        "invalid-basis-profile",
    )?;
    for (i, a) in actors.iter().enumerate() {
        require(
            !actors[..i].iter().any(|b| a["publicKey"] == b["publicKey"]),
            "invalid-basis-profile",
        )?;
    }
    let expected = json!({"arkId":d["arkId"],"activePredicate":predicate(),"initialValues":[initial],
        "context":{"application":{"application":{"formation":d["formation"]},"coordinationProfile":d["coordinationProfile"]},"identityProfile":d["identityProfile"]}});
    require(
        genesis["signed"]["payload"]["genesisInput"] == expected,
        "invalid-basis-profile",
    )?;
    coordination::initial(genesis, anchor)
}
