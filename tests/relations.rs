use serde_json::{Value, json};
use temple3::{Result, federation};

fn resolve(relation: &str, inputs: &Value, evidence: &Value) -> Option<Result<bool>> {
    (relation == "example-equality").then(|| Ok(inputs["expected"] == evidence["observed"]))
}

fn must_not_run(_: &str, _: &Value, _: &Value) -> Option<Result<bool>> {
    panic!("unchecked proof reached the application resolver");
}

#[test]
fn extension_dispatch_requires_a_checked_envelope_and_trusted_resolver() {
    let stmt = federation::statement(&json!("example-equality"), &json!({"expected":"value"})).unwrap();
    let mut proof = json!({
        "kind":"temple3.proof-envelope/v0.1",
        "system":"temple3.transparent/v0.1",
        "statementId":stmt["statementId"],
        "evidence":{"observed":"value"}
    });
    assert!(!federation::verify_proof(&stmt, &proof).unwrap());
    assert!(federation::verify_proof_with_relations(&stmt, &proof, resolve).unwrap());
    proof["evidence"]["observed"] = json!("wrong");
    assert!(!federation::verify_proof_with_relations(&stmt, &proof, resolve).unwrap());
    proof["statementId"] = json!("wrong");
    assert!(!federation::verify_proof_with_relations(&stmt, &proof, must_not_run).unwrap());

    let built_in = federation::statement(&json!("private-opening"), &json!({})).unwrap();
    proof["statementId"] = built_in["statementId"].clone();
    assert!(!federation::verify_proof_with_relations(&built_in, &proof, must_not_run).unwrap());
}
