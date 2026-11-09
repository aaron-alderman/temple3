//! Relationship Navigator static Context-runtime structure boundary.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"kind":"temple3.relnav-context-runtime-structure-policy/v0.1","evidence":"historical-static-source-only","missingService":"explicit-throw","wrongProvider":"explicit-throw","unknownActor":"empty-string-by-expression","basis":"last-returned-manifest-layer-by-expression","sourceGraph":"explicitly-excluded-by-expression","durableCollision":"later-durable-witness-wins-by-expression","failedInspection":"visible-failure-not-function-evidence","runtimeExecution":"not-established","runtimeRecords":"not-established","captureAuthority":"not-established","sourceCompleteness":"not-established","migration":"not-started"})
}
fn source() -> Value {
    json!({"path":"context-actions.runtime.mjs","sourceHash":"795c1273d2ed11e3d2878802e6d42d68a9a92bf0c35e5cf49dd1b9eb2ad445d9"})
}
fn inspections() -> Value {
    json!([{"symbol":"basisOf","symbolHash":"f6c7391cb5e56815ff61073e4ae6710fd9cb78e76f645b39ab9eebf734bca6dc","productEffect":"none"},{"symbol":"contextForActor","symbolHash":"f6e5015a0ee3c2d48a37f8b4df4cd3301991fd3b71e3b41d598d40fcfde30000","productEffect":"none"},{"symbol":"contextRuntime","symbolHash":"af56f7ea3804eb1ac9e1cfdbe0e7a51ede5ea1f04440b7107b26e2119b54dab9","productEffect":"none"},{"symbol":"propositionShells","symbolHash":"8dfd5ccd58c0254a72ab85e5f8660fe221d4bde5d8bd0aa30bdebf314f99b274","productEffect":"none"}])
}
fn failed() -> Value {
    json!({"requestedSymbol":"routeActions","tool":"sourcery_imported_surface","outcome":"failed","resultHash":"486b3dbd1cab162458a469105bef1026b51cac430aaa43f69dd8baecd431db1b","disposition":"no-source-structure-obtained"})
}
fn request() -> Value {
    json!({"requestId":"request-ba58-8530-da47","stoppedRevision":"8","stoppedHash":"c8291b677d33379fa6e8b10113f8bb9f3157e846c62bec5be70be06dc5894546","activeLeaseAfterStop":false})
}
pub fn audit(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind application artifactHash source inspections failedInspection request policy",
    )?;
    require(
        x["kind"] == "temple3.relnav-context-runtime-structure/v0.1"
            && x["application"] == "app.relnav"
            && x["artifactHash"]
                == "c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7",
        "invalid-relnav-context-runtime-structure",
    )?;
    require(
        x["source"] == source()
            && x["inspections"] == inspections()
            && x["failedInspection"] == failed()
            && x["request"] == request()
            && x["policy"] == policy(),
        "invalid-relnav-context-runtime-structure",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-context-runtime-structure-audit/v0.1","status":"qualified-static-runtime-structure-boundary","serviceLookupShape":true,"providerProtocolShape":true,"actorLookupShape":true,"basisLookupShape":true,"sourceGraphExclusionShape":true,"durableCollisionShape":true,"routeActionsInspected":false,"runtimeExecution":false,"runtimeRecords":false,"captureAuthority":false,"sourceCompleteness":false,"readyForMigration":false,"refusal":{"code":"relnav-context-runtime-authority-unavailable","missing":["route-actions-source-structure","provider-runtime-execution","runtime-records","capture-authority","source-completeness"]},"policy":x["policy"]}),
    )
}
