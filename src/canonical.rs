use crate::{Result, err, require};
use serde_json::Value;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

/// Protocol JSON: UTF-8 sorted object keys, NFC strings, and no numbers.
pub fn canonical_json(value: &Value) -> Result<String> {
    fn write(v: &Value, out: &mut String) -> Result<()> {
        match v {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::String(s) => {
                require(s.nfc().eq(s.chars()), "noncanonical-value")?;
                out.push_str(&serde_json::to_string(s).map_err(|_| err("noncanonical-value"))?);
            }
            Value::Array(a) => {
                out.push('[');
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(v, out)?;
                }
                out.push(']');
            }
            Value::Object(m) => {
                let mut keys: Vec<_> = m.keys().collect();
                keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
                out.push('{');
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(&Value::String((*k).clone()), out)?;
                    out.push(':');
                    write(&m[*k], out)?;
                }
                out.push('}');
            }
            Value::Number(_) => return Err(err("noncanonical-value")),
        }
        Ok(())
    }
    let mut out = String::new();
    write(value, &mut out)?;
    Ok(out)
}
pub fn identify(kind: &str, value: &Value) -> Result<String> {
    require(
        kind.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
            && kind
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-'),
        "invalid-identity-kind",
    )?;
    Ok(format!(
        "{kind}:sha256:{}",
        hex::encode(Sha256::digest(canonical_json(value)?.as_bytes()))
    ))
}
/// Decodes protocol wire bytes, rejecting whitespace, duplicate keys and all
/// alternate spellings by requiring an exact canonical round trip.
pub fn decode(bytes: &[u8]) -> Result<Value> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| err("noncanonical-wire"))?;
    require(
        canonical_json(&value)?.as_bytes() == bytes,
        "noncanonical-wire",
    )?;
    Ok(value)
}
pub fn is_id(v: &Value, kind: &str) -> bool {
    v.as_str()
        .and_then(|s| s.strip_prefix(&format!("{kind}:sha256:")))
        .is_some_and(|s| {
            s.len() == 64
                && s.bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        })
}
