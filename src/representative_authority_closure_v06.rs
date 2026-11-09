//! Exact representative authority-closure v0.6 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"priorClosure":"exact-v05-assessment-retained-as-historical-checkpoint","replacement":"relnav-v06-supersedes-relnav-row-only","historicalVsOperative":"admitted-artifact-identity-is-not-current-source-or-runtime-authority","sourceValidity":"current-source-validation-refusal-is-not-runtime-record-absence-proof-or-source-or-migration-authority","authoringCustody":"never-promote-to-application-runtime-record-custody","authoringJournalMutation":"bounded-governed-authoring-history-not-product-runtime-mutation","missingRuntimeAuthority":"refuse-migration","forgiveness":{"disposition":"refuse","scope":"representative-authority-successor"},"activation":"unavailable","migration":"not-started"})
}
fn effects() -> Value {
    json!({"activation":false,"adoption":false,"authoringRequestJournalMutation":true,"binding":false,"capabilityAcquisition":false,"mapping":false,"materialization":false,"migration":false,"productMutation":false})
}
fn unavailable() -> Value {
    json!([
        "executed-authenticated-provider-route-journey",
        "authoritative-runtime-record-head-and-completeness",
        "runtime-record-export-or-capture-authority",
        "actual-runtime-corpus-bytes",
        "operational-writer-and-record-custody",
        "transport-and-delivery-receipts",
        "executed-replay-and-recovery",
        "executed-backout"
    ])
}
fn retained() -> Value {
    json!([{"application":"app.inventoryConservationLedgerReferenceV3","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started","operativeAuthority":"unavailable","requestHead":"66410285090648b08a2a3ec5b9bb220f3d92c45576fae1e32a9b82e919ff7873","witnessHash":"0d224ad320176b7fa5f72dcbea9a45e7b03e95c9127af43605fd0bcb6e22c0e9"},{"application":"app.claimApprovalReference20260831V2","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started","operativeAuthority":"unavailable","requestHead":"855c24eeeed1e30806a06699a63955bb67a4020298c8a6fe0e28b966a392086c","witnessHash":"96cc54de374ed557b2a4f144c9f5f9049e392df5303592415d3f146ce0053e07"},{"application":"app.actorScopedCaseAuthorityReference","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started","operativeAuthority":"unavailable","requestHead":"d91b961a02eeee51b591b59522bc2b7f66e4c67a0f34f3c843eadf09634f948c","witnessHash":"7e071d604924e3ba1c7db8bba5faa29e0b003cd319e1f16be5f25da5b99f9d6d"}])
}
fn replacement() -> Value {
    json!({"application":"app.relnav","artifactHash":"c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7","witnessKind":"temple3.sourcery-relnav-governed-runtime-authority-refusal/v0.6","witnessHash":"ce0cdb8c11a78a6c2dd52e4306adda8d4fcba043878327b6214c877df9f47b1e","assessmentHash":"ed9c759a9996cd3a2a18a25c472329058d46215930813dd20794713da6e10e67","currentSourceStatus":"refused-current-source-validation","currentSourceIssueCount":"6","currentSourceAuthority":"authenticated-current-source-refusal-not-runtime-absence-proof","requestId":"request-2fb8-9e2c-4082","requestHead":"e9f5582b6272b39cb8fcca6ad2606561741949119f84a7a42347b308243b1a9c","requestRevision":"4","requestStatus":"stopped","activeLease":false,"runtimeCorpusBytes":null,"operativeAuthority":"unavailable-not-proven-absent","capacity":"unmeasured","custody":"bounded-authoring-request-custody-not-application-runtime-record-custody","authoringRequestJournalRevision":"54","authoringRequestStateBytes":"150487","authoringRequestMaximumBytes":"2097152","authoringRequestJournalMutation":true,"unavailableOperationalEvidence":unavailable(),"migration":"not-started"})
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind policy priorClosure relnavReplacement retainedApplications effects",
    )?;
    require(
        x["kind"] == "temple3.representative-authority-closure/v0.6",
        "invalid-representative-authority-closure-v06",
    )?;
    require(
        x["priorClosure"]
            == json!({"assessmentHash":"24554e77fbe752b6067238a326666fe8962dd6f5eabea84cc9a7c4bf41817e4e","status":"refused-operative-authority-and-capacity-incomplete","readyForMigration":false}),
        "invalid-representative-authority-closure-v06",
    )?;
    require(
        x["relnavReplacement"] == replacement(),
        "invalid-representative-authority-closure-v06",
    )?;
    require(
        x["retainedApplications"] == retained(),
        "invalid-representative-authority-closure-v06",
    )?;
    require(
        x["effects"] == effects(),
        "invalid-representative-authority-closure-v06",
    )?;
    require(
        x["policy"] == policy(),
        "invalid-representative-authority-policy-v06",
    )?;
    let r = &x["relnavReplacement"];
    let mut apps = x["retainedApplications"].as_array().unwrap().clone();
    apps.push(json!({"application":r["application"],"witnessHash":r["witnessHash"],"requestHead":r["requestHead"],"operativeAuthority":r["operativeAuthority"],"capacity":r["capacity"],"custody":r["custody"],"migration":r["migration"]}));
    Ok(
        json!({"kind":"temple3.representative-authority-closure-audit/v0.6","status":"refused-current-source-invalid-and-representative-operative-authority-incomplete","readyForMigration":false,"applications":apps,"relnavReplacement":{"priorClosureAssessmentHash":x["priorClosure"]["assessmentHash"],"assessmentHash":r["assessmentHash"],"currentSourceStatus":r["currentSourceStatus"],"currentSourceIssueCount":r["currentSourceIssueCount"],"requestStatus":r["requestStatus"],"activeLease":r["activeLease"],"runtimeCorpusBytes":"unavailable","unavailableOperationalEvidence":r["unavailableOperationalEvidence"],"status":"accepted-as-newer-refusal-boundary-not-source-or-runtime-authority"},"effects":x["effects"],"refusal":{"code":"current-source-invalid-and-representative-operative-authority-incomplete","issues":r["currentSourceIssueCount"],"missing":r["unavailableOperationalEvidence"]},"policy":x["policy"]}),
    )
}
