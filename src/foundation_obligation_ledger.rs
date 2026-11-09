//! Exact Temple3 foundation-obligation ledger recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};
const PRIOR: &str = "f65176c8e86d0d66873bb07d3ac7a1aa4bc03b5f967172d96ea2bf955b6b3420";
fn area_ids() -> Vec<String> {
    (1..=8).map(|i| format!("F{i}")).collect()
}
fn case_ids() -> Vec<String> {
    (1..=24).map(|i| format!("FQ{i:02}")).collect()
}
fn complete() -> Vec<String> {
    [5, 6, 7, 8, 14, 15, 16, 18, 19, 22]
        .iter()
        .map(|i| format!("FQ{i:02}"))
        .collect()
}
fn policy() -> Value {
    json!({"missingRequiredCriterion":"refuse","hiddenDefaults":"reject","historicalVsOperative":"prior-manifest-is-provenance-not-current-authority","selectedComplete":"scope-bound-not-global","unavailable":"retain-open-with-visible-obligation","forgiveness":{"disposition":"refuse","scope":"foundation-obligation-ledger"},"migration":"not-started"})
}
fn strings(v: &Value, nonempty: bool) -> Result<()> {
    let xs = v
        .as_array()
        .ok_or_else(|| crate::err("invalid-foundation-obligation-ledger"))?;
    require(
        !nonempty || !xs.is_empty(),
        "invalid-foundation-obligation-ledger",
    )?;
    let mut prior: Option<&str> = None;
    for x in xs {
        let s = x
            .as_str()
            .ok_or_else(|| crate::err("invalid-foundation-obligation-ledger"))?;
        require(
            !s.is_empty() && prior.is_none_or(|p| p < s),
            "invalid-foundation-obligation-ledger",
        )?;
        prior = Some(s);
    }
    Ok(())
}
fn entries(v: &Value, ids: &[String], is_case: bool) -> Result<()> {
    let xs = v
        .as_array()
        .ok_or_else(|| crate::err("invalid-foundation-obligation-ledger"))?;
    require(
        xs.len() == ids.len(),
        "invalid-foundation-obligation-ledger",
    )?;
    let selected = complete();
    for (i, x) in xs.iter().enumerate() {
        shape(x, "id status evidence remaining")?;
        require(x["id"] == ids[i], "invalid-foundation-obligation-ledger")?;
        let done = is_case && selected.contains(&ids[i]);
        require(
            x["status"]
                == if done {
                    "selected-complete"
                } else {
                    "in-progress"
                },
            "invalid-foundation-obligation-ledger",
        )?;
        strings(&x["evidence"], true)?;
        strings(&x["remaining"], !done)?;
        require(
            if done {
                x["remaining"].as_array().is_some_and(Vec::is_empty)
            } else {
                x["remaining"].as_array().is_some_and(|a| !a.is_empty())
            },
            "invalid-foundation-obligation-ledger",
        )?;
    }
    Ok(())
}
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(x, "kind priorIntegratedManifest areas cases policy")?;
    require(
        x["kind"] == "temple3.foundation-obligation-ledger/v0.1"
            && x["priorIntegratedManifest"] == PRIOR,
        "invalid-foundation-obligation-ledger",
    )?;
    let areas = area_ids();
    let cases = case_ids();
    let selected = complete();
    let open: Vec<String> = cases
        .iter()
        .filter(|x| !selected.contains(x))
        .cloned()
        .collect();
    entries(&x["areas"], &areas, false)?;
    entries(&x["cases"], &cases, true)?;
    require(
        x["policy"] == policy(),
        "invalid-foundation-obligation-policy",
    )?;
    let missing: Vec<String> = areas.iter().chain(open.iter()).cloned().collect();
    Ok(
        json!({"kind":"temple3.foundation-obligation-assessment/v0.1","status":"refused-incomplete-foundation","priorIntegratedManifest":x["priorIntegratedManifest"],"priorManifestRole":"qualified-provenance-not-current-operative-authority","areaCount":"8","caseCount":"24","inProgressAreaCount":"8","inProgressCaseCount":open.len().to_string(),"selectedCompleteCaseCount":selected.len().to_string(),"openAreas":areas,"openCases":open,"selectedCompleteCases":selected,"readyForAF1":false,"readyForAF4":false,"readyForAF5":false,"readyForAF6":false,"migration":"not-started","refusal":{"code":"foundation-obligations-incomplete","missing":missing},"policy":x["policy"]}),
    )
}
