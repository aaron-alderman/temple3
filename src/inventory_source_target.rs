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
        "kind application artifactHash buildReceiptHash productLoweringReceiptHash authoringSnapshotHash authoringIntentHash lineageHash factCount requirement decisions authorityClaims stewardshipManifest evidenceBoundary",
    )?;
    crate::canonical::canonical_json(v).map_err(|_| "invalid".to_string())?;
    let expected = json!({"kind":"temple3.inventory-source-intent/v0.1","application":"app.inventoryConservationLedgerReferenceV3","artifactHash":"8234a80e4c1a5744a25cf61255cc66107a7ffe878be89806d5248570f8844731","buildReceiptHash":"4dbea0eba768e7f84183e8bd30575fa2eef9b0e0b011b7d8bc749729a763814e","productLoweringReceiptHash":"a97ad9cdd6ab3e17277bd2cb879f91ca84dbbe207ae44dc387caa28d788700f8","authoringSnapshotHash":"d7e46251b82814c237db2283293b4cc8d44eec5235e751525314ced7080d3588","authoringIntentHash":"c106b051e6b48aa12963c800fb7baee04a6c57bafa3875830ed95cf71cfb8fd2","lineageHash":"568badc6619454bab437cf6f81df75f3a442163574f55560963a9f1b931c4f35","factCount":"28400","requirement":"inventory-conservation","decisions":[{"lens":"audience-boundary","option":"shared","provenance":"template.sourcery.application-start.v1"},{"lens":"authority-boundary","option":"sourcery-owned","provenance":"template.sourcery.application-start.v1"},{"lens":"collaboration-boundary","option":"single-writer","provenance":"template.sourcery.application-start.v1"}],"authorityClaims":[],"stewardshipManifest":"absent","evidenceBoundary":"admitted-intent-not-definition-or-runtime"});
    if *v == expected { Ok(()) } else { fail() }
}
pub fn assess(x: &Value) -> R<Value> {
    exact(
        x,
        "source formation selection expectedSourceId expectedFormationId",
    )?;
    crate::canonical::canonical_json(x).map_err(|_| "invalid".to_string())?;
    source(&x["source"])?;
    let state = crate::inventory::formation(&x["formation"])?;
    let p = &state["policy"];
    let s = &x["selection"];
    exact(
        s,
        "kind conservation unit stock commandAuthority correction postedCorrection forgiveness retention amendment retry disclosure custody activation",
    )?;
    let selected = json!({"kind":"temple3.inventory-source-target-selection/v0.1","conservation":"closed-internal-posted-transfer-equation","unit":"ITEM","stock":"nonnegative","commandAuthority":"source-owner-drafts-reviewer-posts-roles-may-overlap","correction":"draft-only","postedCorrection":"new-independent-transfer","forgiveness":{"disposition":"refuse","scope":"inventory-transfer-history"},"retention":"retain-all-transition-evidence","amendment":"fixed-policy","retry":"reject-stale-basis","disclosure":"unselected-requires-separate-profile","custody":"requires-separate-qualified-host","activation":"unavailable"});
    if *s != selected {
        return fail();
    }
    if p["unit"] != s["unit"]
        || p["stock"] != s["stock"]
        || p["authority"] != s["commandAuthority"]
        || p["correction"] != s["correction"]
        || p["postedCorrection"] != s["postedCorrection"]
        || p["forgiveness"] != s["forgiveness"]
        || p["retention"] != s["retention"]
        || p["amendment"] != s["amendment"]
        || p["retry"] != s["retry"]
    {
        return fail();
    }
    let source_id = crate::canonical::identify("inventory-source-intent", &x["source"])
        .map_err(|_| "invalid".to_string())?;
    let formation_id = crate::canonical::identify("inventory-formation", &x["formation"])
        .map_err(|_| "invalid".to_string())?;
    if x["expectedSourceId"] != source_id || x["expectedFormationId"] != formation_id {
        return fail();
    }
    Ok(
        json!({"kind":"temple3.inventory-source-target-assessment/v0.1","sourceId":x["expectedSourceId"],"formationId":x["expectedFormationId"],"status":"candidate-target-with-declared-intent-gaps","preserved":[{"semantic":"inventory-conservation-intent","source":"admitted requirement only","target":"closed internal posted-transfer conservation equation","evidence":"target-computation-and-signed-profile","qualification":"candidate-preservation-not-source-equivalence"}],"changed":[{"semantic":"durable-authority","source":"Sourcery-owned with template provenance","target":"relation admission plus a separately qualified host","reason":"authority transfer is not implied"},{"semantic":"collaboration","source":"effectively one writer with template provenance","target":"one ordered lifecycle with explicit stale-basis refusal","reason":"source conflict behavior remains unknown"}],"added":[{"semantic":"quantity-and-unit","target":"bounded canonical integers in one explicit indivisible unit"},{"semantic":"command-authority","target":"source owner drafts; declared reviewer posts"},{"semantic":"forgiveness-and-retention","target":"visible refusal and retained transition history"}],"unsupported":["exact-source-conservation-equation-and-included-lifecycle","exact-source-command-guards-effects-and-diagnostics","authentic-runtime-records-and-consistent-head","runtime-actor-identity-mapping","source-shared-audience-field-membership","source-custody-export-and-writer-transfer","issuance-destruction-fractional-units-and-external-settlement"],"authority":{"source":"retained-governed-admission-observation-not-runtime-authority","target":"candidate-profile-not-operative-import","disclosure":s["disclosure"],"custody":s["custody"],"activation":"unavailable"},"migration":"not-started"}),
    )
}
