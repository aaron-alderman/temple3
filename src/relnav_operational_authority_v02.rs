//! Exact Relationship Navigator operative-authority boundary recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"historicalVsOperative":"admission-observation-not-runtime-authority","authoringCustody":"never-promote-to-application-record-custody","signingCustody":"never-promote-to-runtime-export-authority","unknownRuntimeBytes":"refuse-not-zero","chunking":"none-unless-explicitly-qualified","forgiveness":{"disposition":"refuse","scope":"relnav-operative-runtime-authority"},"migration":"not-started"})
}
fn expected() -> Value {
    json!({
"source":{"application":"app.relnav","baselineHash":"a6d4f7e39faeaf24a758c60d95820f3e5da963095b33978f30d5361c8e915aec","artifactHash":"c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7","factCount":"40412","factCorpusHash":"1f67adfa26de7c81fa3c54a952372b976789535f4591be0cd0e5251a3338a16c","subjectCount":"8185","subjectInventoryHash":"63eeddeca9da07c70ebfff934f5e3b27d6b8536ffc91fb3ee21cc81d22e3c766","priorIntent":"unknown-inference-forbidden","authority":"observation-only-no-semantic-determination-or-product-mutation"},
"discovery":{"catalogHash":"d1e3d40384690bffc320d41ce827a5e824117e4267aeda724178edeedd1ccfc6","intentHash":"21489dd986abed4486bfc35b42248d5ba6fb355e3b9e7edfe9e3ae54d9b1c101","matchedClaims":"24","unresolvedClaims":"23","observedEvidenceCount":"0","observedEvidenceHash":"37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570","declaredOperationalCapabilities":"0","externalAuthorityReconciliation":"gap"},
"binding":{"catalogHash":"644a4d3ac420931d351e8d08b116646338aa4efafc4dccdeb66ebd6b77ce2f6b","discoveryHash":"814d59a9a41b7ad89dd39db8d122d87d2288b1580d7d697c9ba8aa6a5d95e154","candidateCount":"1","offerHash":"e8c5ef79c8411f64fe0d189d5db05d7c36198cd2bc7c4966e06d2ac5ad4f1f6c","capability":"signing.after-commit","environment":"development","lifecycle":"observe"},
"journal":{"revision":"36","requestCount":"7","eventCount":"36","stateBytes":"103227","maximumStateBytes":"2097152","remainingStateBytes":"1993925","activeLeaseCount":"0","compactionAvailable":true,"compactionRecommended":false,"queuePolicyHash":"b267ef375dfe927a6487933860f89c727ae0b9392721f77ac101f1f9d98b6e85","autoClaim":false,"authority":"retained-authoring-request-custody-not-application-runtime-record-custody"},
"refusal":{"code":"sourcery-capability-binding-operation","category":"capability-or-realization-gap","status":"not-applied-or-effect-unproven","adjudicationHash":"81b0c65f4ae7b41ddf7cdad6e1a76670eafca6146816a5fbaaba878a0abd613e","productEffect":"not-claimed"},
"request":{"requestId":"request-1a9b-269f-5b5d","initialIntentHash":"91a5455df6e51946b3b1333069ac48f20aeca38985c147e76e9d235d1c4456e2","stoppedRevision":"5","stoppedHash":"981e6622969d2ea92bba6c43fcccdcd846a6334b130baf1f8d2edd41795df774","activeLeaseAfterStop":false},
"effects":{"adoption":false,"binding":false,"materialization":false,"mapping":false,"migration":false,"activation":false,"productChange":false}})
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind source discovery binding journal refusal request effects runtimeCorpusBytes policy",
    )?;
    require(
        x["kind"] == "temple3.relnav-operational-authority-boundary/v0.2",
        "invalid-relnav-operational-authority-v02",
    )?;
    let e = expected();
    for k in [
        "source",
        "discovery",
        "binding",
        "journal",
        "refusal",
        "request",
        "effects",
    ] {
        require(x[k] == e[k], "invalid-relnav-operational-authority-v02")?;
    }
    require(
        x["runtimeCorpusBytes"].is_null(),
        "invented-runtime-corpus-bytes",
    )?;
    require(
        x["policy"] == policy(),
        "invalid-relnav-operational-authority-policy-v02",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-operational-authority-assessment/v0.2","status":"refused-operative-runtime-authority-unavailable","sourceAuthority":"unavailable","runtimeCorpusBytes":"unavailable","authoringRequestCustody":{"status":"bounded-retained","bytes":x["journal"]["stateBytes"],"maximumBytes":x["journal"]["maximumStateBytes"],"authority":x["journal"]["authority"]},"effects":x["effects"],"readyForMigration":false,"migration":"not-started","refusal":{"code":"relnav-operative-runtime-authority-unavailable","missing":["executed-authenticated-provider-route-journey","authoritative-runtime-record-head-and-completeness","runtime-record-export-or-capture-authority","actual-runtime-corpus-byte-size","operational-writer-and-record-custody","transport-and-delivery-receipts","replay-and-recovery","backout"]},"policy":x["policy"]}),
    )
}
