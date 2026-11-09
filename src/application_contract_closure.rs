//! Exact application-contract-closure/v0.1 recognizer.
use serde_json::{Value, json};
const ID: &str = "application-contract-closure:sha256:330ccca4874e582d6ecd1a240c2532b8d73c6f5e66fcfacd2e4dc84b484f4a17";
pub fn audit(x: &Value) -> std::result::Result<Value, String> {
    let id = crate::canonical::identify("application-contract-closure", x)
        .map_err(|_| "invalid-application-contract-closure".to_string())?;
    if x["kind"] != "temple3.application-contract-closure/v0.1" || id != ID {
        return Err("invalid-application-contract-closure".into());
    }
    Ok(
        json!({"kind":"temple3.application-contract-closure-assessment/v0.1","status":"closed-local-metadata-obligations","closureId":ID,"areas":x["areas"],"externalObligations":x["externalObligations"],"readyForBoundProfile":true,"readyForAF1":false,"readyForAuthenticSourceMapping":false,"readyForMigration":false,"refusal":{"code":"broader-foundation-source-and-migration-obligations-open","disposition":"refuse"},"policy":x["policy"]}),
    )
}
