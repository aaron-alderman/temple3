//! Temple3 laboratory protocol. All state advancement recomputes constitutional
//! admission; signatures alone never authorize a transition.
pub mod canonical;
pub mod commitment;
pub mod coordination;
pub mod federation;
pub mod identity;
pub mod journal;
pub mod kernel;
pub mod pilot;
pub mod predicate;
pub mod replica;
pub mod vectors;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub code: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.code)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
pub(crate) fn err(code: &str) -> Error {
    Error { code: code.into() }
}
pub(crate) fn require(ok: bool, code: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(err(code)) }
}
pub(crate) fn string<'a>(v: &'a Value, code: &str) -> Result<&'a str> {
    v.as_str().ok_or_else(|| err(code))
}
pub(crate) fn array<'a>(v: &'a Value, code: &str) -> Result<&'a Vec<Value>> {
    v.as_array().ok_or_else(|| err(code))
}
pub(crate) fn shape(v: &Value, fields: &str) -> Result<()> {
    let expected: std::collections::BTreeSet<_> = fields.split_whitespace().collect();
    require(
        v.as_object().is_some_and(|m| {
            m.keys()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>()
                == expected
        }),
        "invalid-envelope",
    )
}
pub(crate) fn without(v: &Value, field: &str) -> Result<Value> {
    let mut m = v
        .as_object()
        .ok_or_else(|| err("invalid-envelope"))?
        .clone();
    m.remove(field);
    Ok(Value::Object(m))
}
pub(crate) fn bind(kind: &str, field: &str, mut payload: Value) -> Result<Value> {
    let id = canonical::identify(kind, &payload)?;
    payload
        .as_object_mut()
        .ok_or_else(|| err("invalid-envelope"))?
        .insert(field.into(), id.into());
    Ok(payload)
}
pub(crate) fn decimal(v: &Value) -> Result<num_bigint::BigUint> {
    let s = string(v, "invalid-sequence")?;
    require(
        !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit()) && (s == "0" || !s.starts_with('0')),
        "invalid-sequence",
    )?;
    num_bigint::BigUint::parse_bytes(s.as_bytes(), 10).ok_or_else(|| err("invalid-sequence"))
}
