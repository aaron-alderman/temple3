//! Language-neutral corpus runners. Expectations are only compared after execution.
use crate::{
    Result, array, commitment, coordination, err, federation, identity, kernel, migration, require,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
};

pub fn load(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).map_err(|e| err(&format!("{}: {e}", path.display())))?)
        .map_err(|e| err(&format!("{}: {e}", path.display())))
}
fn compare(name: &Value, actual: &Value, expected: &Value) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(err(&format!(
            "vector {name} mismatch\nactual: {actual}\nexpected: {expected}"
        )))
    }
}
fn result(value: Result<Value>) -> Value {
    value.unwrap_or_else(|e| json!({"error":e.code}))
}
pub fn kernel_vectors(root: &Path) -> Result<usize> {
    let c = load(&root.join("conformance/vectors/kernel-v0.1.json"))?;
    require(
        c["schema"] == "temple3.conformance-vectors/v0.1",
        "invalid-corpus",
    )?;
    compare(
        &json!("ark registry"),
        &kernel::ark_registry(),
        &c["arkRegistry"],
    )?;
    compare(
        &json!("obligations"),
        &kernel::obligations(),
        &c["arkObligations"],
    )?;
    let mut count = 0;
    for case in array(&c["arkCases"], "invalid-corpus")? {
        let d = kernel::evaluate_ark(&case["input"])?;
        let code = if d["outcome"] == "rejected" {
            json!(kernel::ark_failure(&d)?)
        } else {
            Value::Null
        };
        compare(
            &case["name"],
            &json!({"decision":d,"code":code}),
            &case["expected"],
        )?;
        count += 1;
    }
    for case in array(&c["transitionProofCases"], "invalid-corpus")? {
        let actual = result(
            kernel::evaluate_proof_transition(&case["head"], &case["input"], &case["proof"])
                .map(|r| json!({"accepted":r.accepted,"head":r.temple,"witness":r.witness})),
        );
        compare(&case["name"], &actual, &case["expected"])?;
        count += 1;
    }
    for case in array(&c["proofCases"], "invalid-corpus")? {
        compare(
            &case["name"],
            &json!(commitment::verify_cell_proof(
                &case["proof"],
                &case["expectedRoot"]
            )?),
            &case["expected"],
        )?;
        count += 1;
    }
    for s in array(&c["scenarios"], "invalid-corpus")? {
        let mut temple = kernel::seal_genesis(&s["genesis"]["input"])?;
        compare(
            &s["name"],
            &json!({"arkId":temple["arkId"],"templeId":temple["templeId"],"stateRoot":temple["stateRoot"],"witness":temple["witnesses"][0]}),
            &s["genesis"]["expected"],
        )?;
        count += 1;
        for step in array(&s["steps"], "invalid-corpus")? {
            let r = kernel::evaluate_transition(&temple, &step["input"])?;
            compare(
                &step["name"],
                &json!({"accepted":r.accepted,"sequence":r.temple["sequence"],"stateRoot":r.temple["stateRoot"],"witness":r.witness}),
                &step["expected"],
            )?;
            temple = r.temple;
            count += 1;
        }
    }
    Ok(count)
}
pub fn identity_vectors(root: &Path) -> Result<usize> {
    let c = load(&root.join("conformance/vectors/identity-v0.1.json"))?;
    require(
        c["kind"] == "temple3.identity-vectors/v0.1",
        "invalid-corpus",
    )?;
    let mut count = 0;
    for case in array(&c["primitiveCases"], "invalid-corpus")? {
        identity::verify_raw(&case["publicKey"], &case["signature"], &case["message"])?;
        count += 1;
    }
    for case in array(&c["signatureCases"], "invalid-corpus")? {
        compare(
            &case["name"],
            &json!(identity::verify_envelope(
                &case["envelope"],
                crate::string(&case["purpose"], "invalid-corpus")?,
                &case["publicKey"]
            )),
            &case["expected"],
        )?;
        count += 1;
    }
    for case in array(&c["replayCases"], "invalid-corpus")? {
        compare(
            &case["name"],
            &result(identity::replay(&case["records"], &case["anchor"])),
            &case["expected"],
        )?;
        count += 1;
    }
    Ok(count)
}
pub fn coordination_vectors(root: &Path) -> Result<usize> {
    let c = load(&root.join("conformance/vectors/coordination-v0.1.json"))?;
    require(
        c["kind"] == "temple3.coordination-vectors/v0.1",
        "invalid-corpus",
    )?;
    let mut count = 0;
    for category in ["profileCases", "certificateCases", "replayCases"] {
        for case in array(&c[category], "invalid-corpus")? {
            let actual = result((|| -> Result<Value> {
                match category {
                    "profileCases" => {
                        coordination::validate_profile(&case["profile"])?;
                        Ok(json!({"valid":true}))
                    }
                    "certificateCases" => {
                        let mut state = coordination::initial(
                            &case["genesis"],
                            &case["anchor"]["expectedTempleId"],
                        )?;
                        for parent in array(&case["parents"], "invalid-corpus")? {
                            state = coordination::advance(&state, parent)?;
                        }
                        let r = coordination::verify_certificate(&state, &case["certificate"])?;
                        Ok(
                            json!({"valid":true,"stateRoot":r["temple"]["stateRoot"],"sequence":r["temple"]["sequence"],"finalityId":case["certificate"]["finalityId"]}),
                        )
                    }
                    _ => coordination::replay(&case["records"], &case["anchor"]),
                }
            })());
            compare(&case["name"], &actual, &case["expected"])?;
            count += 1;
        }
    }
    Ok(count)
}
pub fn federation_vectors(root: &Path) -> Result<usize> {
    let c = load(&root.join("conformance/vectors/federation-v0.1.json"))?;
    require(
        c["kind"] == "temple3.federation-vectors/v0.1",
        "invalid-corpus",
    )?;
    let mut count = 0;
    for category in [
        "proofCases",
        "foreignCases",
        "outcomeCases",
        "transitionCases",
    ] {
        for case in array(&c[category], "invalid-corpus")? {
            let actual=result(match category {
                "proofCases"=>federation::verify_proof(&case["statement"],&case["proof"]).map(|v|json!(v)),
                "foreignCases"=>federation::foreign_chain(&case["bundle"],&case["expectedTempleId"]).and_then(|s|federation::public_head(&s)),
                "outcomeCases"=>federation::outcome(&case["plan"],&case["bundles"]),
                _=>kernel::evaluate_transition(&case["temple"],&case["proposal"]).map(|r|json!({"accepted":r.accepted,"stateRoot":r.temple["stateRoot"],"witness":r.witness})),
            });
            compare(&case["name"], &actual, &case["expected"])?;
            count += 1;
        }
    }
    Ok(count)
}
pub fn migration_vectors(root: &Path) -> Result<usize> {
    let legacy = load(&root.join("archaeology/legacy-observations.json"))?;
    let report = load(&root.join("archaeology/migration-report.json"))?;
    require(
        report["kind"] == "temple3.migration-report/v0.1",
        "invalid-corpus",
    )?;
    let m = migration::manifest(&legacy["sourcePins"])?;
    compare(&json!("manifest"), &m, &report["manifest"])?;
    let cases = array(&report["cases"], "invalid-corpus")?;
    let sources = array(&legacy["cases"], "invalid-corpus")?;
    let observations = array(&legacy["observations"], "invalid-corpus")?;
    require(
        cases.len() == sources.len() && cases.len() == observations.len(),
        "invalid-corpus",
    )?;
    let mut counts = std::collections::BTreeMap::from([
        ("equivalent-active-facts", 0usize),
        ("intentional-narrowing", 0),
        ("both-not-admitted", 0),
    ]);
    for ((case, source), obs) in cases.iter().zip(sources).zip(observations) {
        let name = &case["name"];
        compare(name, name, &source["name"])?;
        compare(name, name, &obs["name"])?;
        compare(name, &case["legacy"], &obs["result"])?;
        let state = coordination::initial(&case["genesis"], &case["expectedTempleId"])?;
        let values: Vec<_> = migration::active_facts(&source["basis"])?
            .iter()
            .map(|f| migration::fact_value(&m, f))
            .collect();
        let cells = values
            .iter()
            .map(commitment::cell_for)
            .collect::<Result<Vec<_>>>()?;
        compare(
            name,
            &state["temple"]["state"]["cells"],
            &json!(commitment::normalize_cells(&json!(cells))?),
        )?;
        compare(
            name,
            &state["temple"]["state"]["activePredicate"],
            &migration::covenant(&m, &values)?,
        )?;
        let construction = result((|| -> Result<Value> {
            require(
                obs["lowering"]["status"] == "ready",
                "legacy-lowering-incomplete",
            )?;
            Ok(
                json!({"proposal":migration::construct(&m,&state["temple"],&obs["lowering"]["command"],&source["args"])?}),
            )
        })());
        compare(name, &construction, &case["construction"])?;
        let comparison;
        let actual;
        if construction.get("error").is_some() {
            require(
                case["proposal"].is_null() && case["certificate"].is_null(),
                "invalid-corpus",
            )?;
            actual = json!({"status":"not-admitted","reason":construction["error"],"stateRoot":state["temple"]["stateRoot"]});
            comparison = if obs["result"]["status"] == "planned" {
                "intentional-narrowing"
            } else {
                "both-not-admitted"
            };
        } else {
            compare(
                name,
                &case["proposal"]["payload"]["delta"],
                &construction["proposal"]["delta"],
            )?;
            compare(name, &case["certificate"]["proposal"], &case["proposal"])?;
            let after = coordination::advance(&state, &case["certificate"])?;
            let mut active: Vec<_> = array(&after["temple"]["state"]["cells"], "invalid-state")?
                .iter()
                .map(|c| {
                    let mut f = c["value"]["fact"].clone();
                    f["pol"] = json!("is");
                    f
                })
                .collect();
            active.sort_by(|a, b| {
                ["s", "p", "o"]
                    .map(|f| a[f].as_str())
                    .cmp(&["s", "p", "o"].map(|f| b[f].as_str()))
            });
            compare(name, &json!(active), &obs["result"]["active"])?;
            actual = json!({"status":"finalized","stateRoot":after["temple"]["stateRoot"],"finalityId":after["finalityId"],"active":active});
            comparison = "equivalent-active-facts";
        }
        compare(name, &actual, &case["result"])?;
        compare(name, &json!(comparison), &case["comparison"])?;
        *counts
            .get_mut(comparison)
            .ok_or_else(|| err("invalid-corpus"))? += 1;
    }
    let counts: serde_json::Map<String, Value> = counts
        .into_iter()
        .map(|(k, v)| (k.into(), json!(v.to_string())))
        .collect();
    compare(
        &json!("migration counts"),
        &Value::Object(counts),
        &report["counts"],
    )?;
    Ok(cases.len())
}
