use crate::{Result, canonical, coordination, err, identity, require, shape};
use num_bigint::BigUint;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::str::FromStr;

fn ident(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        let Some((k, h)) = s.split_once(":sha256:") else {
            return false;
        };
        !k.is_empty()
            && k.bytes().next().is_some_and(|c| c.is_ascii_lowercase())
            && k.bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b".-".contains(&c))
            && h.len() == 64
            && h.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    })
}
fn dec(v: &Value) -> Option<BigUint> {
    let s = v.as_str()?;
    if s.is_empty() || s.starts_with('0') {
        return None;
    }
    BigUint::from_str(s).ok()
}
fn exact(v: &Value, fields: &str) -> Result<()> {
    shape(v, fields).map_err(|_| err("invalid-af6-synthetic-execution-transcript-bridge"))
}
fn profile_v01(v: &Value) -> Result<String> {
    shape(v,"kind instance stagingKey backoutKey maximumAttempts maximumRecordBytes maximumJournalBytes maximumStagingBytes retention recovery backout authority forgiveness").map_err(|_|err("invalid-representative-runtime-evidence-staging-profile"))?;
    shape(&v["forgiveness"], "disposition scope")
        .map_err(|_| err("invalid-representative-runtime-evidence-staging-profile"))?;
    identity::key_id(&v["stagingKey"])?;
    identity::key_id(&v["backoutKey"])?;
    require(
        v["kind"] == "temple3.representative-runtime-evidence-staging-profile/v0.1"
            && v["instance"].as_str().is_some_and(|s| {
                s.len() == 32
                    && s.bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            })
            && dec(&v["maximumAttempts"]).is_some_and(|n| n <= BigUint::from(64u8))
            && dec(&v["maximumRecordBytes"])
                .is_some_and(|n| n <= BigUint::from(4u32 * 1024 * 1024))
            && dec(&v["maximumJournalBytes"])
                .is_some_and(|n| n <= BigUint::from(128u32 * 1024 * 1024))
            && dec(&v["maximumStagingBytes"])
                .is_some_and(|n| n <= BigUint::from(16u32 * 1024 * 1024))
            && v["retention"] == "retain-all-preparation-and-backout-evidence-no-pruning"
            && v["recovery"]
                == "caller-anchored-fixed-suffix-read-only-discovery-exact-selection-no-adoption"
            && v["backout"] == "signed-preactivation-isolated-staging-discard-retain-evidence"
            && v["authority"]
                == "target-staging-laboratory-only-not-source-product-activation-or-migration-authority"
            && v["forgiveness"]
                == json!({"disposition":"refuse","scope":"representative-runtime-evidence-staging-v0.1"}),
        "invalid-representative-runtime-evidence-staging-profile",
    )?;
    canonical::identify("representative-runtime-evidence-staging-profile", v)
}
fn profile_v02(v: &Value) -> Result<String> {
    shape(v,"kind instance recoveryKey maximumIntents maximumRecordBytes maximumJournalBytes retention recovery authority forgiveness").map_err(|_|err("invalid-representative-runtime-evidence-backout-recovery-profile"))?;
    shape(&v["forgiveness"], "disposition scope")
        .map_err(|_| err("invalid-representative-runtime-evidence-backout-recovery-profile"))?;
    identity::key_id(&v["recoveryKey"])?;
    require(
        v["kind"] == "temple3.representative-runtime-evidence-backout-recovery-profile/v0.2"
            && v["instance"].as_str().is_some_and(|s| {
                s.len() == 32
                    && s.bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            })
            && dec(&v["maximumIntents"]).is_some_and(|n| n <= BigUint::from(64u8))
            && dec(&v["maximumRecordBytes"])
                .is_some_and(|n| n <= BigUint::from(4u32 * 1024 * 1024))
            && dec(&v["maximumJournalBytes"])
                .is_some_and(|n| n <= BigUint::from(128u32 * 1024 * 1024))
            && v["retention"] == "retain-all-intents-and-completions-no-pruning"
            && v["recovery"]
                == "read-only-discovery-exact-caller-selection-no-implicit-retry-or-adoption"
            && v["authority"]
                == "target-staging-backout-recovery-laboratory-only-no-source-product-activation-or-migration-authority"
            && v["forgiveness"]
                == json!({"disposition":"refuse","scope":"representative-runtime-evidence-backout-recovery-v0.2"}),
        "invalid-representative-runtime-evidence-backout-recovery-profile",
    )?;
    canonical::identify(
        "representative-runtime-evidence-backout-recovery-profile",
        v,
    )
}

