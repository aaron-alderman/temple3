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
        "kind application artifactHash buildReceiptHash productLoweringReceiptHash authoringSnapshotHash authoringIntentHash factCount requirement decisions authorityClaims stewardshipManifest evidenceBoundary",
    )?;
    crate::canonical::canonical_json(v).map_err(|_| "invalid".to_string())?;
    let expected = json!({"kind":"temple3.actor-authority-source-intent/v0.1","application":"app.actorScopedCaseAuthorityReference","artifactHash":"ac0984ec7c5c9b22ab86d8fc2f24dfd9307f9b8122098e87a6ddb87c75a9ab7d","buildReceiptHash":"fdea71405917c59a12bf306ed1fe21c210d70e9786b67971367f376f30498b3c","productLoweringReceiptHash":"9d77e2faff3ba10a03eb65bca38f2f998a2efb75ba8ddd7bbd5bdf05f5b5aced","authoringSnapshotHash":"ede939d91fce1fc0539911259a12491d874b4d171cd14d27b18e90620a39e48b","authoringIntentHash":"3e2463780a0e6b837c89b0094075fcfccb9ff75215c93783ad1059ddc591849c","factCount":"28399","requirement":"actor-scoped-case-authority","decisions":[{"lens":"audience-boundary","option":"shared","provenance":"template.sourcery.application-start.v1"},{"lens":"authority-boundary","option":"sourcery-owned","provenance":"template.sourcery.application-start.v1"},{"lens":"collaboration-boundary","option":"single-writer","provenance":"template.sourcery.application-start.v1"}],"authorityClaims":[],"stewardshipManifest":"absent","evidenceBoundary":"admitted-intent-not-runtime-enforcement"});
    if *v == expected { Ok(()) } else { fail() }
}
pub fn assess(x: &Value) -> R<Value> {
    exact(
        x,
        "source formation selection expectedSourceId expectedFormationId",
    )?;
    crate::canonical::canonical_json(x).map_err(|_| "invalid".to_string())?;
    source(&x["source"])?;
    let state = crate::claim_lifecycle::run(&json!({"op":"formation","input":x["formation"]}))?;
    let p = &state["formation"]["policy"];
    let s = &x["selection"];
    exact(
        s,
        "kind commandAuthority delivery collaboration documentationGuard forgiveness retention custody activation",
    )?;
    let selected = json!({"kind":"temple3.actor-authority-target-selection/v0.1","commandAuthority":"author-reviewer-distinct","delivery":"participants-all-state","collaboration":"reject-stale-basis","documentationGuard":"required","forgiveness":{"disposition":"refuse","scope":"all-claims-events-and-approvals"},"retention":"retain-all-events-and-approvals","custody":"requires-separate-qualified-host","activation":"unavailable"});
    if *s != selected {
        return fail();
    }
    if p["author"] == p["reviewer"]
        || p["separation"] != "author-reviewer-distinct"
        || p["disclosure"] != s["delivery"]
        || p["retry"] != s["collaboration"]
        || p["documentationGuard"] != s["documentationGuard"]
        || p["retention"] != s["retention"]
        || p["forgiveness"] != s["forgiveness"]
    {
        return fail();
    }
    let source_id = crate::canonical::identify("actor-authority-source-intent", &x["source"])
        .map_err(|_| "invalid".to_string())?;
    let formation_id = crate::canonical::identify("claim-lifecycle-formation", &x["formation"])
        .map_err(|_| "invalid".to_string())?;
    if x["expectedSourceId"] != source_id || x["expectedFormationId"] != formation_id {
        return fail();
    }
    Ok(
        json!({"kind":"temple3.actor-authority-target-assessment/v0.1","sourceId":x["expectedSourceId"],"formationId":x["expectedFormationId"],"status":"compatible-with-declared-intent-gaps","preserved":[{"semantic":"actor-scoped-command-authority","source":"admitted semantic intent","target":"distinct author and reviewer command roles","evidence":"target-computation-and-signed-profile"}],"changed":[{"semantic":"shared-authoritative-delivery","source":"shared audience with template provenance and unknown field membership","target":"all retained claim state disclosed to configured participants","reason":"explicit target audience; no claim of source field equivalence"},{"semantic":"single-writer","source":"effectively one writer with template provenance","target":"distinct actor commands with stale-basis refusal in one ordered lifecycle","reason":"target collaboration policy is explicit and not attributed to source runtime"}],"added":[{"semantic":"forgiveness","target":"visible refusal with retained history"},{"semantic":"retry-and-retention","target":"reject stale basis and retain all events and approvals"},{"semantic":"documentation-guard","target":"author-marked documentation required before ready; not attributed to source"}],"unsupported":["exact-source-handlers-and-effects","runtime-actor-identity-mapping","source-read-and-disclosure-field-membership","source-refusal-behavior","authentic-runtime-records-and-head","source-custody-and-writer-transfer"],"authority":{"source":"historical-admitted-intent-only","target":"candidate-profile-not-operative-import","custody":s["custody"],"activation":"unavailable"},"migration":"not-started"}),
    )
}
