//! Exact Relationship Navigator operational-authority v0.5 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};

fn policy() -> Value {
    json!({"historicalVsOperative":"admission-and-declarations-are-not-executed-runtime-authority","semanticAdoption":"explicit-governed-choice-never-inferred","declarationsNotExecution":"exact-observed-declarations-never-promoted","observedZero":"absence-of-observed-evidence-is-not-proof-of-nonexistence","signingCustody":"never-promote-to-runtime-record-custody","authoringCustody":"never-promote-to-application-runtime-record-custody","contradictions":"retained-unresolved","unknownRuntimeBytes":"refuse-not-zero","forgiveness":{"disposition":"refuse","scope":"executed-relnav-operational-authority"},"migration":"not-started"})
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind admission intentDiscovery runtimeDeclarationClosure capabilityDiscovery request authoringRequestCustody unavailableOperationalEvidence effects runtimeCorpusBytes migration conclusion policy",
    )?;
    require(
        x["kind"] == "temple3.relnav-operational-authority-boundary/v0.5",
        "invalid-relnav-operational-authority-v05",
    )?;
    // Commit to the exact canonical witness fields without embedding the source receipt.
    let fields = [
        "admission",
        "authoringRequestCustody",
        "capabilityDiscovery",
        "conclusion",
        "effects",
        "intentDiscovery",
        "migration",
        "request",
        "runtimeDeclarationClosure",
        "unavailableOperationalEvidence",
    ];
    let witness = Value::Object(fields.into_iter().map(|key| (key.into(), x[key].clone())).collect());
    require(
        crate::canonical::identify("authority-evidence", &witness)?
            == "authority-evidence:sha256:5c916f38cffc92a90fedcb8cec1ba73d750a3503e6c633621daadee062cd2a65",
        "invalid-relnav-operational-authority-v05",
    )?;
    require(
        x["runtimeCorpusBytes"].is_null(),
        "invented-runtime-corpus-bytes",
    )?;
    require(
        x["policy"] == policy(),
        "invalid-relnav-operational-authority-policy-v05",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-operational-authority-assessment/v0.5","status":x["conclusion"],"declarationClosure":"exact-observed-not-executed","sourceAuthority":"unavailable","semanticAdoption":"not-selected","runtimeCorpusBytes":"unavailable","signingCustody":"available-after-commit-only-not-runtime-record-custody","authoringRequestCustody":{"status":"bounded-retained","bytes":x["authoringRequestCustody"]["stateBytes"],"maximumBytes":x["authoringRequestCustody"]["maximumStateBytes"],"authority":x["authoringRequestCustody"]["authority"]},"unavailableOperationalEvidence":x["unavailableOperationalEvidence"],"effects":x["effects"],"readyForMigration":false,"migration":"not-started","refusal":{"code":"executed-relnav-operational-authority-unavailable","missing":x["unavailableOperationalEvidence"]},"policy":x["policy"]}),
    )
}
