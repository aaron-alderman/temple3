use crate::{
    Result, array, canonical::identify, decimal, err, federation::RelationVerifier, identity, kernel, predicate::Scope, require,
    shape, string, without,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

const DECISION: &str = "templeId profileId height parentFinalityId proposalId witnessId initialStateRoot finalStateRoot";
pub fn validate_profile(p: &Value) -> Result<()> {
    shape(p, "kind mode members quorum faultBound")?;
    let members = array(&p["members"], "invalid-coordination-profile")?;
    require(
        p["kind"] == "temple3.coordination-profile/v0.1"
            && !members.is_empty()
            && members.len() <= 32,
        "invalid-coordination-profile",
    )?;
    let number = |f: &str| -> Result<usize> {
        let s = string(&p[f], "invalid-coordination-profile")?;
        require(
            s.len() <= 2
                && !s.is_empty()
                && (s == "0" || !s.starts_with('0'))
                && s.bytes().all(|b| b.is_ascii_digit()),
            "invalid-coordination-profile",
        )?;
        s.parse().map_err(|_| err("invalid-coordination-profile"))
    };
    let q = number("quorum")?;
    let f = number("faultBound")?;
    let n = members.len();
    let mut previous = None;
    let mut keys = BTreeSet::new();
    for m in members {
        shape(m, "voter publicKey")?;
        identity::token(&m["voter"])?;
        identity::key_id(&m["publicKey"])?;
        let voter = string(&m["voter"], "invalid-coordination-profile")?;
        require(
            keys.insert(string(&m["publicKey"], "invalid-coordination-profile")?)
                && previous.is_none_or(|p| p < voter),
            "invalid-coordination-profile",
        )?;
        previous = Some(voter);
    }
    require(
        match p["mode"].as_str() {
            Some("single-instance") => n == 1 && q == 1 && f == 0,
            Some("unanimous-joint") => n >= 2 && q == n && f == n - 1,
            Some("threshold-custody") => f < n && q >= 1 && q <= n - f && 2 * q > n + f,
            _ => false,
        },
        "invalid-coordination-profile",
    )
}
pub fn initial(genesis: &Value, expected_temple: &Value) -> Result<Value> {
    let (temple, identity) = identity::verify_genesis(genesis, expected_temple)?;
    let profile = &genesis["signed"]["payload"]["genesisInput"]["context"]["application"]["coordinationProfile"];
    validate_profile(profile)?;
    let pid = identify("coordination-profile", profile)?;
    let fid = identify(
        "finality-anchor",
        &json!({"templeId":temple["templeId"],"profileId":pid,"genesisWitnessId":temple["witnesses"][0]["witnessId"]}),
    )?;
    Ok(
        json!({"profile":profile,"profileId":pid,"identityProfile":identity,"temple":temple,"finalityId":fid,"requests":[],"history":[{"temple":temple,"finalityId":fid}]}),
    )
}
pub fn candidate(state: &Value, proposal: &Value) -> Result<Value> {
    candidate_scoped(state, proposal, &mut Scope::default())
}
pub fn candidate_with_relations(
    state: &Value, proposal: &Value, relations: RelationVerifier,
) -> Result<Value> {
    candidate_scoped(state, proposal, &mut Scope::new(Some(relations)))
}
pub(crate) fn candidate_scoped(
    state: &Value,
    proposal: &Value,
    scope: &mut Scope,
) -> Result<Value> {
    require(proposal["purpose"] == "proposal", "invalid-purpose")?;
    let slot = identity::authenticate(
        &state["identityProfile"],
        &state["temple"]["templeId"],
        proposal,
    )?;
    require(
        !array(&state["requests"], "invalid-state")?
            .iter()
            .any(|r| r["slot"] == slot),
        "invalid-candidate",
    )?;
    let p = &proposal["payload"];
    let result = kernel::evaluate_scoped(
        &state["temple"],
        &json!({"identity":p["actor"],"delta":p["delta"],"context":p["context"]}),
        None,
        scope,
    )?;
    require(result.accepted, "invalid-candidate")?;
    Ok(
        json!({"decision":{"templeId":result.temple["templeId"],"profileId":state["profileId"],"height":result.temple["sequence"],"parentFinalityId":state["finalityId"],"proposalId":identity::signed_id(proposal)?,"witnessId":result.witness["witnessId"],"initialStateRoot":state["temple"]["stateRoot"],"finalStateRoot":result.temple["stateRoot"]},"witness":result.witness,"temple":result.temple,"slot":slot}),
    )
}
pub fn check_vote(profile: &Value, decision: &Value, vote: &Value) -> Result<String> {
    let p = &vote["payload"];
    shape(p, &format!("{DECISION} voter"))?;
    let member = array(&profile["members"], "invalid-coordination-profile")?
        .iter()
        .find(|m| m["voter"] == p["voter"]);
    require(
        member.is_some()
            && without(p, "voter")? == *decision
            && member.is_some_and(|m| {
                identity::verify_envelope(vote, "coordination-vote", &m["publicKey"])
            }),
        "invalid-coordination-vote",
    )?;
    Ok(string(&p["voter"], "invalid-coordination-vote")?.into())
}
pub fn verify_certificate(state: &Value, certificate: &Value) -> Result<Value> {
    certificate_scoped(state, certificate, &mut Scope::default())
}
pub fn verify_certificate_with_relations(
    state: &Value, certificate: &Value, relations: RelationVerifier,
) -> Result<Value> {
    certificate_scoped(state, certificate, &mut Scope::new(Some(relations)))
}
/// Assemble already-issued votes. Insufficient or duplicate votes are errors.
/// Issuing votes requires a separate durable height-lock service.
pub fn assemble_certificate(state: &Value, proposal: &Value, votes: &[Value]) -> Result<Value> {
    assemble_with(state, proposal, votes, None)
}
pub fn assemble_certificate_with_relations(
    state: &Value, proposal: &Value, votes: &[Value], relations: RelationVerifier,
) -> Result<Value> {
    assemble_with(state, proposal, votes, Some(relations))
}
pub(crate) fn assemble_with(
    state: &Value, proposal: &Value, votes: &[Value], relations: Option<RelationVerifier>,
) -> Result<Value> {
    let result = candidate_scoped(state, proposal, &mut Scope::new(relations))?;
    let mut ordered = votes.to_vec();
    for vote in &ordered {
        check_vote(&state["profile"], &result["decision"], vote)?;
    }
    ordered.sort_by(|a, b| {
        a["payload"]["voter"]
            .as_str()
            .cmp(&b["payload"]["voter"].as_str())
    });
    let certificate = crate::bind(
        "finality-certificate",
        "certificateId",
        json!({"kind":"temple3.finality-certificate/v0.1","decision":result["decision"],"proposal":proposal,"witness":result["witness"],"votes":ordered,"finalityId":identify("finality",&result["decision"])?}),
    )?;
    certificate_scoped(state, &certificate, &mut Scope::new(relations))?;
    Ok(certificate)
}
pub(crate) fn certificate_scoped(state: &Value, c: &Value, scope: &mut Scope) -> Result<Value> {
    shape(
        c,
        "kind decision proposal witness votes finalityId certificateId",
    )?;
    shape(&c["decision"], DECISION)?;
    require(
        c["kind"] == "temple3.finality-certificate/v0.1"
            && c["decision"]["parentFinalityId"] == state["finalityId"],
        "invalid-finality-certificate",
    )?;
    let result = candidate_scoped(state, &c["proposal"], scope)?;
    require(
        c["decision"] == result["decision"] && c["witness"] == result["witness"],
        "invalid-finality-certificate",
    )?;
    let votes = array(&c["votes"], "invalid-finality-certificate")?;
    let profile = &state["profile"];
    require(
        votes.len() <= array(&profile["members"], "invalid-coordination-profile")?.len(),
        "invalid-finality-certificate",
    )?;
    let mut previous: Option<String> = None;
    for v in votes {
        let voter = check_vote(profile, &result["decision"], v)?;
        require(
            previous.as_ref().is_none_or(|p| *p < voter),
            "invalid-finality-certificate",
        )?;
        previous = Some(voter);
    }
    let q = string(&profile["quorum"], "invalid-coordination-profile")?
        .parse::<usize>()
        .map_err(|_| err("invalid-coordination-profile"))?;
    require(
        votes.len() >= q
            && c["finalityId"] == identify("finality", &result["decision"])?
            && c["certificateId"]
                == identify("finality-certificate", &without(c, "certificateId")?)?,
        "invalid-finality-certificate",
    )?;
    Ok(result)
}
pub fn advance(state: &Value, certificate: &Value) -> Result<Value> {
    advance_scoped(state, certificate, &mut Scope::default())
}
pub fn advance_with_relations(
    state: &Value, certificate: &Value, relations: RelationVerifier,
) -> Result<Value> {
    advance_scoped(state, certificate, &mut Scope::new(Some(relations)))
}
pub(crate) fn advance_scoped(state: &Value, c: &Value, scope: &mut Scope) -> Result<Value> {
    require(state["halted"] != true, "halted-coordination")?;
    let result = certificate_scoped(state, c, scope)?;
    let mut next = state.clone();
    next["temple"] = result["temple"].clone();
    next["finalityId"] = c["finalityId"].clone();
    let mut requests = array(&state["requests"], "invalid-state")?.clone();
    requests.push(json!({"slot":result["slot"],"proposalId":c["decision"]["proposalId"],"height":c["decision"]["height"]}));
    next["requests"] = json!(requests);
    let mut history = array(&state["history"], "invalid-state")?.clone();
    history.push(json!({"temple":result["temple"],"finalityId":c["finalityId"]}));
    next["history"] = json!(history);
    Ok(next)
}
pub fn basis(state: &Value, finality: &Value) -> Result<Value> {
    let head = array(&state["history"], "invalid-state")?
        .iter()
        .find(|h| h["finalityId"] == *finality)
        .ok_or_else(|| err("unknown-finality-parent"))?;
    let height = decimal(&head["temple"]["sequence"])?;
    let mut result = state.clone();
    result["temple"] = head["temple"].clone();
    result["finalityId"] = head["finalityId"].clone();
    for (field, sequence) in [("requests", false), ("history", true)] {
        let mut entries = vec![];
        for e in array(&state[field], "invalid-state")? {
            if decimal(if sequence {
                &e["temple"]["sequence"]
            } else {
                &e["height"]
            })? <= height
            {
                entries.push(e.clone());
            }
        }
        result[field] = json!(entries);
    }
    Ok(result)
}
fn historical(state: &Value, c: &Value, relations: Option<RelationVerifier>) -> Result<()> {
    certificate_scoped(&basis(state, &c["decision"]["parentFinalityId"])?, c, &mut Scope::new(relations))?;
    Ok(())
}
pub fn replay(records: &Value, anchor: &Value) -> Result<Value> {
    let state = replay_state(records, anchor)?;
    Ok(
        json!({"templeId":state["temple"]["templeId"],"stateRoot":state["temple"]["stateRoot"],"sequence":state["temple"]["sequence"],"finalityId":state["finalityId"],"tip":state["tip"],"votes":array(&state["votes"],"invalid-state")?.len().to_string(),"equivocations":array(&state["equivocations"],"invalid-state")?.len().to_string(),"certificates":array(&state["certificates"],"invalid-state")?.len().to_string(),"conflicts":array(&state["conflicts"],"invalid-state")?.len().to_string(),"halted":state["halted"]}),
    )
}
/// Reconstructs the complete replica state, including durable height votes.
pub fn replay_state(records: &Value, anchor: &Value) -> Result<Value> {
    replay_state_with(records, anchor, None)
}
pub fn replay_state_with_relations(
    records: &Value, anchor: &Value, relations: RelationVerifier,
) -> Result<Value> {
    replay_state_with(records, anchor, Some(relations))
}
pub(crate) fn replay_state_with(
    records: &Value, anchor: &Value, relations: Option<RelationVerifier>,
) -> Result<Value> {
    let tip = identity::verify_records(records, anchor)?;
    let records = array(records, "invalid-journal")?;
    let mut state = initial(
        &records.first().ok_or_else(|| err("missing-genesis"))?["entry"],
        &anchor["expectedTempleId"],
    )?;
    state["votes"] = json!([]);
    state["equivocations"] = json!([]);
    state["certificates"] = json!([]);
    state["conflicts"] = json!([]);
    apply_entries_with(state, &records[1..], tip, relations)
}
/// Internal continuation of an already validated prefix. The caller verifies
/// the complete on-disk record chain and exact prefix before using this path.
pub(crate) fn apply_entries_with(
    mut state: Value, records: &[Value], tip: Value, relations: Option<RelationVerifier>,
) -> Result<Value> {
    let mut votes = array(&state["votes"], "invalid-state")?.clone();
    let mut equivocations = array(&state["equivocations"], "invalid-state")?.clone();
    let mut certificates = array(&state["certificates"], "invalid-state")?.clone();
    let mut conflicts = array(&state["conflicts"], "invalid-state")?.clone();
    for r in records {
        let e = &r["entry"];
        match e["kind"].as_str() {
            Some("coordination-vote") => {
                shape(e, "kind proposal vote")?;
                let v = &e["vote"];
                let result = candidate_scoped(
                    &basis(&state, &v["payload"]["parentFinalityId"])?,
                    &e["proposal"],
                    &mut Scope::new(relations),
                )?;
                check_vote(&state["profile"], &result["decision"], v)?;
                require(
                    !votes
                        .iter()
                        .any(|old| old["vote"]["payload"] == v["payload"]),
                    "replayed-coordination-message",
                )?;
                for old in &votes {
                    if ["voter", "height"]
                        .iter()
                        .all(|f| old["vote"]["payload"][*f] == v["payload"][*f])
                    {
                        equivocations.push(json!({"voter":v["payload"]["voter"],"height":v["payload"]["height"],"first":old["vote"],"second":v}));
                    }
                }
                votes.push(json!({"proposal":e["proposal"],"vote":v}));
            }
            Some("coordination-finality") => {
                shape(e, "kind certificate")?;
                let c = &e["certificate"];
                historical(&state, c, relations)?;
                let existing = array(&state["history"], "invalid-state")?
                    .iter()
                    .find(|h| h["temple"]["sequence"] == c["decision"]["height"]);
                require(
                    existing.is_none_or(|h| h["finalityId"] != c["finalityId"]),
                    "replayed-coordination-message",
                )?;
                require(state["halted"] != true, "halted-coordination")?;
                if let Some(h) = existing {
                    let first = certificates
                        .iter()
                        .find(|c| c["finalityId"] == h["finalityId"])
                        .ok_or_else(|| err("invalid-finality-conflict"))?;
                    conflicts.push(json!({"first":first,"second":c}));
                    state["halted"] = json!(true);
                } else {
                    require(
                        c["decision"]["parentFinalityId"] == state["finalityId"],
                        "unknown-finality-parent",
                    )?;
                    state = advance_scoped(&state, c, &mut Scope::new(relations))?;
                }
                certificates.push(c.clone());
            }
            Some("coordination-conflict") => {
                shape(e, "kind certificates")?;
                let cs = array(&e["certificates"], "invalid-finality-conflict")?;
                require(
                    cs.len() == 2 && state["halted"] != true,
                    "invalid-finality-conflict",
                )?;
                let (a, b) = (&cs[0], &cs[1]);
                historical(&state, a, relations)?;
                historical(&state, b, relations)?;
                require(
                    a["decision"]["height"] == b["decision"]["height"]
                        && a["decision"]["parentFinalityId"] == b["decision"]["parentFinalityId"]
                        && a["finalityId"] != b["finalityId"],
                    "invalid-finality-conflict",
                )?;
                conflicts.push(json!({"first":a,"second":b}));
                state["halted"] = json!(true);
            }
            _ => return Err(err("invalid-coordination-entry")),
        }
    }
    state["votes"] = json!(votes);
    state["equivocations"] = json!(equivocations);
    state["certificates"] = json!(certificates);
    state["conflicts"] = json!(conflicts);
    state["halted"] = json!(state["halted"] == true);
    state["tip"] = tip;
    Ok(state)
}
