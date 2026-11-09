//! Relationship Navigator governed static-observation/target compatibility.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn fail<T>() -> R<T> {
    Err("invalid-relnav-source-target".into())
}
fn expected_source() -> Value {
    json!({"kind":"temple3.relnav-source-observation/v0.1","application":"app.relnav","artifactHash":"c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7","baselineHash":"a6d4f7e39faeaf24a758c60d95820f3e5da963095b33978f30d5361c8e915aec","buildReceiptHash":"b5f5da95173dac2e3604e667926ed6a7223a0339a80481bef43fe05074df31da","lineageHash":"7de357781e36c79999ca1e24bb258fbfed356bb00b8c63ab1b37086cb95780af","bundleProjectionSha256":"b69af8f4caa2958e6ecceb87711e8eace2da331ceed64435da0aa4962da59df1","sourceInventorySha256":"40e5256749fbc79d73639fcf93342dbf34d20ae3605e15858d4251897804044e","sourceInventoryListingHash":"022fd97891876bb7eb00567c3d55a4403151b13507906967f49bc82f8c989049","sourceListDigest":"1e266b005dabd683ef87ca5633df3a474c823edc3cc3047c032ff553bdc87279","javascriptSymbolHash":"c7854c1eb31692bdd93186991a271e39dbabefd0f983460c3a21d0cc5321e122","offerProjectionSha256":"233fc97e9a1f63c0406a81dd94d8a6ac9a3da6e2b30b2fcab4e584c55d171559","offerAssessmentHashes":["8749d90f801b1767046d30f4c9ae006e55c8e46b0038c658a54f995fe0ac3168","3389ae241978c8c50f54e0c13500f11a757d4085869aee11e58f5d4f543281fd","1e2d015880645e7409629b697d5cbed637d8857c89f98843a441032f7d47bbec","1440a5d2fbac7e4c394d604d40327e216726bbd36614ab23346a903241d5c1b9","955c2cf2f02b70f0da4872a1e6d7071c86debe8d761204de1a2e05b518b81c10"],"consentProjectionSha256":"f428261b2366ad213ed2e7d7676e6bfca4fb268901b3b068e3c76c011be1031b","consentAssessmentHashes":["115a2572ec6a03be508eeee04d50e0c48aeb29acba9d875ac4a5eef1d8bc8269","ed78235cae7a8c11ba9fa9b12302df9e4c08a4dfd8bc83bfb1c7d20645a7ec60","7576414e724e13ca79d093900d6d18cdff26bc5af1d1f6061522cea8eb252b7a","00c573e5d0028752dd2f16fd6e833260107d0a1906034fd5f0289f5fa92ed6c4"],"outsideContradictions":"20","evidenceBoundary":"governed-static-declarations-and-structure-not-runtime"})
}
fn selection() -> Value {
    json!({"kind":"temple3.relnav-source-target-selection/v0.1","meaning":"personal-forgiveness-expression","participation":"optional-no-completion-obligation","authority":"record-owner-only","repeat":"reject-already-offered","withdrawal":"unavailable","consequence":"expression-only-no-judgment-release-or-data-erasure","disclosure":"participants-all-state","forgiveness":{"disposition":"refuse","scope":"origin-records-and-expression-history"},"retention":"retain-origin-and-all-expressions","amendment":"fixed-records-owners-text-process-and-policy","basis":"exact-state-identity","retry":"fresh-consent-after-stale-or-unknown-outcome","formation":"asserted-initial-snapshot-not-proof-of-source-truth-or-prior-consent","capacity":"signed-context-reservation-required-65536","custody":"selected-laboratory-writer-not-source-transfer","activation":"unavailable"})
}
pub fn assess(x: &Value) -> R<Value> {
    crate::canonical::canonical_json(x).map_err(|_| "invalid".to_string())?;
    crate::shape(
        x,
        "source formation selection expectedSourceId expectedFormationId",
    )
    .map_err(|_| "invalid".to_string())?;
    if x["source"] != expected_source() || x["selection"] != selection() {
        return fail();
    }
    let state = crate::voluntary_expression::formation(&x["formation"])?;
    let p = &state["policy"];
    let s = &x["selection"];
    for k in [
        "meaning",
        "participation",
        "authority",
        "repeat",
        "withdrawal",
        "consequence",
        "disclosure",
        "forgiveness",
        "retention",
        "amendment",
        "basis",
        "retry",
        "formation",
    ] {
        if p[k] != s[k] {
            return fail();
        }
    }
    let source_id = crate::canonical::identify("relnav-source-observation", &x["source"])
        .map_err(|_| "invalid".to_string())?;
    let formation_id =
        crate::canonical::identify("voluntary-expression-formation", &x["formation"])
            .map_err(|_| "invalid".to_string())?;
    if x["expectedSourceId"] != source_id || x["expectedFormationId"] != formation_id {
        return fail();
    }
    Ok(
        json!({"kind":"temple3.relnav-source-target-assessment/v0.1","sourceId":x["expectedSourceId"],"formationId":x["expectedFormationId"],"status":"candidate-target-with-static-source-and-runtime-gaps","preserved":[{"semantic":"personal-forgiveness-offer-declaration","source":"offerForgiveness command and semantic declaration","target":"personal-forgiveness-expression","qualification":"candidate-preservation-not-runtime-equivalence"},{"semantic":"offered-status-declaration","source":"generic emit template sets argument-1 status offered","target":"owner offer changes only selected Forgiveness status","qualification":"candidate-preservation-not-handler-equivalence"},{"semantic":"ownership-authority-declaration","source":"argument-1 command context with authority.own","target":"record-owner-only admission","qualification":"candidate-preservation-not-runtime-enforcement-equivalence"},{"semantic":"event-target-binding","source":"button event argument binds displayed record subject","target":"command binds recordId and exact full-state basis","qualification":"candidate-preservation-with-stronger-target-basis"}],"changed":[{"semantic":"repeat-policy","source":"repeat presentation observed; predecessor/runtime guard unverified","target":"reject-already-offered","reason":"explicit selected target policy, not attributed to source runtime"},{"semantic":"conflict-and-consent-basis","source":"refreshConfirm, needs-confirmation and zero retries declared","target":"exact-state identity with fresh consent after stale or unknown outcome","reason":"target enforcement is stricter and independently qualified"},{"semantic":"participation-and-completion","source":"narrow return-path assertion permits zero completion targets","target":"optional participation with no completion obligation","reason":"global non-coercion is not established by the source observation"},{"semantic":"delivery-and-custody","source":"parent projection, transport and writer/export authority unresolved","target":"participants-all-state under selected laboratory custody","reason":"target choice is not source delivery or authority equivalence"}],"added":[{"semantic":"typed-record-membership","target":"Forgiveness-only complete membership validation"},{"semantic":"signed-actor-admission","target":"distinct authenticated participant keys and exact consumed/created state"},{"semantic":"forgiveness-and-retention-policy","target":"visible refusal and retained origin/expression history"},{"semantic":"capacity-reservation","target":"formation recognition reserves every reachable signed context through 65536 bytes"},{"semantic":"uncertain-outcome-recovery","target":"retained request reconciliation before explicit resume or fresh consent"}],"removed":[],"removalAssessment":"no-authenticated-source-removal-claim","unsupported":["exact-rel-command-and-policy-execution","source-argument-coercion-malformed-and-unknown-target-outcomes","source-predecessor-repeat-and-idempotency-behavior","runtime-actor-identity-and-authority-enforcement","authentic-runtime-records-and-current-source-head","source-head-completeness-and-later-generation-drift","parent-projection-raw-delivery-and-route-enforcement","global-rendered-non-coercion-reminder-and-completion-behavior","source-withdrawal-succession-forgiveness-and-erasure-semantics","source-custody-export-and-writer-transfer","twenty-retained-polarity-contradictions","authentic-record-corpus-fit-and-partitioning"],"authority":{"source":"governed-historical-static-declarations-not-runtime-authority","target":"candidate-profile-not-operative-import","custody":s["custody"],"activation":"unavailable"},"readyForMigration":false,"migration":"not-started"}),
    )
}
