//! Independent signed representative-composition-profile/v0.2 recognition.
use serde_json::{Value, json};
const CLOSURE_ID: &str = "application-contract-closure:sha256:330ccca4874e582d6ecd1a240c2532b8d73c6f5e66fcfacd2e4dc84b484f4a17";
fn predicate() -> Value {
    json!({"op":"all","values":[crate::representative_composition_profile::predicate(),{"op":"eq","left":{"op":"read","path":["context","statement","publicInputs","after","components","3"]},"right":{"op":"literal","value":null}},{"op":"eq","left":{"op":"read","path":["context","statement","publicInputs","after","links","2"]},"right":{"op":"literal","value":null}}]})
}
pub fn expected_input(d: &Value) -> crate::Result<Value> {
    crate::canonical::canonical_json(d)?;
    let keys = d
        .as_object()
        .ok_or_else(|| crate::Error {
            code: "invalid-representative-composition-profile-v02".into(),
        })?
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let expected = "arkId contractClosure coordinationProfile formation identityProfile"
        .split_whitespace()
        .collect::<std::collections::BTreeSet<_>>();
    if keys != expected {
        return Err(crate::Error {
            code: "invalid-representative-composition-profile-v02".into(),
        });
    }
    let initial =
        crate::typed_composition::representative_formation(&d["formation"]).map_err(|_| {
            crate::Error {
                code: "invalid-representative-composition-profile-v02".into(),
            }
        })?;
    let cid = crate::canonical::identify("application-contract-closure", &d["contractClosure"])?;
    if cid != CLOSURE_ID
        || d["contractClosure"]["kind"] != "temple3.application-contract-closure/v0.1"
        || d["arkId"].as_str().is_none_or(str::is_empty)
    {
        return Err(crate::Error {
            code: "invalid-representative-composition-profile-v02".into(),
        });
    }
    let actors = d["identityProfile"]["actors"]
        .as_array()
        .ok_or_else(|| crate::Error {
            code: "invalid-representative-composition-profile-v02".into(),
        })?;
    let names: Vec<_> = actors.iter().map(|a| a["actor"].clone()).collect();
    if json!(names) != initial["participants"] {
        return Err(crate::Error {
            code: "invalid-representative-composition-profile-v02".into(),
        });
    }
    for (i, a) in actors.iter().enumerate() {
        if actors[..i].iter().any(|b| a["publicKey"] == b["publicKey"]) {
            return Err(crate::Error {
                code: "invalid-representative-composition-profile-v02".into(),
            });
        }
    }
    Ok(
        json!({"arkId":d["arkId"],"activePredicate":predicate(),"initialValues":[initial],"context":{"application":{"application":{"formation":d["formation"],"contractClosure":d["contractClosure"],"contractClosureId":CLOSURE_ID},"coordinationProfile":d["coordinationProfile"]},"identityProfile":d["identityProfile"]}}),
    )
}
pub fn recognize(genesis: &Value, d: &Value, anchor: &Value) -> crate::Result<Value> {
    if anchor.as_str().is_none_or(str::is_empty) {
        return Err(crate::Error {
            code: "missing-representative-composition-v02-trust-anchor".into(),
        });
    }
    if genesis["signed"]["payload"]["genesisInput"] != expected_input(d)? {
        return Err(crate::Error {
            code: "invalid-representative-composition-profile-v02".into(),
        });
    }
    crate::coordination::initial(genesis, anchor)
}
