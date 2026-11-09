//! Exact Relationship Navigator operational-authority v0.4 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};

fn policy() -> Value {
    json!({"historicalVsOperative":"admission-and-declarations-are-not-executed-runtime-authority","semanticAdoption":"explicit-governed-choice-never-inferred","declarationsNotExecution":"exact-observed-declarations-never-promoted","observedZero":"absence-of-observed-evidence-is-not-proof-of-nonexistence","signingCustody":"never-promote-to-runtime-record-custody","authoringCustody":"never-promote-to-application-runtime-record-custody","contradictions":"retained-unresolved","unknownRuntimeBytes":"refuse-not-zero","forgiveness":{"disposition":"refuse","scope":"executed-relnav-operational-authority"},"migration":"not-started"})
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind admission intentDiscovery operationalDeclarationClosure capabilityDiscovery request authoringRequestCustody unavailableOperationalEvidence effects runtimeCorpusBytes migration policy",
    )?;
    require(
        x["kind"] == "temple3.relnav-operational-authority-boundary/v0.4",
        "invalid-relnav-operational-authority-v04",
    )?;
    // Commit to the exact canonical witness fields without embedding the source receipt.
    let fields = [
        "admission",
        "authoringRequestCustody",
        "capabilityDiscovery",
        "effects",
        "intentDiscovery",
        "migration",
        "operationalDeclarationClosure",
        "request",
        "unavailableOperationalEvidence",
    ];
    let witness = Value::Object(fields.into_iter().map(|key| (key.into(), x[key].clone())).collect());
    require(
        crate::canonical::identify("authority-evidence", &witness)?
            == "authority-evidence:sha256:e7efbde4dcd82d5d91bcb71d1770f9547b69d55216441658442c86b2c1f5a50f",
        "invalid-relnav-operational-authority-v04",
    )?;
    require(
        x["runtimeCorpusBytes"].is_null(),
        "invented-runtime-corpus-bytes",
    )?;
    require(
        x["policy"] == policy(),
        "invalid-relnav-operational-authority-policy-v04",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-operational-authority-assessment/v0.4","status":"refused-executed-operational-authority-unavailable","declarationClosure":"exact-observed-not-executed","sourceAuthority":"unavailable","semanticAdoption":"not-selected","runtimeCorpusBytes":"unavailable","signingCustody":"available-after-commit-only-not-runtime-record-custody","authoringRequestCustody":{"status":"bounded-retained","bytes":x["authoringRequestCustody"]["stateBytes"],"maximumBytes":x["authoringRequestCustody"]["maximumStateBytes"],"authority":x["authoringRequestCustody"]["authority"]},"unavailableOperationalEvidence":x["unavailableOperationalEvidence"],"effects":x["effects"],"readyForMigration":false,"migration":"not-started","refusal":{"code":"executed-relnav-operational-authority-unavailable","missing":x["unavailableOperationalEvidence"]},"policy":x["policy"]}),
    )
}
