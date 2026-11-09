//! Cross-application formation closure audit; never source or migration authority.
use serde_json::{Value, json};
use std::collections::BTreeSet;
type R<T> = Result<T, String>;
fn fail<T>() -> R<T> {
    Err("invalid".into())
}
fn exact(v: &Value, fields: &str) -> R<()> {
    let Some(o) = v.as_object() else {
        return fail();
    };
    let a: BTreeSet<&str> = o.keys().map(String::as_str).collect();
    let e: BTreeSet<&str> = fields.split_whitespace().collect();
    if a == e { Ok(()) } else { fail() }
}
fn policy() -> Value {
    json!({"kind":"temple3.representative-formation-closure-policy/v0.1","missingDependency":"refuse-formation-closure","unavailableExecutable":"report-unavailable","historicalEvidence":"non-operative","sourceAuthority":"never-infer-from-target-compatibility","migration":"not-started","activation":"unavailable"})
}
fn execution(name: &str) -> Value {
    match name {
        "inventory" => {
            json!({"computation":"qualified-three-language","signedAdmission":"qualified-three-language","writer":"qualified-laboratory-writer","custody":"selected-laboratory-retained-history-not-source-transfer","delivery":"participant-signed-current-finality-full-state-candidate-not-source-equivalence","activation":"unavailable","migration":"not-started"})
        }
        "claimApproval" | "actorScoped" => {
            json!({"computation":"qualified-three-language","signedAdmission":"qualified-three-language","writer":"qualified-laboratory-writer","custody":"selected-laboratory-retained-history-not-source-transfer","delivery":"participants-all-state-candidate-not-source-equivalence","activation":"unavailable","migration":"not-started"})
        }
        _ => {
            json!({"computation":"unavailable-semantic-target","signedAdmission":"unavailable","writer":"unavailable","custody":"qualified-exact-observation-not-source-authority","delivery":"listed-inspector-exact-capsule","activation":"unavailable","migration":"not-started"})
        }
    }
}
fn deps(name: &str) -> Value {
    let rows: &[(&str, &str)] = match name {
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
            ("semantic-target-formation", "unavailable"),
            ("source-capture-authority", "unavailable"),
            ("observation-custody", "qualified"),
        ],
    };
    Value::Array(rows.iter().map(|(n,s)|json!({"name":n,"status":s,"authority":if *s=="qualified"{"selected-laboratory-evidence"}else{"no-operative-authority"}})).collect())
}
fn validate_entry(v: &Value, name: &str, assess: fn(&Value) -> R<Value>) -> R<Value> {
    exact(v, "assessment execution dependencies")?;
    if v["execution"] != execution(name) || v["dependencies"] != deps(name) {
        return fail();
    }
    let a = assess(&v["assessment"])?;
    if a["migration"] != "not-started" || a["authority"]["activation"] != "unavailable" {
        return fail();
    }
    Ok(a)
}
fn summary(app: &str, source: &Value, formation: &Value, e: &Value) -> Value {
    json!({"application":app,"sourceId":source,"formationId":formation,"formationStatus":if formation=="unavailable"{"unavailable"}else{"candidate-validated"},"writer":e["writer"],"custody":e["custody"],"delivery":e["delivery"],"activation":"unavailable","migration":"not-started"})
}
pub fn audit(x: &Value) -> R<Value> {
    crate::canonical::canonical_json(x).map_err(|_| "invalid".to_string())?;
    exact(
        x,
        "kind inventory claimApproval actorScoped relationshipNavigator policy",
    )?;
    if x["kind"] != "temple3.representative-formation-closure/v0.1" || x["policy"] != policy() {
        return fail();
    }
    let i = validate_entry(
        &x["inventory"],
        "inventory",
        crate::inventory_source_target::assess,
    )?;
    let c = validate_entry(
        &x["claimApproval"],
        "claimApproval",
        crate::claim_approval_source_target::assess,
    )?;
    let a = validate_entry(
        &x["actorScoped"],
        "actorScoped",
        crate::actor_authority_target::assess,
    )?;
    let r = &x["relationshipNavigator"];
    exact(r, "source observation execution dependencies")?;
    let rs = json!({"application":"app.relnav","admittedArtifactHash":"c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7","baselineHash":"a6d4f7e39faeaf24a758c60d95820f3e5da963095b33978f30d5361c8e915aec","projectionSha256":"b69af8f4caa2958e6ecceb87711e8eace2da331ceed64435da0aa4962da59df1"});
    let ro = json!({"capsuleId":"source-observation-capsule:sha256:24b278dc808c4d930e16212b9c0a5dbfc20a08fcab923bff546c350a59bb1912","custodyId":"source-capsule-custody:sha256:26e614750a9a648bea766321b3bdd4237cdb26e01b20770330a890c2a49073d9","sourceAuthority":"not-verified","runtime":"unavailable","activation":"unavailable"});
    if r["source"] != rs
        || r["observation"] != ro
        || r["execution"] != execution("relationshipNavigator")
        || r["dependencies"] != deps("relationshipNavigator")
    {
        return fail();
    }
    let rel_source = Value::String(format!(
        "admitted-artifact:sha256:{}",
        r["source"]["admittedArtifactHash"]
            .as_str()
            .ok_or("invalid")?
    ));
    let unavailable = Value::String("unavailable".into());
    let apps = json!([
        summary(
            "app.inventoryConservationLedgerReferenceV3",
            &i["sourceId"],
            &i["formationId"],
            &x["inventory"]["execution"]
        ),
        summary(
            "app.claimApprovalReference20260831V2",
            &c["sourceId"],
            &c["formationId"],
            &x["claimApproval"]["execution"]
        ),
        summary(
            "app.actorScopedCaseAuthorityReference",
            &a["sourceId"],
            &a["formationId"],
            &x["actorScoped"]["execution"]
        ),
        summary("app.relnav", &rel_source, &unavailable, &r["execution"])
    ]);
    Ok(
        json!({"kind":"temple3.representative-formation-closure-audit/v0.1","status":"audited-with-explicit-unavailable-dependencies","readyForMigration":false,"applications":apps,"refusal":{"code":"representative-formation-incomplete","reason":"missing-source-runtime-and-operational-authority"},"policy":x["policy"]}),
    )
}
