//! Exact authentic Relationship Navigator corpus assessment; no source authority.
use crate::{Result, require, shape};
use serde_json::{Value, json};

pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(x, "kind source admission observation profile policy")?;
    require(
        x["kind"] == "temple3.relnav-authentic-corpus-capacity/v0.1",
        "invalid-relnav-authentic-corpus-capacity",
    )?;
    shape(
        &x["source"],
        "application artifactHash sourceCount aggregateBytes listingHash",
    )?;
    require(
        x["source"]
            == json!({"application":"app.relnav","artifactHash":"c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7","sourceCount":"33","aggregateBytes":"1483559","listingHash":"022fd97891876bb7eb00567c3d55a4403151b13507906967f49bc82f8c989049"}),
        "invalid-relnav-authentic-corpus-capacity",
    )?;
    shape(
        &x["admission"],
        "baselineHash factCount factCorpusHash subjectCount subjectInventoryHash runtimeCorpusBytes exclusions",
    )?;
    require(
        x["admission"]["baselineHash"]
            == "a6d4f7e39faeaf24a758c60d95820f3e5da963095b33978f30d5361c8e915aec"
            && x["admission"]["factCount"] == "40412"
            && x["admission"]["factCorpusHash"]
                == "1f67adfa26de7c81fa3c54a952372b976789535f4591be0cd0e5251a3338a16c"
            && x["admission"]["subjectCount"] == "8185"
            && x["admission"]["subjectInventoryHash"]
                == "63eeddeca9da07c70ebfff934f5e3b27d6b8536ffc91fb3ee21cc81d22e3c766",
        "invalid-relnav-authentic-corpus-capacity",
    )?;
    require(
        x["admission"]["runtimeCorpusBytes"].is_null()
            && x["admission"]["exclusions"]
                == json!([
                    "historical-intent",
                    "runtime-state",
                    "implicit-product-authority"
                ]),
        "authentic-runtime-corpus-bytes-unavailable",
    )?;
    shape(
        &x["observation"],
        "projectionSha256 rawBytes encodedBytes sourceAuthority runtime activation",
    )?;
    require(
        x["observation"]
            == json!({"projectionSha256":"b69af8f4caa2958e6ecceb87711e8eace2da331ceed64435da0aa4962da59df1","rawBytes":"8378","encodedBytes":"17219","sourceAuthority":"not-verified","runtime":"unavailable","activation":"unavailable"}),
        "invalid-relnav-authentic-corpus-capacity",
    )?;
    let validation = json!({"op":"reserve","profile":x["profile"],"state":{"eventCount":"0","acknowledgedCount":"0"}});
    crate::runtime_export_capacity::evaluate(&validation)
        .map_err(|_| crate::err("wrong-selected-runtime-export-profile"))?;
    require(
        x["profile"]
            == json!({"kind":"temple3.runtime-export-capacity-profile/v0.1","maxEvents":"64","maxSealBytes":"65536","maxEventBytes":"65536","maxReceiptBytes":"32768","maxExportRecordBytes":"196608","maxAckRecordBytes":"4096","maxJournalBytes":"12845056","maxPendingBytes":"12582912","maxDeliveryRecords":"8","maxDeliveryBytes":"1572864","overflow":"refuse-before-source-command","retention":"retain-all-no-pruning","forgiveness":{"disposition":"refuse","scope":"runtime-export-capacity"}}),
        "wrong-selected-runtime-export-profile",
    )?;
    require(
        x["policy"]
            == json!({"representation":"whole-corpus-single-seal","chunking":"unsupported-refuse","truncation":"forbidden","missingRuntimeBytes":"refuse-not-zero","forgiveness":{"disposition":"refuse","scope":"authentic-corpus-capacity"}}),
        "invalid-authentic-corpus-capacity-policy",
    )?;
    let source_bytes = 1_483_559u64;
    let seal_limit = 65_536u64;
    let record_limit = 196_608u64;
    Ok(
        json!({"kind":"temple3.relnav-authentic-corpus-capacity-assessment/v0.1","status":"refused-authentic-runtime-corpus-fit","source":{"classification":"authentic-static-source-inventory-not-runtime-record-corpus","bytes":"1483559","singleSealFit":false,"singleRecordFit":false,"sealOverflowBytes":(source_bytes-seal_limit).to_string(),"recordOverflowBytes":(source_bytes-record_limit).to_string(),"refusal":"authentic-source-corpus-exceeds-selected-envelope"},"observation":{"classification":"governed-observation-custody-only","rawBytes":"8378","encodedBytes":"17219","selectedEnvelopeFit":true,"authority":"not-runtime-export-or-source-authority"},"runtimeCorpus":{"bytes":"unavailable","fit":"unmeasured","refusal":"authentic-runtime-corpus-bytes-unavailable"},"policy":x["policy"],"authority":{"source":"verified-static-inventory-only","producer":"unavailable","transport":"unavailable","activation":"unavailable"},"readyForProducer":false,"readyForMigration":false,"migration":"not-started","refusal":{"code":"relnav-authentic-runtime-corpus-fit-unproven","missing":["runtime-corpus-bytes","runtime-record-export-authority","source-head-completeness","operational-producer-custody","transport-authority"]}}),
    )
}
