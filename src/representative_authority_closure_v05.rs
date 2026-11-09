//! Exact representative authority-closure v0.5 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"priorClosure":"exact-v04-assessment-retained-as-historical-checkpoint","replacement":"relnav-v05-supersedes-relnav-row-only","historicalVsOperative":"admission-and-declarations-are-not-executed-runtime-authority","observedZero":"no-evidence-observed-not-proof-of-nonexistence","signingCustody":"never-promote-to-runtime-record-export-writer-transport-recovery-or-backout-custody","authoringCustody":"never-promote-to-application-runtime-record-custody","authoringJournalMutation":"bounded-governed-authoring-history-not-product-runtime-mutation","missingRuntimeAuthority":"refuse-migration","forgiveness":{"disposition":"refuse","scope":"representative-authority-successor"},"activation":"unavailable","migration":"not-started"})
}
fn effects() -> Value {
    json!({"activation":false,"adoption":false,"authoringRequestJournalMutation":true,"binding":false,"mapping":false,"materialization":false,"migration":false,"productMutation":false})
}
fn expected() -> Value {
    json!({"priorClosure":{"assessmentHash":"d59b63b3279167d9b4455e72149b0caac29a7995e5c74a73b2c677b0ee43c428","status":"refused-operative-authority-and-capacity-incomplete","readyForMigration":false},"relnavReplacement":{"application":"app.relnav","artifactHash":"c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7","witnessKind":"temple3.sourcery-relnav-governed-runtime-evidence-retry/v0.5","witnessHash":"7895dcc74eac8fb2cc417f8d5d15f6c3d5235481b849c17f46982bee977ee011","assessmentHash":"54d932e1d14f2693b762ae997020f2db6be6be73a6ab7ad0a447e3c91bd016ec","declarationAssessmentHash":"2c329b64020c0598fc65e07b665544f61e3975f59576d4c517be3eb4433ea7d0","declarationFactCount":"89","declarationFactCorpusHash":"80cf52be1b1725099eb320c090f016365d17677b6c75ac75d46c4163af927fb8","requestId":"request-4a02-3eff-49d2","requestHead":"c175028bed93c726b1b2da4da0985c73bae2b643fbb886c5d34ca1423db0bad9","requestRevision":"7","activeLease":false,"observedEvidenceCount":"0","declaredOperationalCapabilities":"0","declarationClosure":"exact-observed-not-executed","capabilityDisposition":"signing-custody-only-not-runtime-record-export-capture-writer-transport-recovery-or-backout","runtimeCorpusBytes":null,"operativeAuthority":"unavailable","capacity":"unmeasured","custody":"bounded-authoring-request-custody-not-application-runtime-record-custody","authoringRequestStateBytes":"139487","authoringRequestMaximumBytes":"2097152","authoringRequestJournalMutation":true,"migration":"not-started"},"retainedApplications":[{"application":"app.inventoryConservationLedgerReferenceV3","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started","operativeAuthority":"unavailable","requestHead":"66410285090648b08a2a3ec5b9bb220f3d92c45576fae1e32a9b82e919ff7873","witnessHash":"0d224ad320176b7fa5f72dcbea9a45e7b03e95c9127af43605fd0bcb6e22c0e9"},{"application":"app.claimApprovalReference20260831V2","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started","operativeAuthority":"unavailable","requestHead":"855c24eeeed1e30806a06699a63955bb67a4020298c8a6fe0e28b966a392086c","witnessHash":"96cc54de374ed557b2a4f144c9f5f9049e392df5303592415d3f146ce0053e07"},{"application":"app.actorScopedCaseAuthorityReference","capacity":"unmeasured","custody":"selected-target-laboratory-not-source-transfer","migration":"not-started","operativeAuthority":"unavailable","requestHead":"d91b961a02eeee51b591b59522bc2b7f66e4c67a0f34f3c843eadf09634f948c","witnessHash":"7e071d604924e3ba1c7db8bba5faa29e0b003cd319e1f16be5f25da5b99f9d6d"}]})
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind policy priorClosure relnavReplacement retainedApplications effects",
    )?;
    require(
        x["kind"] == "temple3.representative-authority-closure/v0.5",
        "invalid-representative-authority-closure-v05",
    )?;
    let e = expected();
    for k in ["priorClosure", "relnavReplacement", "retainedApplications"] {
        require(x[k] == e[k], "invalid-representative-authority-closure-v05")?
    }
    require(
        x["effects"] == effects(),
        "invalid-representative-authority-closure-v05",
    )?;
    require(
        x["policy"] == policy(),
        "invalid-representative-authority-policy-v05",
    )?;
    let r = &x["relnavReplacement"];
    let relnav = json!({"application":r["application"],"witnessHash":r["witnessHash"],"requestHead":r["requestHead"],"operativeAuthority":"unavailable","capacity":"unmeasured","custody":r["custody"],"migration":"not-started"});
    let mut apps = x["retainedApplications"].as_array().unwrap().clone();
    apps.push(relnav);
    Ok(
        json!({"kind":"temple3.representative-authority-closure-audit/v0.5","status":"refused-operative-authority-and-capacity-incomplete","readyForMigration":false,"applications":apps,"relnavReplacement":{"priorClosureAssessmentHash":x["priorClosure"]["assessmentHash"],"assessmentHash":r["assessmentHash"],"declarationAssessmentHash":r["declarationAssessmentHash"],"declarationFactCount":r["declarationFactCount"],"status":"accepted-as-newer-refusal-boundary"},"effects":x["effects"],"refusal":{"code":"representative-operative-authority-and-capacity-incomplete","missing":["authenticated-source-runtime-head-and-completeness","runtime-record-export-and-capture-authority","source-writer-and-record-custody","bounded-whole-corpus-or-explicit-chunked-capacity","transport-and-delivery-receipts","replay-recovery-and-backout"]},"policy":x["policy"]}),
    )
}
