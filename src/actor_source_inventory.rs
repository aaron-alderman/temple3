//! Authentic Actor Scoped static-source inventory boundary.
use crate::{Result, require, shape};
use serde_json::{Value, json};

fn policy() -> Value {
    json!({"kind":"temple3.actor-source-inventory-policy/v0.1","staticSource":"historical-structural-evidence","unboundFile":"never-promote-to-declared-source","handlerExecution":"not-established","runtimeRecords":"not-established","sourceHeadCompleteness":"not-established","exportOrCustodyAuthority":"not-established","unsupportedInspection":"visible-refusal","migration":"not-started"})
}
fn sources() -> Value {
    json!([
        [
            "README.md",
            "documentation",
            "959",
            "e78c326c9a68385ba8279d16800fcdec97e771a925c3525949afed6d707a433e",
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
            "8968",
            "6c504d08cdb09c6cce0455e262ea4f586c5a606154f983440a40b4befb4e1bda",
            "declared-application-source"
        ],
        [
            "definition.rel",
            "rel",
            "1608",
            "2413163b827e288b7ca00732b05f7d39cb52fa2efeca0a1f7d180816de599f17",
            "declared-application-source"
        ],
        [
            "plugin.rel",
            "rel",
            "8748",
            "a6f982880a844a526f4aa6525d5575418f8bdd08ff60f9f3ec0900b00af74491",
            "declared-application-source"
        ],
        [
            "product.rel",
            "rel",
            "71641",
            "1a6bbf473c137ff7514257dc2ac3ee25c4b2274e4448c1baeb2cc9510fe8cd2a",
            "declared-application-source"
        ],
        [
            "project.rel",
            "rel",
            "994",
            "bb041fc107373c06bc5b5459663a5242748a4b3e2142064599bd2e7d43928c71",
            "declared-application-source"
        ],
        [
            "routes.rel",
            "rel",
            "2305",
            "f15be07ad0a802b3e5f103764724a96bd89e6966228b8686a7002c77e180ed03",
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
            "1362",
            "32456f82f46a4e1e2f074a5adb15d1e9aa010e29391f93bf82cd8d3b822f6e4a",
            "observed-unbound-application-file"
        ],
        [
            "ui-library-adoption.plugin.rel",
            "rel",
            "2514",
            "2d86a62b9a93b937f8dbf49dfb30ff03a9bd014da8878a7d8e6f94eda36a3ff5",
            "observed-unbound-application-file"
        ],
        [
            "ui-library-adoption.rel",
            "rel",
            "22781",
            "1f75d06f5ebea4e1ecd189b7883637a26faf8c6b33c0bcf1c1d78bc6752c3414",
            "observed-unbound-application-file"
        ],
        [
            "ui.rel",
            "rel",
            "38800",
            "9802ed1c188d27d9940d3e613a2b3f903115c5ebbfb35d5be610824ce4d6c05e",
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
        x["kind"] == "temple3.actor-source-inventory/v0.1"
            && x["application"] == "app.actorScopedCaseAuthorityReference"
            && x["artifactHash"]
                == "ac0984ec7c5c9b22ab86d8fc2f24dfd9307f9b8122098e87a6ddb87c75a9ab7d"
            && x["policy"] == policy(),
        "invalid-actor-source-inventory",
    )?;
    let inventory = &x["inventory"];
    shape(
        inventory,
        "protocol sourceCount aggregateBytes listingHash sources",
    )?;
    require(
        inventory["protocol"] == "sourcery-imported-surface-source-inventory/1"
            && inventory["sourceCount"] == "13"
            && inventory["aggregateBytes"] == "160770"
            && inventory["listingHash"]
                == "dfe42d0b84d5e4fd7ed16815d87156c63c0e1ac00aea2b0fdaa07e112c48985d",
        "invalid-actor-source-inventory",
    )?;
    let actual = inventory["sources"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-actor-source-inventory"))?;
    let expected = sources();
    let expected = expected.as_array().expect("literal sources");
    require(
        actual.len() == expected.len(),
        "invalid-actor-source-inventory",
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
            "invalid-actor-source-inventory",
        )?;
    }
    require(
        x["javascriptInspection"]
            == json!({"path":"source.semantic.sourcery.mjs","exports":["default"],"functions":[],"routes":[],"symbolInspection":"refused-no-unambiguous-top-level-function-class-or-method"})
            && x["relInspection"] == "unsupported-by-javascript-only-source-inspector"
            && x["request"]
                == json!({"requestId":"request-53ee-5cf3-dbda","stoppedRevision":"6","stoppedHash":"181862ecc85458e9357f6f2a997ff3fc0be7f4bd675decc10f82083a0e22f7bf","activeLeaseAfterStop":false}),
        "invalid-actor-source-inventory",
    )?;
    Ok(
        json!({"kind":"temple3.actor-source-inventory-audit/v0.1","status":"qualified-static-source-boundary","staticSourceIdentity":true,"declaredApplicationSources":"8","observedUnboundFiles":"5","emptyObservedLogFiles":"1","handlerExecution":false,"runtimeRecords":false,"sourceHeadCompleteness":false,"exportOrCustodyAuthority":false,"readyForMigration":false,"refusal":{"code":"actor-source-runtime-evidence-unavailable","missing":["inspectable-handler-symbols","executed-runtime-records","source-head-completeness","export-or-custody-authority"]},"policy":x["policy"]}),
    )
}
