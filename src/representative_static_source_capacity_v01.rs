//! Independent selected representative static-source capacity recognizer v0.1.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"kind":"temple3.representative-static-source-capacity-profile/v0.1","selectedApplications":"4","maximumFilesPerApplication":"40","maximumBytesPerApplication":"2000000","maximumAggregateFiles":"72","maximumAggregateBytes":"2000000","overflow":"refuse-whole-assessment-before-capacity-promotion","retention":"retain-exact-source-inventory-evidence-no-pruning","recovery":"replay-exact-assessment-input-no-rescan-no-tail-adoption","backout":"assessment-only-no-product-or-source-mutation","runtimeCorpus":"unavailable-refuse-runtime-capacity","forgiveness":{"disposition":"refuse","scope":"representative-authentic-static-source-capacity-v0.1"}})
}
fn expected() -> Value {
    json!([
     {"application":"app.inventoryConservationLedgerReferenceV3","evidencePath":"pilot/evidence/inventory-source-inventory-differential.json","evidenceKind":"temple3.inventory-source-inventory-differential-evidence/v0.1","evidenceSha256":"7427565bb2732d26842c6915581c8b386d3d5d1cb4a6d44b50deab1f91e44c34","sourceCount":"13","aggregateBytes":"189117","listingHash":"99fbb0ffe65929957a7aa9dbdbff488adccce698dabc18778fdc65798c3c05d1","staticSourceIdentity":true,"runtimeRecords":false,"exportOrCustodyAuthority":false,"readyForMigration":false},
     {"application":"app.claimApprovalReference20260831V2","evidencePath":"pilot/evidence/claim-source-inventory-differential.json","evidenceKind":"temple3.claim-source-inventory-differential-evidence/v0.1","evidenceSha256":"66b3d50b8e839e0efc07c5510ec4d1179182432718da43a55a9a503cb0bced17","sourceCount":"13","aggregateBytes":"161539","listingHash":"840144b8419a55562231147feccabcecfff1a4c892f89a11c9183e7da2cfd15b","staticSourceIdentity":true,"runtimeRecords":false,"exportOrCustodyAuthority":false,"readyForMigration":false},
     {"application":"app.actorScopedCaseAuthorityReference","evidencePath":"pilot/evidence/actor-source-inventory-differential.json","evidenceKind":"temple3.actor-source-inventory-differential-evidence/v0.1","evidenceSha256":"96428be93f8a3e509f3c5ea9380995df32293e7aa57103ea90ed4d512d08ec09","sourceCount":"13","aggregateBytes":"160770","listingHash":"dfe42d0b84d5e4fd7ed16815d87156c63c0e1ac00aea2b0fdaa07e112c48985d","staticSourceIdentity":true,"runtimeRecords":false,"exportOrCustodyAuthority":false,"readyForMigration":false},
     {"application":"app.relnav","evidencePath":"pilot/evidence/relnav-source-inventory-differential.json","evidenceKind":"temple3.relnav-source-inventory-differential-evidence/v0.1","evidenceSha256":"f48a59e38e8d43ce8c90320481ebaba173b58554d172cb8c540bca56c01e8ace","sourceCount":"33","aggregateBytes":"1483559","listingHash":"022fd97891876bb7eb00567c3d55a4403151b13507906967f49bc82f8c989049","staticSourceIdentity":true,"runtimeRecords":false,"exportOrCustodyAuthority":false,"readyForMigration":false}
    ])
}
fn integer(value: &Value) -> Result<u64> {
    let s = value.as_str().ok_or_else(|| temple3::Error {
        code: "non-canonical-capacity-integer".into(),
    })?;
    require(
        s == "0" || (!s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit())),
        "non-canonical-capacity-integer",
    )?;
    s.parse().map_err(|_| temple3::Error {
        code: "non-canonical-capacity-integer".into(),
    })
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(x, "kind applications policy")?;
    require(
        x["kind"] == "temple3.representative-static-source-capacity/v0.1"
            && x["policy"] == policy()
            && x["applications"].is_array(),
        "representative-static-source-capacity-refused",
    )?;
    let p = &x["policy"];
    let apps = x["applications"].as_array().unwrap();
    require(
        apps.len() as u64 == integer(&p["selectedApplications"])?,
        "representative-application-count-refused",
    )?;
    let mut files = 0u64;
    let mut bytes = 0u64;
    let mut rows = Vec::new();
    let expected = expected();
    let expected_apps = expected.as_array().unwrap();
    for (index, app) in apps.iter().enumerate() {
        shape(
            app,
            "application evidencePath evidenceKind evidenceSha256 sourceCount aggregateBytes listingHash candidateSourceCount candidateAggregateBytes staticSourceIdentity runtimeRecords exportOrCustodyAuthority readyForMigration",
        )?;
        let want = &expected_apps[index];
        for key in [
            "application",
            "evidencePath",
            "evidenceKind",
            "evidenceSha256",
            "sourceCount",
            "aggregateBytes",
            "listingHash",
            "staticSourceIdentity",
            "runtimeRecords",
            "exportOrCustodyAuthority",
            "readyForMigration",
        ] {
            require(
                app[key] == want[key],
                "substituted-representative-static-source-inventory",
            )?;
        }
        let observed_f = integer(&app["sourceCount"])?;
        let observed_b = integer(&app["aggregateBytes"])?;
        let f = integer(&app["candidateSourceCount"])?;
        let b = integer(&app["candidateAggregateBytes"])?;
        require(
            f >= observed_f && b >= observed_b,
            "representative-candidate-below-observed-inventory",
        )?;
        require(
            f <= integer(&p["maximumFilesPerApplication"])?,
            "representative-application-file-capacity",
        )?;
        require(
            b <= integer(&p["maximumBytesPerApplication"])?,
            "representative-application-byte-capacity",
        )?;
        files += f;
        bytes += b;
        rows.push(json!({"application":app["application"],"staticSourceFiles":app["candidateSourceCount"],"staticSourceBytes":app["candidateAggregateBytes"],"withinDeclaredStaticLimits":true,"runtimeCorpusBytes":"unavailable","runtimeCapacityQualified":false}));
    }
    let max_files = integer(&p["maximumAggregateFiles"])?;
    let max_bytes = integer(&p["maximumAggregateBytes"])?;
    require(files <= max_files, "representative-aggregate-file-capacity")?;
    require(bytes <= max_bytes, "representative-aggregate-byte-capacity")?;
    let missing = json!([
        "authoritative-runtime-record-head-and-completeness",
        "actual-runtime-corpus-bytes",
        "operational-writer-and-export-custody",
        "executed-authenticated-representative-journeys",
        "executed-replay-and-recovery",
        "executed-backout"
    ]);
    Ok(
        json!({"kind":"temple3.representative-static-source-capacity-assessment/v0.1","status":"qualified-exact-static-source-capacity-runtime-capacity-refused","assessmentId":crate::canonical::identify("representative-static-source-capacity-v0.1",x)?,"applications":rows,"totals":{"applications":apps.len().to_string(),"staticSourceFiles":files.to_string(),"staticSourceBytes":bytes.to_string(),"remainingAggregateByteHeadroom":(max_bytes-bytes).to_string()},"recovery":{"replay":"exact-input-deterministic","rescan":false,"tailAdoption":false},"backout":{"required":false,"reason":"assessment-only-no-mutation"},"readyForRepresentativeJourney":false,"readyForAF5":false,"readyForMigration":false,"productionAuthorized":false,"refusal":{"code":"representative-runtime-capacity-and-authority-unavailable","missing":missing},"policy":p}),
    )
}
