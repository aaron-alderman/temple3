//! Exact representative authority-closure v0.4 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};

fn policy() -> Value {
    json!({"priorClosure":"exact-v03-assessment-retained-as-historical-checkpoint","replacement":"relnav-v04-supersedes-relnav-row-only","historicalVsOperative":"admission-and-declarations-are-not-executed-runtime-authority","observedZero":"no-evidence-observed-not-proof-of-nonexistence","signingCustody":"never-promote-to-runtime-record-or-export-custody","authoringCustody":"never-promote-to-application-runtime-record-custody","missingRuntimeAuthority":"refuse-migration","forgiveness":{"disposition":"refuse","scope":"representative-authority-successor"},"activation":"unavailable","migration":"not-started"})
}
fn effects() -> Value {
    json!({"adoption":false,"binding":false,"materialization":false,"mapping":false,"migration":false,"activation":false,"productChange":false})
}
fn expected() -> Value {
    json!({
     "priorClosure":{"assessmentHash":"214d1d6b0275c2a761de274c7eff20aaa45482a9134cba16af5bcbb0566bfee6","status":"refused-operative-authority-and-capacity-incomplete","readyForMigration":false},
     "relnavReplacement":{"application":"app.relnav","artifactHash":"c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7","witnessHash":"7e2f7d435e3c9ef9fb47b547289bc593836cdbb267427fbc7ab28716659ed4bd","assessmentHash":"12d8ca911be4c52f9d90dcdc83cc2e8fb0cd5a89d6836028d0ea40fd285e3a6d","declarationAssessmentHash":"f84f49fc99591de272e1febcd14beaa83131ddb95f79fe48eea5e64575d59512","requestHead":"57c6408b8ec9e925b13607ed4bf6e08eba5c55a460650eb73d6f4cc53a267e81","requestRevision":"4","activeLease":false,"observedEvidenceCount":"0","declaredOperationalCapabilities":"0","declarationClosure":"exact-observed-not-executed","capabilityDisposition":"signing-custody-not-runtime-record-export-capture-writer-or-custody","runtimeCorpusBytes":null,"operativeAuthority":"unavailable","capacity":"unmeasured","custody":"bounded-retained-authoring-request-custody-not-application-runtime-record-custody","migration":"not-started"},
     "retainedApplications":[{"application":"app.inventoryConservationLedgerReferenceV3","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started","operativeAuthority":"unavailable","requestHead":"66410285090648b08a2a3ec5b9bb220f3d92c45576fae1e32a9b82e919ff7873","witnessHash":"0d224ad320176b7fa5f72dcbea9a45e7b03e95c9127af43605fd0bcb6e22c0e9"},{"application":"app.claimApprovalReference20260831V2","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started","operativeAuthority":"unavailable","requestHead":"855c24eeeed1e30806a06699a63955bb67a4020298c8a6fe0e28b966a392086c","witnessHash":"96cc54de374ed557b2a4f144c9f5f9049e392df5303592415d3f146ce0053e07"},{"application":"app.actorScopedCaseAuthorityReference","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started","operativeAuthority":"unavailable","requestHead":"d91b961a02eeee51b591b59522bc2b7f66e4c67a0f34f3c843eadf09634f948c","witnessHash":"7e071d604924e3ba1c7db8bba5faa29e0b003cd319e1f16be5f25da5b99f9d6d"}]
    })
}

pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind policy priorClosure relnavReplacement retainedApplications effects",
    )?;
    require(
        x["kind"] == "temple3.representative-authority-closure/v0.4",
        "invalid-representative-authority-closure-v04",
    )?;
    let e = expected();
    for k in ["priorClosure", "relnavReplacement", "retainedApplications"] {
        require(x[k] == e[k], "invalid-representative-authority-closure-v04")?;
    }
    require(
        x["effects"] == effects(),
        "invalid-representative-authority-closure-v04",
    )?;
    require(
        x["policy"] == policy(),
        "invalid-representative-authority-policy-v04",
    )?;
    let r = &x["relnavReplacement"];
    let relnav = json!({"application":r["application"],"witnessHash":r["witnessHash"],"requestHead":r["requestHead"],"operativeAuthority":"unavailable","capacity":"unmeasured","custody":r["custody"],"migration":"not-started"});
    let mut applications = x["retainedApplications"].as_array().unwrap().clone();
    applications.push(relnav);
    Ok(
        json!({"kind":"temple3.representative-authority-closure-audit/v0.4","status":"refused-operative-authority-and-capacity-incomplete","readyForMigration":false,"applications":applications,"relnavReplacement":{"priorClosureAssessmentHash":x["priorClosure"]["assessmentHash"],"assessmentHash":r["assessmentHash"],"declarationAssessmentHash":r["declarationAssessmentHash"],"status":"accepted-as-newer-refusal-boundary"},"effects":x["effects"],"refusal":{"code":"representative-operative-authority-and-capacity-incomplete","missing":["authenticated-source-runtime-head-and-completeness","runtime-record-export-and-capture-authority","source-writer-and-record-custody","bounded-whole-corpus-or-explicit-chunked-capacity","transport-and-delivery-receipts","replay-recovery-and-backout"]},"policy":x["policy"]}),
    )
}
