//! Authentic Relationship Navigator static-source inventory boundary.
use crate::{Result, require, shape};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
fn policy() -> Value {
    json!({"kind":"temple3.relnav-source-inventory-policy/v0.1","staticSource":"historical-structural-evidence","unboundFile":"never-promote-to-declared-source","javascriptStructure":"static-only-not-execution","relExecution":"not-established","runtimeRecords":"not-established","sourceHeadCompleteness":"not-established","writerTransfer":"not-established","exportOrCustodyAuthority":"not-established","unsupportedInspection":"visible-refusal","semanticTarget":"not-selected","migration":"not-started"})
}
fn javascript() -> Value {
    json!({"path":"context-actions.runtime.mjs","sourceHash":"795c1273d2ed11e3d2878802e6d42d68a9a92bf0c35e5cf49dd1b9eb2ad445d9","exports":["__testing","pageSurfaces","routeActions","serverExperienceRuntime"],"functions":["basisOf","contextForActor","contextRuntime","pageSurfaces","propositionShells"],"routes":[],"inspectedSymbol":"pageSurfaces","symbolHash":"c7854c1eb31692bdd93186991a271e39dbabefd0f983460c3a21d0cc5321e122"})
}
fn refusals() -> Value {
    json!([{"code":"sourcery-authoring-surface-failure","condition":"invalid-symbol-format","disposition":"not-applied-or-effect-unproven"},{"code":"sourcery-imported-surface-source-inspection-binding","condition":"non-javascript-source-inspection","disposition":"not-applied-or-effect-unproven"}])
}
fn request() -> Value {
    json!({"requestId":"request-8c5b-7b7a-f8f2","stoppedRevision":"7","stoppedHash":"e508a81d61014e976145712a36dc82a6f27715d2b25aada507445252621afb7c","activeLeaseAfterStop":false})
}
fn correction() -> Value {
    json!({"supersedesStopReasonCountsOnly":true,"declaredApplicationSources":"22","observedUnboundFiles":"11","requestId":"request-26bc-0221-ba44","stoppedRevision":"3","stoppedHash":"eed259d5c4cd8f201a138aa972a7d74b9c88cd7f75eba659b631f28756925356","activeLeaseAfterStop":false})
}
pub fn audit(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind application artifactHash inventory javascriptInspection refusals request countCorrection policy",
    )?;
    require(
        x["kind"] == "temple3.relnav-source-inventory/v0.1"
            && x["application"] == "app.relnav"
            && x["artifactHash"]
                == "c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7"
            && x["policy"] == policy(),
        "invalid-relnav-source-inventory",
    )?;
    let i = &x["inventory"];
    shape(i, "protocol sourceCount aggregateBytes listingHash sources")?;
    require(
        i["protocol"] == "sourcery-imported-surface-source-inventory/1"
            && i["sourceCount"] == "33"
            && i["aggregateBytes"] == "1483559"
            && i["listingHash"]
                == "022fd97891876bb7eb00567c3d55a4403151b13507906967f49bc82f8c989049",
        "invalid-relnav-source-inventory",
    )?;
    let sources = i["sources"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-relnav-source-inventory"))?;
    require(sources.len() == 33, "invalid-relnav-source-inventory")?;
    for s in sources {
        shape(s, "path kind bytes sourceHash authority")?;
        let b = s["bytes"].as_str().unwrap_or("");
        let h = s["sourceHash"].as_str().unwrap_or("");
        require(
            !b.is_empty()
                && b.bytes().all(|c| c.is_ascii_digit())
                && (b == "0" || !b.starts_with('0'))
                && h.len() == 64
                && h.bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                && (s["authority"] == "declared-application-source"
                    || s["authority"] == "observed-unbound-application-file"),
            "invalid-relnav-source-inventory",
        )?;
    }
    let raw = crate::canonical::canonical_json(&i["sources"])?;
    require(
        hex::encode(Sha256::digest(raw.as_bytes()))
            == "1e266b005dabd683ef87ca5633df3a474c823edc3cc3047c032ff553bdc87279",
        "invalid-relnav-source-inventory",
    )?;
    require(
        x["javascriptInspection"] == javascript()
            && x["refusals"] == refusals()
            && x["request"] == request()
            && x["countCorrection"] == correction(),
        "invalid-relnav-source-inventory",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-source-inventory-audit/v0.1","status":"qualified-static-source-boundary","staticSourceIdentity":true,"javascriptStructure":true,"declaredApplicationSources":"22","observedUnboundFiles":"11","relExecution":false,"runtimeRecords":false,"sourceHeadCompleteness":false,"writerTransfer":false,"exportOrCustodyAuthority":false,"semanticTargetSelected":false,"readyForMigration":false,"refusal":{"code":"relnav-source-runtime-evidence-unavailable","missing":["executed-rel-semantics","executed-runtime-records","source-head-completeness","source-writer-transfer","export-or-custody-authority","selected-semantic-target"]},"policy":x["policy"]}),
    )
}
