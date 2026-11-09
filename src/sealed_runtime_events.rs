//! Independent verification of source commit seals and shadow reconciliation.
use crate::{
    Result, err, require, runtime_events as runtime,
    source_json::{self, SourceValue as S},
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn shape(v: &S, names: &str) -> Result<()> {
    let S::Object(fields) = v else {
        return Err(err("source-seal-shape"));
    };
    require(
        fields.len() == names.split_whitespace().count()
            && names.split_whitespace().all(|k| v.field(k).is_ok()),
        "source-seal-shape",
    )
}
fn text(v: &S, k: &str) -> Result<String> {
    v.field(k)?.text()
}
fn from(v: &Value) -> Result<S> {
    source_json::parse(&v.to_string())
}
fn hash(bytes: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn hex_string(s: &str, n: usize) -> bool {
    s.len() == n
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn decimal(s: &str) -> bool {
    s == "0" || (!s.starts_with('0') && !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
}
fn revision(s: &str) -> bool {
    s.strip_prefix("facts:").is_some_and(decimal)
}
pub fn initial_cursor(contract: &Value, anchor: &Value) -> Result<Value> {
    crate::shape(anchor, "publicKey admissionHash baselineHash streamId previousSealId nextSealIndex mutationId revision").map_err(|_| err("source-seal-shape"))?;
    crate::identity::key_id(&anchor["publicKey"])?;
    require(
        ["admissionHash", "baselineHash"]
            .iter()
            .all(|k| anchor[*k].as_str().is_some_and(|s| hex_string(s, 64))),
        "source-anchor",
    )?;
    require(
        anchor["baselineHash"] == contract["baselineHash"]
            && anchor["streamId"] == contract["streamId"],
        "source-anchor-scope",
    )?;
    require(
        anchor["nextSealIndex"].as_str().is_some_and(decimal)
            && anchor["revision"].as_str().is_some_and(revision)
            && anchor["mutationId"].as_str().is_some_and(|s| !s.is_empty())
            && anchor["previousSealId"]
                .as_str()
                .is_some_and(|s| s == "empty" || hex_string(s, 64)),
        "source-anchor",
    )?;
    Ok(
        json!({"runtime":runtime::initial_cursor(contract)?,"source":{"previousSealId":anchor["previousSealId"],"nextSealIndex":anchor["nextSealIndex"],"mutationId":anchor["mutationId"],"revision":anchor["revision"]}}),
    )
}
/// Cursor and anchor must come from independently configured, verified replay.
pub fn reconcile(
    contract: &Value,
    anchor: &Value,
    cursor: &Value,
    seal: &S,
    event: &Value,
) -> Result<Value> {
    require(
        source_json::canonical(seal)?.len() <= 65536,
        "source-seal-size",
    )?;
    shape(seal, "payload sealId signature")?;
    let p = seal.field("payload")?;
    shape(p, "kind index previousSealId keyId authority evidence")?;
    shape(p.field("authority")?, "admissionHash baselineHash streamId")?;
    let evidence = p.field("evidence")?;
    shape(evidence, "source receipt")?;
    require(
        text(p, "kind")? == "sourcery.after-commit-seal/1"
            && p.field("authority")?
                == &from(
                    &json!({"admissionHash":anchor["admissionHash"],"baselineHash":anchor["baselineHash"],"streamId":anchor["streamId"]}),
                )?,
        "source-seal-authority",
    )?;
    let public = crate::string(&anchor["publicKey"], "source-anchor")?;
    let der = hex::decode(format!("302a300506032b6570032100{public}"))
        .map_err(|_| err("source-anchor"))?;
    require(text(p, "keyId")? == hash(&der), "source-seal-key")?;
    let signature = text(seal, "signature")?;
    require(hex_string(&signature, 128), "source-seal-signature")?;
    let key_bytes: [u8; 32] = hex::decode(public)
        .map_err(|_| err("source-anchor"))?
        .try_into()
        .map_err(|_| err("source-anchor"))?;
    let key = VerifyingKey::from_bytes(&key_bytes).map_err(|_| err("source-anchor"))?;
    let sig =
        Signature::from_slice(&hex::decode(&signature).map_err(|_| err("source-seal-signature"))?)
            .map_err(|_| err("source-seal-signature"))?;
    key.verify(source_json::canonical(p)?.as_bytes(), &sig)
        .map_err(|_| err("source-seal-signature"))?;
    let id = text(seal, "sealId")?;
    let signed = S::Object(vec![
        ("payload".encode_utf16().collect(), p.clone()),
        (
            "signature".encode_utf16().collect(),
            seal.field("signature")?.clone(),
        ),
    ]);
    require(
        id == hash(source_json::canonical(&signed)?),
        "source-seal-id",
    )?;
    let source = evidence.field("source")?;
    let receipt = evidence.field("receipt")?;
    require(
        matches!(source, S::Object(_))
            && text(receipt, "kind").is_ok_and(|s| s == "commit")
            && text(receipt, "status").is_ok_and(|s| s == "committed"),
        "source-commit-receipt",
    )?;
    require(
        ["mutationId", "proposalId", "admissionId", "requestId"]
            .iter()
            .all(|k| {
                text(source, k).is_ok_and(|s| !s.is_empty())
                    && source.field(k).ok() == receipt.field(k).ok()
            }),
        "source-commit-identity",
    )?;
    let committed = text(source, "committedRevision").map_err(|_| err("source-commit-effects"))?;
    let delta = source
        .field("delta")
        .map_err(|_| err("source-commit-effects"))?;
    require(
        receipt.field("committedRevision").ok() == source.field("committedRevision").ok()
            && matches!(delta, S::Array(_))
            && receipt.field("delta").ok() == Some(delta),
        "source-commit-effects",
    )?;
    require(
        event["payload"]["requestId"] == format!("seal:{id}"),
        "source-event-binding",
    )?;
    let mut result = runtime::reconcile(contract, &cursor["runtime"], event)?;
    require(
        source.field("command").ok() == Some(&from(&event["payload"]["command"])?)
            && source.field("args").ok() == Some(&from(&event["payload"]["args"])?)
            && delta == &from(&event["payload"]["effects"])?,
        "source-event-effects",
    )?;
    result["sourceSealId"] = json!(id);
    if result["status"] == "replay" {
        result["cursor"] = cursor.clone();
        return Ok(result);
    }
    require(
        [
            ("index", "nextSealIndex"),
            ("previousSealId", "previousSealId"),
        ]
        .iter()
        .all(|(k, c)| text(p, k).ok().as_deref() == cursor["source"][c].as_str())
            && [
                ("previousMutationId", "mutationId"),
                ("previousRevision", "revision"),
            ]
            .iter()
            .all(|(k, c)| text(source, k).ok().as_deref() == cursor["source"][c].as_str()),
        "source-event-gap",
    )?;
    let previous = text(source, "previousRevision")?;
    require(
        revision(&committed) && revision(&previous),
        "source-event-revision",
    )?;
    require(
        crate::decimal(&json!(&committed[6..]))?
            == crate::decimal(&json!(&previous[6..]))?
                + num_bigint::BigUint::from(delta.array()?.len()),
        "source-event-revision",
    )?;
    let next = (crate::decimal(&cursor["source"]["nextSealIndex"])?
        + num_bigint::BigUint::from(1u8))
    .to_string();
    result["cursor"] = json!({"runtime":result["cursor"],"source":{"previousSealId":id,"nextSealIndex":next,"mutationId":text(source,"mutationId")?,"revision":committed}});
    Ok(result)
}
pub fn replay_journal(
    contract: &Value,
    anchor: &Value,
    records: &Value,
    expected_tip: &Value,
) -> Result<Value> {
    let tip_anchor = if expected_tip.is_null() {
        json!({})
    } else {
        json!({"expectedTip":expected_tip})
    };
    let tip = crate::identity::verify_records(records, &tip_anchor)?;
    let mut cursor = initial_cursor(contract, anchor)?;
    for record in crate::array(records, "invalid-journal")? {
        let entry = &record["entry"];
        crate::shape(entry, "kind sealJson event receipt").map_err(|_| err("source-seal-shape"))?;
        require(
            entry["kind"] == "sourcery-sealed-shadow-reconciliation",
            "sealed-receiver-record",
        )?;
        let encoded = crate::string(&entry["sealJson"], "source-seal-size")?;
        require(encoded.len() <= 65536, "source-seal-size")?;
        let seal = source_json::parse(encoded)?;
        require(
            source_json::canonical(&seal)? == encoded,
            "source-seal-encoding",
        )?;
        let result = reconcile(contract, anchor, &cursor, &seal, &entry["event"])?;
        require(
            result["status"] == "recorded" && combined_receipt(&result) == entry["receipt"],
            "sealed-receiver-receipt",
        )?;
        cursor = result["cursor"].clone();
    }
    Ok(json!({"cursor":cursor,"tip":tip}))
}
fn combined_receipt(result: &Value) -> Value {
    json!({"kind":"temple3.sealed-shadow-receipt/v0.1","sourceSealId":result["sourceSealId"],"runtime":result["receipt"]})
}
pub struct SealedRuntimeReceiver {
    journal: crate::journal::FileJournal,
    contract: Value,
    anchor: Value,
}
impl SealedRuntimeReceiver {
    pub fn new(
        journal: crate::journal::FileJournal,
        contract: Value,
        anchor: Value,
    ) -> Result<Self> {
        initial_cursor(&contract, &anchor)?;
        Ok(Self {
            journal,
            contract,
            anchor,
        })
    }
    pub fn read(&self, tip: &Value) -> Result<Value> {
        require(tip.is_string(), "receiver-tip-required")?;
        replay_journal(
            &self.contract,
            &self.anchor,
            &json!(self.journal.read()?),
            tip,
        )
    }
    pub fn receive(&self, seal: &S, event: &Value, tip: &Value) -> Result<Value> {
        require(tip.is_string(), "receiver-tip-required")?;
        self.journal.transaction(|journal| {
            let before = replay_journal(&self.contract,&self.anchor,&json!(journal.read()?),tip)?;
            let mut result = reconcile(&self.contract,&self.anchor,&before["cursor"],seal,event)?;
            result["tip"] = if result["status"] == "replay" { before["tip"].clone() } else {
                journal.append(&json!({"kind":"sourcery-sealed-shadow-reconciliation","sealJson":source_json::canonical(seal)?,"event":event,"receipt":combined_receipt(&result)}),&before["tip"])?["recordId"].clone()
            };
            result["receipt"] = combined_receipt(&result);
            Ok(result)
        })
    }
}
