//! Read-only journal/consent/reservation audit; no voting or recovery mutation.
use crate::{Result, array, err, require, shape};
use crate::{canonical, coordination, identity};
use serde_json::{Value, json};
fn size(v: &Value) -> Result<u128> {
    Ok(canonical::canonical_json(v)?.len() as u128)
}
fn charge(basis: &Value, e: &Value, voter: &Value) -> Result<u128> {
    let proposal = &e["submission"]["proposal"];
    let c = coordination::candidate(basis, proposal)?;
    let public = &array(&basis["profile"]["members"], "invalid-profile")?
        .iter()
        .find(|m| m["voter"] == *voter)
        .ok_or_else(|| err("wrong-voter"))?["publicKey"];
    let mut decision = c["decision"].clone();
    decision["voter"] = voter.clone();
    let vote = json!({"kind":"temple3.signed/v0.1","suite":"temple3.ed25519-sha256/v0.1","purpose":"coordination-vote","keyId":identity::key_id(public)?,"publicKey":public,"payload":decision,"signature":"0".repeat(128)});
    let mut cert = json!({"kind":"temple3.finality-certificate/v0.1","decision":c["decision"],"proposal":proposal,"witness":c["witness"],"votes":[vote.clone()],"finalityId":canonical::identify("finality",&c["decision"])?});
    cert["certificateId"] = json!(canonical::identify("finality-certificate", &cert)?);
    let mut total = 0;
    for entry in [
        e.clone(),
        json!({"kind":"coordination-vote","proposal":proposal,"vote":vote}),
        json!({"kind":"coordination-finality","certificate":cert}),
    ] {
        total += size(&identity::record_for(
            &entry,
            9007199254740991,
            &json!(format!("journal-record:sha256:{}", "0".repeat(64))),
        )?)?;
    }
    Ok(total)
}
fn same(a: &Value, b: &Value) -> bool {
    a["payload"]["actor"] == b["payload"]["actor"]
        && a["payload"]["requestId"] == b["payload"]["requestId"]
}
pub fn audit(x: &Value) -> Result<Value> {
    shape(
        x,
        "genesis descriptor expectedTempleId policy expectedPolicyId coordinationTip requestTip coordinationRecords requestRecords voter",
    )?;
    require(
        x["coordinationTip"].is_string() && x["requestTip"].is_string(),
        "missing-anchor",
    )?;
    let policy =
        crate::claim_lifecycle_host_policy::validate(&x["policy"], &x["expectedTempleId"])?;
    let pid = canonical::identify("claim-lifecycle-host-policy", &policy)?;
    require(x["expectedPolicyId"] == pid, "wrong-host-policy")?;
    crate::claim_lifecycle_compact_profile::recognize(
        &x["genesis"],
        &x["descriptor"],
        &x["expectedTempleId"],
    )?;
    let records = array(&x["coordinationRecords"], "invalid-records")?;
    require(
        records.first().is_some_and(|r| r["entry"] == x["genesis"]),
        "wrong-genesis",
    )?;
    let state = coordination::replay_state(
        &x["coordinationRecords"],
        &json!({"expectedTempleId":x["expectedTempleId"],"expectedTip":x["coordinationTip"]}),
    )?;
    require(
        state["profile"]["mode"] == "single-instance" && state["halted"] == false,
        "unsupported-history",
    )?;
    require(
        array(&state["profile"]["members"], "invalid-profile")?
            .iter()
            .any(|m| m["voter"] == x["voter"]),
        "wrong-voter",
    )?;
    let requests = array(&x["requestRecords"], "invalid-requests")?;
    let max = |field: &str| -> Result<u128> {
        policy["limits"][field]
            .as_str()
            .ok_or_else(|| err("invalid-limit"))?
            .parse()
            .map_err(|_| err("invalid-limit"))
    };
    require(
        requests.len() as u128 <= max("maximumRequests")?,
        "request-capacity",
    )?;
    identity::verify_records(
        &x["requestRecords"],
        &json!({"expectedTip":x["requestTip"]}),
    )?;
    let mut submissions: Vec<&Value> = vec![];
    let mut reserved = size(&records[0])?;
    for record in requests {
        let e = &record["entry"];
        shape(e, "kind basisFinalityId submission")?;
        require(
            e["kind"] == "temple3.retained-claim-lifecycle-submission/v0.1",
            "invalid-retention",
        )?;
        let s = &e["submission"];
        require(
            crate::claim_lifecycle_host_policy::verify(
                &json!({"identityProfile":x["descriptor"]["identityProfile"],"expectedTempleId":x["expectedTempleId"],"expectedPolicyId":pid,"policy":policy,"submission":s}),
            ),
            "invalid-consent",
        )?;
        reserved += charge(
            &coordination::basis(&state, &e["basisFinalityId"])?,
            e,
            &x["voter"],
        )?;
        require(
            !submissions
                .iter()
                .any(|old| same(&old["proposal"], &s["proposal"])),
            "duplicate-request",
        )?;
        submissions.push(s);
    }
    let votes = array(&state["votes"], "invalid-state")?;
    let certificates = array(&state["certificates"], "invalid-state")?;
    for p in votes
        .iter()
        .map(|v| &v["proposal"])
        .chain(certificates.iter().map(|c| &c["proposal"]))
    {
        require(
            submissions.iter().any(|s| s["proposal"] == *p),
            "unretained-authority",
        )?;
    }
    let mut actual = 0;
    for r in records.iter().chain(requests) {
        actual += size(r)?;
    }
    require(
        actual <= reserved && reserved <= max("maximumRetainedBytes")?,
        "retention-capacity",
    )?;
    let mut outcomes = vec![];
    for s in submissions {
        let proposal = &s["proposal"];
        let p = &proposal["payload"];
        let mut out = json!({"actor":p["actor"],"requestId":p["requestId"]});
        if let Some(cert) = certificates.iter().find(|c| same(&c["proposal"], proposal)) {
            require(cert["proposal"] == *proposal, "outcome-conflict")?;
            out["status"] = json!("committed");
            out["certificate"] = cert.clone();
        } else if coordination::candidate(&state, proposal).is_ok() {
            out["status"] = json!("pending");
            out["submission"] = s.clone();
        } else {
            out["status"] = json!("stale");
        }
        outcomes.push(out);
    }
    Ok(
        json!({"coordinationTip":x["coordinationTip"],"requestTip":x["requestTip"],"hostPolicyId":pid,"reservedBytes":reserved.to_string(),"outcomes":outcomes}),
    )
}
