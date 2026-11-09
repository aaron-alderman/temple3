use crate::{
    Result, array, bind,
    canonical::{canonical_json, identify},
    commitment::{self, Multiproof},
    decimal, err,
    predicate::{self, Scope},
    require, string,
};
use serde_json::{Value, json};

pub const ARK: &str = "ark:temple3:v0.1-laboratory";
const INVARIANTS: [(&str, &str, &[&str]); 5] = [
    ("known-ark", "unsupported-ark", &["2", "6"]),
    (
        "predicate-fixed-during-judgment",
        "predicate-changed",
        &["6", "7"],
    ),
    ("basis-matches-initial-state", "stale-basis", &["3", "9"]),
    (
        "fixed-delta-application",
        "invalid-application",
        &["3", "5"],
    ),
    (
        "claimed-final-root-matches-result",
        "final-state-root-mismatch",
        &["3", "9"],
    ),
];
pub fn ark_registry() -> Value {
    json!({"kind":"temple3.ark-invariants/v0.1","arkId":ARK,"invariants":INVARIANTS.iter().map(|(id,code,c)|json!({"id":id,"code":code,"commandments":c})).collect::<Vec<_>>()})
}
pub fn obligations() -> Value {
    let entries: [(&str, &str, &[&str]); 9] = [
        ("canonical-proposal", "transport", &["3", "4", "6"]),
        ("bounded-context", "transport", &["4", "10"]),
        ("canonical-state", "state", &["3", "5", "6"]),
        ("canonical-delta", "delta", &["3", "5", "7"]),
        ("structural-application", "application", &["3", "5", "7"]),
        ("declared-basis", "visibility", &["4", "10"]),
        ("old-covenant-admission", "covenant", &["1", "2", "6", "7"]),
        ("atomic-genesis", "genesis", &["1", "8", "9"]),
        (
            "witnessed-continuation",
            "continuation",
            &["6", "7", "8", "9"],
        ),
    ];
    json!({"kind":"temple3.ark-obligations/v0.1","obligations":entries.iter().map(|(id,stage,c)|json!({"id":id,"stage":stage,"commandments":c})).collect::<Vec<_>>()})
}
fn ark_decision(failed: Vec<Value>) -> Value {
    json!({"outcome":if failed.is_empty(){"accepted"}else{"rejected"},"checks":INVARIANTS.iter().map(|v|v.0).collect::<Vec<_>>(),"failed":failed})
}
pub fn evaluate_ark(f: &Value) -> Result<Value> {
    for field in [
        "arkId",
        "basisStateRoot",
        "initialStateRoot",
        "claimedFinalStateRoot",
        "activePredicateId",
        "evaluatedPredicateId",
    ] {
        require(
            f[field].as_str().is_some_and(|s| !s.is_empty()),
            "invalid-ark-fact",
        )?;
    }
    let checks = [
        f["arkId"] == ARK,
        f["activePredicateId"] == f["evaluatedPredicateId"],
        f["basisStateRoot"] == f["initialStateRoot"],
        f["computedFinalStateRoot"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        f["claimedFinalStateRoot"] == f["computedFinalStateRoot"],
    ];
    Ok(ark_decision(
        checks
            .iter()
            .enumerate()
            .filter(|(_, ok)| !**ok)
            .map(|(i, _)| json!(INVARIANTS[i].0))
            .collect(),
    ))
}
pub fn ark_failure(decision: &Value) -> Result<&'static str> {
    INVARIANTS
        .iter()
        .find(|v| decision["failed"][0] == v.0)
        .map(|v| v.1)
        .ok_or_else(|| err("invalid-ark-decision"))
}
pub fn predicate_id(p: &Value) -> Result<String> {
    predicate::validate(p)?;
    identify("predicate", p)
}
pub fn normalize_state(state: &Value) -> Result<Value> {
    predicate::validate(&state["activePredicate"])?;
    Ok(
        json!({"activePredicate":state["activePredicate"],"cells":commitment::normalize_cells(&state["cells"])?}),
    )
}
pub fn state_commitment(state: &Value) -> Result<Value> {
    let pid = predicate_id(&state["activePredicate"])?;
    let root = commitment::cell_set_root(&state["cells"])?;
    Ok(
        json!({"predicateId":pid,"cellSetRoot":root,"stateRoot":commitment::state_root_from(&json!(pid),&json!(root))?}),
    )
}
pub fn state_root(state: &Value) -> Result<String> {
    Ok(string(&state_commitment(state)?["stateRoot"], "invalid-state")?.into())
}
fn compact_commitment(state: &Value) -> Result<Value> {
    let pid = predicate_id(&state["activePredicate"])?;
    Ok(
        json!({"predicateId":pid,"cellSetRoot":state["cellSetRoot"],"stateRoot":commitment::state_root_from(&json!(pid),&state["cellSetRoot"])?}),
    )
}
pub fn compact(temple: &Value) -> Result<Value> {
    let mut t = temple.clone();
    t["state"] = json!({"activePredicate":temple["state"]["activePredicate"],"cellSetRoot":commitment::cell_set_root(&temple["state"]["cells"])?});
    Ok(t)
}
fn context_id(context: &Value) -> Result<String> {
    require(
        canonical_json(context)?.len() <= 65_536,
        "context-too-large",
    )?;
    identify("context", context)
}
pub fn normalize_delta(delta: &Value) -> Result<Value> {
    require(delta.is_object(), "invalid-delta")?;
    for f in ["basisStateRoot", "claimedFinalStateRoot"] {
        require(
            delta[f].as_str().is_some_and(|s| !s.is_empty()),
            "invalid-delta",
        )?;
    }
    let empty = json!([]);
    let list = |f: &str| -> Result<Vec<Value>> {
        let mut v = array(delta.get(f).unwrap_or(&empty), "invalid-delta")?.clone();
        require(
            v.iter().all(|v| v.as_str().is_some_and(|s| !s.is_empty())),
            "invalid-delta",
        )?;
        v.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        require(
            !v.windows(2).any(|p| p[0] == p[1]),
            &format!("duplicate-{f}"),
        )?;
        Ok(v)
    };
    let read = list("read")?;
    let consume = list("consume")?;
    let create = delta.get("create").unwrap_or(&empty);
    require(create.is_array(), "invalid-delta")?;
    let create = commitment::normalize_cells(create)?;
    require(
        !create.iter().any(|c| consume.contains(&c["id"])),
        "consume-create-overlap",
    )?;
    let next = &delta["nextPredicate"];
    if !next.is_null() {
        predicate::validate(next)?;
    }
    require(
        !consume.is_empty() || !create.is_empty() || !next.is_null(),
        "empty-delta",
    )?;
    Ok(
        json!({"basisStateRoot":delta["basisStateRoot"],"read":read,"consume":consume,"create":create,"nextPredicate":next,"claimedFinalStateRoot":delta["claimedFinalStateRoot"]}),
    )
}
pub fn apply_delta(state: &Value, delta: &Value) -> Result<Value> {
    let mut cells = array(&state["cells"], "invalid-state")?.clone();
    for id in array(&delta["read"], "invalid-delta")? {
        require(cells.iter().any(|c| c["id"] == *id), "missing-read-cell")?;
    }
    for id in array(&delta["consume"], "invalid-delta")? {
        let i = cells
            .iter()
            .position(|c| c["id"] == *id)
            .ok_or_else(|| err("missing-consumed-cell"))?;
        cells.remove(i);
    }
    for c in array(&delta["create"], "invalid-delta")? {
        require(
            !cells.iter().any(|old| old["id"] == c["id"]),
            "existing-created-cell",
        )?;
        cells.push(c.clone());
    }
    normalize_state(
        &json!({"activePredicate":if delta["nextPredicate"].is_null(){&state["activePredicate"]}else{&delta["nextPredicate"]},"cells":cells}),
    )
}
/// Constructs and normalizes an unsigned delta; admission still runs separately.
pub fn build_delta(temple: &Value, effects: &Value) -> Result<Value> {
    let mut draft = effects
        .as_object()
        .ok_or_else(|| err("invalid-delta"))?
        .clone();
    draft.insert("basisStateRoot".into(), temple["stateRoot"].clone());
    draft.insert("claimedFinalStateRoot".into(), json!("pending"));
    let mut delta = normalize_delta(&Value::Object(draft))?;
    delta["claimedFinalStateRoot"] = json!(state_root(&apply_delta(&temple["state"], &delta)?)?);
    Ok(delta)
}
pub fn seal_genesis(input: &Value) -> Result<Value> {
    let ark = input.get("arkId").cloned().unwrap_or(json!(ARK));
    require(
        ark.as_str().is_some_and(|s| !s.is_empty()),
        "invalid-ark-identity",
    )?;
    require(ark == ARK, "unsupported-ark")?;
    let empty = json!([]);
    let cells = array(
        input.get("initialValues").unwrap_or(&empty),
        "invalid-state",
    )?
    .iter()
    .map(commitment::cell_for)
    .collect::<Result<Vec<_>>>()?;
    let state =
        normalize_state(&json!({"activePredicate":input["activePredicate"],"cells":cells}))?;
    let root = state_root(&state)?;
    let ctx = context_id(input.get("context").unwrap_or(&json!({})))?;
    let id = identify(
        "temple",
        &json!({"arkId":ARK,"stateRoot":root,"genesisContextId":ctx}),
    )?;
    let w = bind(
        "witness",
        "witnessId",
        json!({"kind":"temple3.genesis-witness/v0.1","templeId":id,"arkId":ARK,"sequence":"0","predicateId":predicate_id(&state["activePredicate"])?,"initialStateRoot":"uninstantiated","deltaId":"genesis","contextId":ctx,"outcome":"accepted","arkDecision":ark_decision(vec![]),"covenantDecision":{"outcome":"genesis"},"finalStateRoot":root}),
    )?;
    Ok(
        json!({"arkId":ARK,"templeId":id,"sequence":"0","state":state,"stateRoot":root,"witnesses":[w]}),
    )
}
#[derive(Debug, Clone)]
pub struct Transition {
    pub accepted: bool,
    pub temple: Value,
    pub witness: Value,
}
/// Evaluate against a trusted full state. Failed admission preserves that state.
pub fn evaluate_transition(temple: &Value, proposal: &Value) -> Result<Transition> {
    evaluate_scoped(temple, proposal, None, &mut Scope::default())
}
/// Use application-provided relations when evaluating a full-state transition.
pub fn evaluate_transition_with_relations(
    temple: &Value,
    proposal: &Value,
    relations: crate::federation::RelationVerifier,
) -> Result<Transition> {
    evaluate_scoped(
        temple,
        proposal,
        None,
        &mut Scope { relations: Some(relations), ..Scope::default() },
    )
}
pub fn evaluate_proof_transition(
    head: &Value,
    proposal: &Value,
    proof: &Value,
) -> Result<Transition> {
    evaluate_scoped(head, proposal, Some(proof), &mut Scope::default())
}
pub(crate) fn evaluate_scoped(
    t: &Value,
    p: &Value,
    proof: Option<&Value>,
    scope: &mut Scope,
) -> Result<Transition> {
    if proof.is_some() {
        require(
            compact_commitment(&t["state"])?["stateRoot"] == t["stateRoot"],
            "invalid-state-commitment",
        )?;
    }
    require(
        p["identity"].as_str().is_some_and(|s| !s.is_empty()),
        "invalid-identity",
    )?;
    canonical_json(&p["identity"])?;
    canonical_json(&p["delta"])?;
    let context = p.get("context").cloned().unwrap_or(json!({}));
    let ctx = context_id(&context)?;
    let pid = predicate_id(&t["state"]["activePredicate"])?;
    let reject = |code: &str,
                  did: String,
                  claimed: Value,
                  ark: Value,
                  covenant: Option<Value>,
                  stage: &str|
     -> Result<Transition> {
        let stage = if covenant.is_some() {
            "covenant"
        } else {
            stage
        };
        let w = bind(
            "witness",
            "witnessId",
            json!({"kind":"temple3.transition-witness/v0.1","templeId":t["templeId"],"arkId":t["arkId"],"sequence":t["sequence"],"identity":p["identity"],"predicateId":pid,"initialStateRoot":t["stateRoot"],"deltaId":did,"contextId":ctx,"outcome":"rejected","arkDecision":ark,"covenantDecision":covenant.unwrap_or(json!({"outcome":"not-evaluated"})),"reason":code,"failure":{"kind":"temple3.error/v0.1","stage":stage,"code":code},"claimedFinalStateRoot":claimed,"finalStateRoot":t["stateRoot"]}),
        )?;
        Ok(Transition {
            accepted: false,
            temple: t.clone(),
            witness: w,
        })
    };
    let delta = match normalize_delta(&p["delta"]) {
        Ok(d) => d,
        Err(e) => {
            return reject(
                &e.code,
                identify("delta", &p["delta"])?,
                json!("unknown"),
                ark_decision(vec![json!(e.code)]),
                None,
                "delta",
            );
        }
    };
    let did = identify("delta", &delta)?;
    if delta["basisStateRoot"] != t["stateRoot"] {
        return reject(
            "stale-basis",
            did,
            delta["claimedFinalStateRoot"].clone(),
            ark_decision(vec![json!("basis-matches-initial-state")]),
            None,
            "ark",
        );
    }
    let mut disclosed = Value::Null;
    let applied = (|| -> Result<Value> {
        if let Some(proof) = proof {
            let mut ids = array(&delta["read"], "invalid-delta")?.clone();
            ids.extend(array(&delta["consume"], "invalid-delta")?.clone());
            ids.extend(
                array(&delta["create"], "invalid-delta")?
                    .iter()
                    .map(|c| c["id"].clone()),
            );
            let m = Multiproof::authenticate(proof, &t["state"]["cellSetRoot"], &ids)?;
            disclosed = json!(m.cells);
            let local = apply_delta(
                &json!({"activePredicate":t["state"]["activePredicate"],"cells":disclosed}),
                &delta,
            )?;
            Ok(
                json!({"activePredicate":local["activePredicate"],"cellSetRoot":m.successor(&delta["consume"],&delta["create"]) ?}),
            )
        } else {
            apply_delta(&t["state"], &delta)
        }
    })();
    let final_state = match applied {
        Ok(s) => s,
        Err(e)
            if [
                "missing-read-cell",
                "missing-consumed-cell",
                "existing-created-cell",
            ]
            .contains(&e.code.as_str()) =>
        {
            return reject(
                &e.code,
                did,
                delta["claimedFinalStateRoot"].clone(),
                ark_decision(vec![json!(e.code)]),
                None,
                "application",
            );
        }
        Err(e) => return Err(e),
    };
    let commitment = if proof.is_some() {
        compact_commitment
    } else {
        state_commitment
    };
    let result = commitment(&final_state)?;
    let ark = evaluate_ark(
        &json!({"arkId":t["arkId"],"basisStateRoot":delta["basisStateRoot"],"initialStateRoot":t["stateRoot"],"claimedFinalStateRoot":delta["claimedFinalStateRoot"],"computedFinalStateRoot":result["stateRoot"],"activePredicateId":pid,"evaluatedPredicateId":pid}),
    )?;
    if ark["outcome"] == "rejected" {
        return reject(
            ark_failure(&ark)?,
            did,
            delta["claimedFinalStateRoot"].clone(),
            ark,
            None,
            "ark",
        );
    }
    let mut basis = commitment(&t["state"])?;
    let cells = if proof.is_some() {
        &disclosed
    } else {
        &t["state"]["cells"]
    };
    basis["cells"] = json!(
        array(cells, "invalid-state")?
            .iter()
            .filter(|c| delta["read"]
                .as_array()
                .is_some_and(|a| a.contains(&c["id"]))
                || delta["consume"]
                    .as_array()
                    .is_some_and(|a| a.contains(&c["id"])))
            .collect::<Vec<_>>()
    );
    let permitted = predicate::evaluate_scoped(
        &t["state"]["activePredicate"],
        &json!({"identity":p["identity"],"basis":basis,"delta":delta,"result":result,"context":context}),
        scope,
    );
    match permitted {
        Err(e)
            if ["predicate-fuel-exhausted", "predicate-result-not-boolean"]
                .contains(&e.code.as_str()) =>
        {
            return reject(
                &e.code,
                did,
                delta["claimedFinalStateRoot"].clone(),
                ark,
                Some(json!({"outcome":"error","reason":e.code})),
                "covenant",
            );
        }
        Err(e) => return Err(e),
        Ok(false) => {
            return reject(
                "covenant-rejected",
                did,
                delta["claimedFinalStateRoot"].clone(),
                ark,
                Some(json!({"outcome":"rejected"})),
                "covenant",
            );
        }
        Ok(true) => (),
    }
    let sequence = (decimal(&t["sequence"])? + 1u8).to_string();
    let witness = bind(
        "witness",
        "witnessId",
        json!({"kind":"temple3.transition-witness/v0.1","templeId":t["templeId"],"arkId":t["arkId"],"sequence":sequence,"identity":p["identity"],"predicateId":pid,"initialStateRoot":t["stateRoot"],"deltaId":did,"contextId":ctx,"outcome":"accepted","arkDecision":ark,"covenantDecision":{"outcome":"accepted"},"finalStateRoot":result["stateRoot"]}),
    )?;
    let mut next = t.clone();
    next["sequence"] = json!(sequence);
    next["state"] = final_state;
    next["stateRoot"] = result["stateRoot"].clone();
    let mut history = array(&t["witnesses"], "invalid-state")?.clone();
    history.push(witness.clone());
    next["witnesses"] = json!(history);
    Ok(Transition {
        accepted: true,
        temple: next,
        witness,
    })
}
