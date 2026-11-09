//! Explicit fixed-scale arithmetic. Currency identifiers do not imply scales.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn check(ok: bool) -> R<()> {
    if ok {
        Ok(())
    } else {
        Err("invalid-exact-money".into())
    }
}
fn exact(v: &Value, fields: &[&str]) -> R<()> {
    let o = v.as_object().ok_or("invalid-exact-money")?;
    check(o.len() == fields.len() && fields.iter().all(|k| o.contains_key(*k)))
}
fn text<'a>(v: &'a Value, k: &str) -> R<&'a str> {
    v[k].as_str().ok_or("invalid-exact-money".into())
}
fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit())
}
fn money(v: &Value, p: &Value, scale: usize, maximum: i128) -> R<Value> {
    exact(v, &["currency", "decimal"])?;
    check(v["currency"] == p["currency"])?;
    let s = text(v, "decimal")?;
    check(s.len() <= 42)?;
    let negative = s.starts_with('-');
    let magnitude = s.strip_prefix('-').unwrap_or(s);
    let parts: Vec<&str> = magnitude.split('.').collect();
    check(parts.len() == if scale == 0 { 1 } else { 2 })?;
    check(digits(parts[0]) && (parts[0] == "0" || !parts[0].starts_with('0')))?;
    if scale > 0 {
        check(digits(parts[1]) && parts[1].len() == scale)?;
    }
    let raw = parts
        .concat()
        .parse::<i128>()
        .map_err(|_| "invalid-exact-money")?;
    check(raw <= maximum && (!negative || raw > 0 && p["negative"] == "allow"))?;
    let minor = if negative { -raw } else { raw };
    Ok(
        json!({"kind":"temple3.exact-money/v0.1","currency":p["currency"],"scale":p["scale"],"minor":minor.to_string()}),
    )
}
pub fn classify(v: &Value) -> R<Value> {
    crate::canonical::canonical_json(v).map_err(|_| "invalid-exact-money")?;
    exact(v, &["amount", "policy", "threshold"])?;
    let p = &v["policy"];
    exact(
        p,
        &[
            "currency",
            "kind",
            "maximumMinor",
            "negative",
            "rounding",
            "scale",
        ],
    )?;
    let currency = text(p, "currency")?;
    let scale = text(p, "scale")?;
    let maximum = text(p, "maximumMinor")?;
    check(
        p["kind"] == "temple3.exact-money-policy/v0.1"
            && currency.len() == 3
            && currency.bytes().all(|c| c.is_ascii_uppercase())
            && scale.len() == 1
            && digits(scale)
            && digits(maximum)
            && maximum.len() <= 30
            && !maximum.starts_with('0')
            && (p["negative"] == "allow" || p["negative"] == "reject")
            && p["rounding"] == "reject-inexact",
    )?;
    let scale = scale.parse::<usize>().map_err(|_| "invalid-exact-money")?;
    let maximum = maximum.parse::<i128>().map_err(|_| "invalid-exact-money")?;
    let amount = money(&v["amount"], p, scale, maximum)?;
    let threshold = money(&v["threshold"], p, scale, maximum)?;
    let a = text(&amount, "minor")?
        .parse::<i128>()
        .map_err(|_| "invalid-exact-money")?;
    let b = text(&threshold, "minor")?
        .parse::<i128>()
        .map_err(|_| "invalid-exact-money")?;
    let comparison = if a < b {
        "below"
    } else if a > b {
        "above"
    } else {
        "equal"
    };
    let policy_id =
        crate::canonical::identify("exact-money-policy", p).map_err(|_| "invalid-exact-money")?;
    Ok(
        json!({"kind":"temple3.exact-money-classification/v0.1","policyId":policy_id,"amount":amount,"threshold":threshold,"comparison":comparison}),
    )
}
