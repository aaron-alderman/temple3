//! Exact admitted-intent compatibility. This never observes or migrates source state.
use serde_json::{Value, json};
use std::collections::BTreeSet;
type R<T> = Result<T, String>;
fn fail<T>() -> R<T> {
    Err("invalid".into())
}
fn exact(v: &Value, fields: &str) -> R<()> {
    let Some(o) = v.as_object() else {
        return fail();
    };
    let a: BTreeSet<&str> = o.keys().map(String::as_str).collect();
    let e: BTreeSet<&str> = fields.split_whitespace().collect();
    if a == e { Ok(()) } else { fail() }
}
fn source(v: &Value) -> R<()> {
    exact(
        v,
        "kind application artifactHash buildReceiptHash productLoweringReceiptHash authoringSnapshotHash authoringIntentHash directIntentHash lineageHash factCount directIntent requirements excludedRequirement decisions authorityClaims stewardshipManifest evidenceBoundary",
    )?;
    crate::canonical::canonical_json(v).map_err(|_| "invalid".to_string())?;
    let expected = json!({"kind":"temple3.claim-approval-source-intent/v0.1","application":"app.claimApprovalReference20260831V2","artifactHash":"f15382f7de81b4c4fb8f25fc65c366516b4d62907d926c05f668802b262db2a5","buildReceiptHash":"88c120ebe44656fc85e215733b595156631b6c4502cbcb737ce7804373ee48a3","productLoweringReceiptHash":"c33a0bafe625d6c12595e93dbc799eb3e56180c94a45e101aaf88aa3c0ba61c8","authoringSnapshotHash":"b467318688c5a0c9dee3400458ed224cfe59c35e634abb95ec5ffd693fd5528d","authoringIntentHash":"203927c41992a301e1d9afc6eee097a1fb7c09638f547dfc368f98915ecdec0b","directIntentHash":"5017ff61305bf46aead61a55d7f183cd7f1a54d4b9a76807057219de91a2924b","lineageHash":"789214b594b76c17be13a14facb487be705ab8d840ce0a8f05db656fa5ad8355","factCount":"28386","directIntent":"Build a record-scoped claim approval application with a typed AUD money field, strict numeric threshold classification above AUD 1000, and a guarded ready-for-payment transition. Use Sourcery-owned durable records and reversible defaults.","requirements":["bounded-write-conflict-policy","numeric-threshold-classification","record-scoped-approval-action","shared-authoritative-delivery","sourcery-owned-durable-authority","typed-money-field"],"excludedRequirement":{"text":"reversible defaults","reason":"authoring-process-instruction-not-product-meaning"},"decisions":[{"lens":"audience-boundary","option":"shared","provenance":"template.sourcery.application-start.v1"},{"lens":"authority-boundary","option":"sourcery-owned","provenance":"template.sourcery.application-start.v1"},{"lens":"collaboration-boundary","option":"single-writer","provenance":"template.sourcery.application-start.v1"}],"authorityClaims":[],"stewardshipManifest":"absent","evidenceBoundary":"admitted-parameterized-intent-not-definition-or-runtime"});
    if *v == expected { Ok(()) } else { fail() }
}
pub fn assess(x: &Value) -> R<Value> {
    exact(
        x,
        "source formation selection expectedSourceId expectedFormationId",
    )?;
    crate::canonical::canonical_json(x).map_err(|_| "invalid".to_string())?;
    source(&x["source"])?;
    let f = crate::claim_readiness::run(&json!({"op":"formation","input":x["formation"]}))?;
    let p = &f["formation"]["policy"];
    let s = &x["selection"];
    exact(
        s,
        "kind money threshold comparison readiness approvalBasis separation repeat reversal forgiveness retention disclosure retry custody paymentEffect activation",
    )?;
    let selected = json!({"kind":"temple3.claim-approval-source-target-selection/v0.1","money":{"kind":"temple3.exact-money-policy/v0.1","currency":"AUD","scale":"2","maximumMinor":"999999999999","negative":"reject","rounding":"reject-inexact"},"threshold":{"currency":"AUD","decimal":"1000.00"},"comparison":"strictly-above","readiness":"guarded-current-approval-above-threshold-or-explicit-none-at-or-below","approvalBasis":"amount-and-evidence-revision","separation":"author-reviewer-distinct","repeat":"reject-already-ready","reversal":"author-reopen-invalidates-approval","forgiveness":{"disposition":"refuse","scope":"all-claims-events-and-approvals"},"retention":"retain-all-events-and-approvals","disclosure":"participants-all-state-candidate-not-source-equivalence","retry":"reject-stale-basis","custody":"requires-separate-qualified-host","paymentEffect":"none-readiness-only","activation":"unavailable"});
    if *s != selected {
        return fail();
    }
    if p["money"] != s["money"]
        || p["threshold"] != s["threshold"]
        || p["comparison"] != s["comparison"]
        || p["approvalBasis"] != s["approvalBasis"]
        || p["separation"] != s["separation"]
        || p["repeat"] != s["repeat"]
        || p["reversal"] != s["reversal"]
        || p["forgiveness"] != s["forgiveness"]
        || p["retention"] != s["retention"]
        || p["retry"] != s["retry"]
    {
        return fail();
    }
    let source_id = crate::canonical::identify("claim-approval-source-intent", &x["source"])
        .map_err(|_| "invalid".to_string())?;
    let formation_id = crate::canonical::identify("claim-readiness-formation", &x["formation"])
        .map_err(|_| "invalid".to_string())?;
    if x["expectedSourceId"] != source_id || x["expectedFormationId"] != formation_id {
        return fail();
    }
    Ok(
        json!({"kind":"temple3.claim-approval-source-target-assessment/v0.1","sourceId":x["expectedSourceId"],"formationId":x["expectedFormationId"],"status":"candidate-target-with-declared-intent-gaps","preserved":[{"semantic":"typed-aud-strict-threshold-intent","source":"full admitted parameterized intent","target":"AUD scale 2 with strict above 1000.00 classification","evidence":"target-computation-and-signed-profile","qualification":"candidate-preservation-not-source-representation-equivalence"},{"semantic":"guarded-ready-intent","source":"full admitted parameterized intent","target":"current approval required above threshold; explicit none at or below","evidence":"target-computation-and-signed-profile","qualification":"candidate-preservation-not-source-handler-equivalence"}],"changed":[{"semantic":"durable-authority","source":"Sourcery-owned with template provenance","target":"signed relation plus separately qualified host","reason":"authority transfer is not implied"},{"semantic":"actor-and-collaboration-policy","source":"effectively one writer; runtime actors unspecified","target":"distinct author/reviewer and stale-basis refusal","reason":"intentional target policy, not attributed to source runtime"},{"semantic":"shared-delivery","source":"shared audience with unknown field membership","target":"configured participants receive candidate target state","reason":"delivery equivalence remains unproved"}],"added":[{"semantic":"exact-money-representation","target":"canonical decimal, fixed scale, range, sign and inexact refusal"},{"semantic":"readiness-lifecycle","target":"repeat refusal, edit lock and explicit reopening invalidation"},{"semantic":"forgiveness-retention-retry","target":"visible refusal, retained history and stale-basis rejection"},{"semantic":"effect-boundary","target":"readiness emits no payment effect"}],"unsupported":["exact-source-money-storage-scale-range-and-malformed-input-policy","exact-source-threshold-and-ready-command-handlers-diagnostics","authentic-runtime-claims-approvals-records-and-consistent-head","runtime-actor-key-and-authority-mapping","source-shared-audience-field-membership-and-live-delivery","source-custody-export-and-writer-transfer","source-repeat-reversal-edit-and-forgiveness-semantics","payment-provider-effect-and-completion"],"authority":{"source":"retained-governed-admission-observation-not-runtime-authority","target":"candidate-profile-not-operative-import","custody":s["custody"],"paymentEffect":s["paymentEffect"],"activation":"unavailable"},"migration":"not-started"}),
    )
}
