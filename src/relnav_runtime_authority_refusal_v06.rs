//! Exact Relationship Navigator governed current-source refusal v0.6 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};

fn policy() -> Value {
    json!({"historicalVsOperative":"admitted-artifact-identity-is-not-current-source-or-runtime-authority","sourceValidity":"current-source-validation-refusal-is-not-runtime-absence-proof","closedContract":"invalid-attempt-shape-is-not-capability-evidence","unknownRuntimeBytes":"refuse-not-zero","forgiveness":{"disposition":"refuse","scope":"executed-relnav-runtime-authority"},"migration":"not-started"})
}
pub fn assess(value: &Value) -> Result<Value> {
    crate::canonical::canonical_json(value)?;
    shape(
        value,
        "kind application historicalAuthority currentSourceDiscovery auxiliaryClosedContractRefusal request authoringRequestCustody unavailableOperationalEvidence effects runtimeCorpusBytes migration remainingRoute conclusion policy",
    )?;
    // Commit to the exact canonical witness fields without embedding the source receipt.
    let fields = [
        "application",
        "authoringRequestCustody",
        "auxiliaryClosedContractRefusal",
        "conclusion",
        "currentSourceDiscovery",
        "effects",
        "historicalAuthority",
        "kind",
        "migration",
        "remainingRoute",
        "request",
        "runtimeCorpusBytes",
        "unavailableOperationalEvidence",
    ];
    let witness = Value::Object(fields.into_iter().map(|key| (key.into(), value[key].clone())).collect());
    require(
        crate::canonical::identify("authority-evidence", &witness)?
            == "authority-evidence:sha256:5a00ea17259f55d4306953488bb9f5bf2472e82f7b7542035f8aec4050b9a249",
        "invalid-relnav-runtime-authority-refusal-v06",
    )?;
    require(
        value["runtimeCorpusBytes"].is_null(),
        "invented-runtime-corpus-bytes",
    )?;
    require(
        value["policy"] == policy(),
        "invalid-relnav-runtime-authority-refusal-policy-v06",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-runtime-authority-refusal-assessment/v0.6","status":value["conclusion"],"historicalAuthority":"authenticated-admitted-artifact-only","currentSourceAuthority":"refused-validation","executedRuntimeAuthority":"unavailable-not-proven-absent","runtimeCorpusBytes":"unavailable","authoringRequestCustody":value["authoringRequestCustody"],"unavailableOperationalEvidence":value["unavailableOperationalEvidence"],"remainingRoute":value["remainingRoute"],"effects":value["effects"],"readyForMigration":false,"migration":"not-started","refusal":{"code":"current-source-invalid-and-executed-operational-authority-unavailable","issues":value["currentSourceDiscovery"]["issueCount"]},"policy":value["policy"]}),
    )
}
