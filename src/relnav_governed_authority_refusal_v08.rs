//! Independent byte-bound governed Relationship Navigator authority refusal v0.8.
use crate::{Result, require, shape};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const PATH: &str = "pilot/evidence/sourcery-relnav-governed-authority-capability-audit-v0.8.json";
const BYTES: usize = 9_497;
const SHA: &str = "4368478e9e098d987101866cb71327495292fd6519b112fb08a7b5766e6c5398";
fn searches() -> Value {
    json!([
     {"query":"runtime record export capture writer custody","candidates":0,"matches":0,"returned":0,"registryHash":"ea390dfd0a3a0c8691de000baa8db27d2e4478686e251299506c61bf9472e720"},
     {"query":"semantic boundary selection human decision authority","candidates":0,"matches":0,"returned":0,"registryHash":"4379d833809202802273bbe9e8655b73f2fed3210386da9cd16bcf1f589f9cff"},
     {"query":"non-mutating source materialization preview validation","candidates":3,"matches":0,"returned":0,"registryHash":"49ec849479308cc97eaddd7c806ad84f8b6c11becad3c8b530b6c7de6b6068d2"}
    ])
}
fn output_searches() -> Value {
    json!([
     {"query":"runtime record export capture writer custody","candidates":"0","matches":"0","returned":"0","registryHash":"ea390dfd0a3a0c8691de000baa8db27d2e4478686e251299506c61bf9472e720"},
     {"query":"semantic boundary selection human decision authority","candidates":"0","matches":"0","returned":"0","registryHash":"4379d833809202802273bbe9e8655b73f2fed3210386da9cd16bcf1f589f9cff"},
     {"query":"non-mutating source materialization preview validation","candidates":"3","matches":"0","returned":"0","registryHash":"49ec849479308cc97eaddd7c806ad84f8b6c11becad3c8b530b6c7de6b6068d2"}
    ])
}
fn prohibited() -> Value {
    json!([
        "semanticSelection",
        "governedPreview",
        "materialization",
        "runtimeRecords",
        "runtimeExport",
        "runtimeCapture",
        "writerCustody",
        "repair",
        "adoption",
        "migration",
        "activation",
        "productEffect"
    ])
}
fn policy() -> Value {
    json!({"witness":"exact-bytes-and-sha256","discovery":"authoring-orientation-is-not-runtime-or-product-authority","signing":"after-commit-signing-custody-is-not-runtime-writer-custody","searches":"zero-matches-remain-visible-refusals","operationDrift":"typed-limitation-not-authority","forgiveness":{"disposition":"refuse","prohibitedPromotions":prohibited()},"migration":"not-started"})
}
fn parse(value: &Value) -> Result<Value> {
    shape(value, "path bytes sha256 raw")?;
    let bytes_text = BYTES.to_string();
    require(
        value["path"] == PATH
            && value["bytes"].as_str() == Some(bytes_text.as_str())
            && value["sha256"] == SHA,
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    let raw = value["raw"]
        .as_str()
        .ok_or_else(|| crate::err("invalid-relnav-governed-authority-refusal-v08"))?;
    require(
        !raw.starts_with('\u{feff}')
            && !raw.contains('\r')
            && raw.starts_with("{\n")
            && raw.ends_with("}\n")
            && raw.lines().all(|x| !x.ends_with(' ') && !x.ends_with('\t')),
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    require(
        raw.len() == BYTES && hex::encode(Sha256::digest(raw.as_bytes())) == SHA,
        "relnav-governed-authority-witness-digest-mismatch",
    )?;
    serde_json::from_str(raw)
        .map_err(|_| crate::err("invalid-relnav-governed-authority-refusal-v08"))
}
pub fn assess(value: &Value) -> Result<Value> {
    crate::canonical::canonical_json(value)?;
    shape(value, "kind witness policy")?;
    require(
        value["kind"] == "temple3.relnav-governed-authority-refusal/v0.8"
            && value["policy"] == policy(),
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    let w = parse(&value["witness"])?;
    require(
        w["kind"] == "temple3.sourcery-relnav-governed-authority-capability-audit/v0.8"
            && w["lane"] == "governed-sourcery-mcp-read-only-capability-audit"
            && w["application"] == "app.relnav",
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    require(
        w["scope"]
            == json!({"projectId":"prj_e4f71823e9c84404b60a0c5e9b77334b","workspaceId":"wsp_aee16f3ba1af4325a95f9eac5baf82c9","role":"steward","scopes":["authoring.read","authoring.write","governance.decide"],"scopeMode":"application-authority-exact-workspace-join-v1"}),
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    require(
        w["request"]
            == json!({"requestId":"request-36b1-cdce-9c98","initialIntentHash":"0c5e64b0f8db837b12b03be9b7bf74854059ddf0cb6e32dadf97d6025b009439","initialIntentBytes":676,"authority":"authoring-request-journal-only"}),
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    let o = &w["orientation"];
    require(
        o["catalogHash"] == "df9edbf815ddf52807426d76b5f4997e246f5c13e4005cbe84a9f4123f9aab41"
            && o["sourceHash"]
                == "53e323cb26961d51db531630ec25693418624f1eb3e2e3094af08b9600a99146"
            && o["route"] == json!({"status":"clarify","action":"surface-hidden-intent"})
            && o["planPreview"]["status"] == "blocked",
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    require(
        o["observedEvidence"]["count"] == 0
            && o["observedEvidence"]["proofTiers"] == json!([])
            && o["observedEvidence"]["uses"] == json!([])
            && o["capabilityHorizon"]["declaredOperationalCapabilities"] == 0
            && o["importedAdmission"]["status"]
                == "semantic-adoption-required-before-managed-evolution"
            && o["importedAdmission"]["authority"] == "observation-only",
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    require(
        o["executionAdmission"]["callHash"]
            == "ac7c566fddf6f0b46dca2a58b640faae905aaad9c24b7b44fabd92ddfcefdc39"
            && o["executionAdmission"]["admissionHash"]
                == "22f663815351c59b2f4f5a79c4d88422a05a7f54239a4020baf562b980ac75ab",
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    let b = &w["operationalBindingDiscovery"];
    require(
        b["totalCandidates"] == 1
            && b["catalogHash"]
                == "644a4d3ac420931d351e8d08b116646338aa4efafc4dccdeb66ebd6b77ce2f6b"
            && b["discoveryHash"]
                == "814d59a9a41b7ad89dd39db8d122d87d2288b1580d7d697c9ba8aa6a5d95e154"
            && b["candidate"]["key"] == "offer.windows-export-custody.v1"
            && b["candidate"]["offerHash"]
                == "e8c5ef79c8411f64fe0d189d5db05d7c36198cd2bc7c4966e06d2ac5ad4f1f6c"
            && b["candidate"]["capability"] == "signing.after-commit"
            && b["candidate"]["lifecycle"] == json!(["observe"])
            && b["journalEffect"] == "two-authoring-request-checkpoints-only"
            && b["productEffect"] == false,
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    let actual = w["semanticRegistrySearches"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-relnav-governed-authority-refusal-v08"))?;
    let expected = searches();
    let expected = expected.as_array().unwrap();
    require(
        actual.len() == 3
            && actual.iter().zip(expected).all(|(a, e)| {
                e.as_object().unwrap().iter().all(|(k, v)| a[k] == *v)
                    && a["projectHash"]
                        == "f83760770c878550f0aeb184c6c0c4ed9cce736655661ab0c176dbb2fefebbb9"
            }),
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    let d = &w["admissionInspectionAttempt"];
    require(
        d["followedExactNextCall"] == true
            && d["status"] == "refused"
            && d["code"] == "sourcery-authoring-request-operation-drift"
            && d["productEffect"] == false
            && d["authorityGained"] == "none",
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    require(
        w["authorityAssessment"].as_object().is_some_and(|m| {
            m.iter().all(|(k, x)| {
                if k == "authoringDiscovery" {
                    x["availableNow"] == true
                        && x["authority"] == "observation-and-authoring-orientation-only"
                } else {
                    x["availableNow"] == false
                }
            })
        }),
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    let r = &w["requestCustody"];
    require(
        r["finalStatus"] == "stopped"
            && r["finalRevision"] == 5
            && r["finalHeadHash"]
                == "c2caf3e8576d7d49ca3e456cf55bf52d41850d54c5f9e946882194ad92b40b13"
            && r["finalLease"].is_null()
            && r["inboxJournalRevision"] == 66
            && r["inboxActiveLeaseCount"] == 0
            && r["eligibleRequestCount"] == 0,
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    require(
        w["effects"].as_object().is_some_and(|m| {
            m.iter()
                .all(|(k, x)| x.as_bool() == Some(k == "authoringRequestJournalMutation"))
        }) && w["conclusion"]
            .as_str()
            .is_some_and(|x| x.ends_with("No migration or retrofit demonstration has begun.")),
        "invalid-relnav-governed-authority-refusal-v08",
    )?;
    let authority = json!({"semanticSelection":false,"governedPreview":false,"materialization":false,"runtimeRecords":false,"runtimeExport":false,"runtimeCapture":false,"writerCustody":false,"repair":false,"adoption":false,"migration":false,"activation":false,"productEffect":false});
    Ok(
        json!({"kind":"temple3.relnav-governed-authority-assessment/v0.8","status":"refused-authoring-discovery-only","witness":{"path":PATH,"bytes":BYTES.to_string(),"sha256":SHA},"request":{"requestId":w["request"]["requestId"],"finalRevision":"5","finalHeadHash":r["finalHeadHash"],"finalStatus":"stopped","finalLease":Value::Null,"inboxRevision":"66","activeLeaseCount":"0"},"discovery":{"route":"clarify","plan":"blocked","observedEvidenceCount":"0","declaredOperationalCapabilityCount":"0","authority":"authoring-orientation-only"},"binding":{"candidateCount":"1","key":b["candidate"]["key"],"capability":"signing.after-commit","authority":"after-commit-signing-not-runtime-writer-custody"},"searches":output_searches(),"importedAdmission":"semantic-adoption-required-before-managed-evolution","operationDrift":{"status":"refused","code":d["code"],"authorityGained":"none"},"prohibitedPromotions":prohibited(),"authority":authority,"effects":w["effects"],"readyForMigration":false,"migration":"not-started","refusal":{"code":"governed-runtime-and-migration-authority-unavailable"},"policy":value["policy"]}),
    )
}
