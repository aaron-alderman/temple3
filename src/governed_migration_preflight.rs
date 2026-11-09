//! Current-evidence governed migration preflight. Refusal has no effects.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"kind":"temple3.governed-migration-preflight-policy/v0.1","authorityBasis":"recompute-exact-four-admission-closure","sourceRead":"forbid-before-authorized-preflight","targetWrite":"forbid-on-refusal","activation":"separate-explicit-authority","backout":"required-before-admission","forgiveness":{"disposition":"refuse","scope":"migration-preflight-attempts"},"historicalEvidence":"non-operative","retry":"fresh-authority-and-source-head-required","migration":"not-started"})
}
fn target() -> Value {
    json!({"kind":"temple3.target-qualification-snapshot/v0.1","checks":"110","sourceManifestSha256":"c539becf6a7dc8a9fa84d54625fbf5ce90bc1999464d897858a334e9c52680e2","changedSourcePaths":[],"signedImport":"qualified-javascript-python-rust","retainedAdmission":"qualified-selected-laboratory","authorityClosure":"refused-missing-operative-source-authority"})
}
fn plan() -> Value {
    json!({"kind":"temple3.governed-migration-plan-boundary/v0.1","sourceRead":"none-until-authorized","targetWrite":"none-until-authorized","activation":"separate-not-requested","recovery":"resume-only-from-retained-authorized-checkpoint","backout":"restore-source-unchanged-target-unactivated","dataMeaningAuthority":"required-before-admission","productionCutover":"not-authorized","arkSealing":"not-authorized"})
}
pub fn audit(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(x, "kind authorityInput target plan policy")?;
    require(
        x["kind"] == "temple3.governed-migration-preflight/v0.1"
            && x["policy"] == policy()
            && x["target"] == target()
            && x["plan"] == plan(),
        "invalid-governed-migration-preflight",
    )?;
    let a = crate::representative_authority_closure::audit(&x["authorityInput"])?;
    require(
        a["readyForMigration"] == false
            && a["refusal"]["code"] == "representative-authority-incomplete",
        "invalid-governed-migration-preflight",
    )?;
    Ok(
        json!({"kind":"temple3.governed-migration-preflight-result/v0.1","admitted":false,"status":"refused-before-effects","refusal":{"code":"governed-migration-source-authority-unavailable","basis":a["refusal"]},"effects":{"sourceReads":"0","targetWrites":"0","activations":"0","backouts":"0"},"nextRequired":["authenticated-source-head-and-completeness","source-writer-export-and-custody-authority","bounded-transport-and-receipts","replay-recovery-and-backout-evidence"],"target":x["target"],"policy":x["policy"]}),
    )
}
