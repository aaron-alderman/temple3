//! Versioned representative candidate-formation closure v0.2.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn fail<T>() -> R<T> {
    Err("invalid".into())
}
fn policy() -> Value {
    json!({"kind":"temple3.representative-formation-closure-policy/v0.2","missingDependency":"refuse-formation-closure","unavailableExecutable":"report-unavailable","historicalEvidence":"non-operative","sourceAuthority":"never-infer-from-target-compatibility","targetCompatibility":"candidate-not-source-equivalence","migration":"not-started","activation":"unavailable"})
}
fn execution(n: &str) -> Value {
    match n {
        "inventory" => {
            json!({"computation":"qualified-three-language","signedAdmission":"qualified-three-language","writer":"qualified-laboratory-writer","custody":"selected-laboratory-retained-history-not-source-transfer","delivery":"participant-signed-current-finality-full-state-candidate-not-source-equivalence","activation":"unavailable","migration":"not-started"})
        }
        "claimApproval" | "actorScoped" => {
            json!({"computation":"qualified-three-language","signedAdmission":"qualified-three-language","writer":"qualified-laboratory-writer","custody":"selected-laboratory-retained-history-not-source-transfer","delivery":"participants-all-state-candidate-not-source-equivalence","activation":"unavailable","migration":"not-started"})
        }
        _ => {
            json!({"computation":"qualified-three-language","signedAdmission":"qualified-three-language","writer":"qualified-laboratory-writer","custody":"selected-laboratory-retained-history-not-source-transfer","delivery":"policy-first-loopback-target-not-source-equivalence","activation":"unavailable","migration":"not-started"})
        }
    }
}
fn deps(n: &str) -> Value {
    let rows: &[(&str, &str)] = match n {
        "inventory" => &[
            ("authentic-runtime-records", "unavailable"),
            ("source-writer-transfer", "unavailable"),
            ("target-custody-host", "qualified"),
            ("target-disclosure-profile", "qualified"),
        ],
        "claimApproval" => &[
            ("authentic-runtime-records", "unavailable"),
            ("payment-effect", "unsupported"),
            ("source-writer-transfer", "unavailable"),
            ("target-laboratory-writer", "qualified"),
        ],
        "actorScoped" => &[
            ("authentic-runtime-actors", "unavailable"),
            ("source-disclosure-enforcement", "unavailable"),
            ("source-writer-transfer", "unavailable"),
            ("target-laboratory-writer", "qualified"),
        ],
        _ => &[
            ("authentic-runtime-records", "unavailable"),
            ("source-writer-transfer", "unavailable"),
            ("source-disclosure-enforcement", "unavailable"),
            ("retained-source-contradictions", "unsupported"),
            ("observation-custody", "qualified"),
            ("target-semantic-formation", "qualified"),
            ("target-signed-admission", "qualified"),
            ("target-capacity-reservation", "qualified"),
            ("target-writer-recovery", "qualified"),
            ("target-loopback-delivery-and-rendering", "qualified"),
        ],
    };
    Value::Array(rows.iter().map(|(name,status)|json!({"name":name,"status":status,"authority":if *status=="qualified"{"selected-laboratory-evidence"}else{"no-operative-source-authority"}})).collect())
}
fn entry(v: &Value, n: &str, f: fn(&Value) -> R<Value>) -> R<Value> {
    crate::shape(v, "assessment execution dependencies").map_err(|_| "invalid".to_string())?;
    if v["execution"] != execution(n) || v["dependencies"] != deps(n) {
        return fail();
    }
    let a = f(&v["assessment"])?;
    if a["migration"] != "not-started" || a["authority"]["activation"] != "unavailable" {
        return fail();
    }
    Ok(a)
}
fn summary(app: &str, a: &Value, e: &Value) -> Value {
    json!({"application":app,"sourceId":a["sourceId"],"formationId":a["formationId"],"formationStatus":"candidate-validated","writer":e["writer"],"custody":e["custody"],"delivery":e["delivery"],"activation":"unavailable","migration":"not-started"})
}
pub fn audit(x: &Value) -> R<Value> {
    crate::canonical::canonical_json(x).map_err(|_| "invalid".to_string())?;
    crate::shape(
        x,
        "kind inventory claimApproval actorScoped relationshipNavigator policy",
    )
    .map_err(|_| "invalid".to_string())?;
    if x["kind"] != "temple3.representative-formation-closure/v0.2" || x["policy"] != policy() {
        return fail();
    }
    let i = entry(
        &x["inventory"],
        "inventory",
        crate::inventory_source_target::assess,
    )?;
    let c = entry(
        &x["claimApproval"],
        "claimApproval",
        crate::claim_approval_source_target::assess,
    )?;
    let a = entry(
        &x["actorScoped"],
        "actorScoped",
        crate::actor_authority_target::assess,
    )?;
    let r = entry(
        &x["relationshipNavigator"],
        "relationshipNavigator",
        crate::relnav_source_target::assess,
    )?;
    Ok(
        json!({"kind":"temple3.representative-formation-closure-audit/v0.2","status":"audited-target-formations-with-explicit-source-gaps","readyForMigration":false,"applications":[summary("app.inventoryConservationLedgerReferenceV3",&i,&x["inventory"]["execution"]),summary("app.claimApprovalReference20260831V2",&c,&x["claimApproval"]["execution"]),summary("app.actorScopedCaseAuthorityReference",&a,&x["actorScoped"]["execution"]),summary("app.relnav",&r,&x["relationshipNavigator"]["execution"])],"refusal":{"code":"representative-source-authority-incomplete","reason":"target-formations-qualified-source-runtime-and-operational-authority-missing"},"policy":x["policy"]}),
    )
}