fn target_execution(v: &Value) -> Result<String> {
    exact(
        v,
        "admission checkpointSelection descriptor expectedResults expectedTempleId genesis kind native steps",
    )?;
    let steps = v["steps"]
        .as_array()
        .ok_or_else(|| err("af6-execution-target-transcript"))?;
    let expected = v["expectedResults"]
        .as_array()
        .ok_or_else(|| err("af6-execution-target-transcript"))?;
    require(
        v["kind"] == "temple3.af6-synthetic-target-execution/v0.1"
            && v["expectedTempleId"].is_string()
            && !steps.is_empty()
            && steps.len() == expected.len(),
        "af6-execution-target-transcript",
    )?;
    exact(
        &v["admission"],
        "certificateId commitName finalityId proposalId proposalName requestId",
    )?;
    exact(
        &v["native"],
        "certificateId commitName predecessorCommitName predecessorFinalityId proposalId proposalName requestId resultingFinalityId",
    )?;
    exact(
        &v["checkpointSelection"],
        "candidateId disposition checkpointId",
    )?;
    for x in [
        &v["admission"]["certificateId"],
        &v["admission"]["finalityId"],
        &v["admission"]["proposalId"],
        &v["native"]["certificateId"],
        &v["native"]["proposalId"],
        &v["native"]["predecessorFinalityId"],
        &v["native"]["resultingFinalityId"],
        &v["checkpointSelection"]["candidateId"],
        &v["checkpointSelection"]["checkpointId"],
    ] {
        require(ident(x), "af6-execution-target-transcript-identity")?
    }
    require(
        v["admission"]["certificateId"] != v["admission"]["finalityId"]
            && v["native"]["certificateId"] != v["native"]["resultingFinalityId"]
            && v["native"]["resultingFinalityId"] != v["native"]["predecessorFinalityId"]
            && v["checkpointSelection"]["disposition"] == "response-lost-exact-selected",
        "af6-execution-target-transcript-binding",
    )?;
    let mut state = crate::claim_import_application_profile::recognize(
        &v["genesis"],
        &v["descriptor"],
        &v["expectedTempleId"],
    )?;
    let mut results = Vec::new();
    let mut named: BTreeMap<String, &Value> = BTreeMap::new();
    let mut accepted: BTreeMap<String, bool> = BTreeMap::new();
    for step in steps {
        let name = step["name"]
            .as_str()
            .ok_or_else(|| err("af6-execution-signed-step-order"))?
            .to_owned();
        require(
            named.insert(name.clone(), step).is_none(),
            "af6-execution-signed-step-order",
        )?;
        match step["kind"].as_str(){Some("proposal")=>{let r=match coordination::candidate(&state,&step["proposal"]){Ok(c)=>json!({"accepted":true,"witnessId":c["witness"]["witnessId"]}),Err(_)=>json!({"accepted":false})};accepted.insert(name,r["accepted"]==true);results.push(r)},Some("certificate")=>results.push(json!({"accepted":coordination::verify_certificate(&state,&step["certificate"]).is_ok()})),Some("commit")=>{let proposal=&step["certificate"]["proposal"];require(named.iter().any(|(n,s)|s["kind"]=="proposal"&&accepted.get(n)==Some(&true)&&s["proposal"]==*proposal),"af6-execution-signed-step-order")?;state=coordination::advance(&state,&step["certificate"])?;results.push(json!({"stateRoot":state["temple"]["stateRoot"],"finalityId":state["finalityId"],"sequence":state["temple"]["sequence"]}))},_=>return Err(err("af6-execution-signed-step-order"))}
    }
    require(
        Value::Array(results.clone()) == v["expectedResults"],
        "af6-execution-signed-replay-result-substitution",
    )?;
    let a = &v["admission"];
    let n = &v["native"];
    let get = |name: &Value| -> Result<&Value> {
        named
            .get(
                name.as_str()
                    .ok_or_else(|| err("af6-execution-signed-replay-binding"))?,
            )
            .copied()
            .ok_or_else(|| err("af6-execution-signed-replay-binding"))
    };
    let ap = get(&a["proposalName"])?;
    let ac = get(&a["commitName"])?;
    let np = get(&n["proposalName"])?;
    let nc = get(&n["commitName"])?;
    let pc = get(&n["predecessorCommitName"])?;
    let index = |needle: &Value| {
        steps
            .iter()
            .position(|x| std::ptr::eq(x, needle))
            .ok_or_else(|| err("af6-execution-signed-replay-binding"))
    };
    let ar = &expected[index(ac)?];
    let nr = &expected[index(nc)?];
    let pr = &expected[index(pc)?];
    require(
        ap["kind"] == "proposal"
            && ac["kind"] == "commit"
            && np["kind"] == "proposal"
            && nc["kind"] == "commit"
            && pc["kind"] == "commit"
            && ap["proposal"]["payload"]["requestId"] == a["requestId"]
            && identity::signed_id(&ap["proposal"])? == a["proposalId"]
            && ac["certificate"]["proposal"] == ap["proposal"]
            && ac["certificate"]["certificateId"] == a["certificateId"]
            && ar["finalityId"] == a["finalityId"]
            && np["proposal"]["payload"]["requestId"] == n["requestId"]
            && identity::signed_id(&np["proposal"])? == n["proposalId"]
            && nc["certificate"]["proposal"] == np["proposal"]
            && nc["certificate"]["certificateId"] == n["certificateId"]
            && pr["finalityId"] == n["predecessorFinalityId"]
            && nr["finalityId"] == n["resultingFinalityId"],
        "af6-execution-signed-replay-binding",
    )?;
    canonical::identify("af6-target-execution", v)
}

