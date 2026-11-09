//! Independent bounded computation over UTF-16 code units (including surrogates).
use crate::{Result, array, require, shape, string};
use serde_json::{Value, json};
const ERROR: &str = "invalid-workgarden-command";
fn text(v: &Value, empty: bool) -> Result<Vec<u16>> {
    let s = string(v, ERROR)?;
    require(
        s.len() <= 2000
            && s.len() % 4 == 0
            && (empty || !s.is_empty())
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        ERROR,
    )?;
    let bytes = hex::decode(s).map_err(|_| crate::err(ERROR))?;
    Ok(bytes
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect())
}
fn sequence(s: &Value) -> Result<u32> {
    let raw = string(s, ERROR)?;
    let n = raw.parse::<u32>().map_err(|_| crate::err(ERROR))?;
    require(n <= 1_000_000 && n.to_string() == raw, ERROR)?;
    Ok(n)
}
fn validate(s: &Value) -> Result<()> {
    shape(s, "kind sequence records")?;
    require(s["kind"] == "temple3.workgarden-state/v0.1", ERROR)?;
    sequence(&s["sequence"])?;
    let records = array(&s["records"], ERROR)?;
    require(records.len() <= 8, ERROR)?;
    for (i, r) in records.iter().enumerate() {
        let expected_order = (i + 1).to_string();
        shape(r, "subject label completed member order")?;
        require(
            r["subject"] == format!("todo.{}", i + 1)
                && r["order"].as_str() == Some(expected_order.as_str())
                && r["completed"].is_boolean()
                && r["member"].is_boolean(),
            ERROR,
        )?;
        text(&r["label"], false)?;
    }
    Ok(())
}
pub fn next_state(before: &Value, command: &Value) -> Result<Value> {
    validate(before)?;
    shape(command, "op args")?;
    let encoded = array(&command["args"], ERROR)?;
    let args: Vec<Vec<u16>> = encoded
        .iter()
        .map(|v| text(v, true))
        .collect::<Result<_>>()?;
    let n = sequence(&before["sequence"])?;
    require(n < 1_000_000, ERROR)?;
    let mut after = before.clone();
    after["sequence"] = json!((n + 1).to_string());
    let records = after["records"]
        .as_array_mut()
        .ok_or_else(|| crate::err(ERROR))?;
    match command["op"].as_str() {
        Some("create") => {
            require(
                args.len() == 1 && !args[0].is_empty() && records.len() < 8,
                ERROR,
            )?;
            let id = records.len() + 1;
            records.push(json!({"subject":format!("todo.{id}"),"label":encoded[0],"completed":false,"member":true,"order":id.to_string()}));
        }
        Some("delete") => {
            require(args.len() <= 1, ERROR)?;
            let target = args.first().cloned().unwrap_or_default();
            for r in records {
                if string(&r["subject"], ERROR)?
                    .encode_utf16()
                    .eq(target.iter().copied())
                {
                    r["member"] = json!(false);
                }
            }
        }
        Some(op @ ("edit" | "status")) => {
            require(args.len() == 2, ERROR)?;
            let r = records
                .iter_mut()
                .find(|r| {
                    r["subject"]
                        .as_str()
                        .is_some_and(|s| s.encode_utf16().eq(args[0].iter().copied()))
                })
                .ok_or_else(|| crate::err(ERROR))?;
            if op == "edit" {
                require(!args[1].is_empty(), ERROR)?;
                r["label"] = encoded[1].clone();
            } else {
                let yes: Vec<u16> = "true".encode_utf16().collect();
                let no: Vec<u16> = "false".encode_utf16().collect();
                require(args[1] == yes || args[1] == no, ERROR)?;
                r["completed"] = json!(args[1] == yes);
            }
        }
        _ => return Err(crate::err(ERROR)),
    }
    validate(&after)?;
    Ok(after)
}
pub fn verify(inputs: &Value, evidence: &Value) -> Result<bool> {
    shape(inputs, "before command after")?;
    shape(evidence, "")?;
    Ok(next_state(&inputs["before"], &inputs["command"])? == inputs["after"])
}
