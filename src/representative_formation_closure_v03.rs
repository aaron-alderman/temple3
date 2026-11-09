//! Representative closure v0.3 with authentic Relationship Navigator declarations.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"kind":"temple3.representative-formation-closure-policy/v0.3","priorClosure":"v0.2-historical-non-operative","authenticDeclarations":"candidate-source-intent-not-runtime-authority","missingOperationalJourney":"refuse-migration-readiness","sourceAuthority":"never-infer-from-target-compatibility-or-declarations","migration":"not-started","activation":"unavailable"})
}
fn deps() -> Value {
    json!([{"name":"source-context-route-declarations","status":"qualified","authority":"verified-admitted-declaration-evidence"},{"name":"authentic-runtime-records","status":"unavailable","authority":"no-operative-source-authority"},{"name":"executed-source-route-enforcement","status":"unavailable","authority":"no-operative-source-authority"},{"name":"source-writer-transfer","status":"unavailable","authority":"no-operative-source-authority"},{"name":"source-disclosure-enforcement","status":"unavailable","authority":"no-operative-source-authority"},{"name":"retained-source-contradictions","status":"unsupported","authority":"no-operative-source-authority"},{"name":"target-semantic-formation","status":"qualified","authority":"selected-laboratory-evidence"},{"name":"target-signed-admission","status":"qualified","authority":"selected-laboratory-evidence"},{"name":"target-capacity-reservation","status":"qualified","authority":"selected-laboratory-evidence"},{"name":"target-writer-recovery","status":"qualified","authority":"selected-laboratory-evidence"},{"name":"target-loopback-delivery-and-rendering","status":"qualified","authority":"selected-laboratory-evidence"}])
}
pub fn audit(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind prior relationshipNavigator relationshipNavigatorDependencies policy",
    )?;
    require(
        x["kind"] == "temple3.representative-formation-closure/v0.3"
            && x["policy"] == policy()
            && x["relationshipNavigatorDependencies"] == deps(),
        "invalid-representative-formation-closure-v03",
    )?;
    let prior = crate::representative_formation_closure_v02::audit(&x["prior"])
        .map_err(|_| crate::err("invalid-representative-formation-closure-v03"))?;
    let r = crate::relnav_source_target_v03::assess(&x["relationshipNavigator"])?;
    let old = prior["applications"]
        .as_array()
        .and_then(|a| a.iter().find(|v| v["application"] == "app.relnav"))
        .ok_or_else(|| crate::err("invalid-representative-formation-closure-v03"))?;
    require(
        old["sourceId"] == r["sourceId"]
            && old["formationId"] == r["formationId"]
            && prior["readyForMigration"] == false
            && r["readyForMigration"] == false,
        "invalid-representative-formation-closure-v03",
    )?;
    let apps: Vec<Value> = prior["applications"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            if a["application"] == "app.relnav" {
                let mut m = a.as_object().unwrap().clone();
                m.insert(
                    "formationStatus".into(),
                    json!("candidate-validated-with-authentic-declaration-reconciliation"),
                );
                m.insert("declarationsId".into(), r["declarationsId"].clone());
                Value::Object(m)
            } else {
                a.clone()
            }
        })
        .collect();
    Ok(
        json!({"kind":"temple3.representative-formation-closure-audit/v0.3","status":"audited-target-formations-with-authentic-relnav-declarations-and-operational-gaps","readyForMigration":false,"applications":apps,"relationshipNavigatorDependencies":x["relationshipNavigatorDependencies"],"refusal":{"code":"representative-operational-source-authority-incomplete","reason":"candidate-formations-and-relnav-declarations-qualified-but-runtime-records-route-enforcement-and-operational-custody-missing"},"policy":x["policy"]}),
    )
}
