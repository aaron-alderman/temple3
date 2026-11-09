use crate::{Result, array, canonical::canonical_json, err, federation::RelationVerifier, require};
use serde_json::{Value, json};

/// Internal deterministic proof budget. Public entry points always start fresh.
#[derive(Debug)]
pub(crate) struct Scope {
    pub depth: usize,
    pub remaining: usize,
    pub relations: Option<RelationVerifier>,
}
impl Default for Scope {
    fn default() -> Self {
        Self {
            depth: 0,
            remaining: 8,
            relations: None,
        }
    }
}
pub fn literal(value: Value) -> Value {
    json!({"op":"literal", "value":value})
}
pub fn read(path: &[&str]) -> Value {
    json!({"op":"read", "path":path})
}
pub fn eq(left: Value, right: Value) -> Value {
    json!({"op":"eq", "left":left, "right":right})
}
pub fn validate(node: &Value) -> Result<()> {
    fn visit(n: &Value, depth: usize, count: &mut usize) -> Result<()> {
        *count += 1;
        require(
            depth <= 32 && *count <= 512 && n.is_object(),
            "invalid-predicate",
        )?;
        match n["op"].as_str() {
            Some("literal") => require(n.get("value").is_some(), "invalid-predicate")?,
            Some("read") => {
                let p = array(&n["path"], "invalid-predicate")?;
                require(
                    !p.is_empty()
                        && p.iter().all(|v| v.as_str().is_some_and(|s| !s.is_empty()))
                        && ["identity", "basis", "delta", "result", "context"]
                            .iter()
                            .any(|s| p[0] == *s),
                    "invalid-predicate",
                )?;
            }
            Some("eq" | "verify") => {
                let fields = if n["op"] == "eq" {
                    ["left", "right"]
                } else {
                    ["statement", "proof"]
                };
                for f in fields {
                    visit(&n[f], depth + 1, count)?;
                }
            }
            Some("not") => visit(&n["value"], depth + 1, count)?,
            Some("all" | "any") => {
                let a = array(&n["values"], "invalid-predicate")?;
                require(!a.is_empty(), "invalid-predicate")?;
                for v in a {
                    visit(v, depth + 1, count)?;
                }
            }
            _ => return Err(err("invalid-predicate")),
        }
        Ok(())
    }
    visit(node, 0, &mut 0)
}
pub fn evaluate(predicate: &Value, environment: &Value) -> Result<bool> {
    evaluate_scoped(predicate, environment, &mut Scope::default())
}
pub(crate) fn evaluate_scoped(p: &Value, env: &Value, scope: &mut Scope) -> Result<bool> {
    validate(p)?;
    fn run(n: &Value, env: &Value, fuel: &mut usize, scope: &mut Scope) -> Result<Value> {
        *fuel = fuel
            .checked_sub(1)
            .ok_or_else(|| err("predicate-fuel-exhausted"))?;
        Ok(match n["op"].as_str() {
            Some("literal") => n["value"].clone(),
            Some("read") => {
                let mut value = env;
                for part in array(&n["path"], "invalid-predicate")? {
                    let part = part.as_str().ok_or_else(|| err("invalid-predicate"))?;
                    value = if let Some(m) = value.as_object() {
                        m.get(part)
                    } else if let Some(a) = value.as_array() {
                        if part == "0"
                            || (!part.starts_with('0') && part.bytes().all(|b| b.is_ascii_digit()))
                        {
                            part.parse::<usize>().ok().and_then(|i| a.get(i))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                    .unwrap_or(&Value::Null);
                }
                value.clone()
            }
            Some("eq") => {
                let a = run(&n["left"], env, fuel, scope)?;
                let b = run(&n["right"], env, fuel, scope)?;
                json!(canonical_json(&a)? == canonical_json(&b)?)
            }
            Some("verify") => {
                let a = run(&n["statement"], env, fuel, scope)?;
                let b = run(&n["proof"], env, fuel, scope)?;
                json!(crate::federation::verify_scoped(&a, &b, scope)?)
            }
            Some("not") => json!(run(&n["value"], env, fuel, scope)? != true),
            Some("all" | "any") => {
                let all = n["op"] == "all";
                let mut result = all;
                for v in array(&n["values"], "invalid-predicate")? {
                    let b = run(v, env, fuel, scope)? == true;
                    if b != all {
                        result = b;
                        break;
                    }
                }
                json!(result)
            }
            _ => return Err(err("invalid-predicate")),
        })
    }
    run(p, env, &mut 2048, scope)?
        .as_bool()
        .ok_or_else(|| err("predicate-result-not-boolean"))
}
