//! Pure recognition of the AF6 synthetic seven-artifact envelope.
//! Successful recognition never supplies authentic authority or performs effects.
use crate::{Result, canonical, err, identity, require, shape};
use num_bigint::BigUint;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn canonical_hash(value: &Value) -> Result<String> {
    Ok(hash(canonical::canonical_json(value)?.as_bytes()))
}
fn hex64(value: &Value) -> bool {
    value.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
fn decimal(value: &Value) -> Option<BigUint> {
    let s = value.as_str()?;
    if s == "0" || (!s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit())) {
        BigUint::parse_bytes(s.as_bytes(), 10)
    } else {
        None
    }
}
fn token(value: &Value) -> bool {
    value
        .as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 160)
}
fn exact_identity(value: &Value) -> bool {
    let Some(s) = value.as_str() else {
        return false;
    };
    let Some((prefix, digest)) = s.rsplit_once(":sha256:") else {
        return false;
    };
    !prefix.is_empty()
        && prefix.as_bytes()[0].is_ascii_lowercase()
        && prefix
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
        && digest.len() == 64
        && digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn source_head(source: &Value, corpus: &Value) -> Result<String> {
    canonical_hash(
        &json!({"kind":"temple3.af6-synthetic-source-head/v0.1","application":source["application"],
        "generation":source["generation"],"orderedRecordDigests":corpus["orderedRecordDigests"],
        "corpusSha256":corpus["sha256"],"recordCount":corpus["recordCount"],"bytes":corpus["bytes"],
        "completeness":source["completeness"]}),
    )
}
fn validate_policy(value: &Value) -> Result<()> {
    shape(
        value,
        "backout forgiveness kind maximumCorpusBytes maximumRecords recovery retention",
    )?;
    shape(&value["forgiveness"], "disposition scope")?;
    require(
        value["kind"] == "temple3.af6-migration-manifest-policy/v0.1"
            && decimal(&value["maximumCorpusBytes"]).is_some_and(|n| n > BigUint::from(0u8))
            && decimal(&value["maximumRecords"]).is_some_and(|n| n > BigUint::from(0u8))
            && value["retention"]
                == "retain-exact-synthetic-corpus-and-manifest-identities-no-pruning"
            && value["recovery"] == "exact-checkpoint-only-no-tail-adoption-or-automatic-retry"
            && value["backout"] == "preactivation-isolated-stage-only"
            && value["forgiveness"]
                == json!({"disposition":"refuse","scope":"af6-migration-manifest-v0.1"}),
        "invalid-af6-migration-policy",
    )
}
fn validate_source(value: &Value, policy: &Value) -> Result<(Value, Vec<String>)> {
    shape(
        value,
        "application completeness generation runtimeHead corpus",
    )?;
    shape(&value["completeness"], "scope statementId status")?;
    require(
        token(&value["application"])
            && token(&value["generation"])
            && hex64(&value["runtimeHead"])
            && value["completeness"]["status"] == "complete"
            && value["completeness"]["scope"] == "exact-runtime-head"
            && hex64(&value["completeness"]["statementId"]),
        "invalid-af6-source",
    )?;
    let corpus = &value["corpus"];
    shape(
        corpus,
        "bytes encoding orderedRecordDigests recordCount records sha256",
    )?;
    let records = corpus["records"]
        .as_array()
        .ok_or_else(|| err("invalid-af6-corpus"))?;
    require(
        corpus["encoding"] == "ordered-opaque-hex-records-v1"
            && !records.is_empty()
            && decimal(&corpus["recordCount"]) == Some(BigUint::from(records.len()))
            && decimal(&corpus["bytes"]).is_some(),
        "invalid-af6-corpus",
    )?;
    let mut bytes = BigUint::from(0u8);
    let mut digests = Vec::new();
    for record in records {
        shape(record, "payloadHex payloadSha256 recordId")?;
        let encoded = record["payloadHex"]
            .as_str()
            .ok_or_else(|| err("invalid-af6-corpus-framing"))?;
        require(
            !encoded.is_empty()
                && encoded.len() % 2 == 0
                && encoded
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "invalid-af6-corpus-framing",
        )?;
        let payload = hex::decode(encoded).map_err(|_| err("invalid-af6-corpus-framing"))?;
        let digest = hash(&payload);
        require(
            record["payloadSha256"] == digest && record["recordId"] == digest,
            "invalid-af6-corpus-record-digest",
        )?;
        bytes += BigUint::from(payload.len());
        digests.push(digest);
    }
    require(
        digests.iter().collect::<BTreeSet<_>>().len() == digests.len()
            && corpus["orderedRecordDigests"] == json!(digests)
            && corpus["sha256"] == canonical_hash(&corpus["records"])?
            && decimal(&corpus["bytes"]) == Some(bytes.clone())
            && BigUint::from(records.len())
                <= decimal(&policy["maximumRecords"]).unwrap_or_default()
            && bytes <= decimal(&policy["maximumCorpusBytes"]).unwrap_or_default(),
        "invalid-af6-corpus-identity-or-capacity",
    )?;
    require(
        value["runtimeHead"] == source_head(value, corpus)?,
        "invalid-af6-runtime-head",
    )?;
    Ok((corpus.clone(), digests))
}
fn authority_statement(source: &Value, corpus: &Value, authority: &Value) -> Value {
    json!({
    "kind":"temple3.af6-synthetic-authority-bundle/v0.1","application":source["application"],"generation":source["generation"],
    "runtimeHead":source["runtimeHead"],"corpusSha256":corpus["sha256"],"recordCount":corpus["recordCount"],"bytes":corpus["bytes"],
    "orderedRecordDigests":corpus["orderedRecordDigests"],"completeness":source["completeness"],
    "roles":{"exporterKey":authority["exporterKey"],"captureKey":authority["captureKey"],"custodyKey":authority["custodyKey"],"transportKey":authority["transportKey"]},
    "scope":"synthetic-fixture-only-never-authentic-admission"})
}
fn receipt_statement(kind: &str, grant_id: &str, source: &Value, corpus: &Value) -> Value {
    json!({
    "kind":format!("temple3.af6-synthetic-{kind}-receipt/v0.1"),"grantId":grant_id,"application":source["application"],
    "generation":source["generation"],"runtimeHead":source["runtimeHead"],"corpusSha256":corpus["sha256"],
    "recordCount":corpus["recordCount"],"bytes":corpus["bytes"],"orderedRecordDigests":corpus["orderedRecordDigests"],
    "scope":"synthetic-fixture-only-never-authentic-admission"})
}
fn validate_authority(
    value: &Value,
    source: &Value,
    corpus: &Value,
) -> Result<(String, String, String)> {
    shape(
        value,
        "captureKey captureReceipt custodyKey custodyReceipt exporterKey exportReceipt governorKey grant retainedAnchor terminalReceipt transportKey transportReceipt",
    )?;
    let names = [
        "governorKey",
        "exporterKey",
        "captureKey",
        "custodyKey",
        "transportKey",
    ];
    require(
        names.iter().all(|n| hex64(&value[*n])),
        "invalid-af6-authority-key",
    )?;
    require(
        names
            .iter()
            .map(|n| value[*n].as_str().unwrap())
            .collect::<BTreeSet<_>>()
            .len()
            == 5,
        "invalid-af6-authority-role-ambiguity",
    )?;
    require(
        identity::verify_envelope(&value["grant"], "attestation", &value["governorKey"])
            && value["grant"]["payload"] == authority_statement(source, corpus, value),
        "invalid-af6-authority-grant",
    )?;
    let grant_id = identity::signed_id(&value["grant"])?;
    for (kind, key, envelope) in [
        ("export", "exporterKey", "exportReceipt"),
        ("capture", "captureKey", "captureReceipt"),
        ("custody", "custodyKey", "custodyReceipt"),
        ("transport", "transportKey", "transportReceipt"),
    ] {
        require(
            identity::verify_envelope(&value[envelope], "attestation", &value[key])
                && value[envelope]["payload"] == receipt_statement(kind, &grant_id, source, corpus),
            "invalid-af6-authority-receipt",
        )?;
    }
    let terminal = receipt_statement("terminal-source", &grant_id, source, corpus);
    require(
        identity::verify_envelope(
            &value["terminalReceipt"],
            "attestation",
            &value["transportKey"],
        ) && value["terminalReceipt"]["payload"] == terminal,
        "invalid-af6-terminal-source-receipt",
    )?;
    let terminal_id = identity::signed_id(&value["terminalReceipt"])?;
    let anchor = json!({"kind":"temple3.af6-synthetic-retained-anchor/v0.1","grantId":grant_id,"terminalReceiptId":terminal_id,
        "runtimeHead":source["runtimeHead"],"corpusSha256":corpus["sha256"],"recordCount":corpus["recordCount"],"bytes":corpus["bytes"],
        "scope":"synthetic-fixture-only-never-authentic-admission"});
    require(
        identity::verify_envelope(
            &value["retainedAnchor"],
            "attestation",
            &value["custodyKey"],
        ) && value["retainedAnchor"]["payload"] == anchor,
        "invalid-af6-retained-anchor",
    )?;
    Ok((
        grant_id,
        terminal_id,
        identity::signed_id(&value["retainedAnchor"])?,
    ))
}
fn validate_ledger(value: &Value, digests: &[String]) -> Result<Value> {
    shape(value, "added changed preserved removed unsupported")?;
    let mut seen = Vec::new();
    for (kind, data, meaning) in [
        ("preserved", "copy", "equivalent"),
        ("changed", "transform", "redesigned"),
        ("removed", "no-target", "removed-with-explicit-reason"),
        ("unsupported", "quarantine", "unresolved"),
    ] {
        let rows = value[kind]
            .as_array()
            .ok_or_else(|| err("invalid-af6-ledger"))?;
        for row in rows {
            shape(
                row,
                "authority data evidenceRecordId meaning sourceRecordId targetRepresentation",
            )?;
            require(
                row["sourceRecordId"] == row["evidenceRecordId"]
                    && digests.iter().any(|x| row["sourceRecordId"] == *x)
                    && ["targetRepresentation", "authority", "data", "meaning"]
                        .iter()
                        .all(|n| token(&row[*n])),
                "invalid-af6-ledger-row",
            )?;
            require(
                row["data"] == data && row["meaning"] == meaning,
                "invalid-af6-ledger-classification",
            )?;
            seen.push(row["sourceRecordId"].as_str().unwrap().to_string());
        }
    }
    let seen_set = seen.iter().collect::<BTreeSet<_>>();
    let digest_set = digests.iter().collect::<BTreeSet<_>>();
    require(
        seen_set.len() == seen.len() && seen_set == digest_set,
        "invalid-af6-ledger-coverage-or-ambiguity",
    )?;
    let added = value["added"]
        .as_array()
        .ok_or_else(|| err("invalid-af6-ledger-added"))?;
    let mut added_ids = Vec::new();
    for row in added {
        shape(
            row,
            "authority data meaning targetRecordId targetRepresentation",
        )?;
        require(
            hex64(&row["targetRecordId"])
                && token(&row["targetRepresentation"])
                && token(&row["authority"])
                && row["data"] == "target-only"
                && row["meaning"] == "added-no-source-equivalence",
            "invalid-af6-ledger-added-row",
        )?;
        added_ids.push(row["targetRecordId"].as_str().unwrap());
    }
    require(
        added_ids.iter().collect::<BTreeSet<_>>().len() == added_ids.len(),
        "invalid-af6-ledger-added-ambiguity",
    )?;
    Ok(
        json!({"preserved":value["preserved"].as_array().unwrap().len().to_string(),"changed":value["changed"].as_array().unwrap().len().to_string(),
        "removed":value["removed"].as_array().unwrap().len().to_string(),"unsupported":value["unsupported"].as_array().unwrap().len().to_string(),"added":added.len().to_string()}),
    )
}
pub fn recognize(value: &Value) -> Result<Value> {
    canonical::canonical_json(value)?;
    shape(
        value,
        "authority backout continuation finality kind ledger mode policy recovery source target",
    )?;
    require(
        value["kind"] == "temple3.af6-cross-language-artifact-recognition/v0.1"
            && value["mode"] == "synthetic-only-never-authentic-admission",
        "invalid-af6-manifest-mode",
    )?;
    validate_policy(&value["policy"])?;
    let (corpus, digests) = validate_source(&value["source"], &value["policy"])?;
    let (grant_id, terminal_id, anchor_id) =
        validate_authority(&value["authority"], &value["source"], &corpus)?;
    let counts = validate_ledger(&value["ledger"], &digests)?;
    let target = &value["target"];
    shape(
        target,
        "checkpointId finalityId mappingId requestId requestNonce targetFormationId targetHead",
    )?;
    require(
        target.as_object().unwrap().values().all(exact_identity),
        "invalid-af6-target-identity",
    )?;
    let bound = json!({"kind":"temple3.af6-synthetic-target-request-binding/v0.1","application":value["source"]["application"],"generation":value["source"]["generation"],
        "runtimeHead":value["source"]["runtimeHead"],"retainedAnchorId":anchor_id,"targetFormationId":target["targetFormationId"],"targetHead":target["targetHead"],
        "requestNonce":target["requestNonce"],"mappingId":target["mappingId"],"checkpointId":target["checkpointId"],"finalityId":target["finalityId"]});
    require(
        target["requestId"] == canonical::identify("af6-request", &bound)?,
        "invalid-af6-target-request-binding",
    )?;
    let recovery = &value["recovery"];
    shape(
        recovery,
        "checkpointId mappingId requestId retry sourceHead targetHead",
    )?;
    require(
        *recovery
            == json!({"checkpointId":target["checkpointId"],"mappingId":target["mappingId"],"requestId":target["requestId"],
        "retry":"exact-checkpoint-only-no-automatic-retry","sourceHead":value["source"]["runtimeHead"],"targetHead":target["targetHead"]}),
        "invalid-af6-recovery-checkpoint",
    )?;
    let finality = &value["finality"];
    shape(
        finality,
        "certificateId checkpointId finalityId mappingId requestId targetHead",
    )?;
    require(
        *finality
            == json!({"certificateId":target["finalityId"],"checkpointId":recovery["checkpointId"],"finalityId":target["finalityId"],
        "mappingId":recovery["mappingId"],"requestId":recovery["requestId"],"targetHead":recovery["targetHead"]}),
        "invalid-af6-finality-binding",
    )?;
    let continuation = &value["continuation"];
    shape(
        continuation,
        "commandId predecessorFinalityId resultingFinalityId status",
    )?;
    require(
        exact_identity(&continuation["commandId"])
            && exact_identity(&continuation["resultingFinalityId"])
            && continuation["predecessorFinalityId"] == target["finalityId"]
            && continuation["resultingFinalityId"] != target["finalityId"]
            && continuation["status"] == "native-command-finalized",
        "invalid-af6-native-continuation",
    )?;
    let backout = &value["backout"];
    shape(
        backout,
        "activation authority backoutId checkpointId externalEffects mappingId requestId sourceDisposition targetDisposition",
    )?;
    require(
        exact_identity(&backout["backoutId"])
            && backout["checkpointId"] == recovery["checkpointId"]
            && backout["requestId"] == target["requestId"]
            && backout["mappingId"] == target["mappingId"]
            && backout["sourceDisposition"] == "unchanged"
            && backout["targetDisposition"] == "discard-isolated-unactivated-staging-only"
            && backout["activation"] == "unavailable"
            && backout["externalEffects"] == "none"
            && backout["authority"] == "synthetic-separate-backout-authority",
        "invalid-af6-backout",
    )?;
    Ok(
        json!({"kind":"temple3.af6-cross-language-artifact-recognition-assessment/v0.1","status":"synthetic-schema-recognized-authentic-admission-refused",
        "artifactId":canonical::identify("af6-cross-language-artifact-recognition",value)?,"application":value["source"]["application"],"generation":value["source"]["generation"],
        "runtimeHead":value["source"]["runtimeHead"],"corpus":{"recordCount":corpus["recordCount"],"bytes":corpus["bytes"],"sha256":corpus["sha256"]},
        "authority":{"grantId":grant_id,"terminalReceiptId":terminal_id,"retainedAnchorId":anchor_id},"semanticCounts":counts,
        "target":{"requestId":target["requestId"],"mappingId":target["mappingId"],"checkpointId":target["checkpointId"],"finalityId":target["finalityId"]},
        "continuation":continuation,"backout":{"backoutId":backout["backoutId"],"targetDisposition":backout["targetDisposition"]},"policy":value["policy"],
        "schemaRecognized":true,"syntheticOnly":true,"authenticAuthority":false,"af6Started":false,"readyForAF6":false,"readyForMigration":false,"productionAuthorized":false,
        "effects":{"sourceReads":"0","runtimeCaptures":"0","targetWrites":"0","activations":"0","backouts":"0"},
        "refusal":{"code":"synthetic-schema-never-authentic-source-authority","missing":["governed-sourcery-read-export-or-capture-authority","authentic-runtime-corpus-and-terminal-source-receipt","operative-target-mapping-and-writer-authority","executed-authentic-recovery-finality-continuation-and-backout"]}}),
    )
}
