//! Versioned foundation-obligation ledger successor.
use crate::{Result, require, shape};
use serde_json::{Value, json};
fn policy() -> Value {
    json!({"priorLedger":"v0.1-historical-provenance","advancement":"only-independently-qualified-exact-selected-scope","selectedComplete":"scope-bound-not-global","missingRequiredCriterion":"refuse","hiddenDefaults":"reject","historicalVsOperative":"successor-integration-required-for-operative-authority","forgiveness":{"disposition":"refuse","scope":"foundation-obligation-ledger-v0.2"},"migration":"not-started"})
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(x, "kind prior formationSurface policy")?;
    require(
        x["kind"] == "temple3.foundation-obligation-ledger-update/v0.2" && x["policy"] == policy(),
        "invalid-foundation-obligation-ledger-v02",
    )?;
    let prior = crate::foundation_obligation_ledger::assess(&x["prior"])?;
    let surface = crate::selected_formation_surface::assess(&x["formationSurface"])?;
    require(
        prior["status"] == "refused-incomplete-foundation"
            && surface["FQ01"] == "selected-complete"
            && surface["FQ02"] == "selected-complete"
            && surface["readyForAF1"] == false
            && surface["readyForAF6"] == false,
        "invalid-foundation-obligation-ledger-v02",
    )?;
    let mut cases = x["prior"]["cases"].as_array().unwrap().clone();
    for c in &mut cases {
        if c["id"] == "FQ01" || c["id"] == "FQ02" {
            let m = c.as_object_mut().unwrap();
            m.insert("status".into(), "selected-complete".into());
            let mut evidence = m["evidence"].as_array().unwrap().clone();
            evidence.push("specification/SELECTED-FORMATION-SURFACE-CONTRACT.md".into());
            evidence.sort_by(|a, b| a.as_str().unwrap().cmp(b.as_str().unwrap()));
            m.insert("evidence".into(), Value::Array(evidence));
            m.insert("remaining".into(), json!([]));
        }
    }
    let open: Vec<Value> = cases
        .iter()
        .filter(|c| c["status"] == "in-progress")
        .map(|c| c["id"].clone())
        .collect();
    let selected: Vec<Value> = cases
        .iter()
        .filter(|c| c["status"] == "selected-complete")
        .map(|c| c["id"].clone())
        .collect();
    let areas = x["prior"]["areas"].as_array().unwrap().clone();
    let area_ids: Vec<Value> = areas.iter().map(|a| a["id"].clone()).collect();
    let mut missing = area_ids.clone();
    missing.extend(open.clone());
    Ok(
        json!({"kind":"temple3.foundation-obligation-ledger/v0.2","status":"refused-incomplete-foundation","priorLedgerId":crate::canonical::identify("foundation-obligation-ledger-v0.1",&x["prior"])? ,"formationSurfaceId":crate::canonical::identify("selected-formation-surface-v0.1",&x["formationSurface"])? ,"areas":areas,"cases":cases,"counts":{"areas":"8","inProgressAreas":"8","cases":"24","inProgressCases":open.len().to_string(),"selectedCompleteCases":selected.len().to_string()},"openAreas":area_ids,"openCases":open,"selectedCompleteCases":selected,"readyForAF1":false,"readyForAF4":false,"readyForAF5":false,"readyForAF6":false,"migration":"not-started","refusal":{"code":"foundation-obligations-incomplete","missing":missing},"policy":x["policy"]}),
    )
}
