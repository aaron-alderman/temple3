//! Independent supplied Sourcery /3 authority-bundle recognition; never product admission.
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

fn failure(code: &str) -> String {
    code.to_string()
}
fn need(ok: bool, code: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(failure(code)) }
}
fn exact<'a>(value: &'a Value, fields: &str, code: &str) -> Result<&'a Map<String, Value>, String> {
    let map = value.as_object().ok_or_else(|| failure(code))?;
    let expected: Vec<&str> = fields.split_whitespace().collect();
    need(
        map.len() == expected.len() && map.keys().all(|key| expected.contains(&key.as_str())),
        code,
    )?;
    Ok(map)
}
fn text<'a>(value: &'a Value, field: &str, code: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| failure(code))
}
fn hash_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn product_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let mut encoded =
        serde_json::to_string_pretty(value).map_err(|_| failure("relnav-v3-canonical"))?;
    encoded.push('\n');
    Ok(encoded.into_bytes())
}
fn hash(value: &Value) -> Result<String, String> {
    Ok(hash_bytes(&product_bytes(value)?))
}
fn content_id(kind: &str, value: &Value) -> Result<String, String> {
    Ok(format!("{kind}:sha256:{}", hash(value)?))
}
fn is_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
fn subject(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_.:/#@+$-".contains(&byte))
}
fn identity(value: &str) -> bool {
    let Some((kind, digest)) = value.split_once(":sha256:") else {
        return false;
    };
    !kind.is_empty()
        && kind.as_bytes()[0].is_ascii_lowercase()
        && kind
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte))
        && is_hash(digest)
}
fn decimal(value: &str, code: &str) -> Result<u64, String> {
    need(
        !value.is_empty()
            && value.len() <= 16
            && (value == "0"
                || (value.as_bytes()[0] != b'0'
                    && value.bytes().all(|byte| byte.is_ascii_digit()))),
        code,
    )?;
    value.parse::<u64>().map_err(|_| failure(code))
}
fn legacy_mutation(value: &str) -> bool {
    let Some(uuid) = value.strip_prefix("mutation:") else {
        return false;
    };
    let parts: Vec<&str> = uuid.split('-').collect();
    if parts.len() != 5
        || [8, 4, 4, 4, 12]
            .iter()
            .enumerate()
            .any(|(i, len)| parts[i].len() != *len)
    {
        return false;
    }
    if !parts.iter().all(|part| {
        part.bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    }) {
        return false;
    }
    matches!(parts[2].as_bytes()[0], b'1'..=b'5')
        && matches!(parts[3].as_bytes()[0], b'8' | b'9' | b'a' | b'b')
}