fn backout_execution(v: &Value) -> Result<String> {
    exact(
        v,
        "after application before candidate confirmedExit kind preparationReceipt receipt request v01Profile v02Profile",
    )?;
    require(
        v["kind"] == "temple3.af6-synthetic-preactivation-backout-execution/v0.1",
        "af6-execution-backout-transcript",
    )?;
    exact(&v["request"], "authorization requestId")?;
    exact(
        &v["before"],
        "activation externalEffects staging v01AcknowledgedAnchor v01CurrentAnchor v02Anchor",
    )?;
    exact(
        &v["after"],
        "activation externalEffects staging v01AcknowledgedAnchor v01CurrentAnchor v02Anchor",
    )?;
    exact(&v["confirmedExit"], "exitCode signal")?;
    let auth = &v["request"]["authorization"];
    let p = &auth["payload"];
    let receipt = &v["receipt"];
    exact(
        receipt,
        "anchor application attemptId authenticAdmission completionRecordId effects intentRecordId kind readyForAF5 readyForAF6 readyForMigration status",
    )?;
    exact(
        &receipt["effects"],
        "activations productWrites sourceReads stagingDeletes",
    )?;
    require(
        profile_v01(&v["v01Profile"])? == p["v01ProfileId"]
            && profile_v02(&v["v02Profile"])? == p["profileId"]
            && identity::verify_envelope(auth, "attestation", &v["v02Profile"]["recoveryKey"]),
        "af6-execution-backout-authorization",
    )?;
    require(
        v["request"]["requestId"] == p["requestId"]
            && v["candidate"] == p["candidate"]
            && v["before"]["v01AcknowledgedAnchor"] == p["v01Anchor"],
        "af6-execution-backout-request-binding",
    )?;
    require(
        receipt["kind"] == "temple3.representative-runtime-evidence-backout-recovery-receipt/v0.2"
            && receipt["status"] == "backed-out-before-activation-intent-and-completion-retained"
            && ident(&receipt["intentRecordId"])
            && ident(&receipt["completionRecordId"])
            && receipt["anchor"]["tip"] == receipt["completionRecordId"]
            && receipt["anchor"]["count"] == "2"
            && receipt["application"] == v["application"]
            && receipt["application"] == v["candidate"]["application"]
            && receipt["attemptId"] == v["candidate"]["attemptId"]
            && receipt["effects"]
                == json!({"stagingDeletes":"1","sourceReads":"0","productWrites":"0","activations":"0"})
            && [
                "authenticAdmission",
                "readyForAF5",
                "readyForAF6",
                "readyForMigration",
            ]
            .iter()
            .all(|k| receipt[*k] == false),
        "af6-execution-backout-receipt-binding",
    )?;
    let before = json!({"activation":"unavailable","externalEffects":"none","staging":"present","v01AcknowledgedAnchor":p["v01Anchor"],"v01CurrentAnchor":v["preparationReceipt"]["anchor"],"v02Anchor":v["before"]["v02Anchor"]});
    let after = json!({"activation":"unavailable","externalEffects":"none","staging":"discarded","v01AcknowledgedAnchor":p["v01Anchor"],"v01CurrentAnchor":v["preparationReceipt"]["anchor"],"v02Anchor":receipt["anchor"]});
    require(
        v["preparationReceipt"]["recordId"] == v["candidate"]["recordId"]
            && v["preparationReceipt"]["attemptId"] == v["candidate"]["attemptId"]
            && v["before"] == before
            && v["after"] == after
            && v["confirmedExit"] == json!({"exitCode":"0","signal":null}),
        "af6-execution-backout-derived-state",
    )?;
    canonical::identify("af6-preactivation-backout-execution", v)
}

