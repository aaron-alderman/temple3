//! Independent Relationship Navigator source-target reconciliation v0.4.
use serde_json::{Value, json};
type Result<T> = std::result::Result<T, String>;
fn req(v: bool) -> Result<()> {
    if v {
        Ok(())
    } else {
        Err("invalid-relnav-source-target-v0.4".into())
    }
}
fn policy() -> Value {
    json!({"successor":"exact-v03-semantic-reconciliation-retained","historicalVsOperative":"historical-admission-and-declarations-never-become-operative-authority","declarationsNotExecution":"exact-declaration-closure-is-not-an-executed-route","observedZero":"zero-observed-evidence-is-not-proof-of-nonexistence","signingCustody":"after-commit-signing-is-not-runtime-record-custody","authoringCustody":"bounded-request-journal-is-not-application-runtime-record-custody","unknownRuntimeBytes":"unavailable-never-zero","forgiveness":{"disposition":"refuse","scope":"authentic-relnav-source-target-journey"},"migration":"not-started"})
}
fn preserved() -> Value {
    json!([{"semantic":"authenticated-route-intent","source":"commit and view routes declare authenticated access","target":"signed actor admission and authenticated participant inspection","qualification":"candidate-preservation-not-executed-enforcement-equivalence"},{"semantic":"explicit-formation-intent","source":"Context journal bootstrap declares explicitSeedOnly","target":"formation is a separate explicit operation with no implicit open default","qualification":"candidate-preservation-not-source-formation-equivalence"},{"semantic":"retained-recovery-intent","source":"content-addressed retention, bundle-before-receipt publication and fail-closed linear recovery declared","target":"retained laboratory writer and recovery qualification","qualification":"candidate-preservation-not-operational-custody-equivalence"}])
}
fn changed() -> Value {
    json!([{"semantic":"disclosure-scope","source":"route text declares signed-in participant own private Context and authorized closure","target":"participants-all-state selected voluntary-expression disclosure","reason":"target policy differs and source runtime delivery is unverified"},{"semantic":"writer-custody","source":"singleAuthoritativeProcessV1 declared","target":"selected laboratory writer with no source-writer transfer","reason":"a declaration does not establish custody identity or authority transfer"},{"semantic":"post-command-delivery","source":"recordAnswer declares reload-authorized-route-after-completion","target":"policy-first loopback response and explicit retained-outcome recovery","reason":"delivery mechanisms are intentionally different and neither proves source execution"}])
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x).map_err(|e| e.to_string())?;
    let o = x.as_object().ok_or("invalid")?;
    let keys = [
        "kind",
        "prior",
        "operationalBoundary",
        "operationalWitnessSha256",
        "expectedOperationalBoundaryId",
        "expectedOperationalAssessmentId",
        "representativeClosure",
        "expectedRepresentativeClosureId",
        "expectedRepresentativeAssessmentId",
        "policy",
    ];
    req(o.len() == keys.len()
        && keys.iter().all(|k| o.contains_key(*k))
        && x["kind"] == "temple3.relnav-source-target/v0.4"
        && x["policy"] == policy())?;
    let prior = crate::relnav_source_target_v03::assess(&x["prior"]).map_err(|e| e.to_string())?;
    req(
        prior["declarationReconciliation"]["preserved"] == preserved()
            && prior["declarationReconciliation"]["changed"] == changed()
            && prior["declarationReconciliation"]["added"] == json!([])
            && prior["declarationReconciliation"]["removed"] == json!([]),
    )?;
    let operational = crate::relnav_operational_authority_v04::assess(&x["operationalBoundary"])
        .map_err(|e| e.to_string())?;
    let obi = crate::canonical::identify(
        "relnav-operational-authority-boundary",
        &x["operationalBoundary"],
    )
    .map_err(|e| e.to_string())?;
    let oai = crate::canonical::identify("relnav-operational-authority-assessment", &operational)
        .map_err(|e| e.to_string())?;
    req(x["operationalWitnessSha256"]
        == "7e2f7d435e3c9ef9fb47b547289bc593836cdbb267427fbc7ab28716659ed4bd"
        && x["expectedOperationalBoundaryId"] == obi
        && x["expectedOperationalAssessmentId"] == oai)?;
    let representative =
        crate::representative_authority_closure_v04::assess(&x["representativeClosure"])
            .map_err(|e| e.to_string())?;
    let rci = crate::canonical::identify(
        "representative-authority-closure",
        &x["representativeClosure"],
    )
    .map_err(|e| e.to_string())?;
    let rai = crate::canonical::identify(
        "representative-authority-closure-assessment",
        &representative,
    )
    .map_err(|e| e.to_string())?;
    req(x["expectedRepresentativeClosureId"] == rci
        && x["expectedRepresentativeAssessmentId"] == rai)?;
    let relnav = representative["applications"]
        .as_array()
        .and_then(|a| a.iter().find(|v| v["application"] == "app.relnav"))
        .ok_or("missing-relnav")?;
    let operational_hash = oai.split(":sha256:").nth(1).ok_or("bad-operational-id")?;
    req(relnav["witnessHash"] == x["operationalWitnessSha256"]
        && representative["relnavReplacement"]["assessmentHash"] == operational_hash)?;
    Ok(
        json!({"kind":"temple3.relnav-source-target-assessment/v0.4","status":"refused-authentic-source-target-journey-incomplete","sourceId":prior["sourceId"],"formationId":prior["formationId"],"structureId":prior["structureId"],"declarationsId":prior["declarationsId"],"priorAssessmentKind":prior["kind"],"operationalBoundaryId":obi,"operationalAssessmentId":oai,"representativeClosureId":rci,"representativeAssessmentId":rai,"reconciliation":{"preserved":preserved(),"changed":changed(),"added":[],"removed":[],"removalAssessment":"no-authenticated-source-removal-claim","unsupported":["executed-authenticated-context-routes","authentic-runtime-record-export-and-capture","current-source-head-completeness","operational-single-writer-custody-and-recovery","private-actor-map-completeness-and-authority","source-to-target-record-corpus-fit","transport-and-delivery-receipts","replay-and-recovery","backout"]},"evidenceBoundary":{"declarationClosure":operational["declarationClosure"],"declarationsNotExecution":true,"observedEvidence":"zero-observed-not-nonexistence","sourceAuthority":operational["sourceAuthority"],"requestFinality":"stopped-exact-head-no-active-lease-not-runtime-finality","authoringCustody":operational["authoringRequestCustody"]["authority"],"signingCustody":operational["signingCustody"],"runtimeCorpusBytes":"unavailable","representativeAuthority":representative["status"]},"authority":{"source":"verified-admission-and-declarations-not-operative-runtime-authority","target":"candidate-profile-not-operative-import","activation":"unavailable"},"authenticJourney":"refused","readyForMigration":false,"migration":"not-started","refusal":{"code":"relnav-authentic-operational-journey-unavailable","missing":operational["unavailableOperationalEvidence"]},"policy":x["policy"]}),
    )
}
