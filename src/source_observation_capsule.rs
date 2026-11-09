use serde_json::{Value, json};
use sha2::{Digest, Sha256};
type R<T> = Result<T, String>;
fn need(ok: bool) -> R<()> {
    if ok {
        Ok(())
    } else {
        Err("invalid-capsule".into())
    }
}
fn shape(v: &Value, fields: &str) -> R<()> {
    let obj = v.as_object().ok_or("object-required")?;
    let names: Vec<_> = fields.split_whitespace().collect();
    need(obj.len() == names.len() && names.iter().all(|k| obj.contains_key(*k)))
}
fn canonical(v: &Value) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|e| format!("{e:?}"))
}
fn id(kind: &str, v: &Value) -> R<String> {
    crate::canonical::identify(kind, v).map_err(|e| format!("{e:?}"))
}
fn text(v: &Value) -> R<&str> {
    v.as_str().ok_or("string-required".into())
}
fn count(v: &Value, zero: bool) -> R<u64> {
    let s = text(v)?;
    need(
        !s.is_empty()
            && s.len() <= 16
            && s.bytes().all(|b| b.is_ascii_digit())
            && (s == "0" && zero || !s.starts_with('0')),
    )?;
    let n = s.parse::<u64>().map_err(|_| "invalid-count")?;
    need(n <= 9_007_199_254_740_991)?;
    Ok(n)
}
fn sha(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
pub fn inspect(x: &Value) -> R<Value> {
    shape(
        x,
        "capsule policy expectedPolicyId expectedBindingId expectedContentSha256",
    )?;
    let c = &x["capsule"];
    let p = &x["policy"];
    canonical(c)?;
    canonical(p)?;
    shape(
        p,
        "kind maximumRawBytes maximumEncodedBytes forgiveness retention disclosure activation",
    )?;
    need(p["kind"] == "temple3.source-observation-capsule-policy/v0.1")?;
    let raw_limit = count(&p["maximumRawBytes"], false)?;
    let encoded_limit = count(&p["maximumEncodedBytes"], false)?;
    need(
        p["forgiveness"] == json!({"disposition":"refuse","scope":"capsule-and-inspection"})
            && p["retention"] == "retain-exact-capsule"
            && p["disclosure"] == "trusted-inspector-only"
            && p["activation"] == "unavailable",
    )?;
    need(id("source-observation-capsule-policy", p)? == x["expectedPolicyId"])?;
    shape(c, "kind binding encoding rawBytes contentSha256 payload")?;
    need(
        c["kind"] == "temple3.source-observation-capsule/v0.1" && c["encoding"] == "lowercase-hex",
    )?;
    let b = &c["binding"];
    shape(
        b,
        "kind application admittedArtifactHash evidenceKind runtime",
    )?;
    need(
        b["kind"] == "temple3.source-observation-capsule-binding/v0.1"
            && !text(&b["application"])?.is_empty()
            && sha(&b["admittedArtifactHash"]),
    )?;
    need(
        b["evidenceKind"] == "tool-observation-projection"
            && b["runtime"] == json!({"availability":"unavailable"}),
    )?;
    need(id("source-observation-capsule-binding", b)? == x["expectedBindingId"])?;
    let length = count(&c["rawBytes"], true)?;
    need(length <= raw_limit)?;
    let encoded = canonical(c)?.len() as u64;
    need(encoded <= encoded_limit)?;
    let payload = text(&c["payload"])?;
    need(
        payload.len() as u64 == 2 * length
            && payload
                .bytes()
                .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v)),
    )?;
    let raw = hex::decode(payload).map_err(|_| "invalid-hex")?;
    let hash = hex::encode(Sha256::digest(&raw));
    need(
        sha(&c["contentSha256"])
            && sha(&x["expectedContentSha256"])
            && hash == c["contentSha256"]
            && c["contentSha256"] == x["expectedContentSha256"],
    )?;
    Ok(
        json!({"kind":"temple3.source-observation-capsule-inspection/v0.1","capsuleId":id("source-observation-capsule",c)?,"policyId":x["expectedPolicyId"],"bindingId":x["expectedBindingId"],"contentSha256":c["contentSha256"],"rawBytes":c["rawBytes"],"encodedBytes":encoded.to_string(),"content":"exact-bytes-verified","sourceAuthority":"not-verified","runtime":"unavailable","activation":"unavailable"}),
    )
}