pub fn recognize(v: &Value) -> Result<Value> {
    canonical::canonical_json(v)?;
    exact(
        v,
        "artifact backoutExecution bridge recognition targetExecution",
    )?;
    let recognition =
        crate::af6_cross_language_artifact_recognition_v01::recognize(&v["artifact"])?;
    let zero = json!({"sourceReads":"0","runtimeCaptures":"0","targetWrites":"0","activations":"0","backouts":"0"});
    require(
        recognition == v["recognition"]
            && recognition["schemaRecognized"] == true
            && recognition["syntheticOnly"] == true
            && recognition["authenticAuthority"] == false
            && recognition["af6Started"] == false
            && recognition["effects"] == zero,
        "af6-execution-recognition-substitution",
    )?;
    require(
        identity::verify_envelope(
            &v["bridge"],
            "attestation",
            &v["artifact"]["authority"]["governorKey"],
        ),
        "af6-execution-bridge-signature",
    )?;
    let target_id = target_execution(&v["targetExecution"])?;
    let backout_id = backout_execution(&v["backoutExecution"])?;
    let p = &v["bridge"]["payload"];
    exact(
        p,
        "artifactId backout backoutExecutionId forgiveness kind scope target targetExecutionId",
    )?;
    let expected = json!({"kind":"temple3.af6-synthetic-execution-bridge/v0.1","artifactId":p["artifactId"],"targetExecutionId":p["targetExecutionId"],"backoutExecutionId":p["backoutExecutionId"],"target":p["target"],"backout":p["backout"],"forgiveness":{"disposition":"refuse","scope":"af6-synthetic-execution-transcript-bridge-v0.1"},"scope":"synthetic-only-never-authentic-admission"});
    require(
        *p == expected
            && p["artifactId"] == recognition["artifactId"]
            && p["targetExecutionId"] == target_id
            && p["backoutExecutionId"] == backout_id,
        "af6-execution-bridge-payload",
    )?;
    let t = &p["target"];
    exact(
        t,
        "certificateId continuationCommandId continuationPredecessorFinalityId continuationResultFinalityId manifestCheckpointId manifestFinalityId manifestMappingId manifestRequestId selectedCheckpointTranscriptId selectionDisposition targetTranscriptId",
    )?;
    for (k, x) in t
        .as_object()
        .ok_or_else(|| err("invalid-af6-execution-target-identity"))?
    {
        if k != "selectionDisposition" {
            require(ident(x), "invalid-af6-execution-target-identity")?
        }
    }
    require(
        t["manifestRequestId"] == recognition["target"]["requestId"]
            && t["manifestMappingId"] == recognition["target"]["mappingId"]
            && t["manifestCheckpointId"] == recognition["target"]["checkpointId"]
            && t["manifestFinalityId"] == recognition["target"]["finalityId"]
            && t["certificateId"] != t["manifestFinalityId"]
            && t["continuationResultFinalityId"] != t["continuationPredecessorFinalityId"]
            && t["selectionDisposition"] == "response-lost-exact-selected",
        "af6-execution-manifest-target-substitution",
    )?;
    let te = &v["targetExecution"];
    require(
        t["targetTranscriptId"] == target_id
            && t["selectedCheckpointTranscriptId"] == te["checkpointSelection"]["checkpointId"]
            && t["certificateId"] == te["admission"]["certificateId"]
            && t["continuationCommandId"] == te["native"]["proposalId"]
            && t["continuationPredecessorFinalityId"] == te["native"]["predecessorFinalityId"]
            && t["continuationResultFinalityId"] == te["native"]["resultingFinalityId"],
        "af6-execution-native-transcript-substitution",
    )?;
    let b = &p["backout"];
    exact(
        b,
        "application attemptId branchId completionRecordId effects intentRecordId manifestBackoutId receiptId status",
    )?;
    exact(
        &b["effects"],
        "activations productWrites sourceReads stagingDeletes",
    )?;
    let receipt = &v["backoutExecution"]["receipt"];
    require(
        [
            "branchId",
            "receiptId",
            "intentRecordId",
            "completionRecordId",
        ]
        .iter()
        .all(|k| ident(&b[*k]))
            && b["manifestBackoutId"] == recognition["backout"]["backoutId"]
            && b["branchId"] != t["targetTranscriptId"]
            && b["branchId"] != t["selectedCheckpointTranscriptId"]
            && b["status"] == "backed-out-before-activation-intent-and-completion-retained"
            && b["receiptId"] == canonical::identify("af6-v02-backout-receipt", receipt)?
            && b["intentRecordId"] == receipt["intentRecordId"]
            && b["completionRecordId"] == receipt["completionRecordId"]
            && b["application"] == receipt["application"]
            && b["attemptId"] == receipt["attemptId"]
            && b["effects"] == receipt["effects"],
        "af6-execution-backout-receipt-substitution",
    )?;
    let mut out = Map::new();
    for (k, x) in [
        (
            "kind",
            json!("temple3.af6-synthetic-execution-transcript-bridge-assessment/v0.1"),
        ),
        (
            "status",
            json!("synthetic-transcript-recognized-authentic-admission-refused"),
        ),
        (
            "bridgeId",
            json!(canonical::identify(
                "af6-synthetic-execution-transcript-bridge",
                &v["bridge"]
            )?),
        ),
        ("artifactId", recognition["artifactId"].clone()),
        ("targetExecutionId", json!(target_id)),
        ("backoutExecutionId", json!(backout_id)),
        (
            "checkpointSelection",
            json!({"transcriptId":t["selectedCheckpointTranscriptId"],"disposition":t["selectionDisposition"]}),
        ),
        ("certificateId", t["certificateId"].clone()),
        ("finalityId", t["manifestFinalityId"].clone()),
        (
            "continuation",
            json!({"commandId":t["continuationCommandId"],"predecessorFinalityId":t["continuationPredecessorFinalityId"],"resultingFinalityId":t["continuationResultFinalityId"]}),
        ),
        ("backout", b.clone()),
        ("syntheticOnly", json!(true)),
        ("authenticAuthority", json!(false)),
        ("af6Started", json!(false)),
        ("readyForAF6", json!(false)),
        ("readyForMigration", json!(false)),
        ("productionAuthorized", json!(false)),
        ("effects", zero),
        (
            "refusal",
            json!({"code":"synthetic-execution-transcript-never-authentic-authority","missing":["governed-sourcery-authority","authentic-source-and-target-journey","operative-migration-authority","production-cutover-and-ark-sealing"]}),
        ),
    ] {
        out.insert(k.into(), x);
    }
    Ok(Value::Object(out))
}
