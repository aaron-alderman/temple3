//! Exact Relationship Navigator operational-authority v0.3 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"historicalVsOperative":"admission-observation-not-runtime-authority","semanticAdoption":"explicit-governed-choice-never-inferred","closedInspection":"refuse-runtime-surface-promotion","authoringCustody":"never-promote-to-application-record-custody","unknownRuntimeBytes":"refuse-not-zero","forgiveness":{"disposition":"refuse","scope":"relnav-runtime-surface-inspection"},"migration":"not-started"})
}
fn expected() -> Value {
    json!({
"source":{"application":"app.relnav","baselineHash":"a6d4f7e39faeaf24a758c60d95820f3e5da963095b33978f30d5361c8e915aec","artifactHash":"c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7","factCount":"40412","factCorpusHash":"1f67adfa26de7c81fa3c54a952372b976789535f4591be0cd0e5251a3338a16c","subjectCount":"8185","subjectInventoryHash":"63eeddeca9da07c70ebfff934f5e3b27d6b8536ffc91fb3ee21cc81d22e3c766","priorIntent":"unknown-inference-forbidden","authority":"observation-only-no-semantic-determination-or-product-mutation"},
"discovery":{"catalogHash":"d1e3d40384690bffc320d41ce827a5e824117e4267aeda724178edeedd1ccfc6","intentHash":"4111f157b2e92ca3c883844ed54810986ed595dcca978444e9855bf736ce2436","matchedClaims":"13","unresolvedClaims":"13","routeStatus":"clarify","observedEvidenceCount":"0","observedEvidenceHash":"37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570","declaredOperationalCapabilities":"0","externalAuthorityReconciliation":"gap","pluginSupplyHash":"b4db0a3e9708888dfec1ade92d320ad2885770df44cadff2d78ce8bb203fcc67"},
"subjects":[{"subjectId":"bundle.relnav.admission","factCount":"6","factBagHash":"a31a466d42c1598fcffd0e1076f7fa6899f52251241940e8cac5f17e4ab8a9a0"},{"subjectId":"bundle.relnav.appRoutes","factCount":"7","factBagHash":"eb515e138b490c9ab64f3e1788bf4c6acbd79c685a7fb891bc6291b2d90b60e2"},{"subjectId":"bundle.relnav.contextRuntime","factCount":"6","factBagHash":"d5bca17b32f1221243d2ddcd902f462cce04d4097a1f046e27ad684ac6513a4a"},{"subjectId":"bundle.relnav.mcp","factCount":"10","factBagHash":"a1ecf46a29d45b38addde2f8585b38e74bbe855953ec903084cc2efb47b31a42"},{"subjectId":"bundle.relnav.program","factCount":"11","factBagHash":"316cc1b921f26bb6b5dceb7a6645b59ddde6415ed88a27a857ab543aa02915ac"}],
"inspection":{"status":"closed-for-mapping-proposal","observation":"verified-baseline-only","mapping":"none","determination":"none","productMutation":"none","onlyOfferedContinuation":"sourcery_adoption/start","continuationTaken":false},
"journal":{"revision":"39","requestCount":"8","eventCount":"39","stateBytes":"110862","maximumStateBytes":"2097152","remainingStateBytes":"1986290","activeLeaseCount":"0","compactionAvailable":true,"compactionRecommended":false,"queuePolicyHash":"b267ef375dfe927a6487933860f89c727ae0b9392721f77ac101f1f9d98b6e85","autoClaim":false,"authority":"retained-authoring-request-custody-not-application-runtime-record-custody"},
"request":{"requestId":"request-1d3d-fb26-2ac2","initialIntentHash":"87024724b01fd2d381a8f717400d2513fe35ead7cfe8dc681769b1782c5db9b1","stoppedRevision":"3","stoppedHash":"3d3eb704bb6f24b1d503a55045b9d985cefd2170ef48500bf07f2421b48b3182","activeLeaseAfterStop":false},
"effects":{"adoption":false,"binding":false,"materialization":false,"mapping":false,"migration":false,"activation":false,"productChange":false}})
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind source discovery subjects inspection journal request effects runtimeCorpusBytes policy",
    )?;
    require(
        x["kind"] == "temple3.relnav-operational-authority-boundary/v0.3",
        "invalid-relnav-operational-authority-v03",
    )?;
    let e = expected();
    for k in [
        "source",
        "discovery",
        "subjects",
        "inspection",
        "journal",
        "request",
        "effects",
    ] {
        require(x[k] == e[k], "invalid-relnav-operational-authority-v03")?;
    }
    require(
        x["runtimeCorpusBytes"].is_null(),
        "invented-runtime-corpus-bytes",
    )?;
    require(
        x["policy"] == policy(),
        "invalid-relnav-operational-authority-policy-v03",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-operational-authority-assessment/v0.3","status":"refused-runtime-surface-inspection-without-adoption","sourceAuthority":"unavailable","semanticAdoption":"not-selected","runtimeCorpusBytes":"unavailable","selectedSubjectCount":x["subjects"].as_array().unwrap().len().to_string(),"inspection":x["inspection"],"authoringRequestCustody":{"status":"bounded-retained","bytes":x["journal"]["stateBytes"],"maximumBytes":x["journal"]["maximumStateBytes"],"authority":x["journal"]["authority"]},"effects":x["effects"],"readyForMigration":false,"migration":"not-started","refusal":{"code":"relnav-runtime-surface-inspection-requires-explicit-semantic-adoption","missing":["executed-authenticated-provider-route-journey","authoritative-runtime-record-head-and-completeness","runtime-record-export-or-capture-authority","actual-runtime-corpus-byte-size","operational-writer-and-record-custody","transport-and-delivery-receipts","replay-and-recovery","backout"]},"policy":x["policy"]}),
    )
}
