//! Size calculation using placeholders, never fabricated authorization.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn bytes(v: &Value) -> R<usize> {
    crate::canonical::canonical_json(v)
        .map(|s| s.len())
        .map_err(|_| "invalid-capacity".into())
}
pub fn capacity(f: &Value) -> R<Value> {
    let initial = crate::voluntary_expression::formation(f)?;
    let records = initial["records"].as_array().ok_or("invalid-records")?;
    let pending: Vec<&Value> = records.iter().filter(|r| r["status"] == "notYet").collect();
    let basis = format!("voluntary-expression-state:sha256:{}", "0".repeat(64));
    let mut maximum_statement = 0;
    let mut maximum_context = 0;
    for last in &pending {
        let ordered: Vec<&Value> = pending
            .iter()
            .copied()
            .filter(|r| r["id"] != last["id"])
            .chain(std::iter::once(*last))
            .collect();
        let entries: Vec<Value> = ordered
            .iter()
            .map(|r| json!({"actor":r["owner"],"recordId":r["id"],"basis":basis}))
            .collect();
        let mut after = initial.clone();
        after["records"] = json!(
            records
                .iter()
                .map(|r| {
                    let mut v = r.clone();
                    v["status"] = json!("offered");
                    v
                })
                .collect::<Vec<_>>()
        );
        after["entries"] = json!(entries);
        after["sequence"] = json!(entries.len().to_string());
        let mut before = after.clone();
        for r in before["records"].as_array_mut().ok_or("invalid-records")? {
            if r["id"] == last["id"] {
                r["status"] = json!("notYet");
            }
        }
        before["entries"] = json!(&entries[..entries.len() - 1]);
        before["sequence"] = json!((entries.len() - 1).to_string());
        let statement = json!({"kind":"temple3.proof-statement/v0.1","relation":"voluntary-expression-command-v1","publicInputs":{"before":before,"actor":last["owner"],"command":{"op":"offer","args":{"recordId":last["id"],"basis":basis}},"after":after},"statementId":format!("proof-statement:sha256:{}","0".repeat(64))});
        let proof = json!({"kind":"temple3.proof-envelope/v0.1","system":"temple3.transparent/v0.1","statementId":statement["statementId"],"evidence":{}});
        maximum_statement = maximum_statement.max(bytes(&statement)?);
        maximum_context =
            maximum_context.max(bytes(&json!({"statement":statement,"proof":proof}))?);
    }
    Ok(
        json!({"kind":"temple3.voluntary-expression-capacity/v0.1","remainingOffers":pending.len().to_string(),"maximumStatementBytes":maximum_statement.to_string(),"maximumContextBytes":maximum_context.to_string(),"fits":maximum_statement<=65536 && maximum_context<=65536}),
    )
}
pub fn require_capacity(f: &Value) -> R<Value> {
    let result = capacity(f)?;
    if result["fits"] != true {
        return Err("insufficient-expression-capacity".into());
    }
    Ok(result)
}
