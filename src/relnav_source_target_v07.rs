//! Independent Relationship Navigator source-target reconciliation v0.7.
use serde_json::{Value, json};
type Result<T> = std::result::Result<T, String>;
fn req(v: bool) -> Result<()> {
    if v {
        Ok(())
    } else {
        Err("invalid-relnav-source-target-v0.7".into())
    }
}
fn policy() -> Value {
    json!({"successor":"exact-v06-reconciliation-and-v07-byte-bound-authority-boundary","historicalVsOperative":"v06-five-named-plus-sixth-unavailable-is-historical-and-non-exhaustive-v07-is-operative-refusal-boundary","currentSource":"focused-valid-concurrent-uncommitted-is-observation-not-governed-repair-adoption-or-runtime-authority","sourceAdmission":"authenticated-baseline-identity-does-not-transfer-writer-custody-or-runtime-authority","unknownRuntimeBytes":"unavailable-never-zero","forgiveness":{"disposition":"refuse","scope":"authentic-relnav-source-target-journey-v0.7"},"production":"not-authorized","migration":"not-started"})
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
        "governed-current-source-repair",
        "governed-current-source-adoption",
        "unchanged-source-snapshot-requalification",
        "runtime-executor-admission",
        "production-activation-authority",
        "historical-validator-issue-six-exact-record"
    ])
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x).map_err(|e| e.to_string())?;
    let o = x.as_object().ok_or("invalid")?;
    let keys = [
        "kind",
        "prior",
        "runtimeAuthorityBoundary",
        "expectedRuntimeAuthorityBoundaryId",
        "expectedRuntimeAuthorityAssessmentId",
        "representativeClosure",
        "expectedRepresentativeClosureId",
        "expectedRepresentativeAssessmentId",
        "policy",
    ];
    req(o.len() == keys.len()
        && keys.iter().all(|k| o.contains_key(*k))
        && x["kind"] == "temple3.relnav-source-target/v0.7"
        && x["policy"] == policy())?;
    let prior = crate::relnav_source_target_v06::assess(&x["prior"])?;
    req(
        crate::canonical::identify("relnav-source-target-v06-assessment", &prior)
            .map_err(|e| e.to_string())?
            == "relnav-source-target-v06-assessment:sha256:413f78f4a7e71481dc4812455c28d5e7d14ed44574995702ede2c9181bc69ed3",
    )?;
    req(prior["reconciliation"]["preserved"] == preserved()
        && prior["reconciliation"]["changed"] == changed()
        && prior["reconciliation"]["added"] == json!([])
        && prior["reconciliation"]["removed"] == json!([]))?;
    let runtime =
        crate::relnav_runtime_authority_boundary_v07::assess(&x["runtimeAuthorityBoundary"])
            .map_err(|e| e.to_string())?;
    let rbi = crate::canonical::identify(
        "relnav-runtime-authority-boundary",
        &x["runtimeAuthorityBoundary"],
    )
    .map_err(|e| e.to_string())?;
    let rai = crate::canonical::identify("relnav-runtime-authority-assessment", &runtime)
        .map_err(|e| e.to_string())?;
    req(x["expectedRuntimeAuthorityBoundaryId"] == rbi
        && x["expectedRuntimeAuthorityAssessmentId"] == rai)?;
    let representative =
        crate::representative_authority_closure_v07::assess(&x["representativeClosure"])
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
    req(
        relnav["witnessHash"] == runtime["witnessDigests"]["current"]
            && representative["relnavReplacement"]["assessmentHash"] == runtime_hash,
    )?;
    let mut request = runtime["request"].as_object().unwrap().clone();
    request.insert(
        "authority".into(),
        json!("authoring-finality-only-not-runtime-finality"),
    );
    Ok(
        json!({"kind":"temple3.relnav-source-target-assessment/v0.7","status":"refused-authentic-journey-runtime-authority-and-governed-repair-incomplete","sourceId":prior["sourceId"],"formationId":prior["formationId"],"structureId":prior["structureId"],"declarationsId":prior["declarationsId"],"priorAssessmentKind":prior["kind"],"runtimeAuthorityBoundaryId":rbi,"runtimeAuthorityAssessmentId":rai,"representativeClosureId":rci,"representativeAssessmentId":rcai,"historicalBoundary":{"reportedIssueCount":runtime["historicalIssues"]["reportedCount"],"namedIssues":runtime["historicalIssues"]["named"],"issue6":runtime["historicalIssues"]["issue6"],"exhaustive":false,"authority":"historical-refusal-only-not-current-source-state"},"currentSourceBoundary":{"status":"focused-valid-concurrent-uncommitted","sourceHash":runtime["currentSource"]["governedObservedSourceHash"],"authority":runtime["currentSource"]["authority"],"repair":runtime["currentSource"]["repair"],"interpretation":"observation-not-governed-repair-adoption-source-control-runtime-authority-or-migration-evidence"},"requestFinality":request,"reconciliation":{"preserved":preserved(),"changed":changed(),"added":[],"removed":[],"removalAssessment":"no-authenticated-source-removal-claim","unsupported":unsupported()},"evidenceBoundary":{"historicalAdmission":"identity-only","currentAdmission":runtime["admission"]["authority"],"executedRuntimeAuthority":"unavailable","authoringJournalMutation":true,"runtimeCorpusBytes":"unavailable","representativeAuthority":representative["status"]},"authority":{"source":"concurrent-observation-only","target":"candidate-profile-not-operative-import","activation":"unavailable","production":"not-authorized"},"authenticJourney":"refused","readyForMigration":false,"productionAuthorized":false,"migration":"not-started","refusal":{"code":"relnav-authentic-journey-runtime-authority-and-governed-repair-incomplete","missing":runtime["unavailableOperationalEvidence"],"additionalAuthorityRequired":runtime["additionalAuthorityRequired"]},"policy":x["policy"]}),
    )
}
