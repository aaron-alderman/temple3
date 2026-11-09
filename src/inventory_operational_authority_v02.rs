//! Exact Inventory operational-authority v0.2 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};

fn policy() -> Value {
    json!({"historicalVsOperative":"admitted-definition-and-intent-not-runtime-authority","observedZero":"no-evidence-observed-not-proof-of-nonexistence","signingCustody":"never-promote-to-runtime-record-export-or-custody","changeAuthoring":"explicit-governed-choice-never-inferred","authoringCustody":"never-promote-to-application-runtime-record-custody","unknownRuntimeBytes":"refuse-not-zero","forgiveness":{"disposition":"refuse","scope":"inventory-runtime-authority"},"migration":"not-started"})
}

fn expected() -> Value {
    json!({
        "source":{"application":"app.inventoryConservationLedgerReferenceV3","artifactHash":"8234a80e4c1a5744a25cf61255cc66107a7ffe878be89806d5248570f8844731","factCount":"28400","buildReceiptHash":"4dbea0eba768e7f84183e8bd30575fa2eef9b0e0b011b7d8bc749729a763814e","productLoweringReceiptHash":"a97ad9cdd6ab3e17277bd2cb879f91ca84dbbe207ae44dc387caa28d788700f8","authoringSnapshotHash":"d7e46251b82814c237db2283293b4cc8d44eec5235e751525314ced7080d3588","authoringIntentHash":"c106b051e6b48aa12963c800fb7baee04a6c57bafa3875830ed95cf71cfb8fd2","stewardshipManifestHash":null,"lineageHash":"568badc6619454bab437cf6f81df75f3a442163574f55560963a9f1b931c4f35","parents":[],"productionKind":"change","authority":"admitted-definition-and-intent-not-runtime-record-authority"},
        "discovery":{"catalogHash":"d1e3d40384690bffc320d41ce827a5e824117e4267aeda724178edeedd1ccfc6","claimCount":"9","unresolvedClaimCount":"7","observedEvidenceCount":"0","declaredOperationalCapabilities":"0","pluginSupplyHash":"67c55979bbe6f265912dc7f1011f7854fe3d1621d73bdf63b984a393ab131335","route":"clarify","onlyOfferedContinuation":"sourcery_change_start","continuationTaken":false,"projectionSourceHash":"9699597579acea202a22a90c448c84659fc9cb242a26c361c5bb5bf3aaa481b8","authority":"gap"},
        "capability":{"catalogHash":"644a4d3ac420931d351e8d08b116646338aa4efafc4dccdeb66ebd6b77ce2f6b","discoveryHash":"814d59a9a41b7ad89dd39db8d122d87d2288b1580d7d697c9ba8aa6a5d95e154","presentationHash":"50828c2dd3ce1b14bbd1e8d57bae41ece7324e281d06080ee02142ffb76251fe","totalCandidates":"1","truncated":false,"offerHash":"e8c5ef79c8411f64fe0d189d5db05d7c36198cd2bc7c4966e06d2ac5ad4f1f6c","offerKey":"offer.windows-export-custody.v1","provider":"provider.windows-export-custody","capability":"signing.after-commit","environments":["development"],"authority":"development-signing-custody-not-runtime-record-export-or-custody"},
        "request":{"requestId":"request-e4a1-7447-145c","initialIntentHash":"071d89a1ee9cf5a8dc23a95f7ce4bf18ff3b2591371cdba0f767365c0c9cae4d","initialIntentBytes":"502","initialRevision":"1","initialHeadHash":"3f4f19e31976d5b5bdb22c2aa7a2cf873508d118e756faafc77ff53af85e019f","discoveryRevision":"3","discoveryHeadHash":"dfdf4ee4c05dfe68ed9741dd0f3d9b562d5bfb95156142d23688ee854be84700","stoppedRevision":"4","stoppedHeadHash":"66410285090648b08a2a3ec5b9bb220f3d92c45576fae1e32a9b82e919ff7873","activeLeaseAfterStop":false,"status":"stopped"},
        "journal":{"revision":"17","requestCount":"4","eventCount":"17","stateBytes":"46558","maximumStateBytes":"2097152","remainingStateBytes":"2050594","minimumCompactionEvents":"16","activeLeaseCount":"0","compactionAvailable":true,"compactionRecommended":false,"queuePolicyHash":"b267ef375dfe927a6487933860f89c727ae0b9392721f77ac101f1f9d98b6e85","autoClaim":false,"authority":"bounded-retained-authoring-request-custody-not-application-runtime-record-custody"},
        "effects":{"adoption":false,"binding":false,"materialization":false,"mapping":false,"migration":false,"activation":false,"productChange":false}
    })
}

pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind source discovery capability request journal effects runtimeCorpusBytes policy",
    )?;
    require(
        x["kind"] == "temple3.inventory-operational-authority-boundary/v0.2",
        "invalid-inventory-operational-authority-v02",
    )?;
    let e = expected();
    for k in [
        "source",
        "discovery",
        "capability",
        "request",
        "journal",
        "effects",
    ] {
        require(x[k] == e[k], "invalid-inventory-operational-authority-v02")?;
    }
    require(
        x["runtimeCorpusBytes"].is_null(),
        "invented-runtime-corpus-bytes",
    )?;
    require(
        x["policy"] == policy(),
        "invalid-inventory-operational-authority-policy-v02",
    )?;
    Ok(
        json!({"kind":"temple3.inventory-operational-authority-assessment/v0.2","status":"refused-inventory-runtime-authority-unavailable","sourceAuthority":"unavailable","observedEvidence":"none-observed-not-nonexistence","runtimeCorpusBytes":"unavailable","capabilityDisposition":"development-signing-custody-is-not-runtime-record-authority","authoringRequestCustody":{"status":"bounded-retained","bytes":x["journal"]["stateBytes"],"maximumBytes":x["journal"]["maximumStateBytes"],"authority":x["journal"]["authority"]},"effects":x["effects"],"readyForMigration":false,"migration":"not-started","refusal":{"code":"inventory-runtime-authority-unavailable","missing":["authoritative-runtime-record-head-and-completeness","runtime-record-export-or-capture-authority","actual-runtime-corpus-byte-size","operational-writer-and-record-custody","transport-and-delivery-receipts","replay-and-recovery","backout"]},"policy":x["policy"]}),
    )
}
