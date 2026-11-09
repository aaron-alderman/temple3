//! Independent byte-bound Relationship Navigator runtime-authority v0.7 recognizer.
use crate::{Result, require, shape};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CURRENT_PATH: &str = "pilot/evidence/sourcery-relnav-source-repair-preflight-v0.1.json";
const CURRENT_BYTES: usize = 13_542;
const CURRENT_SHA: &str = "14a632042764810f510ec0671e599d456d695ccce6af680ac9ed03665b38381d";
const HISTORICAL_PATH: &str =
    "pilot/evidence/sourcery-relnav-governed-runtime-authority-refusal-v0.6.json";
const HISTORICAL_BYTES: usize = 5_036;
const HISTORICAL_SHA: &str = "ce0cdb8c11a78a6c2dd52e4306adda8d4fcba043878327b6214c877df9f47b1e";

fn named() -> Value {
    json!([
        "bundle.executableArtifacts.runtime",
        "ExecutableArtifact",
        "ExecutableApiContract",
        "ExecutableDependency",
        "ExecutableTestBinding"
    ])
}
fn unavailable() -> Value {
    json!([
        "executed-authenticated-provider-route-journey",
        "authoritative-runtime-record-head-and-completeness",
        "runtime-record-export-or-capture-authority",
        "actual-runtime-corpus-bytes",
        "operational-writer-and-record-custody",
        "transport-and-delivery-receipts",
        "executed-replay-and-recovery",
        "executed-backout"
    ])
}
fn policy() -> Value {
    json!({"historicalIssues":"five-named-plus-sixth-unavailable-never-exhaustive","currentSource":"valid-local-focused-concurrent-uncommitted-not-governed-repair-or-authority","sourceAdmission":"authenticated-hashes-do-not-transfer-runtime-writer-authority","runtimeUnknown":"null-and-unavailable-never-zero-or-absent","forgiveness":{"disposition":"refuse","scope":"executed-relnav-runtime-authority-v0.7"},"migration":"not-started"})
}
fn parse_raw(value: &Value, path: &str, bytes: usize, sha: &str) -> Result<Value> {
    shape(value, "path bytes sha256 raw")?;
    let bytes_text = bytes.to_string();
    require(
        value["path"] == path
            && value["bytes"].as_str() == Some(bytes_text.as_str())
            && value["sha256"] == sha,
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    let raw = value["raw"]
        .as_str()
        .ok_or_else(|| crate::err("invalid-relnav-runtime-authority-boundary-v07"))?;
    require(
        !raw.starts_with('\u{feff}')
            && !raw.contains('\r')
            && raw.starts_with("{\n")
            && raw.ends_with("}\n"),
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        raw.lines()
            .all(|line| !line.ends_with(' ') && !line.ends_with('\t')),
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        raw.len() == bytes && hex::encode(Sha256::digest(raw.as_bytes())) == sha,
        "relnav-runtime-authority-witness-digest-mismatch",
    )?;
    serde_json::from_str(raw)
        .map_err(|_| crate::err("invalid-relnav-runtime-authority-boundary-v07"))
}
fn no_effects(value: &Value) -> bool {
    value.as_object().is_some_and(|items| {
        items
            .iter()
            .all(|(key, item)| item.as_bool() == Some(key == "authoringRequestJournalMutation"))
    })
}
pub fn assess(value: &Value) -> Result<Value> {
    crate::canonical::canonical_json(value)?;
    shape(value, "kind currentWitness historicalWitness policy")?;
    require(
        value["kind"] == "temple3.relnav-runtime-authority-boundary/v0.7"
            && value["policy"] == policy(),
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    let current = parse_raw(
        &value["currentWitness"],
        CURRENT_PATH,
        CURRENT_BYTES,
        CURRENT_SHA,
    )?;
    let historical = parse_raw(
        &value["historicalWitness"],
        HISTORICAL_PATH,
        HISTORICAL_BYTES,
        HISTORICAL_SHA,
    )?;
    require(
        current["kind"] == "temple3.sourcery-relnav-source-repair-preflight/v0.1"
            && current["lane"]
                == "governed-sourcery-mcp-read-only-preflight-with-local-read-only-corroboration"
            && current["application"] == "app.relnav",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        current["scope"]
            == json!({"projectId":"prj_e4f71823e9c84404b60a0c5e9b77334b","workspaceId":"wsp_aee16f3ba1af4325a95f9eac5baf82c9","role":"steward","scopes":["authoring.read","authoring.write","governance.decide"]}),
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        historical["kind"] == "temple3.sourcery-relnav-governed-runtime-authority-refusal/v0.6"
            && historical["application"] == current["application"]
            && historical["lane"] == "governed-sourcery-mcp-fenced-read-only-investigation",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        historical["scope"]
            == json!({"projectId":current["scope"]["projectId"],"workspaceId":current["scope"]["workspaceId"]}),
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    let h = &current["historicalRefusal"];
    require(
        h["witness"] == HISTORICAL_PATH
            && h["observedAt"] == historical["observedAt"]
            && h["requestId"] == historical["request"]["requestId"]
            && h["requestFinalRevision"] == historical["request"]["finalRevision"]
            && h["requestFinalHeadHash"] == historical["request"]["finalHeadHash"],
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        h["issueCount"] == 6 && historical["currentSourceDiscovery"]["issueCount"] == 6,
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    let records = h["issueRecords"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-relnav-runtime-authority-boundary-v07"))?;
    let five: Vec<Value> = records
        .iter()
        .take(5)
        .map(|item| item["subjectOrSymbol"].clone())
        .collect();
    require(
        Value::Array(five) == named()
            && records.len() == 6
            && records[5]["ordinal"] == 6
            && records[5]["retainedCategory"] == "unavailable"
            && records[5]["subjectOrSymbol"] == "unavailable",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        records.iter().all(|item| {
            item["exactValidatorIssueRecord"]
                .as_str()
                .is_some_and(|text| {
                    text.starts_with("unavailable-from-retained-v0.6-witness-and-request-journal")
                })
        }),
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        h["authority"]
            == "historical-authenticated-refusal-summary-not-current-source-state-or-applied-repair"
            && h["failedSourceSnapshot"]["sourceBlobHash"].is_null()
            && h["failedSourceSnapshot"]["projectManifestHash"].is_null()
            && h["failedSourceSnapshot"]["governedSourceObservationHash"].is_null(),
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        historical["currentSourceDiscovery"]["missingBundle"] == named()[0]
            && historical["currentSourceDiscovery"]["missingExports"]
                == json!([
                    "ExecutableArtifact",
                    "ExecutableApiContract",
                    "ExecutableDependency",
                    "ExecutableTestBinding"
                ]),
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    let discovery = &current["currentGovernedDiscovery"];
    require(
        discovery["result"]["sourceHash"]
            == "3acda2cf5242b0eba88ec28416f7ff3cec0a229af900e6fd90871e1dd40f141b"
            && discovery["executionAdmission"]["callHash"]
                == "7b3d6f550283b9d39b9243d3209ec315f3ba896dc17d205250b8cdac733b2e6e"
            && discovery["executionAdmission"]["admissionHash"]
                == "139427eb3adce4063613d5275a78b2afd7709d39583787d5f6d5f7ef71b7fefc",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        discovery["result"]["observedEvidenceCount"] == 0
            && discovery["result"]["declaredOperationalCapabilityCount"] == 0
            && discovery["result"]["authority"] == "observation-only",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    let admission = &current["currentAuthenticatedAdmissionClosure"];
    require(
        admission["baseline"]["artifactHash"] == historical["historicalAuthority"]["artifactHash"]
            && admission["baseline"]["artifactHash"]
                == "c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        admission["baseline"]["baselineHash"]
            == "a6d4f7e39faeaf24a758c60d95820f3e5da963095b33978f30d5361c8e915aec"
            && admission["baseline"]["inventoryHash"]
                == "63eeddeca9da07c70ebfff934f5e3b27d6b8536ffc91fb3ee21cc81d22e3c766",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        admission["executionAdmission"]["callHash"]
            == "80577284835bf2db4cd7de0e2fbd7aae8525e7b3bf8635032f708591af945931"
            && admission["executionAdmission"]["admissionHash"]
                == "a17590fd3563cfce48672988ba65222c8673b19b4144da702143d6035bab52b7",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        admission["missingSubjectIds"]
            .as_array()
            .is_some_and(|items| items.len() == 6)
            && admission["warning"]
                == "these-six-missing-subjects-are-current-admitted-baseline-closure-results-not-the-six-historical-source-validator-issue-records",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    let source = &current["concurrentCurrentSourceSnapshot"];
    require(
        source["focusedValidation"].as_array().is_some_and(|items| {
            items.len() == 2 && items.iter().all(|item| item["valid"] == true)
        }),
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        source["gitState"]
            == json!({"project.rel":"modified","plugins/sourcery-app-foundation/plugin.rel":"modified","plugins/sourcery-app-foundation/causal-claim-runtime-plugin.rel":"untracked"})
            && source["inferenceAuthority"]
                == "local-read-only-observation-not-governed-adoption-or-authorized-applied-change",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    let repair = &current["repairPreflight"];
    require(
        repair["authenticatedNonMutatingRepairProposalAvailable"] == false
            && repair["authenticatedValidationPreviewAvailable"] == false
            && repair["prohibitedNextCallNotTaken"]["tool"] == "sourcery_adoption",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    let request = &current["requestCustody"];
    require(
        request["requestId"] == "request-7db0-a8a8-86ea"
            && request["finalStatus"] == "stopped"
            && request["finalRevision"] == 7
            && request["finalHeadHash"]
                == "50cd2a963fc7d75dc8aba5035c823db515fa9dcc0450baf2aea203ac33f7c1a5"
            && request["finalLease"].is_null()
            && request["inboxActiveLeaseCount"] == 0,
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        no_effects(&current["effects"])
            && historical["runtimeCorpusBytes"].is_null()
            && historical["unavailableOperationalEvidence"] == unavailable()
            && no_effects(&historical["effects"]),
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    require(
        current["conclusion"]
            == "historical-refusal-preserved-current-baseline-closure-authenticated-repair-route-not-authorized-and-request-stopped-with-zero-active-leases",
        "invalid-relnav-runtime-authority-boundary-v07",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-runtime-authority-assessment/v0.7","status":"refused-executed-runtime-authority-unavailable","witnessDigests":{"current":CURRENT_SHA,"historical":HISTORICAL_SHA},"witnessLanes":{"current":current["lane"],"historical":historical["lane"]},"historicalIssues":{"reportedCount":"6","named":named(),"issue6":"unavailable","exhaustive":false},"currentSource":{"governedObservedSourceHash":discovery["result"]["sourceHash"],"executionCallHash":discovery["executionAdmission"]["callHash"],"executionAdmissionHash":discovery["executionAdmission"]["admissionHash"],"localFocusedValidation":"passed","concurrency":"modified-and-untracked","repair":"not-governed-or-authorized","authority":"observation-only"},"admission":{"artifactHash":admission["baseline"]["artifactHash"],"baselineHash":admission["baseline"]["baselineHash"],"inventoryHash":admission["baseline"]["inventoryHash"],"assessmentHash":admission["assessmentHash"],"executionCallHash":admission["executionAdmission"]["callHash"],"executionAdmissionHash":admission["executionAdmission"]["admissionHash"],"authority":"verified-baseline-only-not-runtime-authority"},"request":{"requestId":request["requestId"],"finalRevision":"7","finalHeadHash":request["finalHeadHash"],"finalStatus":"stopped","finalLease":Value::Null,"activeLeaseCount":"0"},"runtime":{"records":"unavailable","headAndCompleteness":"unavailable","exportOrCaptureAuthority":"unavailable","corpusBytes":Value::Null,"writerAndCustodyAuthority":"unavailable"},"unavailableOperationalEvidence":unavailable(),"effects":{"current":current["effects"],"historical":historical["effects"]},"additionalAuthorityRequired":current["additionalAuthorityRequired"],"readyForMigration":false,"migration":"not-started","refusal":{"code":"current-source-valid-concurrent-uncommitted-and-runtime-authority-unavailable"},"policy":value["policy"]}),
    )
}
