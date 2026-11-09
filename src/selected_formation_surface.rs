//! Exact selected-formation surface recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn kinds() -> Vec<&'static str> {
    vec![
        "temple3.basis-consequence-formation/v0.1",
        "temple3.basis-decision-formation/v0.1",
        "temple3.claim-lifecycle-formation/v0.1",
        "temple3.claim-readiness-formation/v0.1",
        "temple3.disclosure-grant-formation/v0.1",
        "temple3.durable-effect-formation/v0.1",
        "temple3.inventory-formation/v0.1",
        "temple3.preserved-transition-formation/v0.1",
        "temple3.representative-composition-formation/v0.1",
        "temple3.testimony-formation/v0.1",
        "temple3.typed-composition-formation/v0.1",
        "temple3.voluntary-expression-formation/v0.1",
    ]
}
fn meta() -> Vec<[&'static str; 5]> {
    vec![
        [
            "model/basis-consequence.mjs",
            "conformance/basis-consequence-differential.mjs",
            "conformance/vectors/basis-consequence-v0.1.json",
            "policy-{}",
            "policy-null",
        ],
        [
            "model/basis-decision.mjs",
            "conformance/basis-decision-differential.mjs",
            "conformance/vectors/basis-decision-v0.1.json",
            "missing-forgiveness",
            "policy-forgiveness-null",
        ],
        [
            "model/claim-lifecycle.mjs",
            "conformance/claim-lifecycle-differential.mjs",
            "conformance/vectors/claim-lifecycle-v0.1.json",
            "omit-forgiveness",
            "null-forgiveness",
        ],
        [
            "model/claim-readiness.mjs",
            "conformance/claim-readiness-differential.mjs",
            "conformance/vectors/claim-readiness-v0.1.json",
            "omit-policy-forgiveness",
            "null-policy-forgiveness",
        ],
        [
            "model/disclosure-grants.mjs",
            "conformance/disclosure-grants-differential.mjs",
            "conformance/vectors/disclosure-grants-v0.1.json",
            "policy-forgiveness-absent",
            "policy-forgiveness-null",
        ],
        [
            "model/durable-effect.mjs",
            "conformance/durable-effect-differential.mjs",
            "conformance/vectors/durable-effect-v0.1.json",
            "policy-forgiveness-absent",
            "policy-forgiveness-null",
        ],
        [
            "model/inventory.mjs",
            "conformance/inventory-differential.mjs",
            "conformance/vectors/inventory-v0.1.json",
            "policy-forgiveness-absent",
            "policy-forgiveness-null",
        ],
        [
            "model/preserved-transition.mjs",
            "conformance/preserved-transition-differential.mjs",
            "conformance/vectors/preserved-transition-v0.1.json",
            "policy-forgiveness-absent",
            "policy-forgiveness-null",
        ],
        [
            "model/typed-composition.mjs",
            "conformance/representative-composition-differential.mjs",
            "conformance/vectors/representative-composition-v0.1.json",
            "missing-policy-forgiveness",
            "extra-policy-field",
        ],
        [
            "model/authorized-testimony.mjs",
            "conformance/authorized-testimony-differential.mjs",
            "conformance/vectors/authorized-testimony-v0.1.json",
            "missing-policy-forgiveness",
            "extra-policy-field",
        ],
        [
            "model/typed-composition.mjs",
            "conformance/typed-composition-differential.mjs",
            "conformance/vectors/typed-composition-v0.1.json",
            "policy-forgiveness-absent",
            "policy-forgiveness-null",
        ],
        [
            "model/voluntary-expression.mjs",
            "conformance/voluntary-expression-differential.mjs",
            "conformance/vectors/voluntary-expression-v0.1.json",
            "policy-forgiveness-undefined",
            "policy-forgiveness-null",
        ],
    ]
}
fn policy() -> Value {
    json!({"missingForgiveness":"refuse-formation","unknownOrMalformedPolicy":"refuse-before-domain-judgment","hiddenDefaults":"reject","selectedScope":"all-currently-discovered-integrated-laboratory-formations","newFormation":"requires-explicit-registry-update-and-requalification","historicalVsOperative":"prior-manifest-is-provenance-not-current-authority","forgiveness":{"disposition":"refuse","scope":"selected-formation-surface"},"migration":"not-started"})
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind priorIntegratedManifest selectionRule formations policy",
    )?;
    require(
        x["kind"] == "temple3.selected-formation-surface/v0.1"
            && x["priorIntegratedManifest"]
                == "c941af9003c17aee19e2ab16a01a5b525aed3a9d47954526bb6a8957bdb5cf0f"
            && x["selectionRule"]
                == "all-distinct-temple3-formation-kinds-in-model-source-with-integrated-differential",
        "invalid-selected-formation-surface",
    )?;
    let ks = kinds();
    let ms = meta();
    let fs = x["formations"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-selected-formation-surface"))?;
    require(fs.len() == ks.len(), "invalid-selected-formation-surface")?;
    for (i, f) in fs.iter().enumerate() {
        shape(
            f,
            "formationKind source differential vector omissionCase malformedCase",
        )?;
        require(
            f["formationKind"] == ks[i],
            "invalid-selected-formation-surface",
        )?;
        for (k, field) in [
            "source",
            "differential",
            "vector",
            "omissionCase",
            "malformedCase",
        ]
        .iter()
        .enumerate()
        {
            require(f[*field] == ms[i][k], "invalid-selected-formation-surface")?;
        }
    }
    require(
        x["policy"] == policy(),
        "invalid-selected-formation-surface-policy",
    )?;
    Ok(
        json!({"kind":"temple3.selected-formation-surface-assessment/v0.1","status":"selected-formation-boundaries-complete","priorIntegratedManifest":x["priorIntegratedManifest"],"priorManifestRole":"qualified-provenance-not-current-operative-authority","formationCount":ks.len().to_string(),"formationKinds":ks,"FQ01":"selected-complete","FQ02":"selected-complete","scope":"all-currently-discovered-integrated-laboratory-formations","readyForAF1":false,"readyForAF6":false,"migration":"not-started","policy":x["policy"]}),
    )
}
