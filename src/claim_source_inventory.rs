//! Authentic Claim Approval static-source inventory boundary.
use crate::{Result, require, shape};
use serde_json::{Value, json};

fn policy() -> Value {
    json!({"kind":"temple3.claim-source-inventory-policy/v0.1","staticSource":"historical-structural-evidence","unboundFile":"never-promote-to-declared-source","handlerExecution":"not-established","runtimeRecords":"not-established","exportOrCustodyAuthority":"not-established","unsupportedInspection":"visible-refusal","migration":"not-started"})
}
fn sources() -> Value {
    json!([
        [
            "README.md",
            "documentation",
            "958",
            "71b0aac9cb729041797ed8847fc8a4fc0e21b868f6bc41e9d9ae01e01732da89",
            "observed-unbound-application-file"
        ],
        [
            "app.log.rel",
            "rel",
            "0",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "observed-unbound-application-file"
        ],
        [
            "app.rel",
            "rel",
            "8858",
            "8cb7d09a33e759ae555aa03ce843d4eeecebdd1850435dff91c06280cac1a19c",
            "declared-application-source"
        ],
        [
            "definition.rel",
            "rel",
            "1580",
            "3651e3729294f326a3c06e04f95698757da2237698dbc9b2eadebeecb1e14896",
            "declared-application-source"
        ],
        [
            "plugin.rel",
            "rel",
            "8633",
            "0bf91ca94c69295cc8fac1ca45d0e70b97b279379ca2f20e30a0e11f22e54cbc",
            "declared-application-source"
        ],
        [
            "product.rel",
            "rel",
            "60203",
            "7886cb2858f930efd32d836a1809dd5b53d805abec652a3cf73556681696108f",
            "declared-application-source"
        ],
        [
            "project.rel",
            "rel",
            "992",
            "f87a8c50180bd22cd5d49277bc7b9d8955f95760e0009f2ebae49c7044118e74",
            "declared-application-source"
        ],
        [
            "routes.rel",
            "rel",
            "2292",
            "11daa9449e03306bd36c1640ec1ed478e9ff9c624f97a97a5618c3ab38a3dc5d",
            "declared-application-source"
        ],
        [
            "semantic-pack-supply.rel",
            "rel",
            "90",
            "6e03d5d43491b1e9b9cca217cdb7d3c8a04eb3553788183c876b77f07eedca36",
            "declared-application-source"
        ],
        [
            "source.semantic.sourcery.mjs",
            "javascript",
            "1521",
            "6f941104204fde3a33b41b6e78327f051fcb682d14e255c4511d4990790b4f8d",
            "observed-unbound-application-file"
        ],
        [
            "ui-library-adoption.plugin.rel",
            "rel",
            "2487",
            "cc315a1a60172a44d23ebb2078f6138d026ce23cc2e91a3b87191d0744656c47",
            "observed-unbound-application-file"
        ],
        [
            "ui-library-adoption.rel",
            "rel",
            "31579",
            "9c274e1a863630765a57be15941b0caee1e361fd75c2a14f87e3e7ffde5d0c08",
            "observed-unbound-application-file"
        ],
        [
            "ui.rel",
            "rel",
            "42346",
            "70138590e8ea45aeba5eefba3877279cd9e289b887ddd812d9dc253b59a7df01",
            "declared-application-source"
        ]
    ])
}
pub fn audit(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind application artifactHash inventory javascriptInspection relInspection request countReconciliation policy",
    )?;
    require(
        x["kind"] == "temple3.claim-source-inventory/v0.1"
            && x["application"] == "app.claimApprovalReference20260831V2"
            && x["artifactHash"]
                == "f15382f7de81b4c4fb8f25fc65c366516b4d62907d926c05f668802b262db2a5"
            && x["policy"] == policy(),
        "invalid-claim-source-inventory",
    )?;
    let i = &x["inventory"];
    shape(i, "protocol sourceCount aggregateBytes listingHash sources")?;
    require(
        i["protocol"] == "sourcery-imported-surface-source-inventory/1"
            && i["sourceCount"] == "13"
            && i["aggregateBytes"] == "161539"
            && i["listingHash"]
                == "840144b8419a55562231147feccabcecfff1a4c892f89a11c9183e7da2cfd15b",
        "invalid-claim-source-inventory",
    )?;
    let actual = i["sources"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-claim-source-inventory"))?;
    let expected = sources();
    let expected = expected.as_array().expect("literal sources");
    require(
        actual.len() == expected.len(),
        "invalid-claim-source-inventory",
    )?;
    for (source, e) in actual.iter().zip(expected) {
        shape(source, "path kind bytes sourceHash authority")?;
        require(
            json!([
                source["path"],
                source["kind"],
                source["bytes"],
                source["sourceHash"],
                source["authority"]
            ]) == *e,
            "invalid-claim-source-inventory",
        )?;
    }
    require(
        x["javascriptInspection"]
            == json!({"path":"source.semantic.sourcery.mjs","exports":["default"],"functions":[],"routes":[],"symbolInspection":"refused-no-unambiguous-top-level-function-class-or-method"})
            && x["relInspection"] == "unsupported-by-javascript-only-source-inspector"
            && x["request"]
                == json!({"requestId":"request-24e6-ff99-9af8","stoppedRevision":"8","stoppedHash":"6177f743377343ac2709490c7a4b475e2f1c8da9c9b4c388564d3d8e0ab75ef5","activeLeaseAfterStop":false})
            && x["countReconciliation"]
                == json!({"observedAt":"2026-09-09T10:06:35.058Z","requestId":"request-c2bc-f94d-3443","stoppedRevision":"4","stoppedHash":"057da7a0486d49753354698fe769984226cc4aa778470a70ebdf9f89333faae4","activeLeaseAfterStop":false,"sourceCount":"13","aggregateBytes":"161539","listingHash":"840144b8419a55562231147feccabcecfff1a4c892f89a11c9183e7da2cfd15b","declaredApplicationSources":"8","observedUnboundFiles":"5","historicalNarrativeCorrection":"The earlier request's stop reason said seven declared .rel files. The exact returned source classifications from that request and this fresh discovery both establish eight. The earlier source identities remain valid; only that narrative count is superseded."}),
        "invalid-claim-source-inventory",
    )?;
    Ok(
        json!({"kind":"temple3.claim-source-inventory-audit/v0.1","status":"qualified-static-source-boundary","staticSourceIdentity":true,"classificationCountReconciled":true,"declaredApplicationSources":"8","observedUnboundFiles":"5","emptyObservedLogFiles":"1","handlerExecution":false,"runtimeRecords":false,"exportOrCustodyAuthority":false,"readyForMigration":false,"refusal":{"code":"claim-source-runtime-evidence-unavailable","missing":["inspectable-handler-symbols","executed-runtime-records","export-or-custody-authority"]},"policy":x["policy"]}),
    )
}
