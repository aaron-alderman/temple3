use serde_json::{Value, json};
use temple3::{
    canonical, commitment, coordination, federation, identity, kernel, predicate, vectors,
};

fn corpus_root() -> std::path::PathBuf {
    std::env::var_os("TEMPLE3_CORPUS_ROOT")
        .expect("set TEMPLE3_CORPUS_ROOT to the external corpus")
        .into()
}

#[test]
fn canonical_wire_rejects_ambiguous_encodings() {
    for bytes in [
        br#"{"a":true,"a":false}"#.as_slice(),
        b" null",
        b"null\n",
        br#"{"b":null,"a":null}"#,
        br#""\u0061""#,
        b"1",
        br#""\ud800""#,
        b"\xff",
    ] {
        assert!(canonical::decode(bytes).is_err(), "{bytes:?}");
    }
    assert!(canonical::canonical_json(&json!("e\u{301}")).is_err());
    assert!(canonical::canonical_json(&json!({"e\u{301}":null})).is_err());
    let value =
        json!({"é":"é","\u{10000}":true,"\u{e000}":false,"control":"\u{0}\t\n\r\u{8}\u{c}"});
    let wire = canonical::canonical_json(&value).unwrap();
    assert!(wire.find('\u{e000}').unwrap() < wire.find('\u{10000}').unwrap());
    assert_eq!(canonical::decode(wire.as_bytes()).unwrap(), value);
}
#[test]
fn signatures_bind_purpose_payload_and_key() {
    let seed = [7u8; 32];
    let envelope =
        identity::sign_envelope("attestation", &json!({"statement":"test"}), &seed).unwrap();
    let key = envelope["publicKey"].clone();
    assert!(identity::verify_envelope(&envelope, "attestation", &key));
    assert!(!identity::verify_envelope(&envelope, "proposal", &key));
    let mut forged = envelope.clone();
    forged["payload"]["statement"] = json!("tampered");
    assert!(!identity::verify_envelope(&forged, "attestation", &key));
    assert!(!identity::verify_envelope(
        &envelope,
        "attestation",
        &json!("00".repeat(32))
    ));
    assert_eq!(
        identity::sign_envelope("attestation", &json!({"statement":"test"}), &seed).unwrap(),
        envelope
    );
}
#[test]
#[ignore = "requires the external conformance corpus"]
fn rust_signing_reproduces_published_envelope() {
    let corpus =
        vectors::load(&corpus_root().join("conformance/vectors/identity-v0.1.json"))
            .unwrap();
    // RFC 8032 test vector 2, public fixture material.
    let seed: [u8; 32] =
        hex::decode("4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb")
            .unwrap()
            .try_into()
            .unwrap();
    let public =
        identity::sign_envelope("attestation", &Value::Null, &seed).unwrap()["publicKey"].clone();
    let envelope = corpus["signatureCases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["expected"] == true && c["publicKey"] == public)
        .unwrap()["envelope"]
        .clone();
    assert_eq!(
        identity::sign_envelope(
            envelope["purpose"].as_str().unwrap(),
            &envelope["payload"],
            &seed
        )
        .unwrap(),
        envelope
    );
}
#[test]
fn rejection_preserves_input_and_old_policy_controls_replacement() {
    let temple = kernel::seal_genesis(
        &json!({"activePredicate":predicate::literal(json!(false)),"initialValues":["kept"]}),
    )
    .unwrap();
    let saved = temple.clone();
    let delta = kernel::build_delta(
        &temple,
        &json!({"nextPredicate":predicate::literal(json!(true))}),
    )
    .unwrap();
    let result =
        kernel::evaluate_transition(&temple, &json!({"identity":"alice","delta":delta})).unwrap();
    assert!(!result.accepted);
    assert_eq!(result.witness["reason"], "covenant-rejected");
    assert_eq!(result.temple, saved);
    assert_eq!(temple, saved);
}
#[test]
fn undeclared_cells_are_hidden_and_large_sequences_do_not_overflow() {
    let policy = predicate::eq(
        predicate::read(&["basis", "cells", "0"]),
        predicate::literal(Value::Null),
    );
    let mut temple =
        kernel::seal_genesis(&json!({"activePredicate":policy,"initialValues":["secret"]}))
            .unwrap();
    temple["sequence"] = json!("999999999999999999999999999999999999999");
    let delta = kernel::build_delta(
        &temple,
        &json!({"create":[commitment::cell_for(&json!("new")).unwrap()]}),
    )
    .unwrap();
    let r =
        kernel::evaluate_transition(&temple, &json!({"identity":"alice","delta":delta})).unwrap();
    assert!(r.accepted);
    assert_eq!(
        r.temple["sequence"],
        "1000000000000000000000000000000000000000"
    );
}
#[test]
fn proof_budget_failure_is_not_boolean_permission() {
    let stmt=federation::statement(&json!("private-opening"),&json!({"commitment":canonical::identify("private-value",&json!({"salt":"ab".repeat(32),"value":"x"})).unwrap(),"value":"x"})).unwrap();
    let proof = json!({"kind":"temple3.proof-envelope/v0.1","system":"temple3.transparent/v0.1","statementId":stmt["statementId"],"evidence":{"salt":"ab".repeat(32)}});
    let verify = json!({"op":"verify","statement":predicate::literal(stmt),"proof":predicate::literal(proof)});
    let p = json!({"op":"not","value":{"op":"all","values":vec![verify;9]}});
    assert_eq!(
        predicate::evaluate(&p, &json!({})).unwrap_err().code,
        "proof-verification-exhausted"
    );
}
#[test]
#[ignore = "requires the external conformance corpus"]
fn certificate_assembly_reproduces_fixture_and_rejects_duplicate_votes() {
    let corpus = vectors::load(
        &corpus_root().join("conformance/vectors/coordination-v0.1.json"),
    )
    .unwrap();
    let case = corpus["certificateCases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["expected"]["valid"] == true)
        .unwrap();
    let mut state =
        coordination::initial(&case["genesis"], &case["anchor"]["expectedTempleId"]).unwrap();
    for p in case["parents"].as_array().unwrap() {
        state = coordination::advance(&state, p).unwrap();
    }
    let cert = &case["certificate"];
    let votes = cert["votes"].as_array().unwrap();
    assert_eq!(
        coordination::assemble_certificate(&state, &cert["proposal"], votes).unwrap(),
        *cert
    );
    assert!(
        coordination::assemble_certificate(
            &state,
            &cert["proposal"],
            &[votes[0].clone(), votes[0].clone()]
        )
        .is_err()
    );
    let next = coordination::advance(&state, cert).unwrap();
    assert!(coordination::advance(&next, cert).is_err());
}
#[test]
fn malformed_envelopes_and_proofs_fail_without_panicking() {
    for v in [
        Value::Null,
        json!(false),
        json!(0),
        json!("x"),
        json!([]),
        json!({}),
        json!({"kind":"temple3.signed/v0.1"}),
    ] {
        assert!(!identity::verify_envelope(&v, "proposal", &Value::Null));
        assert!(!federation::verify_proof(&v, &v).unwrap());
        assert!(kernel::seal_genesis(&v).is_err());
        assert!(coordination::validate_profile(&v).is_err());
    }
}
