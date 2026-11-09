//! Independent Relationship Navigator source-target reconciliation v0.6.
use serde_json::{Value, json};
type Result<T> = std::result::Result<T, String>;
fn req(v: bool) -> Result<()> {
    if v {
        Ok(())
    } else {
        Err("invalid-relnav-source-target-v0.6".into())
    }
}
fn policy() -> Value {
    json!({"successor":"exact-v05-reconciliation-and-v06-current-source-refusal-boundary","historicalVsOperative":"admitted-artifact-identity-is-not-current-source-or-runtime-authority","sourceValidation":"six-issue-current-source-refusal-is-not-source-behavior-runtime-nonexistence-or-migration-evidence","executableArtifacts":"missing-runtime-bundle-and-four-export-types-remain-explicitly-unsupported","unknownRuntimeBytes":"unavailable-never-zero","forgiveness":{"disposition":"refuse","scope":"authentic-relnav-source-target-journey"},"migration":"not-started"})
}
fn missing_exports() -> Value {
    json!([
        "ExecutableArtifact",
        "ExecutableApiContract",
        "ExecutableDependency",
        "ExecutableTestBinding"
    ])
}
fn preserved() -> Value {
    json!([{"semantic":"authenticated-route-intent","source":"commit and view routes declare authenticated access","target":"signed actor admission and authenticated participant inspection","qualification":"candidate-preservation-not-executed-enforcement-equivalence"},{"semantic":"explicit-formation-intent","source":"Context journal bootstrap declares explicitSeedOnly","target":"formation is a separate explicit operation with no implicit open default","qualification":"candidate-preservation-not-source-formation-equivalence"},{"semantic":"retained-recovery-intent","source":"content-addressed retention, bundle-before-receipt publication and fail-closed linear recovery declared","target":"retained laboratory writer and recovery qualification","qualification":"candidate-preservation-not-operational-custody-equivalence"}])
}
fn changed() -> Value {
    json!([{"semantic":"disclosure-scope","source":"route text declares signed-in participant own private Context and authorized closure","target":"participants-all-state selected voluntary-expression disclosure","reason":"target policy differs and source runtime delivery is unverified"},{"semantic":"writer-custody","source":"singleAuthoritativeProcessV1 declared","target":"selected laboratory writer with no source-writer transfer","reason":"a declaration does not establish custody identity or authority transfer"},{"semantic":"post-command-delivery","source":"recordAnswer declares reload-authorized-route-after-completion","target":"policy-first loopback response and explicit retained-outcome recovery","reason":"delivery mechanisms are intentionally different and neither proves source execution"}])
}
fn unsupported() -> Value {
    json!([
        "executed-authenticated-context-routes",
        "authentic-runtime-record-export-and-capture",
        "authoritative-runtime-record-head-and-completeness",
        "current-source-head-completeness",
        "operational-single-writer-custody-and-recovery",
        "private-actor-map-completeness-and-authority",
        "source-to-target-record-corpus-fit",
        "semantic-adoption-authority",
        "transport-and-delivery-receipts",
        "executed-replay-and-recovery",
        "executed-backout",
        "current-source-runtime-executable-bundle",
        "current-source-export-ExecutableArtifact",
        "current-source-export-ExecutableApiContract",
        "current-source-export-ExecutableDependency",
        "current-source-export-ExecutableTestBinding"
    ])
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x).map_err(|e| e.to_string())?;
    let o = x.as_object().ok_or("invalid")?;
    let keys = [
        "kind",
        "prior",
        "runtimeAuthorityRefusal",
        "runtimeAuthorityWitnessSha256",
        "expectedRuntimeAuthorityBoundaryId",
        "expectedRuntimeAuthorityAssessmentId",
        "representativeClosure",
        "expectedRepresentativeClosureId",
        "expectedRepresentativeAssessmentId",
        "policy",
    ];
    req(o.len() == keys.len()
        && keys.iter().all(|k| o.contains_key(*k))
        && x["kind"] == "temple3.relnav-source-target/v0.6"
        && x["policy"] == policy())?;
    let prior = crate::relnav_source_target_v05::assess(&x["prior"])?;
    req(prior["reconciliation"]["preserved"] == preserved()
        && prior["reconciliation"]["changed"] == changed()
        && prior["reconciliation"]["added"] == json!([])
        && prior["reconciliation"]["removed"] == json!([]))?;
    let runtime =
        crate::relnav_runtime_authority_refusal_v06::assess(&x["runtimeAuthorityRefusal"])
            .map_err(|e| e.to_string())?;
    let rbi = crate::canonical::identify(
        "relnav-runtime-authority-refusal-boundary",
        &x["runtimeAuthorityRefusal"],
    )
    .map_err(|e| e.to_string())?;
    let rai = crate::canonical::identify("relnav-runtime-authority-refusal-assessment", &runtime)
        .map_err(|e| e.to_string())?;
    req(x["runtimeAuthorityWitnessSha256"]
        == "ce0cdb8c11a78a6c2dd52e4306adda8d4fcba043878327b6214c877df9f47b1e"
        && x["expectedRuntimeAuthorityBoundaryId"] == rbi
        && x["expectedRuntimeAuthorityAssessmentId"] == rai)?;
    let d = &x["runtimeAuthorityRefusal"]["currentSourceDiscovery"];
    req(d["status"] == "refused-current-source-validation"
        && d["issueCount"] == "6"
        && d["missingBundle"] == "bundle.executableArtifacts.runtime"
        && d["missingExports"] == missing_exports()
        && d["runtimeDiscoveryReached"] == false
        && d["capabilityHorizonReached"] == false
        && d["authority"] == "authenticated-current-source-refusal-not-runtime-absence-proof")?;
    let representative =
        crate::representative_authority_closure_v06::assess(&x["representativeClosure"])
            .map_err(|e| e.to_string())?;
    let rci = crate::canonical::identify(
        "representative-authority-closure",
        &x["representativeClosure"],
    )
    .map_err(|e| e.to_string())?;
    let rcai = crate::canonical::identify(
        "representative-authority-closure-assessment",
        &representative,
    )
    .map_err(|e| e.to_string())?;
    req(x["expectedRepresentativeClosureId"] == rci
        && x["expectedRepresentativeAssessmentId"] == rcai)?;
    let relnav = representative["applications"]
        .as_array()
        .and_then(|a| a.iter().find(|v| v["application"] == "app.relnav"))
        .ok_or("missing-relnav")?;
    let runtime_hash = rai.split(":sha256:").nth(1).ok_or("bad-runtime-id")?;
    req(relnav["witnessHash"] == x["runtimeAuthorityWitnessSha256"]
        && representative["relnavReplacement"]["assessmentHash"] == runtime_hash)?;
    let q = &x["runtimeAuthorityRefusal"]["request"];
    Ok(
        json!({"kind":"temple3.relnav-source-target-assessment/v0.6","status":"refused-current-source-invalid-and-authentic-journey-incomplete","sourceId":prior["sourceId"],"formationId":prior["formationId"],"structureId":prior["structureId"],"declarationsId":prior["declarationsId"],"priorAssessmentKind":prior["kind"],"runtimeAuthorityBoundaryId":rbi,"runtimeAuthorityAssessmentId":rai,"representativeClosureId":rci,"representativeAssessmentId":rcai,"sourceValidationBoundary":{"status":d["status"],"issueCount":d["issueCount"],"missingBundle":d["missingBundle"],"missingExports":missing_exports(),"runtimeDiscoveryReached":false,"capabilityHorizonReached":false,"authority":d["authority"],"interpretation":"validation-refusal-not-removal-runtime-nonexistence-source-behavior-or-migration-evidence"},"requestFinality":{"requestId":q["requestId"],"revision":q["finalRevision"],"headHash":q["finalHeadHash"],"status":q["status"],"activeLeaseAfterStop":q["activeLeaseAfterStop"],"authority":"authoring-finality-only-not-runtime-finality"},"reconciliation":{"preserved":preserved(),"changed":changed(),"added":[],"removed":[],"removalAssessment":"no-authenticated-source-removal-claim","unsupported":unsupported()},"evidenceBoundary":{"historicalAdmission":"identity-only","currentSourceAuthority":runtime["currentSourceAuthority"],"executedRuntimeAuthority":runtime["executedRuntimeAuthority"],"authoringJournalMutation":true,"authoringCustody":runtime["authoringRequestCustody"]["authority"],"runtimeCorpusBytes":"unavailable","representativeAuthority":representative["status"]},"authority":{"source":"refused-current-source-validation","target":"candidate-profile-not-operative-import","activation":"unavailable"},"authenticJourney":"refused","readyForMigration":false,"migration":"not-started","refusal":{"code":"relnav-current-source-invalid-and-authentic-journey-unavailable","issues":d["issueCount"],"missing":runtime["unavailableOperationalEvidence"],"remainingRoute":runtime["remainingRoute"]},"policy":x["policy"]}),
    )
}
