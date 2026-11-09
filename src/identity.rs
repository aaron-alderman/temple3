use crate::{
    Result, array, bind,
    canonical::{canonical_json, identify},
    err, kernel, require, shape, string, without,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use num_bigint::BigUint;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const SUITE: &str = "temple3.ed25519-sha256/v0.1";
const PURPOSES: [&str; 6] = [
    "genesis",
    "proposal",
    "attestation",
    "finalized-witness",
    "rejection-witness",
    "coordination-vote",
];
pub fn token(v: &Value) -> Result<()> {
    let s = string(v, "invalid-token")?;
    require(!s.is_empty() && s.len() <= 128, "invalid-token")?;
    canonical_json(v)?;
    Ok(())
}
fn bytes<const N: usize>(v: &Value) -> Result<[u8; N]> {
    let s = string(v, "invalid-signature")?;
    require(
        s.len() == N * 2
            && s.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
        "invalid-signature",
    )?;
    hex::decode(s)
        .map_err(|_| err("invalid-signature"))?
        .try_into()
        .map_err(|_| err("invalid-signature"))
}
fn point(v: &Value) -> Result<[u8; 32]> {
    let original = bytes::<32>(v)?;
    let mut y = original;
    y[31] &= 127;
    let low = [
        "00".repeat(32),
        format!("01{}", "00".repeat(31)),
        format!("ec{}7f", "ff".repeat(30)),
        "26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc05".into(),
        "c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac037a".into(),
    ];
    require(
        BigUint::from_bytes_le(&y) < (BigUint::from(1u8) << 255) - 19u8
            && !low.contains(&hex::encode(y)),
        "invalid-public-key",
    )?;
    Ok(original)
}
pub fn key_id(public_key: &Value) -> Result<String> {
    point(public_key)?;
    identify("key", &json!({"suite":SUITE,"publicKey":public_key}))
}
pub fn signed_id(envelope: &Value) -> Result<String> {
    identify("signed", envelope)
}
fn statement_id(envelope: &Value) -> Result<String> {
    identify("signed-statement", &without(envelope, "signature")?)
}
fn message(purpose: &str, key: &Value, payload: &Value) -> Result<Vec<u8>> {
    Ok(canonical_json(&json!({"domain":"temple3.signature/v0.1","suite":SUITE,"purpose":purpose,"keyId":key_id(key)?,"publicKey":key,"payload":payload}))?.into_bytes())
}
/// Signs using a caller-supplied seed. Never persists or generates private keys.
pub fn sign_envelope(purpose: &str, payload: &Value, seed: &[u8; 32]) -> Result<Value> {
    require(PURPOSES.contains(&purpose), "invalid-purpose")?;
    let key = SigningKey::from_bytes(seed);
    let public = json!(hex::encode(key.verifying_key().as_bytes()));
    let mut result = json!({"kind":"temple3.signed/v0.1","suite":SUITE,"purpose":purpose,"keyId":key_id(&public)?,"publicKey":public,"payload":payload,"signature":"0".repeat(128)});
    require(
        canonical_json(&result)?.len() <= 4_194_304,
        "signed-object-too-large",
    )?;
    result["signature"] = json!(hex::encode(
        key.sign(&message(purpose, &public, payload)?).to_bytes()
    ));
    Ok(result)
}
pub fn signed_genesis(input: &Value, seed: &[u8; 32]) -> Result<Value> {
    let temple = kernel::seal_genesis(input)?;
    let signed = sign_envelope(
        "genesis",
        &json!({"genesisInput":input,"templeId":temple["templeId"],"witness":temple["witnesses"][0]}),
        seed,
    )?;
    let entry = json!({"kind":"genesis","signed":signed});
    verify_genesis(&entry, &temple["templeId"])?;
    Ok(entry)
}
pub fn verify_envelope(envelope: &Value, purpose: &str, public_key: &Value) -> bool {
    (|| -> Result<()> {
        shape(
            envelope,
            "kind suite purpose keyId publicKey payload signature",
        )?;
        require(
            canonical_json(envelope)?.len() <= 4_194_304,
            "invalid-signature",
        )?;
        require(
            PURPOSES.contains(&purpose)
                && envelope["purpose"] == purpose
                && envelope["kind"] == "temple3.signed/v0.1"
                && envelope["suite"] == SUITE
                && envelope["publicKey"] == *public_key
                && envelope["keyId"] == key_id(public_key)?,
            "invalid-signature",
        )?;
        let signature = bytes::<64>(&envelope["signature"])?;
        point(&json!(hex::encode(&signature[..32])))?;
        let order = (BigUint::from(1u8) << 252)
            + BigUint::parse_bytes(b"27742317777372353535851937790883648493", 10)
                .ok_or_else(|| err("invalid-signature"))?;
        require(
            BigUint::from_bytes_le(&signature[32..]) < order,
            "invalid-signature",
        )?;
        let key =
            VerifyingKey::from_bytes(&point(public_key)?).map_err(|_| err("invalid-signature"))?;
        key.verify_strict(
            &message(purpose, public_key, &envelope["payload"])?,
            &Signature::from_bytes(&signature),
        )
        .map_err(|_| err("invalid-signature"))
    })()
    .is_ok()
}
pub fn verify_raw(public: &Value, signature: &Value, message_hex: &Value) -> Result<()> {
    let key =
        VerifyingKey::from_bytes(&bytes::<32>(public)?).map_err(|_| err("invalid-signature"))?;
    let msg = hex::decode(string(message_hex, "invalid-signature")?)
        .map_err(|_| err("invalid-signature"))?;
    key.verify(&msg, &Signature::from_bytes(&bytes::<64>(signature)?))
        .map_err(|_| err("invalid-signature"))
}
pub fn validate_profile(profile: &Value) -> Result<()> {
    shape(profile, "kind actors finalizer")?;
    require(
        profile["kind"] == "temple3.identity-profile/v0.1",
        "invalid-profile",
    )?;
    let actors = array(&profile["actors"], "invalid-profile")?;
    key_id(&profile["finalizer"])?;
    let mut previous = None;
    for binding in actors {
        shape(binding, "actor publicKey")?;
        token(&binding["actor"])?;
        key_id(&binding["publicKey"])?;
        let actor = string(&binding["actor"], "invalid-profile")?;
        require(previous.is_none_or(|p| p < actor), "invalid-profile")?;
        previous = Some(actor);
    }
    Ok(())
}
pub fn authenticate(profile: &Value, temple_id: &Value, envelope: &Value) -> Result<String> {
    let purpose = string(&envelope["purpose"], "invalid-purpose")?;
    let p = &envelope["payload"];
    let public;
    let slot;
    match purpose {
        "proposal" | "attestation" => {
            shape(
                p,
                if purpose == "proposal" {
                    "templeId actor requestId delta context"
                } else {
                    "templeId actor slot statement"
                },
            )?;
            token(&p["actor"])?;
            let nonce = &p[if purpose == "proposal" {
                "requestId"
            } else {
                "slot"
            }];
            token(nonce)?;
            public = &array(&profile["actors"], "invalid-profile")?
                .iter()
                .find(|b| b["actor"] == p["actor"])
                .ok_or_else(|| err("unknown-actor"))?["publicKey"];
            slot = json!({"templeId":temple_id,"purpose":purpose,"actor":p["actor"],"nonce":nonce});
        }
        "finalized-witness" | "rejection-witness" => {
            shape(p, "templeId proposalId previousReceiptId witness")?;
            require(
                p["previousReceiptId"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty()),
                "invalid-envelope",
            )?;
            public = &profile["finalizer"];
            slot = json!({"templeId":temple_id,"purpose":"receipt","keyId":key_id(public)?,"previousReceiptId":p["previousReceiptId"]});
        }
        _ => return Err(err("invalid-purpose")),
    }
    require(p["templeId"] == *temple_id, "wrong-temple")?;
    require(
        verify_envelope(envelope, purpose, public),
        "invalid-signature",
    )?;
    identify("signed-slot", &slot)
}
pub fn record_for(entry: &Value, index: usize, tip: &Value) -> Result<Value> {
    bind(
        "journal-record",
        "recordId",
        json!({"kind":"temple3.journal-record/v0.1","index":index.to_string(),"previousRecordId":tip,"entry":entry}),
    )
}
pub fn verify_records(records: &Value, anchor: &Value) -> Result<Value> {
    let mut tip = json!("empty");
    for (i, r) in array(records, "invalid-journal")?.iter().enumerate() {
        shape(r, "kind index previousRecordId entry recordId")?;
        require(*r == record_for(&r["entry"], i, &tip)?, "invalid-journal")?;
        tip = r["recordId"].clone();
    }
    require(
        anchor.get("expectedTip").is_none_or(|t| *t == tip),
        "journal-tip-mismatch",
    )?;
    Ok(tip)
}
pub fn verify_genesis(genesis: &Value, expected: &Value) -> Result<(Value, Value)> {
    require(
        expected.as_str().is_some_and(|s| !s.is_empty()),
        "missing-trust-anchor",
    )?;
    shape(genesis, "kind signed")?;
    require(genesis["kind"] == "genesis", "missing-genesis")?;
    let p = &genesis["signed"]["payload"];
    shape(p, "genesisInput templeId witness")?;
    let profile = &p["genesisInput"]["context"]["identityProfile"];
    validate_profile(profile)?;
    let temple = kernel::seal_genesis(&p["genesisInput"])?;
    require(
        temple["templeId"] == *expected
            && *expected == p["templeId"]
            && p["witness"] == temple["witnesses"][0]
            && verify_envelope(&genesis["signed"], "genesis", &profile["finalizer"]),
        "invalid-genesis",
    )?;
    Ok((temple, profile.clone()))
}
pub fn replay(records: &Value, anchor: &Value) -> Result<Value> {
    let expected = &anchor["expectedTempleId"];
    require(
        expected.as_str().is_some_and(|s| !s.is_empty()),
        "missing-trust-anchor",
    )?;
    let tip = verify_records(records, anchor)?;
    let records = array(records, "invalid-journal")?;
    let genesis = &records.first().ok_or_else(|| err("missing-genesis"))?["entry"];
    let (mut temple, profile) = verify_genesis(genesis, expected)?;
    let mut receipt = signed_id(&genesis["signed"])?;
    let mut observations = BTreeMap::new();
    let mut evidence = BTreeSet::new();
    let observe = |e: &Value, observations: &mut BTreeMap<String, Value>| -> Result<()> {
        let slot = authenticate(&profile, expected, e)?;
        require(!observations.contains_key(&slot), "replayed-statement")?;
        observations.insert(slot, e.clone());
        Ok(())
    };
    for r in &records[1..] {
        let e = &r["entry"];
        match e["kind"].as_str() {
            Some("transition") => {
                shape(e, "kind proposal receipt")?;
                require(e["proposal"]["purpose"] == "proposal", "invalid-purpose")?;
                observe(&e["proposal"], &mut observations)?;
                let p = &e["proposal"]["payload"];
                let result = kernel::evaluate_transition(
                    &temple,
                    &json!({"identity":p["actor"],"delta":p["delta"],"context":p["context"]}),
                )?;
                let payload = json!({"templeId":expected,"proposalId":signed_id(&e["proposal"])?,"previousReceiptId":receipt,"witness":result.witness});
                require(
                    e["receipt"]["purpose"]
                        == if result.accepted {
                            "finalized-witness"
                        } else {
                            "rejection-witness"
                        }
                        && e["receipt"]["payload"] == payload,
                    "invalid-receipt",
                )?;
                observe(&e["receipt"], &mut observations)?;
                receipt = signed_id(&e["receipt"])?;
                temple = result.temple;
            }
            Some("attestation") => {
                shape(e, "kind signed")?;
                require(e["signed"]["purpose"] == "attestation", "invalid-purpose")?;
                observe(&e["signed"], &mut observations)?;
            }
            Some("equivocation") => {
                shape(e, "kind first second")?;
                let a = &e["first"];
                let b = &e["second"];
                let slot = authenticate(&profile, expected, a)?;
                let aid = statement_id(a)?;
                let bid = statement_id(b)?;
                require(
                    slot == authenticate(&profile, expected, b)? && aid != bid,
                    "invalid-equivocation",
                )?;
                require(
                    observations
                        .get(&slot)
                        .map(statement_id)
                        .transpose()?
                        .is_some_and(|id| id == aid),
                    "invalid-equivocation",
                )?;
                let mut ids = vec![aid, bid];
                ids.sort();
                require(
                    evidence.insert(identify("equivocation", &json!(ids))?),
                    "replayed-statement",
                )?;
            }
            _ => return Err(err("invalid-journal-entry")),
        }
    }
    Ok(
        json!({"templeId":expected,"stateRoot":temple["stateRoot"],"sequence":temple["sequence"],"receiptId":receipt,"tip":tip,"observations":observations.len().to_string(),"equivocations":evidence.len().to_string(),"witnessIds":array(&temple["witnesses"],"invalid-state")?.iter().map(|w|&w["witnessId"]).collect::<Vec<_>>() }),
    )
}
