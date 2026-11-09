//! Exact representative source-authority closure audit.
use crate::{Result, require, shape};
use serde_json::{Value, json};

fn policy() -> Value {
    json!({"kind":"temple3.representative-authority-closure-policy/v0.1","admissionEvidence":"historical-definition-and-intent-not-runtime-authority","missingRuntimeAuthority":"refuse-migration","signingCustody":"never-promote-to-record-custody","observationCustody":"never-promote-to-source-authority","activation":"unavailable","migration":"not-started"})
}
fn expected(name: &str) -> Option<Value> {
    match name {
        "app.inventoryConservationLedgerReferenceV3" => Some(json!([
            "8234a80e4c1a5744a25cf61255cc66107a7ffe878be89806d5248570f8844731",
            "d1e3d40384690bffc320d41ce827a5e824117e4267aeda724178edeedd1ccfc6",
            "37b109fc1e9a10d9fcdab68fa9cab60bb9e803809a515fcf4a493b125f1e41bd",
            "39074c2eab7ac18025d80136780868373b447bfb15437f52c002745bb13e25aa",
            [],
            "selected-target-laboratory-not-source-transfer"
        ])),
        "app.claimApprovalReference20260831V2" => Some(json!([
            "f15382f7de81b4c4fb8f25fc65c366516b4d62907d926c05f668802b262db2a5",
            "644a4d3ac420931d351e8d08b116646338aa4efafc4dccdeb66ebd6b77ce2f6b",
            "814d59a9a41b7ad89dd39db8d122d87d2288b1580d7d697c9ba8aa6a5d95e154",
            "855c24eeeed1e30806a06699a63955bb67a4020298c8a6fe0e28b966a392086c",
            ["development-after-commit-signing-custody"],
            "selected-target-laboratory-not-source-transfer"
        ])),
        "app.actorScopedCaseAuthorityReference" => Some(json!([
            "ac0984ec7c5c9b22ab86d8fc2f24dfd9307f9b8122098e87a6ddb87c75a9ab7d",
            "644a4d3ac420931d351e8d08b116646338aa4efafc4dccdeb66ebd6b77ce2f6b",
            "814d59a9a41b7ad89dd39db8d122d87d2288b1580d7d697c9ba8aa6a5d95e154",
            "d91b961a02eeee51b591b59522bc2b7f66e4c67a0f34f3c843eadf09634f948c",
            ["development-after-commit-signing-custody"],
            "selected-target-laboratory-not-source-transfer"
        ])),
        "app.relnav" => Some(json!([
            "c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7",
            "d1e3d40384690bffc320d41ce827a5e824117e4267aeda724178edeedd1ccfc6",
            "c4aceb49accd19249c5e712ab4897c4b89bcbf0cfc7e14fd7579c54d1365adc5",
            "b154af08477d4f42b960c284d67bed74ce3ddcddfaf35472273d496b1cb45e94",
            [],
            "qualified-observation-custody-not-source-authority"
        ])),
        _ => None,
    }
}
pub fn audit(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(x, "kind policy applications")?;
    require(
        x["kind"] == "temple3.representative-authority-closure/v0.1" && x["policy"] == policy(),
        "invalid-representative-authority-closure",
    )?;
    let apps = x["applications"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-representative-authority-closure"))?;
    require(apps.len() == 4, "invalid-representative-authority-closure")?;
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for a in apps {
        shape(
            a,
            "application artifactHash admission operational custody sourceRuntime activation migration",
        )?;
        let n = a["application"]
            .as_str()
            .ok_or_else(|| crate::err("invalid-representative-authority-closure"))?;
        require(
            !seen.contains(&n),
            "invalid-representative-authority-closure",
        )?;
        seen.push(n);
        let e =
            expected(n).ok_or_else(|| crate::err("invalid-representative-authority-closure"))?;
        shape(
            &a["admission"],
            "authorityClaims stewardshipManifest evidenceClass",
        )?;
        shape(
            &a["operational"],
            "catalogHash discoveryHash declaredRequirements observedEvidence offers requestStatus requestHash",
        )?;
        require(
            a["admission"]
                == json!({"authorityClaims":[],"stewardshipManifest":null,"evidenceClass":"historical-definition-and-intent"}),
            "invalid-representative-authority-closure",
        )?;
        require(
            a["operational"]
                == json!({"catalogHash":e[1],"discoveryHash":e[2],"declaredRequirements":[],"observedEvidence":[],"offers":e[4],"requestStatus":"stopped","requestHash":e[3]}),
            "invalid-representative-authority-closure",
        )?;
        require(
            a["artifactHash"] == e[0]
                && a["custody"] == e[5]
                && a["sourceRuntime"] == "unavailable"
                && a["activation"] == "unavailable"
                && a["migration"] == "not-started",
            "invalid-representative-authority-closure",
        )?;
        out.push(json!({"application":n,"artifactHash":a["artifactHash"],"operativeAuthority":"unavailable","custody":a["custody"],"migration":"not-started"}));
    }
    Ok(
        json!({"kind":"temple3.representative-authority-closure-audit/v0.1","status":"refused-missing-operative-source-authority","readyForMigration":false,"applications":out,"refusal":{"code":"representative-authority-incomplete","missing":["source-runtime-head-and-completeness","source-writer-and-custody","runtime-export-capture-and-transport","receipts-replay-recovery-and-backout"]},"policy":x["policy"]}),
    )
}

fn policy_v02() -> Value {
    json!({"kind":"temple3.representative-authority-closure-policy/v0.2","operativeEvidence":"exact-retained-governed-witnesses","historicalEvidence":"definition-and-intent-not-operative-authority","missingRuntimeAuthority":"refuse-migration","signingCustody":"never-promote-to-record-custody","observationCustody":"never-promote-to-source-authority","capacity":"explicit-fit-required-no-implicit-chunking-or-truncation","forgiveness":{"disposition":"refuse","scope":"representative-operative-authority-and-capacity"},"activation":"unavailable","migration":"not-started"})
}
fn absent() -> Value {
    json!({"runtimeHead":"unavailable","exportCapture":"unavailable","writerCustody":"unavailable","transport":"unavailable","receipts":"unavailable","replay":"unavailable","recovery":"unavailable","backout":"unavailable"})
}
fn no_effects() -> Value {
    json!({"adoption":false,"binding":false,"materialization":false,"mapping":false,"migration":false,"activation":false,"productChange":false})
}
fn unmeasured() -> Value {
    json!({"evidenceHash":null,"sourceStaticBytes":null,"maxSealBytes":null,"maxRecordBytes":null,"sourceStaticFit":"unmeasured","runtimeCorpusBytes":null,"runtimeCorpusFit":"unmeasured"})
}
fn expected_v02(name: &str) -> Option<Value> {
    match name {
        "app.inventoryConservationLedgerReferenceV3" => Some(
            json!({"artifactHash":"8234a80e4c1a5744a25cf61255cc66107a7ffe878be89806d5248570f8844731","witnessHash":"38c61e70fcbc4cac37284a763baa18da66e2b56bafb560a8d6425ce49788f8d2","request":{"headHash":"39074c2eab7ac18025d80136780868373b447bfb15437f52c002745bb13e25aa","revision":"4","activeLease":false},"operational":{"catalogHash":"d1e3d40384690bffc320d41ce827a5e824117e4267aeda724178edeedd1ccfc6","discoveryHash":"37b109fc1e9a10d9fcdab68fa9cab60bb9e803809a515fcf4a493b125f1e41bd","observedEvidenceCount":"0","offer":null},"custody":"selected-target-laboratory-not-source-transfer","capacity":unmeasured()}),
        ),
        "app.claimApprovalReference20260831V2" => Some(
            json!({"artifactHash":"f15382f7de81b4c4fb8f25fc65c366516b4d62907d926c05f668802b262db2a5","witnessHash":"96cc54de374ed557b2a4f144c9f5f9049e392df5303592415d3f146ce0053e07","request":{"headHash":"855c24eeeed1e30806a06699a63955bb67a4020298c8a6fe0e28b966a392086c","revision":"4","activeLease":false},"operational":{"catalogHash":"644a4d3ac420931d351e8d08b116646338aa4efafc4dccdeb66ebd6b77ce2f6b","discoveryHash":"814d59a9a41b7ad89dd39db8d122d87d2288b1580d7d697c9ba8aa6a5d95e154","observedEvidenceCount":"0","offer":"development-after-commit-signing-custody"},"custody":"selected-target-laboratory-not-source-transfer","capacity":unmeasured()}),
        ),
        "app.actorScopedCaseAuthorityReference" => Some(
            json!({"artifactHash":"ac0984ec7c5c9b22ab86d8fc2f24dfd9307f9b8122098e87a6ddb87c75a9ab7d","witnessHash":"7e071d604924e3ba1c7db8bba5faa29e0b003cd319e1f16be5f25da5b99f9d6d","request":{"headHash":"d91b961a02eeee51b591b59522bc2b7f66e4c67a0f34f3c843eadf09634f948c","revision":"7","activeLease":false},"operational":{"catalogHash":"644a4d3ac420931d351e8d08b116646338aa4efafc4dccdeb66ebd6b77ce2f6b","discoveryHash":"814d59a9a41b7ad89dd39db8d122d87d2288b1580d7d697c9ba8aa6a5d95e154","observedEvidenceCount":"0","offer":"development-after-commit-signing-custody"},"custody":"selected-target-laboratory-not-source-transfer","capacity":unmeasured()}),
        ),
        "app.relnav" => Some(
            json!({"artifactHash":"c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7","witnessHash":"a9d33cc874662ffa8e5b4cc375cc152fea8c499599d8454a30ba10d61c68f0ab","request":{"headHash":"4bd073422affec65c1ec1a4330d41d55a6d5db7a9d4dd348a6bc21357c8e208a","revision":"6","activeLease":false},"operational":{"catalogHash":"644a4d3ac420931d351e8d08b116646338aa4efafc4dccdeb66ebd6b77ce2f6b","discoveryHash":"814d59a9a41b7ad89dd39db8d122d87d2288b1580d7d697c9ba8aa6a5d95e154","observedEvidenceCount":"0","offer":"development-after-commit-signing-custody"},"custody":"qualified-observation-custody-not-source-authority","capacity":{"evidenceHash":"c20421e12851bfedd61b95354c12cdbc7154d34e05629b4181bfb75fff81014a","sourceStaticBytes":"1483559","maxSealBytes":"65536","maxRecordBytes":"196608","sourceStaticFit":"refused-oversize","runtimeCorpusBytes":null,"runtimeCorpusFit":"unmeasured"}}),
        ),
        _ => None,
    }
}
pub fn audit_v02(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(x, "kind policy applications")?;
    require(
        x["kind"] == "temple3.representative-authority-closure/v0.2" && x["policy"] == policy_v02(),
        "invalid-representative-authority-closure-v02",
    )?;
    let apps = x["applications"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-representative-authority-closure-v02"))?;
    require(
        apps.len() == 4,
        "invalid-representative-authority-closure-v02",
    )?;
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for a in apps {
        shape(
            a,
            "application artifactHash witnessHash request operational authority effects custody capacity sourceRuntime activation migration",
        )?;
        let n = a["application"]
            .as_str()
            .ok_or_else(|| crate::err("invalid-representative-authority-closure-v02"))?;
        require(
            !seen.contains(&n),
            "invalid-representative-authority-closure-v02",
        )?;
        seen.push(n);
        let e = expected_v02(n)
            .ok_or_else(|| crate::err("invalid-representative-authority-closure-v02"))?;
        require(
            a["artifactHash"] == e["artifactHash"]
                && a["witnessHash"] == e["witnessHash"]
                && a["request"] == e["request"]
                && a["operational"] == e["operational"]
                && a["authority"] == absent()
                && a["effects"] == no_effects()
                && a["custody"] == e["custody"]
                && a["capacity"] == e["capacity"]
                && a["sourceRuntime"] == "unavailable"
                && a["activation"] == "unavailable"
                && a["migration"] == "not-started",
            "invalid-representative-authority-closure-v02",
        )?;
        out.push(json!({"application":n,"witnessHash":a["witnessHash"],"requestHead":a["request"]["headHash"],"operativeAuthority":"unavailable","capacity":a["capacity"]["runtimeCorpusFit"],"custody":a["custody"],"migration":"not-started"}));
    }
    Ok(
        json!({"kind":"temple3.representative-authority-closure-audit/v0.2","status":"refused-operative-authority-and-capacity-incomplete","readyForMigration":false,"applications":out,"refusal":{"code":"representative-operative-authority-and-capacity-incomplete","missing":["authenticated-source-runtime-head-and-completeness","runtime-record-export-and-capture-authority","source-writer-and-record-custody","bounded-whole-corpus-or-explicit-chunked-capacity","transport-and-delivery-receipts","replay-recovery-and-backout"]},"policy":x["policy"]}),
    )
}
