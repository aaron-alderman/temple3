use crate::{
    Result, array, bind,
    canonical::{canonical_json, identify},
    commitment, coordination, err, kernel,
    predicate::{Scope, eq, literal, read},
    require, shape, string, without,
};
use serde_json::{Value, json};

pub fn statement(relation: &Value, inputs: &Value) -> Result<Value> {
    bind(
        "proof-statement",
        "statementId",
        json!({"kind":"temple3.proof-statement/v0.1","relation":relation,"publicInputs":inputs}),
    )
}
pub fn foreign_chain(bundle: &Value, temple_id: &Value) -> Result<Value> {
    foreign_scoped(bundle, temple_id, &mut Scope::default())
}
fn foreign_scoped(bundle: &Value, temple_id: &Value, scope: &mut Scope) -> Result<Value> {
    require(
        canonical_json(bundle)?.len() <= 65536,
        "foreign-evidence-too-large",
    )?;
    shape(bundle, "kind genesis certificates")?;
    let certs = array(&bundle["certificates"], "invalid-foreign-evidence")?;
    require(
        bundle["kind"] == "temple3.foreign-bundle/v0.1" && certs.len() <= 8,
        "invalid-foreign-evidence",
    )?;
    let mut state = coordination::initial(&bundle["genesis"], temple_id)?;
    for c in certs {
        state = coordination::advance_scoped(&state, c, scope)?;
    }
    Ok(state)
}
pub fn public_head(state: &Value) -> Result<Value> {
    let last = array(&state["temple"]["witnesses"], "invalid-state")?
        .last()
        .ok_or_else(|| err("invalid-state"))?;
    Ok(
        json!({"templeId":state["temple"]["templeId"],"finalityId":state["finalityId"],"stateRoot":state["temple"]["stateRoot"],"witnessId":last["witnessId"]}),
    )
}
pub fn validate_plan(plan: &Value) -> Result<()> {
    require(
        canonical_json(plan)?.len() <= 16384,
        "invalid-federation-plan",
    )?;
    shape(plan, "kind participants planId")?;
    let members = array(&plan["participants"], "invalid-federation-plan")?;
    require(
        plan["kind"] == "temple3.federation-plan/v0.1"
            && members.len() == 2
            && plan["planId"] == identify("federation-plan", &without(plan, "planId")?)?,
        "invalid-federation-plan",
    )?;
    let mut previous = None;
    for m in members {
        shape(
            m,
            "templeId baseFinalityId baseStateRoot cellSetRoot activePredicate intention",
        )?;
        let id = string(&m["templeId"], "invalid-federation-plan")?;
        let d = &m["intention"]["payload"]["delta"];
        require(
            !id.is_empty()
                && previous.is_none_or(|p| p < id)
                && m["baseFinalityId"].as_str().is_some_and(|s| !s.is_empty())
                && m["baseStateRoot"]
                    == commitment::state_root_from(
                        &json!(kernel::predicate_id(&m["activePredicate"])?),
                        &m["cellSetRoot"],
                    )?
                && m["intention"]["payload"]["templeId"] == m["templeId"]
                && d["basisStateRoot"] == m["baseStateRoot"]
                && kernel::normalize_delta(d)? == *d,
            "invalid-federation-plan",
        )?;
        previous = Some(id);
    }
    Ok(())
}
fn participant<'a>(plan: &'a Value, id: &Value) -> Result<&'a Value> {
    array(&plan["participants"], "invalid-federation-plan")?
        .iter()
        .find(|m| m["templeId"] == *id)
        .ok_or_else(|| err("invalid-federation-plan"))
}
fn commit_fields(m: &Value) -> Value {
    let d = &m["intention"]["payload"]["delta"];
    json!({"read":d["read"],"consume":d["consume"],"create":d["create"],"nextPredicate":if d["nextPredicate"].is_null(){&m["activePredicate"]}else{&d["nextPredicate"]},"claimedFinalStateRoot":d["claimedFinalStateRoot"]})
}
pub fn lock(plan: &Value, id: &Value) -> Result<Value> {
    validate_plan(plan)?;
    let fields = commit_fields(participant(plan, id)?);
    // Array order is protocol data, regardless of a map's serialization order.
    let mut values: Vec<Value> = [
        "read",
        "consume",
        "create",
        "nextPredicate",
        "claimedFinalStateRoot",
    ]
    .iter()
    .map(|f| eq(read(&["delta", f]), literal(fields[*f].clone())))
    .collect();
    values.push(eq(
        read(&["context", "federation", "phase"]),
        literal(json!("commit")),
    ));
    values.push(eq(
        read(&["context", "federation", "planId"]),
        literal(plan["planId"].clone()),
    ));
    values.push(json!({"op":"verify","statement":literal(statement(&json!("federation-ready"),&json!({"plan":plan,"localTempleId":id}))?),"proof":read(&["context","federation","proof"])}));
    Ok(json!({"op":"all","values":values}))
}
fn prepared_root(plan: &Value, m: &Value) -> Result<String> {
    commitment::state_root_from(
        &json!(kernel::predicate_id(&lock(plan, &m["templeId"])?)?),
        &m["cellSetRoot"],
    )
}
pub fn prepare_delta(plan: &Value, id: &Value) -> Result<Value> {
    let m = participant(plan, id)?;
    Ok(
        json!({"basisStateRoot":m["baseStateRoot"],"read":[],"consume":[],"create":[],"nextPredicate":lock(plan,id)?,"claimedFinalStateRoot":prepared_root(plan,m)?}),
    )
}
fn check_base(state: &Value, m: &Value, scope: &mut Scope) -> Result<()> {
    let base = coordination::basis(state, &m["baseFinalityId"])?;
    require(
        base["temple"]["stateRoot"] == m["baseStateRoot"]
            && commitment::cell_set_root(&base["temple"]["state"]["cells"])? == m["cellSetRoot"]
            && base["temple"]["state"]["activePredicate"] == m["activePredicate"],
        "invalid-federation-evidence",
    )?;
    coordination::candidate_scoped(&base, &m["intention"], scope)?;
    Ok(())
}
fn is_preparation(state: &Value, bundle: &Value, plan: &Value, m: &Value) -> Result<bool> {
    let certs = array(&bundle["certificates"], "invalid-foreign-evidence")?;
    let Some(c) = certs.last() else {
        return Ok(false);
    };
    Ok(c["decision"]["parentFinalityId"] == m["baseFinalityId"]
        && state["temple"]["stateRoot"] == prepared_root(plan, m)?
        && c["proposal"]["payload"]["delta"] == prepare_delta(plan, &m["templeId"])?
        && c["proposal"]["payload"]["context"]
            == json!({"federation":{"planId":plan["planId"],"phase":"prepare"}}))
}
fn ready(inputs: &Value, bundle: &Value, scope: &mut Scope) -> Result<bool> {
    shape(inputs, "plan localTempleId")?;
    let plan = &inputs["plan"];
    validate_plan(plan)?;
    participant(plan, &inputs["localTempleId"])?;
    let peer = array(&plan["participants"], "invalid-federation-plan")?
        .iter()
        .find(|m| m["templeId"] != inputs["localTempleId"])
        .ok_or_else(|| err("invalid-federation-plan"))?;
    let state = foreign_scoped(bundle, &peer["templeId"], scope)?;
    check_base(&state, peer, scope)?;
    is_preparation(&state, bundle, plan, peer)
}
pub fn verify_proof(stmt: &Value, proof: &Value) -> Result<bool> {
    verify_scoped(stmt, proof, &mut Scope::default())
}
pub(crate) fn verify_scoped(stmt: &Value, proof: &Value, scope: &mut Scope) -> Result<bool> {
    // Exhaustion must escape every nested verifier, including under `not`.
    require(
        scope.depth < 4 && scope.remaining > 0,
        "proof-verification-exhausted",
    )?;
    scope.remaining -= 1;
    let checked = (|| -> Result<bool> {
        require(
            canonical_json(stmt)?.len() <= 65536 && canonical_json(proof)?.len() <= 65536,
            "proof-statement-too-large",
        )?;
        shape(stmt, "kind relation publicInputs statementId")?;
        shape(proof, "kind system statementId evidence")?;
        require(
            *stmt == statement(&stmt["relation"], &stmt["publicInputs"])?
                && proof["kind"] == "temple3.proof-envelope/v0.1"
                && proof["system"] == "temple3.transparent/v0.1"
                && proof["statementId"] == stmt["statementId"],
            "invalid-proof",
        )?;
        let mut child = Scope {
            depth: scope.depth + 1,
            remaining: 8,
        };
        let i = &stmt["publicInputs"];
        let e = &proof["evidence"];
        match stmt["relation"].as_str() {
            Some("basis-decision-command-v1") => Ok(crate::basis_decision::verify(i, e)),
            Some("basis-consequence-command-v1") => Ok(crate::basis_consequence::verify(i, e)),
            Some("typed-composition-command-v1") => Ok(crate::typed_composition::verify(i, e)),
            Some("representative-composition-command-v1") => {
                Ok(crate::typed_composition::verify_representative(i, e))
            }
            Some("durable-effect-command-v1") => Ok(crate::durable_effect::verify(i, e)),
            Some("inventory-command-v1") => Ok(crate::inventory::verify(i, e)),
            Some("preserved-transition-command-v1") => {
                Ok(crate::preserved_transition::verify(i, e))
            }
            Some("claim-import-application-v1") => {
                Ok(crate::claim_import_application::verify(i, e))
            }
            Some("voluntary-expression-command-v1") => {
                Ok(crate::voluntary_expression::verify(i, e))
            }
            Some("claim-readiness-command-v1") => Ok(crate::claim_readiness::verify(i, e)),
            Some("claim-readiness-local-command-v1") => {
                Ok(crate::claim_readiness_local::verify_bound(i, e))
            }
            Some("claim-readiness-local-command-v2") => {
                Ok(crate::claim_readiness_local::verify_compact(i, e))
            }
            Some("claim-lifecycle-local-command-v1") => {
                Ok(crate::claim_lifecycle_local::verify_compact(i, e))
            }
            Some("disclosure-grant-command-v1") => Ok(crate::disclosure_grants::verify(i, e)),
            Some("authorized-testimony-command-v1") => {
                Ok(crate::authorized_testimony::command_valid(i, e))
            }
            Some("workgarden-command-v1") => crate::workgarden::verify(i, e),
            Some("foreign-finality") => {
                shape(i, "templeId finalityId stateRoot witnessId")?;
                let result = foreign_scoped(e, &i["templeId"], &mut child)?;
                Ok(
                    !array(&e["certificates"], "invalid-foreign-evidence")?.is_empty()
                        && *i == public_head(&result)?,
                )
            }
            Some("federation-ready") => ready(i, e, &mut child),
            Some("private-opening") => {
                shape(i, "commitment value")?;
                shape(e, "salt")?;
                Ok(e["salt"].as_str().is_some_and(|s| {
                    s.len() == 64
                        && s.bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                }) && i["commitment"]
                    == identify(
                        "private-value",
                        &json!({"salt":e["salt"],"value":i["value"]}),
                    )?)
            }
            Some("state-disclosure") => {
                shape(i, "templeId stateRoot predicateId cellSetRoot claims")?;
                shape(e, "proofs")?;
                let claims = array(&i["claims"], "invalid-disclosure")?;
                let paths = array(&e["proofs"], "invalid-disclosure")?;
                require(
                    i["templeId"].as_str().is_some_and(|s| !s.is_empty())
                        && commitment::state_root_from(&i["predicateId"], &i["cellSetRoot"])?
                            == i["stateRoot"]
                        && claims.len() <= 16
                        && claims.len() == paths.len(),
                    "invalid-disclosure",
                )?;
                let mut previous = None;
                for (c, p) in claims.iter().zip(paths) {
                    shape(c, "cellId outcome")?;
                    let id = string(&c["cellId"], "invalid-disclosure")?;
                    require(previous.is_none_or(|prev| prev < id), "invalid-disclosure")?;
                    previous = Some(id);
                    if c["cellId"] != p["cellId"]
                        || c["outcome"] != p["outcome"]
                        || !commitment::verify_cell_proof(p, &i["cellSetRoot"])?
                    {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    })();
    match checked {
        Err(e) if e.code == "proof-verification-exhausted" => Err(e),
        Err(_) => Ok(false),
        ok => ok,
    }
}
pub fn outcome(plan: &Value, bundles: &Value) -> Result<Value> {
    validate_plan(plan)?;
    if !bundles
        .as_array()
        .is_some_and(|b| b.len() == 2 && b.iter().all(|v| !v.is_null()))
    {
        return Ok(json!({"status":"unresolved","reason":"missing-participant-evidence"}));
    }
    let mut phases = vec![];
    for (m, b) in array(&plan["participants"], "invalid-federation-plan")?
        .iter()
        .zip(array(bundles, "invalid-federation-evidence")?)
    {
        let state = foreign_chain(b, &m["templeId"])?;
        check_base(&state, m, &mut Scope::default())?;
        if state["finalityId"] == m["baseFinalityId"] {
            phases.push("unprepared");
        } else if is_preparation(&state, b, plan, m)? {
            phases.push("prepared");
        } else {
            let cs = array(&b["certificates"], "invalid-federation-evidence")?;
            let c = cs
                .last()
                .ok_or_else(|| err("invalid-federation-evidence"))?;
            let mut pb = b.clone();
            pb["certificates"] = json!(&cs[..cs.len() - 1]);
            let parent = foreign_chain(&pb, &m["templeId"])?;
            let mut delta = commit_fields(m);
            delta["basisStateRoot"] = json!(prepared_root(plan, m)?);
            require(
                is_preparation(&parent, &pb, plan, m)?
                    && c["proposal"]["payload"]["delta"] == delta
                    && c["proposal"]["payload"]["context"]["federation"]["planId"]
                        == plan["planId"],
                "invalid-federation-evidence",
            )?;
            phases.push("committed");
        }
    }
    let status = if phases.iter().all(|p| *p == "committed") {
        "committed"
    } else if phases.contains(&"committed") {
        "committing"
    } else if phases.iter().all(|p| *p == "prepared") {
        "prepared"
    } else if phases.iter().all(|p| *p == "unprepared") {
        "unprepared"
    } else {
        "preparing"
    };
    Ok(json!({"status":status,"planId":plan["planId"],"phases":phases}))
}