pub fn recognize(input: &Value) -> Result<Value, String> {
    exact(
        input,
        "authorityLifecyclePin expected kind policy source",
        "relnav-v3-bundle-input-shape",
    )?;
    need(
        text(input, "kind", "relnav-v3-bundle-input-kind")?
            == "temple3.runtime-capture-v3-authority-bundle-input/v0.1",
        "relnav-v3-bundle-input-kind",
    )?;
    let expected = &input["expected"];
    exact(
        expected,
        "admissionHash application binding runtimeHeadId sourceBytes sourceSha256 store",
        "relnav-v3-expected-shape",
    )?;
    need(
        ["application", "binding", "store"]
            .iter()
            .all(|field| text(expected, field, "relnav-v3-expected-shape").is_ok_and(subject))
            && text(expected, "admissionHash", "relnav-v3-expected-shape").is_ok_and(is_hash)
            && text(expected, "runtimeHeadId", "relnav-v3-expected-shape")
                .is_ok_and(|v| v.starts_with("runtime-record-head:sha256:") && identity(v))
            && decimal(
                text(expected, "sourceBytes", "relnav-v3-expected-shape")?,
                "relnav-v3-expected-shape",
            )? <= 1_048_576
            && text(expected, "sourceSha256", "relnav-v3-expected-shape").is_ok_and(is_hash),
        "relnav-v3-expected-shape",
    )?;
    let policy = &input["policy"];
    exact(
        policy,
        "application authority backout binding completeness forgiveness framing kind maximumBytes maximumRecords recovery retention retry roles signer store",
        "relnav-v3-policy-shape",
    )?;
    need(
        text(policy, "kind", "relnav-v3-policy-selection")?
            == "sourcery.runtime-record-capture-policy/3"
            && ["application", "binding", "store"]
                .iter()
                .all(|field| policy[*field] == expected[*field]),
        "relnav-v3-policy-selection",
    )?;
    let roles = &policy["roles"];
    let role_map = exact(
        roles,
        "capture custody exporter governor reader transport writer",
        "relnav-v3-roles",
    )?;
    need(
        role_map
            .values()
            .all(|role| role.as_str().is_some_and(subject)),
        "relnav-v3-roles",
    )?;
    let authority = &policy["authority"];
    exact(
        authority,
        "grantHash grantId operation revision revocationStateHash",
        "relnav-v3-authority",
    )?;
    need(
        text(authority, "operation", "relnav-v3-authority")? == "capture"
            && text(authority, "grantId", "relnav-v3-authority").is_ok_and(identity)
            && text(authority, "grantHash", "relnav-v3-authority").is_ok_and(is_hash)
            && text(authority, "revocationStateHash", "relnav-v3-authority").is_ok_and(is_hash)
            && text(authority, "revision", "relnav-v3-authority").is_ok_and(|v| {
                !v.is_empty()
                    && v.as_bytes()[0] != b'0'
                    && v.bytes().all(|byte| byte.is_ascii_digit())
            }),
        "relnav-v3-authority",
    )?;
    let forgiveness = &policy["forgiveness"];
    exact(forgiveness, "disposition scope", "relnav-v3-forgiveness")?;
    need(
        forgiveness == &json!({"disposition":"refuse","scope":"runtime-record-capture-v2"}),
        "relnav-v3-forgiveness",
    )?;
    let signer = &policy["signer"];
    exact(
        signer,
        "algorithm binding bindingReceiptHash custodyId keyId publicKey",
        "relnav-v3-signer",
    )?;
    let raw = text(signer, "publicKey", "relnav-v3-signer")?;
    let der = if is_hash(raw) {
        let mut der =
            hex::decode("302a300506032b6570032100").map_err(|_| failure("relnav-v3-signer"))?;
        der.extend(hex::decode(raw).map_err(|_| failure("relnav-v3-signer"))?);
        der
    } else {
        Vec::new()
    };
    need(
        text(signer, "algorithm", "relnav-v3-signer")? == "ed25519"
            && text(signer, "binding", "relnav-v3-signer").is_ok_and(subject)
            && text(signer, "custodyId", "relnav-v3-signer").is_ok_and(subject)
            && text(signer, "bindingReceiptHash", "relnav-v3-signer").is_ok_and(is_hash)
            && is_hash(raw)
            && text(signer, "keyId", "relnav-v3-signer")? == hash_bytes(&der),
        "relnav-v3-signer",
    )?;
    need(
        text(policy, "framing", "relnav-v3-policy-mode")?
            == "single-complete-store-opaque-bytes-v1"
            && text(policy, "completeness", "relnav-v3-policy-mode")?
                == "externally-verified-entire-selected-store-at-head-v1"
            && text(policy, "retention", "relnav-v3-policy-mode")?
                == "core-does-not-retain-host-must-retain-before-success-v1"
            && text(policy, "retry", "relnav-v3-policy-mode")?
                == "core-does-not-retry-host-must-replay-exact-request-hash-or-refuse-v1"
            && text(policy, "recovery", "relnav-v3-policy-mode")?
                == "core-does-not-recover-host-must-prove-exact-artifacts-or-refuse-v1"
            && text(policy, "backout", "relnav-v3-policy-mode")?
                == "core-never-mutates-source-host-governs-unpublished-artifact-disposition-v1"
            && decimal(
                text(policy, "maximumRecords", "relnav-v3-capacity")?,
                "relnav-v3-capacity",
            )? == 1
            && (1..=1_050_000).contains(&decimal(
                text(policy, "maximumBytes", "relnav-v3-capacity")?,
                "relnav-v3-capacity",
            )?),
        "relnav-v3-policy-mode",
    )?;
    let policy_hash = hash(policy)?;
    let pin = &input["authorityLifecyclePin"];
    exact(
        pin,
        "grantHash grantId operation revision revocationStateHash",
        "relnav-v3-lifecycle-pin",
    )?;
    need(
        product_bytes(pin)? == product_bytes(authority)?,
        "relnav-v3-lifecycle-pin",
    )?;
    let source = &input["source"];
    exact(
        source,
        "admissionHash application completeness runtimeHead store",
        "relnav-v3-source-shape",
    )?;
    let head = &source["runtimeHead"];
    let head_map = exact(
        head,
        "headId kind lastMutationId revision sourceBytes sourceSha256 store",
        "relnav-v3-head-shape",
    )?;
    let completeness = &source["completeness"];
    exact(
        completeness,
        "empty evidenceHash scope status",
        "relnav-v3-completeness",
    )?;
    let mut body = head_map.clone();
    body.remove("headId");
    let head_id = text(head, "headId", "relnav-v3-head")?;
    let mutation = text(head, "lastMutationId", "relnav-v3-head")?;
    need(
        source["application"] == policy["application"]
            && source["store"] == policy["store"]
            && text(source, "admissionHash", "relnav-v3-head").is_ok_and(is_hash)
            && text(head, "kind", "relnav-v3-head")? == "sourcery.runtime-record-head/1"
            && head["store"] == source["store"]
            && head_id == content_id("runtime-record-head", &Value::Object(body))?
            && head_id == text(expected, "runtimeHeadId", "relnav-v3-head")?
            && head["sourceBytes"] == expected["sourceBytes"]
            && head["sourceSha256"] == expected["sourceSha256"]
            && decimal(text(head, "revision", "relnav-v3-head")?, "relnav-v3-head").is_ok()
            && decimal(
                text(head, "sourceBytes", "relnav-v3-head")?,
                "relnav-v3-head",
            )? <= 1_048_576
            && text(head, "sourceSha256", "relnav-v3-head").is_ok_and(is_hash)
            && (mutation == "none" || identity(mutation) || legacy_mutation(mutation)),
        "relnav-v3-head",
    )?;
    need(
        text(completeness, "status", "relnav-v3-completeness")? == "externally-verified-complete"
            && text(completeness, "scope", "relnav-v3-completeness")?
                == "entire-selected-store-at-exact-head"
            && text(completeness, "evidenceHash", "relnav-v3-completeness").is_ok_and(is_hash)
            && completeness["empty"]
                .as_bool()
                .is_some_and(|v| v == (head["sourceBytes"] == "0")),
        "relnav-v3-completeness",
    )?;
    need(
        source["admissionHash"] == expected["admissionHash"],
        "relnav-v3-source-binding",
    )?;
    Ok(json!({"disposition":"recognized","policyHash":policy_hash,
        "expectedBindingHash":hash(expected)?,"runtimeHeadId":head_id,
        "grantId":authority["grantId"],"productProvenanceEstablished":false}))
}
