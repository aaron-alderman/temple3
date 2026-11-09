//! Exact representative authority-closure v0.7 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"priorClosure":"exact-v06-assessment-retained-as-historical-checkpoint","replacement":"relnav-v07-supersedes-relnav-row-only","historicalVsOperative":"v06-historical-refusal-limitations-remain-visible-v07-byte-bound-current-observation-is-not-runtime-authority","currentSource":"focused-valid-concurrent-uncommitted-not-governed-repair-or-adoption","authoringCustody":"never-promote-to-application-runtime-record-custody","missingRuntimeAuthority":"refuse-migration","forgiveness":{"disposition":"refuse","scope":"representative-authority-successor-v0.7"},"activation":"unavailable","production":"not-authorized","migration":"not-started"})
}
fn effects() -> Value {
    json!({"activation":false,"authoringRequestJournalMutation":true,"changeMaterialization":false,"filesystemSourceEditByThisRequest":false,"mapping":false,"migration":false,"productMutation":false,"repair":false,"semanticAdoption":false})
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
    json!({"application":"app.relnav","artifactHash":"c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7","boundaryKind":"temple3.relnav-runtime-authority-boundary/v0.7","currentWitnessHash":"14a632042764810f510ec0671e599d456d695ccce6af680ac9ed03665b38381d","historicalWitnessHash":"ce0cdb8c11a78a6c2dd52e4306adda8d4fcba043878327b6214c877df9f47b1e","assessmentHash":"1b0291a7fc082ae27095bcec3631a18ae51a4131c92d6fe1684cbaf627ed305d","currentSourceStatus":"focused-valid-concurrent-uncommitted","currentSourceHash":"3acda2cf5242b0eba88ec28416f7ff3cec0a229af900e6fd90871e1dd40f141b","currentSourceAuthority":"observation-only-not-governed-repair-or-adoption","historicalIssueCount":"6","historicalNamedIssueCount":"5","historicalIssue6":"unavailable","historicalIssuesExhaustive":false,"requestId":"request-7db0-a8a8-86ea","requestHead":"50cd2a963fc7d75dc8aba5035c823db515fa9dcc0450baf2aea203ac33f7c1a5","requestRevision":"7","requestStatus":"stopped","activeLeaseCount":"0","runtimeCorpusBytes":null,"operativeAuthority":"unavailable","capacity":"unmeasured","custody":"bounded-governed-authoring-request-custody-not-application-runtime-record-custody","unavailableOperationalEvidence":unavailable(),"migration":"not-started"})
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind policy priorClosure relnavReplacement retainedApplications effects",
    )?;
    require(
        x["kind"] == "temple3.representative-authority-closure/v0.7",
        "invalid-representative-authority-closure-v07",
    )?;
    require(
        x["policy"] == policy()
            && x["effects"] == effects()
            && x["priorClosure"]
                == json!({"assessmentHash":"1ebc44224dbb4f686948d798795bda95a59ab180bf53fd927da1d44dee2629ab","status":"refused-current-source-invalid-and-representative-operative-authority-incomplete","readyForMigration":false})
            && x["relnavReplacement"] == replacement()
            && x["retainedApplications"] == retained(),
        "invalid-representative-authority-closure-v07",
    )?;
    let r = &x["relnavReplacement"];
    let mut apps = x["retainedApplications"].as_array().unwrap().clone();
    apps.push(json!({"application":r["application"],"witnessHash":r["currentWitnessHash"],"requestHead":r["requestHead"],"operativeAuthority":r["operativeAuthority"],"capacity":r["capacity"],"custody":r["custody"],"migration":r["migration"]}));
    Ok(
        json!({"kind":"temple3.representative-authority-closure-audit/v0.7","status":"refused-current-source-uncontrolled-and-representative-operative-authority-incomplete","readyForMigration":false,"productionAuthorized":false,"applications":apps,"relnavReplacement":{"priorClosureAssessmentHash":x["priorClosure"]["assessmentHash"],"assessmentHash":r["assessmentHash"],"currentWitnessHash":r["currentWitnessHash"],"historicalWitnessHash":r["historicalWitnessHash"],"currentSourceStatus":r["currentSourceStatus"],"currentSourceAuthority":r["currentSourceAuthority"],"historicalIssues":{"reportedCount":r["historicalIssueCount"],"namedCount":r["historicalNamedIssueCount"],"issue6":r["historicalIssue6"],"exhaustive":r["historicalIssuesExhaustive"]},"requestStatus":r["requestStatus"],"activeLeaseCount":r["activeLeaseCount"],"runtimeCorpusBytes":"unavailable","unavailableOperationalEvidence":r["unavailableOperationalEvidence"],"status":"accepted-as-byte-bound-refusal-boundary-not-governed-repair-runtime-or-migration-authority"},"effects":x["effects"],"refusal":{"code":"current-source-uncontrolled-and-representative-operative-authority-incomplete","missing":r["unavailableOperationalEvidence"]},"policy":x["policy"]}),
    )
}
