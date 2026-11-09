//! Independent representative static-source semantic capacity recognizer v0.2.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"kind":"temple3.representative-static-source-capacity-profile/v0.2","selectedApplications":"4","maximumFilesPerApplication":"40","maximumBytesPerApplication":"2000000","maximumAggregateFiles":"72","maximumAggregateBytes":"2000000","overflow":"refuse-whole-assessment-before-capacity-promotion","retention":"retain-exact-raw-source-evidence-separately-and-bind-versioned-semantic-identity","recovery":"replay-exact-semantic-assessment-input-no-rescan-no-tail-adoption","backout":"assessment-only-additive-v0.2-removal","runtimeCorpus":"unavailable-refuse-runtime-capacity","forgiveness":{"disposition":"refuse","scope":"representative-authentic-static-source-capacity-v0.2","volatileTopLevelFields":["observedAt"]}})
}
fn expected() -> Value {
    json!([
{"application":"app.inventoryConservationLedgerReferenceV3","evidencePath":"pilot/evidence/inventory-source-inventory-differential.json","evidenceKind":"temple3.inventory-source-inventory-differential-evidence/v0.1","evidenceSemanticId":"static-source-inventory-evidence-v0.1:sha256:a484f249674e66c12f88eedc903ece59474c1bf2b6e7a477c29c58a32035f0a1","sourceCount":"13","aggregateBytes":"189117","listingHash":"99fbb0ffe65929957a7aa9dbdbff488adccce698dabc18778fdc65798c3c05d1","staticSourceIdentity":true,"runtimeRecords":false,"exportOrCustodyAuthority":false,"readyForMigration":false},
{"application":"app.claimApprovalReference20260831V2","evidencePath":"pilot/evidence/claim-source-inventory-differential.json","evidenceKind":"temple3.claim-source-inventory-differential-evidence/v0.1","evidenceSemanticId":"static-source-inventory-evidence-v0.1:sha256:46ec24e36e141df774e314c3fbee0802696bfa40eddf488f08d13061f7d7ccbe","sourceCount":"13","aggregateBytes":"161539","listingHash":"840144b8419a55562231147feccabcecfff1a4c892f89a11c9183e7da2cfd15b","staticSourceIdentity":true,"runtimeRecords":false,"exportOrCustodyAuthority":false,"readyForMigration":false},
{"application":"app.actorScopedCaseAuthorityReference","evidencePath":"pilot/evidence/actor-source-inventory-differential.json","evidenceKind":"temple3.actor-source-inventory-differential-evidence/v0.1","evidenceSemanticId":"static-source-inventory-evidence-v0.1:sha256:a2dac24f43aef8d98d985024cd4c6e9af1d8212475849a472cd5de5e36c94fab","sourceCount":"13","aggregateBytes":"160770","listingHash":"dfe42d0b84d5e4fd7ed16815d87156c63c0e1ac00aea2b0fdaa07e112c48985d","staticSourceIdentity":true,"runtimeRecords":false,"exportOrCustodyAuthority":false,"readyForMigration":false},
{"application":"app.relnav","evidencePath":"pilot/evidence/relnav-source-inventory-differential.json","evidenceKind":"temple3.relnav-source-inventory-differential-evidence/v0.1","evidenceSemanticId":"static-source-inventory-evidence-v0.1:sha256:faa8567207713e9e45e0987f91db70cce951e8b0a25e18dd1a3f411ff4ea9a2e","sourceCount":"33","aggregateBytes":"1483559","listingHash":"022fd97891876bb7eb00567c3d55a4403151b13507906967f49bc82f8c989049","staticSourceIdentity":true,"runtimeRecords":false,"exportOrCustodyAuthority":false,"readyForMigration":false}])
}
fn integer(v: &Value) -> Result<u64> {
    let s = v.as_str().ok_or_else(|| temple3::Error {
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
        x["kind"] == "temple3.representative-static-source-capacity/v0.2"
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
    let wants = expected();
    let wants = wants.as_array().unwrap();
    let mut files = 0;
    let mut bytes = 0;
    let mut rows = Vec::new();
    for (i, a) in apps.iter().enumerate() {
        shape(
            a,
            "application evidencePath evidenceKind evidenceSemanticId sourceCount aggregateBytes listingHash candidateSourceCount candidateAggregateBytes staticSourceIdentity runtimeRecords exportOrCustodyAuthority readyForMigration",
        )?;
        for k in [
            "application",
            "evidencePath",
            "evidenceKind",
            "evidenceSemanticId",
            "sourceCount",
            "aggregateBytes",
            "listingHash",
            "staticSourceIdentity",
            "runtimeRecords",
            "exportOrCustodyAuthority",
            "readyForMigration",
        ] {
            require(
                a[k] == wants[i][k],
                "substituted-representative-static-source-inventory",
            )?;
        }
        let on = integer(&a["sourceCount"])?;
        let ob = integer(&a["aggregateBytes"])?;
        let n = integer(&a["candidateSourceCount"])?;
        let b = integer(&a["candidateAggregateBytes"])?;
        require(
            n >= on && b >= ob,
            "representative-candidate-below-observed-inventory",
        )?;
        require(
            n <= integer(&p["maximumFilesPerApplication"])?
                && b <= integer(&p["maximumBytesPerApplication"])?,
            "representative-application-capacity",
        )?;
        files += n;
        bytes += b;
        rows.push(json!({"application":a["application"],"staticSourceFiles":a["candidateSourceCount"],"staticSourceBytes":a["candidateAggregateBytes"],"withinDeclaredStaticLimits":true,"runtimeCorpusBytes":"unavailable","runtimeCapacityQualified":false}));
    }
    let mf = integer(&p["maximumAggregateFiles"])?;
    let mb = integer(&p["maximumAggregateBytes"])?;
    require(
        files <= mf && bytes <= mb,
        "representative-aggregate-capacity",
    )?;
    let missing = json!([
        "authoritative-runtime-record-head-and-completeness",
        "actual-runtime-corpus-bytes",
        "operational-writer-and-export-custody",
        "executed-authenticated-representative-journeys",
        "executed-replay-and-recovery",
        "executed-backout"
    ]);
    Ok(
        json!({"kind":"temple3.representative-static-source-capacity-assessment/v0.2","status":"qualified-exact-static-source-capacity-runtime-capacity-refused","assessmentId":crate::canonical::identify("representative-static-source-capacity-v0.2",x)?,"applications":rows,"totals":{"applications":apps.len().to_string(),"staticSourceFiles":files.to_string(),"staticSourceBytes":bytes.to_string(),"remainingAggregateByteHeadroom":(mb-bytes).to_string()},"recovery":{"replay":"exact-semantic-input-deterministic","rescan":false,"tailAdoption":false},"backout":{"required":false,"reason":"assessment-only-additive-v0.2-removal"},"readyForRepresentativeJourney":false,"readyForAF5":false,"readyForMigration":false,"productionAuthorized":false,"refusal":{"code":"representative-runtime-capacity-and-authority-unavailable","missing":missing},"policy":p}),
    )
}
