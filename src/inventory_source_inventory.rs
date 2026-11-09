//! Authentic Inventory static-source inventory boundary.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"kind":"temple3.inventory-source-inventory-policy/v0.1","staticSource":"historical-structural-evidence","unboundFile":"never-promote-to-declared-source","sourceEquationExecution":"not-established","runtimeRecords":"not-established","sourceHeadCompleteness":"not-established","writerTransfer":"not-established","exportOrCustodyAuthority":"not-established","unsupportedInspection":"visible-refusal","migration":"not-started"})
}
fn sources() -> Value {
    json!([
        [
            "README.md",
            "documentation",
            "964",
            "bf70ffd948e69d64b7a699782031fac319856291b193f97c2c890456e3811f28",
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
            "9518",
            "7022bfde6d4372d1a975f0487617254a9f30a55e3fa33c6ae12b634013201690",
            "declared-application-source"
        ],
        [
            "definition.rel",
            "rel",
            "1748",
            "3d0f0687ce51c02067f3885f2727aa65de59d453a5477b86e28e701b6308f07f",
            "declared-application-source"
        ],
        [
            "plugin.rel",
            "rel",
            "9323",
            "9981fd38e8656a79aed2e57c2b9668fd8106b44c86a2d26a4600dc4143bb4c1c",
            "declared-application-source"
        ],
        [
            "product.rel",
            "rel",
            "88055",
            "7eabdf37cdd57fc740f5172188db7f1a2208ae83468c5c28cde4df8c6fce9625",
            "declared-application-source"
        ],
        [
            "project.rel",
            "rel",
            "1004",
            "032e1a7f8dacd01e4e373f4cfd68dcff6e4df8ce5ad895e32af990d293cf0bf6",
            "declared-application-source"
        ],
        [
            "routes.rel",
            "rel",
            "2370",
            "24df4f33bc66294121113ff206382c04641a8b8acb7fd1d07cf081f8532a0032",
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
            "1457",
            "4b3c556116b6c583a3f16ce6af77e475cfe028e19568a6e085c8329906e45504",
            "observed-unbound-application-file"
        ],
        [
            "ui-library-adoption.plugin.rel",
            "rel",
            "2649",
            "a4985e1a494fe0bd21aeac98f7943f345a5c6572e13a778687422b267323498e",
            "observed-unbound-application-file"
        ],
        [
            "ui-library-adoption.rel",
            "rel",
            "24575",
            "d769b07dbeac1a38af10766a794c258f57c3acc626441737b80b020d0806e315",
            "observed-unbound-application-file"
        ],
        [
            "ui.rel",
            "rel",
            "47364",
            "041b2f0ea45a8f601a021dc2d9698e8d209d552d9ebeff2dd246d81febc64180",
            "declared-application-source"
        ]
    ])
}
pub fn audit(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind application artifactHash inventory javascriptInspection relInspection request policy",
    )?;
    require(
        x["kind"] == "temple3.inventory-source-inventory/v0.1"
            && x["application"] == "app.inventoryConservationLedgerReferenceV3"
            && x["artifactHash"]
                == "8234a80e4c1a5744a25cf61255cc66107a7ffe878be89806d5248570f8844731"
            && x["policy"] == policy(),
        "invalid-inventory-source-inventory",
    )?;
    let i = &x["inventory"];
    shape(i, "protocol sourceCount aggregateBytes listingHash sources")?;
    require(
        i["protocol"] == "sourcery-imported-surface-source-inventory/1"
            && i["sourceCount"] == "13"
            && i["aggregateBytes"] == "189117"
            && i["listingHash"]
                == "99fbb0ffe65929957a7aa9dbdbff488adccce698dabc18778fdc65798c3c05d1",
        "invalid-inventory-source-inventory",
    )?;
    let a = i["sources"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-inventory-source-inventory"))?;
    let e = sources();
    let e = e.as_array().expect("literal sources");
    require(a.len() == e.len(), "invalid-inventory-source-inventory")?;
    for (s, v) in a.iter().zip(e) {
        shape(s, "path kind bytes sourceHash authority")?;
        require(
            json!([
                s["path"],
                s["kind"],
                s["bytes"],
                s["sourceHash"],
                s["authority"]
            ]) == *v,
            "invalid-inventory-source-inventory",
        )?
    }
    require(
        x["javascriptInspection"]
            == json!({"path":"source.semantic.sourcery.mjs","exports":["default"],"functions":[],"routes":[],"symbolInspection":"refused-no-unambiguous-top-level-function-class-or-method"})
            && x["relInspection"] == "unsupported-by-javascript-only-source-inspector"
            && x["request"]
                == json!({"requestId":"request-8dc8-dd7c-6781","stoppedRevision":"6","stoppedHash":"3d8dc9af4888cb8e3f8da53e3d978712a27a2df4b23d346115a8607e9273e60f","activeLeaseAfterStop":false}),
        "invalid-inventory-source-inventory",
    )?;
    Ok(
        json!({"kind":"temple3.inventory-source-inventory-audit/v0.1","status":"qualified-static-source-boundary","staticSourceIdentity":true,"declaredApplicationSources":"8","observedUnboundFiles":"5","emptyObservedLogFiles":"1","sourceEquationExecution":false,"runtimeRecords":false,"sourceHeadCompleteness":false,"writerTransfer":false,"exportOrCustodyAuthority":false,"readyForMigration":false,"refusal":{"code":"inventory-source-runtime-evidence-unavailable","missing":["executed-source-equation","executed-runtime-records","source-head-completeness","source-writer-transfer","export-or-custody-authority"]},"policy":x["policy"]}),
    )
}
