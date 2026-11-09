//! Independent representative runtime-evidence admission evaluator v0.1.
use crate::{Result, require, shape};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
const APPS: [(&str, &str, &str, &str); 4] = [
    (
        "app.inventoryConservationLedgerReferenceV3",
        "8234a80e4c1a5744a25cf61255cc66107a7ffe878be89806d5248570f8844731",
        "inventory-formation:sha256:7f8188b633c39676e04b63aa34744df45e32c725861fc971339862ce5a3bab93",
        "inventory-runtime-record-v1",
    ),
    (
        "app.claimApprovalReference20260831V2",
        "f15382f7de81b4c4fb8f25fc65c366516b4d62907d926c05f668802b262db2a5",
        "claim-readiness-formation:sha256:f43a7ae7a11531f2f2c55c55dd23e065a1076c9a4f070213411bb8a4b08685b3",
        "claim-approval-runtime-record-v1",
    ),
    (
        "app.actorScopedCaseAuthorityReference",
        "ac0984ec7c5c9b22ab86d8fc2f24dfd9307f9b8122098e87a6ddb87c75a9ab7d",
        "claim-lifecycle-formation:sha256:16f4aecb25c0acc4305258b34c2a9bc4e76e154e64c71870e494893fff2a1e9d",
        "actor-scoped-runtime-record-v1",
    ),
    (
        "app.relnav",
        "c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7",
        "voluntary-expression-formation:sha256:22d8e616e913cd0b5de4666afcfed27e4c1b5399e21a45601b11e037f89d072d",
        "relnav-polarity-multiplicity-runtime-record-v1",
    ),
];
fn n(v: &Value) -> Result<u64> {
    let s = v.as_str().unwrap_or("");
    require(
        s == "0" || (!s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit())),
        "invalid-number",
    )?;
    s.parse().map_err(|_| temple3::Error {
        code: "invalid-number".into(),
    })
}
fn sha(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    })
}
fn hash(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
fn bounded(v: &Value) -> bool {
    v.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 128)
}
fn adapter_payload(adapter: &str, b: &[u8]) -> Result<()> {
    let v: Value = serde_json::from_slice(b).map_err(|_| temple3::Error {
        code: "adapter-refused".into(),
    })?;
    require(
        crate::canonical::canonical_json(&v)?.as_bytes() == b,
        "adapter-refused",
    )?;
    match adapter {
        "inventory-runtime-record-v1" => {
            shape(&v, "kind recordId unit quantity status actor")?;
            require(
                v["kind"] == "inventory-runtime-record"
                    && bounded(&v["recordId"])
                    && v["unit"] == "ITEM"
                    && v["quantity"].as_str().is_some_and(|x| {
                        !x.is_empty()
                            && !x.starts_with('0')
                            && x.len() <= 18
                            && x.bytes().all(|c| c.is_ascii_digit())
                    })
                    && ["draft", "posted", "corrected"]
                        .contains(&v["status"].as_str().unwrap_or(""))
                    && bounded(&v["actor"]),
                "adapter-refused",
            )?
        }
        "claim-approval-runtime-record-v1" => {
            shape(&v, "kind claimId currency minor status approval actor")?;
            require(
                v["kind"] == "claim-approval-runtime-record"
                    && bounded(&v["claimId"])
                    && v["currency"] == "AUD"
                    && v["minor"].as_str().is_some_and(|x| {
                        x == "0"
                            || (!x.is_empty()
                                && !x.starts_with('0')
                                && x.len() <= 18
                                && x.bytes().all(|c| c.is_ascii_digit()))
                    })
                    && ["draft", "approved", "ready"].contains(&v["status"].as_str().unwrap_or(""))
                    && ["none", "historical-source-only"]
                        .contains(&v["approval"].as_str().unwrap_or(""))
                    && bounded(&v["actor"]),
                "adapter-refused",
            )?
        }
        "actor-scoped-runtime-record-v1" => {
            shape(&v, "kind caseId owner commandActor role disclosure")?;
            require(
                v["kind"] == "actor-scoped-runtime-record"
                    && bounded(&v["caseId"])
                    && bounded(&v["owner"])
                    && bounded(&v["commandActor"])
                    && ["author", "reviewer"].contains(&v["role"].as_str().unwrap_or(""))
                    && v["disclosure"] == "explicit-participants",
                "adapter-refused",
            )?
        }
        "relnav-polarity-multiplicity-runtime-record-v1" => {
            shape(
                &v,
                "kind subject relation object polarity ordinal contradiction contextOwner",
            )?;
            require(
                v["kind"] == "relnav-runtime-record"
                    && bounded(&v["subject"])
                    && bounded(&v["relation"])
                    && bounded(&v["object"])
                    && ["positive", "negative"].contains(&v["polarity"].as_str().unwrap_or(""))
                    && v["ordinal"].as_str().is_some_and(|x| {
                        x == "0"
                            || (!x.is_empty()
                                && !x.starts_with('0')
                                && x.len() <= 18
                                && x.bytes().all(|c| c.is_ascii_digit()))
                    })
                    && v["contradiction"].is_boolean()
                    && bounded(&v["contextOwner"]),
                "adapter-refused",
            )?
        }
        _ => require(false, "adapter-refused")?,
    }
    Ok(())
}
pub fn assess(request: &Value) -> Result<Value> {
    crate::canonical::canonical_json(request)?;
    shape(request, "expectedProfileId input")?;
    let expected = &request["expectedProfileId"];
    require(sha(expected), "expected-profile-required")?;
    let x = &request["input"];
    shape(x, "kind profile rows current")?;
    require(
        x["kind"] == "temple3.representative-runtime-evidence-admission/v0.1",
        "invalid-admission",
    )?;
    let p = &x["profile"];
    shape(
        p,
        "kind applications trustRoots maximumRecordsPerApplication maximumBytesPerApplication maximumAggregateBytes maximumRecoveryOverheadBytes retention recovery backout forgiveness",
    )?;
    require(
        p["kind"] == "temple3.representative-runtime-evidence-admission-profile/v0.1"
            && p["applications"] == json!(APPS.map(|a| a.0))
            && p["trustRoots"].as_array().is_some_and(|a| a.len() == 4)
            && p["retention"] == "retain-exact-corpus-and-all-reconciliation-rows-no-pruning"
            && p["recovery"]
                == "exact-attempt-corpus-head-target-head-ledger-checkpoint-no-rescan-or-tail-adoption"
            && p["backout"]
                == "preactivation-isolated-target-staging-only-retain-evidence-postactivation-compensation-required"
            && p["forgiveness"]
                == json!({"disposition":"refuse","scope":"representative-runtime-evidence-admission-v0.1"}),
        "invalid-profile",
    )?;
    require(
        crate::canonical::identify("representative-runtime-evidence-admission-profile-v0.1", p)?
            .rsplit(':')
            .next()
            .unwrap()
            == expected.as_str().unwrap(),
        "profile-substitution-refused",
    )?;
    let input_rows = x["rows"].as_array().ok_or_else(|| temple3::Error {
        code: "invalid-rows".into(),
    })?;
    require(input_rows.len() == 4, "invalid-rows")?;
    let mut rows = Vec::new();
    let mut total = 0u64;
    let mut totals = [0usize; 5];
    for (i, row) in input_rows.iter().enumerate() {
        let (app, artifact, target, adapter) = APPS[i];
        shape(
            row,
            "application artifactHash targetFormationId adapter targetHead corpus authority ledger recovery backout",
        )?;
        require(
            row["application"] == app
                && row["artifactHash"] == artifact
                && row["targetFormationId"] == target
                && row["adapter"] == adapter
                && sha(&row["targetHead"]),
            "wrong-row",
        )?;
        let c = &row["corpus"];
        shape(
            c,
            "encoding records recordCount rawBytes corpusSha256 runtimeHead completeness retainedBytes range",
        )?;
        shape(&c["completeness"], "scope excluded complete")?;
        shape(&c["range"], "firstRecordId lastRecordId recordCount")?;
        require(
            c["encoding"] == "lowercase-hex"
                && c["records"]
                    .as_array()
                    .is_some_and(|records| !records.is_empty())
                && n(&c["recordCount"])? == c["records"].as_array().unwrap().len() as u64
                && sha(&c["runtimeHead"])
                && c["retainedBytes"] == true
                && c["completeness"]
                    == json!({"scope":"exact-selected-application-runtime-head","excluded":[],"complete":true}),
            "wrong-corpus",
        )?;
        let mut ids = Vec::new();
        let mut bytes = 0u64;
        for r in c["records"].as_array().unwrap() {
            shape(r, "id payloadHex payloadSha256")?;
            require(
                r["payloadHex"].as_str().is_some_and(|s| {
                    s.len() % 2 == 0
                        && s.bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                }),
                "bad-hex",
            )?;
            let b = hex::decode(r["payloadHex"].as_str().unwrap_or("")).map_err(|_| {
                temple3::Error {
                    code: "bad-hex".into(),
                }
            })?;
            let payload_hash = hash(&b);
            require(
                r["id"] == payload_hash && r["payloadSha256"] == payload_hash,
                "wrong-record",
            )?;
            adapter_payload(adapter, &b)?;
            bytes += b.len() as u64;
            ids.push(r["id"].as_str().unwrap().to_string())
        }
        let corpus_sha = hash(crate::canonical::canonical_json(&c["records"])?.as_bytes());
        let range = json!({"firstRecordId":ids.first().unwrap(),"lastRecordId":ids.last().unwrap(),"recordCount":ids.len().to_string()});
        let runtime_head=hash(crate::canonical::canonical_json(&json!({"kind":"temple3.representative-runtime-evidence-head-material/v0.1","orderedRecordIds":ids,"corpusSha256":corpus_sha,"range":range,"completeness":c["completeness"]}))?.as_bytes());
        require(
            ids.iter().collect::<BTreeSet<_>>().len() == ids.len()
                && bytes == n(&c["rawBytes"])?
                && c["corpusSha256"] == corpus_sha
                && c["range"] == range
                && c["runtimeHead"] == runtime_head
                && ids.len() as u64 <= n(&p["maximumRecordsPerApplication"])?
                && bytes <= n(&p["maximumBytesPerApplication"])?,
            "corpus-refused",
        )?;
        let a = &row["authority"];
        shape(
            a,
            "governorKey exporterKey captureKey custodyKey transportKey grant export capture custody transport",
        )?;
        let root = &p["trustRoots"][i];
        shape(
            root,
            "application governorKey exporterKey captureKey custodyKey transportKey",
        )?;
        for k in [
            "governorKey",
            "exporterKey",
            "captureKey",
            "custodyKey",
            "transportKey",
        ] {
            require(a[k] == root[k], "trust-root-refused")?
        }
        require(
            root["application"] == app
                && crate::identity::verify_envelope(&a["grant"], "attestation", &a["governorKey"]),
            "grant-refused",
        )?;
        let g = &a["grant"]["payload"];
        shape(
            g,
            "kind application artifactHash runtimeHead corpusSha256 recordCount rawBytes completeness orderedRecordIds range scope exporterKey captureKey custodyKey transportKey",
        )?;
        shape(&g["completeness"], "scope excluded complete")?;
        shape(&g["range"], "firstRecordId lastRecordId recordCount")?;
        require(
            g["kind"] == "temple3.representative-runtime-evidence-grant/v0.1"
                && g["application"] == app
                && g["artifactHash"] == artifact
                && g["runtimeHead"] == c["runtimeHead"]
                && g["corpusSha256"] == c["corpusSha256"]
                && g["recordCount"] == c["recordCount"]
                && g["rawBytes"] == c["rawBytes"]
                && g["completeness"] == c["completeness"]
                && g["orderedRecordIds"] == json!(ids)
                && g["range"] == c["range"]
                && g["scope"] == "exact-runtime-export-capture-custody-transport"
                && ["exporterKey", "captureKey", "custodyKey", "transportKey"]
                    .iter()
                    .all(|k| g[*k] == a[*k]),
            "grant-refused",
        )?;
        let gid = crate::canonical::identify("signed", &a["grant"])?;
        for (name, key) in [
            ("export", "exporterKey"),
            ("capture", "captureKey"),
            ("custody", "custodyKey"),
            ("transport", "transportKey"),
        ] {
            require(
                crate::identity::verify_envelope(&a[name], "attestation", &a[key]),
                "receipt-refused",
            )?;
            let q = &a[name]["payload"];
            shape(
                q,
                "kind application grantId runtimeHead corpusSha256 recordCount rawBytes completeness orderedRecordIds range disposition",
            )?;
            shape(&q["completeness"], "scope excluded complete")?;
            shape(&q["range"], "firstRecordId lastRecordId recordCount")?;
            require(
                q == &json!({"kind":format!("temple3.representative-runtime-evidence-{name}-receipt/v0.1"),"application":app,"grantId":gid,"runtimeHead":c["runtimeHead"],"corpusSha256":c["corpusSha256"],"recordCount":c["recordCount"],"rawBytes":c["rawBytes"],"completeness":c["completeness"],"orderedRecordIds":ids,"range":c["range"],"disposition":format!("{name}-verified-fixture-mechanics-only")}),
                "receipt-refused",
            )?
        }
        let l = &row["ledger"];
        shape(l, "preserved changed removed unsupported added")?;
        let mut seen = Vec::new();
        for (kind, data, meaning) in [
            ("preserved", "copy", "equivalent"),
            ("changed", "transform", "redesigned"),
            ("removed", "no-target", "removed-with-explicit-reason"),
            ("unsupported", "quarantine", "unresolved"),
        ] {
            for z in l[kind].as_array().ok_or_else(|| temple3::Error {
                code: "ledger".into(),
            })? {
                shape(z, "sourceUnitId evidenceRecordId data meaning authority")?;
                shape(&z["data"], "disposition targetRepresentation")?;
                shape(&z["meaning"], "disposition rule reason")?;
                shape(&z["authority"], "source target transfer")?;
                require(
                    sha(&z["sourceUnitId"])
                        && z["sourceUnitId"] == z["evidenceRecordId"]
                        && ids.contains(&z["sourceUnitId"].as_str().unwrap_or("").to_string())
                        && z["data"]["disposition"] == data
                        && z["meaning"]["disposition"] == meaning
                        && bounded(&z["data"]["targetRepresentation"])
                        && bounded(&z["meaning"]["rule"])
                        && bounded(&z["meaning"]["reason"])
                        && bounded(&z["authority"]["source"])
                        && bounded(&z["authority"]["target"])
                        && bounded(&z["authority"]["transfer"]),
                    "ledger",
                )?;
                if kind == "removed" {
                    require(
                        z["authority"]["transfer"] == "explicit-removal-decision",
                        "ledger",
                    )?
                }
                seen.push(z["sourceUnitId"].as_str().unwrap().to_string())
            }
        }
        seen.sort();
        let mut ids2 = ids.clone();
        ids2.sort();
        require(seen == ids2, "coverage")?;
        let added = l["added"].as_array().ok_or_else(|| temple3::Error {
            code: "ledger".into(),
        })?;
        let mut added_ids = Vec::new();
        for z in added {
            shape(z, "targetUnitId data meaning authority")?;
            shape(&z["data"], "disposition targetRepresentation")?;
            shape(&z["meaning"], "disposition rule reason")?;
            shape(&z["authority"], "source target transfer")?;
            require(
                sha(&z["targetUnitId"])
                    && z["data"]["disposition"] == "target-only"
                    && bounded(&z["data"]["targetRepresentation"])
                    && z["meaning"]["disposition"] == "added-no-source-equivalence"
                    && bounded(&z["meaning"]["rule"])
                    && bounded(&z["meaning"]["reason"])
                    && z["authority"]["source"] == "none"
                    && bounded(&z["authority"]["target"])
                    && bounded(&z["authority"]["transfer"]),
                "ledger",
            )?;
            added_ids.push(z["targetUnitId"].as_str().unwrap().to_string())
        }
        require(
            added_ids.iter().collect::<BTreeSet<_>>().len() == added_ids.len(),
            "added-identity",
        )?;
        let counts = json!({"preserved":l["preserved"].as_array().unwrap().len().to_string(),"changed":l["changed"].as_array().unwrap().len().to_string(),"removed":l["removed"].as_array().unwrap().len().to_string(),"unsupported":l["unsupported"].as_array().unwrap().len().to_string(),"added":l["added"].as_array().unwrap().len().to_string()});
        for (j, k) in ["preserved", "changed", "removed", "unsupported", "added"]
            .iter()
            .enumerate()
        {
            totals[j] += l[*k].as_array().unwrap().len()
        }
        let r = &row["recovery"];
        shape(
            r,
            "attemptId checkpointId sourceHead targetHead ledgerId reservedOverheadBytes retry ambiguousPublication sourceDrift",
        )?;
        require(
            sha(&r["attemptId"])
                && sha(&r["checkpointId"])
                && r["sourceHead"] == c["runtimeHead"]
                && r["targetHead"] == row["targetHead"]
                && r["ledgerId"] == hash(crate::canonical::canonical_json(l)?.as_bytes())
                && n(&r["reservedOverheadBytes"])? <= n(&p["maximumRecoveryOverheadBytes"])?
                && r["retry"] == "exact-identities-only"
                && r["ambiguousPublication"] == "discover-read-only-select-exact-or-refuse"
                && r["sourceDrift"] == "new-attempt-required",
            "recovery",
        )?;
        require(
            row["backout"]
                == json!({"source":"never-mutated","target":"discard-isolated-unactivated-staging-only","evidence":"retain","postActivation":"unsupported-without-qualified-compensation","externalEffects":"unsupported","authority":"explicit-separate-backout-authority"}),
            "backout",
        )?;
        rows.push(json!({"application":app,"artifactHash":artifact,"targetFormationId":target,"adapter":adapter,"runtimeHead":c["runtimeHead"],"recordCount":c["recordCount"],"rawBytes":c["rawBytes"],"counts":counts,"ledgerId":r["ledgerId"],"fixtureAuthorityVerified":true,"operativeSourceAuthority":false}));
        total += bytes
    }
    require(
        total <= n(&p["maximumAggregateBytes"])?
            && x["current"]
                == json!({"authenticInputs":"unavailable","relnavBoundary":"unselected","migration":"not-started","activation":"unavailable","production":"not-authorized"}),
        "current-refused",
    )?;
    Ok(
        json!({"kind":"temple3.representative-runtime-evidence-admission-assessment/v0.1","status":"fixture-verification-mechanics-qualified-current-authentic-admission-refused","assessmentId":crate::canonical::identify("representative-runtime-evidence-admission-v0.1",x)?,"expectedProfileId":expected,"rows":rows,"totalRawBytes":total.to_string(),"classificationTotals":{"preserved":totals[0].to_string(),"changed":totals[1].to_string(),"removed":totals[2].to_string(),"unsupported":totals[3].to_string(),"added":totals[4].to_string()},"fixtureMechanicsAccepted":true,"authenticAdmission":false,"eligibleForGovernedMigrationPreflight":false,"readyForAF5":false,"readyForAF6":false,"readyForMigration":false,"productionAuthorized":false,"refusal":{"code":"representative-authentic-runtime-evidence-and-relnav-boundary-unavailable","missing":["four-authentic-runtime-corpora-and-operative-source-authority","selected-relnav-semantic-boundary","production-custody-and-freshness","executed-migration-recovery-and-backout"]},"effects":{"sourceReads":"0","targetWrites":"0","activations":"0","backouts":"0"},"policy":p}),
    )
}
