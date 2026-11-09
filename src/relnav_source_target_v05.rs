//! Independent Relationship Navigator source-target reconciliation v0.5.
use serde_json::{Value, json};
type Result<T> = std::result::Result<T, String>;
fn req(v: bool) -> Result<()> {
    if v {
        Ok(())
    } else {
        Err("invalid-relnav-source-target-v0.5".into())
    }
}
fn policy() -> Value {
    json!({"successor":"exact-v04-reconciliation-and-v05-governed-declaration-boundary","historicalVsOperative":"historical-admission-declarations-and-authoring-events-never-become-operative-authority","declarationsNotExecution":"15-subject-89-fact-closure-is-not-an-executed-route","observedZero":"zero-observed-evidence-is-not-proof-of-nonexistence","signingCustody":"development-after-commit-signing-is-not-runtime-record-custody","authoringCustody":"bounded-request-journal-mutation-is-not-application-runtime-record-custody","contradictions":"20-retained-unresolved-outside-closure","unknownRuntimeBytes":"unavailable-never-zero","forgiveness":{"disposition":"refuse","scope":"authentic-relnav-source-target-journey"},"migration":"not-started"})
}
fn subjects() -> Value {
    json!([
        "bundle.relnav.contextRuntime",
        "capability.sourcery.privateContextCommandEffect.v1",
        "consequence.relnav.recordAnswer.a",
        "consequence.relnav.recordAnswer.b",
        "journal.relnav.contexts",
        "module.relnav.contextActions",
        "policy.relnav.recordAnswer",
        "policy.relnav.recordAnswer.authority",
        "policy.relnav.recordAnswer.authorizedContextProjection",
        "policy.relnav.recordAnswer.privateEvidence",
        "relnav.recordAnswer",
        "routeAction.relnavContextCommit",
        "routeAction.relnavContextView",
        "rt.context.commit",
        "rt.context.view"
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
        "executed-backout"
    ])
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
        && x["kind"] == "temple3.relnav-source-target/v0.5"
        && x["policy"] == policy())?;
    let prior = crate::relnav_source_target_v04::assess(&x["prior"]).map_err(|e| e.to_string())?;
    req(prior["reconciliation"]["preserved"] == preserved()
        && prior["reconciliation"]["changed"] == changed()
        && prior["reconciliation"]["added"] == json!([])
        && prior["reconciliation"]["removed"] == json!([]))?;
    let operational = crate::relnav_operational_authority_v05::assess(&x["operationalBoundary"])
        .map_err(|e| e.to_string())?;
    let obi = crate::canonical::identify(
        "relnav-operational-authority-boundary",
        &x["operationalBoundary"],
    )
    .map_err(|e| e.to_string())?;
    let oai = crate::canonical::identify("relnav-operational-authority-assessment", &operational)
        .map_err(|e| e.to_string())?;
    req(x["operationalWitnessSha256"]
        == "7895dcc74eac8fb2cc417f8d5d15f6c3d5235481b849c17f46982bee977ee011"
        && x["expectedOperationalBoundaryId"] == obi
        && x["expectedOperationalAssessmentId"] == oai)?;
    let d = &x["operationalBoundary"]["runtimeDeclarationClosure"];
    let q = &x["operationalBoundary"]["request"];
    req(d["subjectIds"] == subjects()
        && d["factCount"] == "89"
        && d["factCorpusHash"]
            == "80cf52be1b1725099eb320c090f016365d17677b6c75ac75d46c4163af927fb8"
        && d["assessmentHash"]
            == "2c329b64020c0598fc65e07b665544f61e3975f59576d4c517be3eb4433ea7d0")?;
    req(q["requestId"] == "request-4a02-3eff-49d2"
        && q["finalRevision"] == "7"
        && q["finalHeadHash"]
            == "c175028bed93c726b1b2da4da0985c73bae2b643fbb886c5d34ca1423db0bad9"
        && q["status"] == "stopped"
        && q["activeLeaseAfterStop"] == false
        && q["authoringJournalOnly"] == true)?;
    let representative =
        crate::representative_authority_closure_v05::assess(&x["representativeClosure"])
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
        json!({"kind":"temple3.relnav-source-target-assessment/v0.5","status":"refused-authentic-source-target-journey-incomplete","sourceId":prior["sourceId"],"formationId":prior["formationId"],"structureId":prior["structureId"],"declarationsId":prior["declarationsId"],"priorAssessmentKind":prior["kind"],"operationalBoundaryId":obi,"operationalAssessmentId":oai,"representativeClosureId":rci,"representativeAssessmentId":rai,"declarationBoundary":{"subjectIds":subjects(),"factCount":d["factCount"],"factCorpusHash":d["factCorpusHash"],"assessmentHash":d["assessmentHash"],"contradictionCount":d["contradictionCount"],"contradictionDisposition":d["contradictionDisposition"],"authority":d["authority"]},"requestFinality":{"requestId":q["requestId"],"revision":q["finalRevision"],"headHash":q["finalHeadHash"],"status":q["status"],"activeLeaseAfterStop":q["activeLeaseAfterStop"],"authority":"authoring-finality-only-not-runtime-finality"},"reconciliation":{"preserved":preserved(),"changed":changed(),"added":[],"removed":[],"removalAssessment":"no-authenticated-source-removal-claim","unsupported":unsupported()},"evidenceBoundary":{"declarationsNotExecution":true,"observedEvidence":"zero-observed-not-nonexistence","sourceAuthority":operational["sourceAuthority"],"authoringJournalMutation":true,"authoringCustody":operational["authoringRequestCustody"]["authority"],"signingCustody":operational["signingCustody"],"runtimeCorpusBytes":"unavailable","semanticAdoption":operational["semanticAdoption"],"representativeAuthority":representative["status"]},"authority":{"source":"verified-admission-and-declarations-not-operative-runtime-authority","target":"candidate-profile-not-operative-import","activation":"unavailable"},"authenticJourney":"refused","readyForMigration":false,"migration":"not-started","refusal":{"code":"relnav-authentic-source-target-journey-unavailable","missing":operational["unavailableOperationalEvidence"]},"policy":x["policy"]}),
    )
}
