//! Exact representative authority-closure v0.3 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};

fn policy() -> Value {
    json!({"priorClosure":"exact-v02-assessment-retained-as-historical-checkpoint","replacement":"inventory-v02-supersedes-inventory-row-only","historicalVsOperative":"definition-and-intent-not-runtime-authority","observedZero":"no-evidence-observed-not-proof-of-nonexistence","signingCustody":"never-promote-to-runtime-record-or-export-custody","missingRuntimeAuthority":"refuse-migration","forgiveness":{"disposition":"refuse","scope":"representative-authority-successor"},"activation":"unavailable","migration":"not-started"})
}
fn effects() -> Value {
    json!({"adoption":false,"binding":false,"materialization":false,"mapping":false,"migration":false,"activation":false,"productChange":false})
}
fn expected() -> Value {
    json!({"priorClosure":{"assessmentHash":"47e4ea4008865b059963c053f0ebcda38e25c930eea55f35e630e20c73bc0de2","status":"refused-operative-authority-and-capacity-incomplete","readyForMigration":false},"inventoryReplacement":{"application":"app.inventoryConservationLedgerReferenceV3","artifactHash":"8234a80e4c1a5744a25cf61255cc66107a7ffe878be89806d5248570f8844731","witnessHash":"0d224ad320176b7fa5f72dcbea9a45e7b03e95c9127af43605fd0bcb6e22c0e9","assessmentHash":"cdce7cbf83b78bf32099f5780743954b250a88d8b08aba67e9c46eb374100290","requestHead":"66410285090648b08a2a3ec5b9bb220f3d92c45576fae1e32a9b82e919ff7873","requestRevision":"4","activeLease":false,"observedEvidenceCount":"0","declaredOperationalCapabilities":"0","capabilityDisposition":"development-signing-custody-is-not-runtime-record-authority","runtimeCorpusBytes":null,"operativeAuthority":"unavailable","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started"},"retainedApplications":[{"application":"app.claimApprovalReference20260831V2","witnessHash":"96cc54de374ed557b2a4f144c9f5f9049e392df5303592415d3f146ce0053e07","requestHead":"855c24eeeed1e30806a06699a63955bb67a4020298c8a6fe0e28b966a392086c","operativeAuthority":"unavailable","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started"},{"application":"app.actorScopedCaseAuthorityReference","witnessHash":"7e071d604924e3ba1c7db8bba5faa29e0b003cd319e1f16be5f25da5b99f9d6d","requestHead":"d91b961a02eeee51b591b59522bc2b7f66e4c67a0f34f3c843eadf09634f948c","operativeAuthority":"unavailable","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started"},{"application":"app.relnav","witnessHash":"a9d33cc874662ffa8e5b4cc375cc152fea8c499599d8454a30ba10d61c68f0ab","requestHead":"4bd073422affec65c1ec1a4330d41d55a6d5db7a9d4dd348a6bc21357c8e208a","operativeAuthority":"unavailable","capacity":"unmeasured","custody":"qualified-observation-custody-not-source-authority","migration":"not-started"}]})
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind policy priorClosure inventoryReplacement retainedApplications effects",
    )?;
    require(
        x["kind"] == "temple3.representative-authority-closure/v0.3",
        "invalid-representative-authority-closure-v03",
    )?;
    let e = expected();
    for k in [
        "priorClosure",
        "inventoryReplacement",
        "retainedApplications",
    ] {
        require(x[k] == e[k], "invalid-representative-authority-closure-v03")?;
    }
    require(
        x["effects"] == effects(),
        "invalid-representative-authority-closure-v03",
    )?;
    require(
        x["policy"] == policy(),
        "invalid-representative-authority-policy-v03",
    )?;
    let i = &x["inventoryReplacement"];
    let inventory = json!({"application":i["application"],"witnessHash":i["witnessHash"],"requestHead":i["requestHead"],"operativeAuthority":"unavailable","capacity":"unmeasured","custody":i["custody"],"migration":"not-started"});
    let mut applications = vec![inventory];
    applications.extend(
        x["retainedApplications"]
            .as_array()
            .unwrap()
            .iter()
            .cloned(),
    );
    Ok(
        json!({"kind":"temple3.representative-authority-closure-audit/v0.3","status":"refused-operative-authority-and-capacity-incomplete","readyForMigration":false,"applications":applications,"inventoryReplacement":{"priorClosureAssessmentHash":x["priorClosure"]["assessmentHash"],"assessmentHash":i["assessmentHash"],"status":"accepted-as-newer-refusal-boundary"},"effects":x["effects"],"refusal":{"code":"representative-operative-authority-and-capacity-incomplete","missing":["authenticated-source-runtime-head-and-completeness","runtime-record-export-and-capture-authority","source-writer-and-record-custody","bounded-whole-corpus-or-explicit-chunked-capacity","transport-and-delivery-receipts","replay-recovery-and-backout"]},"policy":x["policy"]}),
    )
}
