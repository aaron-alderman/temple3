//! A local single-voter demonstration using PUBLIC test-only seed material.
//! No durable voting service or operational key management is implied.
use serde_json::json;
use temple3::{Result, canonical, commitment, coordination, identity, kernel, predicate};

fn main() -> Result<()> {
    let seed = [7u8; 32]; // Public demonstration seed; never use for real authority.
    let public = identity::sign_envelope("attestation", &json!({}), &seed)?["publicKey"].clone();
    let initial = commitment::cell_for(&json!({"owner":"alice","asset":"sample"}))?;
    let policy = predicate::eq(
        predicate::read(&["identity"]),
        predicate::literal(json!("alice")),
    );
    let genesis = identity::signed_genesis(
        &json!({"activePredicate":policy,"initialValues":[initial["value"]],"context":{
            "identityProfile":{"kind":"temple3.identity-profile/v0.1","actors":[{"actor":"alice","publicKey":public}],"finalizer":public},
            "application":{"coordinationProfile":{"kind":"temple3.coordination-profile/v0.1","mode":"single-instance","members":[{"voter":"demo","publicKey":public}],"quorum":"1","faultBound":"0"}}
        }}),
        &seed,
    )?;
    let expected = genesis["signed"]["payload"]["templeId"].clone();
    let state = coordination::initial(&genesis, &expected)?;
    let delta = kernel::build_delta(
        &state["temple"],
        &json!({"consume":[initial["id"]],"create":[commitment::cell_for(&json!({"owner":"bob","asset":"sample"}))?]}),
    )?;
    let proposal = identity::sign_envelope(
        "proposal",
        &json!({"templeId":expected,"actor":"alice","requestId":"demo-transfer","delta":delta,"context":{}}),
        &seed,
    )?;
    let candidate = coordination::candidate(&state, &proposal)?;
    let mut vote_payload = candidate["decision"].clone();
    vote_payload["voter"] = json!("demo");
    let vote = identity::sign_envelope("coordination-vote", &vote_payload, &seed)?;
    let certificate = coordination::assemble_certificate(&state, &proposal, &[vote])?;
    let after = coordination::advance(&state, &certificate)?;
    assert_eq!(after["temple"]["sequence"], "1");
    assert_eq!(
        after["temple"]["state"]["cells"][0]["value"]["owner"],
        "bob"
    );
    println!(
        "{}",
        canonical::canonical_json(
            &json!({"templeId":expected,"sequence":after["temple"]["sequence"],"stateRoot":after["temple"]["stateRoot"],"finalityId":after["finalityId"],"cells":after["temple"]["state"]["cells"]})
        )?
    );
    Ok(())
}
