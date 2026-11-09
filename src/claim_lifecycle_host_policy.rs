//! Independent host policy and consent validation, not durable enforcement.
use crate::{Result, require, shape};
use serde_json::{Value, json};
pub fn validate(p: &Value, temple: &Value) -> Result<Value> {
    crate::canonical::canonical_json(p)?;
    shape(
        p,
        "kind instance templeId coordination authorization retention retry recovery forgiveness limits",
    )?;
    require(
        p["kind"] == "temple3.claim-lifecycle-host-policy/v0.1",
        "invalid-host-policy",
    )?;
    require(
        p["instance"].as_str().is_some_and(|s| {
            s.len() == 32
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }),
        "invalid-host-policy",
    )?;
    require(
        temple.as_str().is_some_and(|s| !s.is_empty()) && p["templeId"] == *temple,
        "invalid-host-policy",
    )?;
    for (key, value) in [
        ("coordination", "single-instance"),
        ("authorization", "exact-proposal-and-host-attestation"),
        ("retention", "retain-all-requests-votes-and-certificates"),
        ("retry", "exact-no-rebase"),
        (
            "recovery",
            "finish-retained-vote-no-vote-from-retention-alone",
        ),
    ] {
        require(p[key] == value, "invalid-host-policy")?;
    }
    require(
        p["forgiveness"]
            == json!({"disposition":"refuse","scope":"host-requests-votes-and-certificates"}),
        "invalid-host-policy",
    )?;
    shape(
        &p["limits"],
        "maximumRequests maximumSubmissionBytes maximumRetainedBytes",
    )?;
    for (key, max) in [
        ("maximumRequests", 9007199254740991u64),
        ("maximumSubmissionBytes", 8389632),
        ("maximumRetainedBytes", 9007199254740991),
    ] {
        let s = p["limits"][key]
            .as_str()
            .ok_or_else(|| crate::err("invalid-host-policy"))?;
        require(
            !s.is_empty()
                && s.len() <= 16
                && !s.starts_with('0')
                && s.bytes().all(|b| b.is_ascii_digit()),
            "invalid-host-policy",
        )?;
        require(
            s.parse::<u64>().is_ok_and(|n| n <= max),
            "invalid-host-policy",
        )?;
    }
    Ok(p.clone())
}
fn checked(x: &Value) -> Result<bool> {
    shape(
        x,
        "identityProfile expectedTempleId expectedPolicyId policy submission",
    )?;
    let p = validate(&x["policy"], &x["expectedTempleId"])?;
    let pid = crate::canonical::identify("claim-lifecycle-host-policy", &p)?;
    require(x["expectedPolicyId"] == pid, "invalid-host-policy")?;
    let s = &x["submission"];
    shape(s, "proposal selection")?;
    let max = p["limits"]["maximumSubmissionBytes"]
        .as_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    require(
        crate::canonical::canonical_json(s)?.len() <= max,
        "host-submission-bound",
    )?;
    let proposal = &s["proposal"];
    let selection = &s["selection"];
    require(
        proposal["purpose"] == "proposal" && selection["purpose"] == "attestation",
        "invalid-host-selection",
    )?;
    crate::identity::authenticate(&x["identityProfile"], &x["expectedTempleId"], proposal)?;
    crate::identity::authenticate(&x["identityProfile"], &x["expectedTempleId"], selection)?;
    require(
        selection["payload"]["actor"] == proposal["payload"]["actor"]
            && selection["payload"]["slot"] == proposal["payload"]["requestId"],
        "invalid-host-selection",
    )?;
    Ok(selection["payload"]["statement"]
        == json!({"kind":"temple3.claim-lifecycle-host-selection/v0.1","hostPolicyId":pid,"proposalId":crate::canonical::identify("signed",proposal)?}))
}
pub fn verify(x: &Value) -> bool {
    checked(x).unwrap_or(false)
}
