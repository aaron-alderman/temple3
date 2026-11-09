use crate::{Result, canonical, disclosure_grants, err, identity, require};
use serde_json::{Value, json};

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| err("relnav-af6-final-shape"))
}

pub fn recognize(value: &Value) -> Result<Value> {
    canonical::canonical_json(value)?;
    require(
        text(value, "kind")? == "temple3.relnav-af6-authentic-governed-retrofit-evidence/v0.1"
            && text(value, "status")?
                == "passed-authentic-source-bound-isolated-target-import-finality-native-continuation-and-preactivation-backout",
        "relnav-af6-final-profile",
    )?;
    let input = &value["manifestInput"];
    let authority = &input["targetAuthority"];
    let authority_key = &input["targetAuthorityPublicKey"];
    require(
        identity::verify_envelope(authority, "attestation", authority_key),
        "relnav-af6-target-authority-signature",
    )?;
    let ap = &authority["payload"];
    let signed = &input["signedManifest"];
    require(
        identity::verify_envelope(signed, "attestation", &ap["manifestSignerPublicKey"]),
        "relnav-af6-manifest-signature",
    )?;
    let manifest = &signed["payload"];
    require(
        ap["kind"] == "temple3.relnav-target-authority/v0.2"
            && ap["application"] == "app.relnav"
            && ap["choice"] == "restricted-resource-actor-target-retrofit"
            && ap["sourceReconciliationId"] == manifest["source"]["reconciliationId"]
            && ap["sourceAuthorityTransfer"] == "none"
            && ap["activation"] == "unavailable",
        "relnav-af6-authority",
    )?;
    require(
        manifest["kind"] == "temple3.relnav-authentic-import-manifest/v0.1"
            && manifest["application"] == "app.relnav"
            && manifest["mapping"]["choice"] == ap["choice"]
            && manifest["source"]["factCount"] == "42"
            && manifest["source"]["counts"]
                == json!({"added":"4","changed":"6","preserved":"21","removed":"0","unsupported":"15"})
            && manifest["source"]["sourceAuthorityTransfer"] == "none"
            && manifest["policy"]["activation"] == "unavailable"
            && manifest["policy"]["forgiveness"]
                == json!({"disposition":"refuse","scope":"relnav-authentic-import-manifest-v0.1"}),
        "relnav-af6-manifest",
    )?;
    let candidate: u128 = text(&manifest["target"], "candidateBytes")?
        .parse()
        .map_err(|_| err("relnav-af6-capacity"))?;
    let maximum: u128 = text(&manifest["target"], "maximumBytes")?
        .parse()
        .map_err(|_| err("relnav-af6-capacity"))?;
    let records: u128 = text(&manifest["target"], "recordCount")?
        .parse()
        .map_err(|_| err("relnav-af6-capacity"))?;
    let max_records: u128 = text(&manifest["target"], "maximumRecords")?
        .parse()
        .map_err(|_| err("relnav-af6-capacity"))?;
    require(
        candidate <= maximum && records <= max_records,
        "relnav-af6-capacity",
    )?;
    let manifest_id = identity::signed_id(signed)?;
    let authority_id = identity::signed_id(authority)?;
    let expected_recognition = json!({"kind":"temple3.relnav-authentic-import-manifest-recognition/v0.1","status":"recognized-signed-authentic-source-bound-restricted-target-manifest","manifestId":manifest_id,"targetAuthorityId":authority_id,"requestId":manifest["requestId"],"mappingId":manifest["mapping"]["mappingId"],"intendedFinalityId":manifest["intendedFinalityId"],"source":manifest["source"],"targetFormationId":manifest["target"]["formationId"],"capacity":{"boundary":if candidate==maximum{"exact-capacity"}else{"under-capacity"},"candidateBytes":manifest["target"]["candidateBytes"],"maximumBytes":manifest["target"]["maximumBytes"],"recordCount":manifest["target"]["recordCount"],"maximumRecords":manifest["target"]["maximumRecords"]},"policy":manifest["policy"],"targetMutation":false,"authenticMigration":false,"af6Completed":false,"effects":{"sourceReads":"0","targetWrites":"0","activations":"0","migrations":"0"}});
    require(
        value["recognition"] == expected_recognition,
        "relnav-af6-manifest-recognition",
    )?;
    let journals = &value["journals"];
    let control = journals["control"]
        .as_array()
        .ok_or_else(|| err("relnav-af6-final-journals"))?;
    let target = journals["target"]
        .as_array()
        .ok_or_else(|| err("relnav-af6-final-journals"))?;
    let backs = journals["backout"]
        .as_array()
        .ok_or_else(|| err("relnav-af6-final-journals"))?;
    require(
        control.len() == 1 && target.len() == 2 && backs.len() == 2,
        "relnav-af6-final-journal-counts",
    )?;
    let control_tip = identity::verify_records(&journals["control"], &json!({}))?;
    let target_tip = identity::verify_records(&journals["target"], &json!({}))?;
    let backout_tip = identity::verify_records(&journals["backout"], &json!({}))?;
    let prepared = &control[0];
    let finality = &target[0];
    let native = &target[1];
    let backout_preparation = &backs[0];
    let backout = &backs[1];
    let p = &prepared["entry"];
    require(
        p["kind"] == "temple3.relnav-authentic-retrofit-preparation/v0.1"
            && p["manifestId"] == manifest_id
            && p["requestId"] == manifest["requestId"]
            && p["mappingId"] == manifest["mapping"]["mappingId"]
            && p["intendedFinalityId"] == manifest["intendedFinalityId"],
        "relnav-af6-checkpoint",
    )?;
    let f = &finality["entry"];
    require(
        f["kind"] == "temple3.relnav-authentic-retrofit-finality/v0.1"
            && f["manifestId"] == manifest_id
            && f["requestId"] == manifest["requestId"]
            && f["mappingId"] == manifest["mapping"]["mappingId"]
            && f["intendedFinalityId"] == manifest["intendedFinalityId"]
            && f["activation"] == "unavailable"
            && f["sourceDisposition"] == "unchanged"
            && f["authorityTransfer"] == "none",
        "relnav-af6-finality",
    )?;
    let n = &native["entry"];
    require(
        n["kind"] == "temple3.relnav-authentic-retrofit-native-finality/v0.1"
            && n["manifestId"] == manifest_id
            && n["predecessorFinalityId"] == finality["recordId"]
            && n["activation"] == "unavailable",
        "relnav-af6-native",
    )?;
    let next = disclosure_grants::next_state(
        &f["targetState"],
        &json!("actor.target.steward"),
        &n["command"],
    )
    .map_err(|_| err("relnav-af6-native-transition"))?;
    require(n["targetState"] == next, "relnav-af6-native-transition")?;
    let b = &backout["entry"];
    require(
        backout_preparation["entry"] == prepared["entry"]
            && b == &json!({"kind":"temple3.relnav-authentic-retrofit-preactivation-backout/v0.1","manifestId":manifest_id,"requestId":manifest["requestId"],"mappingId":manifest["mapping"]["mappingId"],"sourceDisposition":"unchanged","targetDisposition":"discard-isolated-unactivated-staging-only","activation":"unavailable","externalEffects":"none"}),
        "relnav-af6-backout",
    )?;
    require(
        value["journey"]["applied"]["finalityId"] == finality["recordId"]
            && value["journey"]["native"]["predecessorFinalityId"] == finality["recordId"]
            && value["journey"]["native"]["finalityId"] == native["recordId"]
            && value["journey"]["backout"]["backoutId"] == backout["recordId"],
        "relnav-af6-result-binding",
    )?;
    require(
        value["qualification"]["semanticCounts"] == manifest["source"]["counts"]
            && value["qualification"]["sourceGrantTransfer"] == "none"
            && value["effects"]
                == json!({"sourceReads":"0","targetWrites":"2","activations":"0","migrations":"1","backouts":"1"})
            && value["boundaries"]["authenticMigration"] == true
            && value["boundaries"]["activation"] == false
            && value["boundaries"]["productionCutover"] == false
            && value["boundaries"]["arkSealed"] == false,
        "relnav-af6-boundary",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-af6-final-artifact-recognition/v0.1","status":"recognized-authentic-source-bound-isolated-migration-artifact","artifactId":canonical::identify("relnav-af6-final-artifact",value)?,"manifestId":manifest_id,"targetAuthorityId":authority_id,"requestId":manifest["requestId"],"mappingId":manifest["mapping"]["mappingId"],"sourceRuntimeHeadId":manifest["source"]["runtimeHeadId"],"sourceReceiptId":manifest["source"]["sourceReceiptId"],"recoveryReceiptId":manifest["source"]["recoveryReceiptId"],"semanticCounts":value["qualification"]["semanticCounts"],"checkpointId":prepared["recordId"],"finalityId":finality["recordId"],"nativeFinalityId":native["recordId"],"backoutId":backout["recordId"],"journalTips":{"control":control_tip,"target":target_tip,"backout":backout_tip},"authenticMigration":true,"nativeContinuation":true,"preactivationBackout":true,"activation":false,"productionAuthorized":false,"arkSealed":false,"af6Completed":false}),
    )
}
