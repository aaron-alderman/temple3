//! Pure claim-import authenticated rollback-set evaluator v0.1.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
pub type Result<T> = std::result::Result<T, String>;
const PROFILE: &str = "temple3.claim-import-authenticated-rollback-set-profile/v0.1";
const KIND: &str = "temple3.claim-import-authenticated-rollback-set/v0.1";
const ROLES: [(&str, &str); 4] = [
    (
        "C",
        "temple3.claim-import-coordination-journal-snapshot/v0.1",
    ),
    ("R", "temple3.claim-import-request-journal-snapshot/v0.1"),
    (
        "K",
        "temple3.claim-import-retained-checkpoint-journal-snapshot/v0.1",
    ),
    (
        "A",
        "temple3.claim-import-caller-recovery-bundle-snapshot/v0.1",
    ),
];
fn fail<T>(code: &str) -> Result<T> {
    Err(code.into())
}
fn require(ok: bool, code: &str) -> Result<()> {
    if ok { Ok(()) } else { fail(code) }
}
fn shape(v: &Value, fields: &str, code: &str) -> Result<()> {
    let expected: BTreeSet<_> = fields.split_whitespace().collect();
    require(
        v.as_object()
            .is_some_and(|x| x.keys().map(String::as_str).collect::<BTreeSet<_>>() == expected),
        code,
    )
}
fn dec(v: &Value) -> bool {
    v.as_str()
        .is_some_and(|s| s == "0" || (!s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit())))
}
fn digest(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn canonical(v: &Value) -> Result<String> {
    temple3::canonical::canonical_json(v).map_err(|_| "noncanonical-value".into())
}
fn identify(kind: &str, v: &Value) -> Result<String> {
    temple3::canonical::identify(kind, v).map_err(|_| "invalid-identity".into())
}
fn profile(v: &Value) -> Result<()> {
    let code = "invalid-claim-import-authenticated-rollback-set-profile";
    canonical(v)?;
    shape(
        v,
        "kind roles maximumAggregateBytes encoding empty retention discovery forgiveness",
        code,
    )?;
    shape(&v["forgiveness"], "disposition scope", code)?;
    let Some(roles) = v["roles"].as_array() else {
        return fail(code);
    };
    require(
        v["kind"] == PROFILE
            && roles.len() == 4
            && dec(&v["maximumAggregateBytes"])
            && v["maximumAggregateBytes"] != "0"
            && v["maximumAggregateBytes"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap_or(u64::MAX)
                <= 16777216
            && v["encoding"] == "lowercase-hex"
            && v["empty"] == "explicit-present-empty-allowed-absence-or-null-refused"
            && v["retention"] == "retain-exact-member-bytes-no-pruning"
            && v["discovery"] == "caller-pinned-exact-role-and-snapshot-identities-no-default"
            && v["forgiveness"]
                == json!({"disposition":"refuse","scope":"claim-import-authenticated-rollback-set-v0.1"}),
        code,
    )?;
    for (i, r) in roles.iter().enumerate() {
        shape(r, "role semanticKind maximumBytes", code)?;
        require(
            r["role"] == ROLES[i].0
                && r["semanticKind"] == ROLES[i].1
                && dec(&r["maximumBytes"])
                && r["maximumBytes"] != "0"
                && r["maximumBytes"]
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap_or(u64::MAX)
                    <= 4194304,
            code,
        )?
    }
    Ok(())
}
pub fn profile_id(v: &Value) -> Result<String> {
    profile(v)?;
    identify("claim-import-authenticated-rollback-set-profile", v)
}
pub fn derive_material(input: &Value, authority: &Value) -> Result<Value> {
    let code = "invalid-claim-import-authenticated-rollback-set";
    canonical(input)?;
    canonical(authority)?;
    shape(input, "profile members", code)?;
    shape(
        authority,
        "expectedProfileId expectedMembers",
        "invalid-claim-import-authenticated-rollback-set-authority",
    )?;
    let Some(ms) = input["members"].as_array() else {
        return fail(code);
    };
    let Some(es) = authority["expectedMembers"].as_array() else {
        return fail(code);
    };
    require(ms.len() == 4 && es.len() == 4, code)?;
    let p = &input["profile"];
    let pid = profile_id(p)?;
    require(
        authority["expectedProfileId"] == pid,
        "claim-import-rollback-set-profile-substitution-refused",
    )?;
    let mut out = Vec::new();
    let mut total = 0u64;
    for i in 0..4 {
        let m = &ms[i];
        let e = &es[i];
        let role = &p["roles"][i];
        shape(
            m,
            "role semanticKind present encoding bytesHex byteCount sha256",
            code,
        )?;
        shape(
            e,
            "role semanticKind byteCount sha256",
            "invalid-claim-import-authenticated-rollback-set-authority",
        )?;
        let hex_text = m["bytesHex"].as_str().ok_or_else(|| code.to_string())?;
        require(
            m["role"] == role["role"]
                && m["semanticKind"] == role["semanticKind"]
                && e["role"] == role["role"]
                && e["semanticKind"] == role["semanticKind"]
                && m["present"] == true
                && m["encoding"] == "lowercase-hex"
                && hex_text.len() % 2 == 0
                && hex_text
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                && dec(&m["byteCount"])
                && digest(&m["sha256"])
                && dec(&e["byteCount"])
                && digest(&e["sha256"]),
            code,
        )?;
        let bytes = hex::decode(hex_text).map_err(|_| code.to_string())?;
        let count = bytes.len() as u64;
        let sha = hash(&bytes);
        require(
            m["byteCount"].as_str().unwrap().parse::<u64>().unwrap() == count
                && m["sha256"] == sha
                && e["byteCount"] == m["byteCount"]
                && e["sha256"] == m["sha256"],
            "claim-import-rollback-set-member-identity-refused",
        )?;
        require(
            count
                <= role["maximumBytes"]
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
            "claim-import-rollback-set-member-capacity",
        )?;
        total = total
            .checked_add(count)
            .ok_or_else(|| "claim-import-rollback-set-aggregate-capacity".to_string())?;
        out.push(json!({"role":m["role"],"semanticKind":m["semanticKind"],"byteCount":m["byteCount"],"sha256":m["sha256"]}))
    }
    require(
        total
            <= p["maximumAggregateBytes"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap(),
        "claim-import-rollback-set-aggregate-capacity",
    )?;
    let material = json!({"kind":"temple3.claim-import-rollback-set-commitment-material/v0.1","domain":"temple3.claim-import.rollback-set/v0.1","profileId":pid,"members":out});
    let commitment = hash(canonical(&material)?.as_bytes());
    Ok(
        json!({"kind":"temple3.claim-import-authenticated-rollback-set-assessment/v0.1","status":"accepted-caller-pinned-exact-constituent-derivation-independent-authority-false","profileId":pid,"members":material["members"],"totalBytes":total.to_string(),"commitmentMaterial":material,"aggregateRollbackSetCommitment":commitment,"aggregateDerivationEstablished":true,"constituentCompleteness":"caller-pinned-exact-role-and-snapshot-identities-only","independentAuthority":false,"jointRollbackResistance":false,"readyForFQ20":false,"readyForFQ21":false,"readyForAF1":false,"readyForAF4":false,"readyForProduction":false,"sourceryAuthority":false,"authenticMigration":false,"effects":{"reads":"0","writes":"0","dispatches":"0","activations":"0"},"refusal":{"code":"independent-production-freshness-authority-and-custody-unavailable","missing":["independently-operated-and-custodied-monotonic-freshness-root","production-checkpoint-key-and-client-custody","hostile-filesystem-power-loss-and-remote-recovery","authentic-sourcery-runtime-export-and-capture-authority"]},"forgiveness":p["forgiveness"]}),
    )
}
pub fn evaluate(input: &Value, authority: &Value) -> Result<Value> {
    let code = "invalid-claim-import-authenticated-rollback-set";
    canonical(input)?;
    shape(
        input,
        "kind profile members proposedAggregateRollbackSetCommitment",
        code,
    )?;
    require(
        input["kind"] == KIND && digest(&input["proposedAggregateRollbackSetCommitment"]),
        code,
    )?;
    let result = derive_material(
        &json!({"profile":input["profile"],"members":input["members"]}),
        authority,
    )?;
    require(
        result["aggregateRollbackSetCommitment"] == input["proposedAggregateRollbackSetCommitment"],
        "claim-import-rollback-set-bridge-commitment-mismatch",
    )?;
    Ok(result)
}
