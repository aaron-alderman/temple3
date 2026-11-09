use crate::{canonical, identity};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
const PROFILE: &str = "temple3.operational-monotonic-head-authority-profile/v0.1";
const STATE: &str = "temple3.operational-monotonic-head-authority-state/v0.1";
const REQUEST: &str = "temple3.operational-monotonic-head-client-request/v0.1";
const OBS: &str = "temple3.operational-monotonic-head-observation/v0.1";
type R<T> = Result<T, String>;
type ObservedRequests = BTreeMap<String, (String, String, Value)>;
fn fail<T>(c: &str) -> R<T> {
    Err(c.into())
}
fn shape(x: &Value, ks: &[&str], c: &str) -> R<()> {
    let Some(o) = x.as_object() else {
        return fail(c);
    };
    if o.len() != ks.len() || ks.iter().any(|k| !o.contains_key(*k)) {
        return fail(c);
    }
    Ok(())
}
fn text<'a>(x: &'a Value, k: &str) -> R<&'a str> {
    x[k].as_str().ok_or_else(|| "invalid".into())
}
fn tok(x: &Value) -> bool {
    x.as_str()
        .map(|v| !v.is_empty() && v.len() <= 128)
        .unwrap_or(false)
}
fn dec(x: &Value, pos: bool) -> bool {
    x.as_str()
        .map(|v| {
            let q = v == "0" || (!v.starts_with('0') && v.bytes().all(|c| c.is_ascii_digit()));
            q && (!pos || v != "0")
        })
        .unwrap_or(false)
}
fn dig_s(v: &str) -> bool {
    v.len() == 64
        && v.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn dig(x: &Value) -> bool {
    x.as_str().map(dig_s).unwrap_or(false)
}
fn id(k: &str, x: &Value) -> R<String> {
    canonical::identify(k, x).map_err(|_| "invalid".into())
}
fn flags() -> Map<String, Value> {
    serde_json::from_value(json!({"independentAuthority":false,"jointRollbackResistance":false,"readyForFQ20":false,"readyForFQ21":false,"readyForAF1":false,"readyForAF4":false,"readyForProduction":false,"sourceryAuthority":false,"authenticMigration":false})).unwrap()
}
fn with_flags(mut x: Map<String, Value>) -> Value {
    for (k, v) in flags() {
        x.insert(k, v);
    }
    Value::Object(x)
}
fn head(x: &Value) -> R<String> {
    let mut b = x.as_object().ok_or("invalid")?.clone();
    b.remove("headId");
    id("operational-monotonic-head", &Value::Object(b))
}
fn observation(e: &Value, p: &Value, c: &str) -> R<(Value, String)> {
    if canonical::canonical_json(e)
        .map_err(|_| c.to_string())?
        .len()
        > text(&p["bounds"], "maximumObservationBytes")?
            .parse::<usize>()
            .map_err(|_| c.to_string())?
    {
        return fail("operational-monotonic-head-observation-capacity");
    }
    if !identity::verify_envelope(e, "attestation", &p["authority"]["publicKey"]) {
        return fail(c);
    }
    let x = &e["payload"];
    shape(
        x,
        &[
            "kind",
            "authorityId",
            "subject",
            "domain",
            "sequence",
            "predecessorObservationId",
            "aggregateRollbackSetCommitment",
            "requestId",
            "requestIdentity",
            "headId",
        ],
        c,
    )?;
    let pred = x["predecessorObservationId"].is_null()
        || x["predecessorObservationId"]
            .as_str()
            .map(|v| {
                v.starts_with("operational-monotonic-head-observation:sha256:")
                    && dig_s(v.rsplit(':').next().unwrap_or(""))
            })
            .unwrap_or(false);
    let rq = x["requestId"].is_null() || tok(&x["requestId"]);
    let ri = x["requestIdentity"].is_null()
        || x["requestIdentity"]
            .as_str()
            .map(|v| {
                v.starts_with("operational-monotonic-head-client-request:sha256:")
                    && dig_s(v.rsplit(':').next().unwrap_or(""))
            })
            .unwrap_or(false);
    if text(x, "kind")? != OBS
        || x["authorityId"] != p["authority"]["authorityId"]
        || x["subject"] != p["subject"]
        || x["domain"] != p["domain"]
        || !dec(&x["sequence"], false)
        || text(x, "sequence")?.parse::<u64>().unwrap_or(u64::MAX)
            > text(&p["bounds"], "maximumSequence")?
                .parse::<u64>()
                .unwrap_or(0)
        || !pred
        || !dig(&x["aggregateRollbackSetCommitment"])
        || !rq
        || !ri
        || (x["requestId"].is_null() != x["requestIdentity"].is_null())
        || text(x, "headId")? != head(x)?
    {
        return fail(c);
    }
    Ok((x.clone(), id("operational-monotonic-head-observation", e)?))
}
pub fn validate_profile(p: &Value) -> R<()> {
    let c = "invalid-operational-monotonic-head-authority-profile";
    canonical::canonical_json(p).map_err(|_| c.to_string())?;
    shape(
        p,
        &[
            "kind",
            "authority",
            "client",
            "subject",
            "domain",
            "genesisTrustAnchor",
            "bounds",
            "ordering",
            "retention",
            "writer",
            "retry",
            "availability",
            "forgiveness",
            "authorityQualification",
            "independentAuthority",
        ],
        c,
    )?;
    shape(&p["authority"], &["authorityId", "publicKey"], c)?;
    shape(&p["client"], &["clientId", "publicKey"], c)?;
    shape(
        &p["genesisTrustAnchor"],
        &["observationId", "observation"],
        c,
    )?;
    shape(
        &p["bounds"],
        &[
            "maximumObservationBytes",
            "maximumRequestBytes",
            "maximumObservations",
            "maximumSequence",
        ],
        c,
    )?;
    shape(&p["forgiveness"], &["disposition", "scope"], c)?;
    let b = &p["bounds"];
    let ok = text(p, "kind")? == PROFILE
        && tok(&p["authority"]["authorityId"])
        && tok(&p["client"]["clientId"])
        && tok(&p["subject"])
        && tok(&p["domain"])
        && text(&p["authority"], "publicKey")?.len() == 64
        && text(&p["client"], "publicKey")?.len() == 64
        && [
            "maximumObservationBytes",
            "maximumRequestBytes",
            "maximumObservations",
            "maximumSequence",
        ]
        .iter()
        .all(|k| dec(&b[*k], true))
        && text(b, "maximumObservationBytes")?
            .parse::<u64>()
            .unwrap_or(u64::MAX)
            <= 65536
        && text(b, "maximumRequestBytes")?
            .parse::<u64>()
            .unwrap_or(u64::MAX)
            <= 65536
        && text(b, "maximumObservations")?
            .parse::<u64>()
            .unwrap_or(u64::MAX)
            <= 1024
        && text(b, "maximumSequence")?
            .parse::<u64>()
            .unwrap_or(u64::MAX)
            <= 1_000_000_000
        && p["ordering"] == "strict-next-sequence-exact-predecessor-cas"
        && p["retention"] == "complete-history-retain-all-no-pruning"
        && p["writer"] == "single-authority-writer"
        && p["retry"] == "no-automatic-append-retry-after-ambiguous"
        && p["availability"] == "explicit-unavailable-or-ambiguous-no-fallback"
        && p["forgiveness"]
            == json!({"disposition":"refuse","scope":"operational-monotonic-head-authority-v0.1"})
        && p["authorityQualification"]
            == "pure-state-machine-not-operated-or-independently-custodied"
        && p["independentAuthority"] == false;
    if !ok
        || identity::key_id(&p["authority"]["publicKey"]).is_err()
        || identity::key_id(&p["client"]["publicKey"]).is_err()
    {
        return fail(c);
    }
    let (g, gid) = observation(&p["genesisTrustAnchor"]["observation"], p, c)?;
    if g["sequence"] != "0"
        || !g["predecessorObservationId"].is_null()
        || !g["requestId"].is_null()
        || !g["requestIdentity"].is_null()
        || p["genesisTrustAnchor"]["observationId"] != gid
    {
        return fail(c);
    }
    Ok(())
}
pub fn profile_id(p: &Value) -> R<String> {
    validate_profile(p)?;
    id("operational-monotonic-head-authority-profile", p)
}
fn request(e: &Value, p: &Value) -> R<(Value, String)> {
    let c = "invalid-operational-monotonic-head-client-request";
    if canonical::canonical_json(e)
        .map_err(|_| c.to_string())?
        .len()
        > text(&p["bounds"], "maximumRequestBytes")?
            .parse::<usize>()
            .unwrap_or(0)
    {
        return fail("operational-monotonic-head-request-capacity");
    }
    if !identity::verify_envelope(e, "proposal", &p["client"]["publicKey"]) {
        return fail(c);
    }
    let x = &e["payload"];
    shape(
        x,
        &[
            "kind",
            "profileId",
            "clientId",
            "requestId",
            "expectedCurrentObservationId",
            "expectedCurrentSequence",
            "aggregateRollbackSetCommitment",
        ],
        c,
    )?;
    let oi = x["expectedCurrentObservationId"]
        .as_str()
        .map(|v| {
            v.starts_with("operational-monotonic-head-observation:sha256:")
                && dig_s(v.rsplit(':').next().unwrap_or(""))
        })
        .unwrap_or(false);
    if x["kind"] != REQUEST
        || text(x, "profileId")? != profile_id(p)?
        || x["clientId"] != p["client"]["clientId"]
        || !tok(&x["requestId"])
        || !oi
        || !dec(&x["expectedCurrentSequence"], false)
        || !dig(&x["aggregateRollbackSetCommitment"])
    {
        return fail(c);
    }
    Ok((
        x.clone(),
        id("operational-monotonic-head-client-request", e)?,
    ))
}
fn state(p: &Value, s: &Value, a: &Value) -> R<(Value, String, ObservedRequests)> {
    validate_profile(p)?;
    shape(
        a,
        &["expectedProfileId"],
        "invalid-operational-monotonic-head-authority-input",
    )?;
    let wanted = profile_id(p)?;
    if text(a, "expectedProfileId")? != wanted {
        return fail("wrong-operational-monotonic-head-authority-profile");
    }
    let c = "invalid-operational-monotonic-head-authority-state";
    shape(s, &["kind", "profileId", "history"], c)?;
    let Some(h) = s["history"].as_array() else {
        return fail(c);
    };
    if s["kind"] != STATE
        || text(s, "profileId")? != wanted
        || h.is_empty()
        || h.len()
            > text(&p["bounds"], "maximumObservations")?
                .parse::<usize>()
                .unwrap_or(0)
    {
        return fail(c);
    }
    let (mut prior, mut prid) =
        observation(&h[0], p, "invalid-operational-monotonic-head-observation")?;
    if prid != p["genesisTrustAnchor"]["observationId"]
        || h[0] != p["genesisTrustAnchor"]["observation"]
    {
        return fail("operational-monotonic-head-history-genesis-mismatch");
    }
    let mut found = BTreeMap::new();
    for e in &h[1..] {
        let (n, nid) = observation(e, p, "invalid-operational-monotonic-head-observation")?;
        if text(&n, "sequence")?.parse::<u64>().unwrap()
            != text(&prior, "sequence")?.parse::<u64>().unwrap() + 1
        {
            return fail("operational-monotonic-head-history-gap-or-reorder");
        }
        if n["predecessorObservationId"] != prid {
            return fail("operational-monotonic-head-history-fork");
        }
        let r = text(&n, "requestId")?.to_string();
        if found.contains_key(&r) {
            return fail("operational-monotonic-head-history-duplicate-request-id");
        }
        found.insert(
            r,
            (text(&n, "requestIdentity")?.into(), nid.clone(), e.clone()),
        );
        prior = n;
        prid = nid
    }
    Ok((prior, prid, found))
}
#[allow(dead_code)]
pub fn form_state(p: &Value, a: &Value) -> R<Value> {
    let wanted = profile_id(p)?;
    let s = json!({"kind":STATE,"profileId":wanted,"history":[p["genesisTrustAnchor"]["observation"].clone()]});
    state(p, &s, a)?;
    Ok(s)
}
fn derive(p: &Value, s: &Value, re: &Value, a: &Value) -> R<Value> {
    let (t, tid, found) = state(p, s, a)?;
    let (r, ri) = request(re, p)?;
    let rq = text(&r, "requestId")?;
    if let Some((stored, oi, o)) = found.get(rq) {
        if stored != &ri {
            return fail("operational-monotonic-head-request-id-reuse");
        }
        return Ok(
            json!({"outcome":"already-committed","request":{"envelope":re,"payload":r,"id":ri},"retained":{"requestId":rq,"requestIdentity":stored,"observationId":oi,"observation":o}}),
        );
    }
    if r["expectedCurrentObservationId"] != tid || r["expectedCurrentSequence"] != t["sequence"] {
        return Ok(
            json!({"outcome":"conflict","current":{"observationId":tid,"sequence":t["sequence"]},"mutation":"none"}),
        );
    }
    let h = s["history"].as_array().unwrap();
    if h.len() + 1
        > text(&p["bounds"], "maximumObservations")?
            .parse::<usize>()
            .unwrap()
    {
        return fail("operational-monotonic-head-history-capacity");
    }
    if text(&t, "sequence")?.parse::<u64>().unwrap() + 1
        > text(&p["bounds"], "maximumSequence")?
            .parse::<u64>()
            .unwrap()
    {
        return fail("operational-monotonic-head-sequence-capacity");
    }
    let mut base = json!({"kind":OBS,"authorityId":p["authority"]["authorityId"],"subject":p["subject"],"domain":p["domain"],"sequence":(text(&t,"sequence")?.parse::<u64>().unwrap()+1).to_string(),"predecessorObservationId":tid,"aggregateRollbackSetCommitment":r["aggregateRollbackSetCommitment"],"requestId":r["requestId"],"requestIdentity":ri});
    let hd = head(&base)?;
    base["headId"] = json!(hd);
    Ok(
        json!({"outcome":"append-candidate","request":{"envelope":re,"payload":r,"id":ri},"payload":base}),
    )
}
pub fn append(p: &Value, s: &Value, re: &Value, proposed: &Value, a: &Value) -> R<Value> {
    let d = derive(p, s, re, a)?;
    if d["outcome"] == "conflict" {
        return Ok(with_flags(json!({"kind":"temple3.operational-monotonic-head-authority-assessment/v0.1","status":"conflict","current":d["current"],"state":s,"mutation":"none","automaticRetry":false}).as_object().unwrap().clone()));
    }
    if d["outcome"] == "already-committed" {
        return Ok(with_flags(json!({"kind":"temple3.operational-monotonic-head-authority-assessment/v0.1","status":"already-committed","observationId":d["retained"]["observationId"],"observation":d["retained"]["observation"],"state":s,"mutation":"none","automaticRetry":false}).as_object().unwrap().clone()));
    }
    let (o, oi) = observation(
        proposed,
        p,
        "invalid-operational-monotonic-head-observation",
    )?;
    if o != d["payload"] {
        return fail("operational-monotonic-head-proposed-observation-mismatch");
    }
    let mut ns = s.clone();
    ns["history"].as_array_mut().unwrap().push(proposed.clone());
    state(p, &ns, a)?;
    Ok(with_flags(json!({"kind":"temple3.operational-monotonic-head-authority-assessment/v0.1","status":"committed","observationId":oi,"observation":proposed,"state":ns,"mutation":"append-one","automaticRetry":false}).as_object().unwrap().clone()))
}
pub fn query(p: &Value, s: &Value, a: &Value) -> R<Value> {
    let (t, ti, _) = state(p, s, a)?;
    Ok(with_flags(json!({"kind":"temple3.operational-monotonic-head-authority-query/v0.1","status":"current-complete-history","profileId":s["profileId"],"observationId":ti,"sequence":t["sequence"],"history":s["history"],"mutation":"none"}).as_object().unwrap().clone()))
}
pub fn reconcile(p: &Value, s: &Value, re: &Value, a: &Value) -> R<Value> {
    let (_, _, found) = state(p, s, a)?;
    let (r, ri) = request(re, p)?;
    let z = found.get(text(&r, "requestId")?);
    if let Some((stored, _, _)) = z {
        if stored != &ri {
            return fail("operational-monotonic-head-request-id-reuse");
        }
    }
    Ok(with_flags(json!({"kind":"temple3.operational-monotonic-head-authority-reconciliation/v0.1","status":if z.is_some(){"already-committed"}else{"conflict"},"reason":if z.is_some(){"exact-request-retained"}else{"request-not-committed"},"observationId":z.map(|x|Value::String(x.1.clone())).unwrap_or(Value::Null),"observation":z.map(|x|x.2.clone()).unwrap_or(Value::Null),"state":s,"mutation":"none","automaticRetry":false}).as_object().unwrap().clone()))
}
pub fn transport(p: &Value, s: &Value, r: &Value, a: &Value) -> R<Value> {
    state(p, s, a)?;
    let c = "invalid-operational-monotonic-head-transport-outcome";
    shape(r, &["kind", "outcome", "reason"], c)?;
    let o = text(r, "outcome")?;
    if r["kind"] != "temple3.operational-monotonic-head-transport-response/v0.1"
        || !(o == "unavailable" || o == "ambiguous")
        || r["reason"]
            != (if o == "unavailable" {
                "dependency-unavailable"
            } else {
                "outcome-unknown-reconcile-required"
            })
    {
        return fail(c);
    }
    Ok(with_flags(json!({"kind":"temple3.operational-monotonic-head-authority-assessment/v0.1","status":o,"state":s,"mutation":if o=="unavailable"{"none-known"}else{"unknown"},"automaticRetry":false,"reconciliationRequired":o=="ambiguous"}).as_object().unwrap().clone()))
}
pub fn invoke(x: &Value) -> R<Value> {
    let raw = x["operation"].as_str().unwrap_or("");
    let pure = raw.ends_with("-purity");
    let op = raw.strip_suffix("-purity").unwrap_or(raw);
    let mut owned = x.clone();
    let result = match op {
        "append" => append(
            &owned["profile"],
            &owned["state"],
            &owned["request"],
            &owned["proposedObservation"],
            &owned["authority"],
        ),
        "query" => query(&owned["profile"], &owned["state"], &owned["authority"]),
        "reconcile" => reconcile(
            &owned["profile"],
            &owned["state"],
            &owned["request"],
            &owned["authority"],
        ),
        "transport" => transport(
            &owned["profile"],
            &owned["state"],
            &owned["response"],
            &owned["authority"],
        ),
        _ => fail("invalid-operational-monotonic-head-invocation"),
    };
    if pure {
        owned["state"]["history"] = json!([]);
        owned["request"] = Value::Null;
        owned["proposedObservation"] = Value::Null;
        owned["response"] = Value::Null;
    }
    result
}
