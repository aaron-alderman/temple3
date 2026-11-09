//! Independent content-addressed selected-AF1 refusal audit v0.2.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
const CLUSTERS: [(&str, &str, &str, &str, &str); 15] = [
    (
        "formation",
        "SELECTED-FORMATION-SURFACE-CONTRACT.md",
        "selected-formation-surface-differential.mjs",
        "selected-formation-surface-v0.1.json",
        "temple3.selected-formation-surface-vectors/v0.1",
    ),
    (
        "import",
        "CLAIM-IMPORT-APPLICATION-CONTRACT.md",
        "claim-import-application-differential.mjs",
        "claim-import-application-v0.1.json",
        "temple3.claim-import-application-vectors/v0.1",
    ),
    (
        "lowering",
        "LOWERING-PRESERVATION-CONTRACT.md",
        "lowering-preservation-differential.mjs",
        "lowering-preservation-v0.1.json",
        "temple3.lowering-preservation-vectors/v0.1",
    ),
    (
        "basis",
        "BASIS-DECISION-CONTRACT.md",
        "basis-decision-differential.mjs",
        "basis-decision-v0.1.json",
        "temple3.basis-decision-vectors/v0.1",
    ),
    (
        "inventory",
        "INVENTORY-CONSERVATION-CONTRACT.md",
        "inventory-differential.mjs",
        "inventory-v0.1.json",
        "temple3.inventory-vectors/v0.1",
    ),
    (
        "testimony",
        "AUTHORIZED-TESTIMONY-CONTRACT.md",
        "authorized-testimony-differential.mjs",
        "authorized-testimony-v0.1.json",
        "temple3.authorized-testimony-vectors/v0.1",
    ),
    (
        "typed",
        "TYPED-COMPOSITION-CONTRACT.md",
        "typed-composition-differential.mjs",
        "typed-composition-v0.1.json",
        "temple3.typed-composition-vectors/v0.1",
    ),
    (
        "representative",
        "REPRESENTATIVE-COMPOSITION-CONTRACT.md",
        "representative-composition-signed-v0.2.mjs",
        "representative-composition-signed-v0.2.json",
        "temple3.representative-composition-signed-vectors/v0.2",
    ),
    (
        "request",
        "DECISION-REQUEST-CONTRACT.md",
        "decision-request-differential.mjs",
        "decision-request-v0.1.json",
        "temple3.decision-request-vectors/v0.1",
    ),
    (
        "effect",
        "DURABLE-EFFECT-CONTRACT.md",
        "durable-effect-differential.mjs",
        "durable-effect-v0.1.json",
        "temple3.durable-effect-vectors/v0.1",
    ),
    (
        "publication",
        "GENERATION-PUBLICATION-CONTRACT.md",
        "generation-publication-differential.mjs",
        "generation-publication-v0.1.json",
        "temple3.generation-publication-vectors/v0.1",
    ),
    (
        "consequence",
        "CONSEQUENCE-RELEASE-CONTRACT.md",
        "basis-consequence-differential.mjs",
        "basis-consequence-v0.1.json",
        "temple3.basis-consequence-vectors/v0.1",
    ),
    (
        "disclosure",
        "DISCLOSURE-GRANTS-CONTRACT.md",
        "disclosure-grants-differential.mjs",
        "disclosure-grants-v0.1.json",
        "temple3.disclosure-grants-vectors/v0.1",
    ),
    (
        "custody",
        "SOURCE-CAPSULE-CUSTODY-CONTRACT.md",
        "source-capsule-custody-differential.mjs",
        "source-capsule-custody-v0.1.json",
        "temple3.source-capsule-custody-vectors/v0.1",
    ),
    (
        "transport",
        "RUNTIME-EXPORT-TRANSPORT-V02-CONTRACT.md",
        "runtime-export-transport-capacity-v0.1-differential.mjs",
        "runtime-export-transport-capacity-v0.1.json",
        "temple3.runtime-export-transport-capacity-vectors/v0.1",
    ),
];
const PINNED_SHA256: [&str; 50] = [
    "ccd8f35cdc094144c836032808c317c32948f4e20a7f3c1aec7edeb715ffe01b",
    "64e8e7e2fb683a918aa1d955d94f22d0fbe0e5b76ccd19e4e8fe1dc5c2b73368",
    "42b908dd6848994d7b5505a3edb3df08db4d9e2a0c28dcc3846a25b3ba21d042",
    "f2c7c068bef26403e0133bcfdbd8c490b0453ae5170c9d5db4023f1c3022bb9e",
    "1e75513fdf9447f9daef0c1c98cabbdc38dc120ab2ff122f95ebc05a34a4de4d",
    "e0bf7f57c058a7a83a35f9cdded2d264f15c614d9d13cc1cfb4da41c1a331539",
    "df02ae35b5672c15ee445c3ded79532130a9b1fb159a70d293ceb0a620d96043",
    "65081fbaadf640d949cdf91465278c9a279234bca262dc39e893d0ddc01b3015",
    "7b6220e9c353f7870b24b7f9d72dbe336f8eaf45916d3225606c9199e98f6e3e",
    "2beb540569b2fdc04a078157a572ecd2e94f95fd69fd32399dcd7e217654ad84",
    "0408aad89e16bb81279ca1e91fd7341fa41b01ea7f34d807cec747a95198072c",
    "a173a5e76a2264682ef412c4e42e84eb66b0ca43a2df9d732339af80c4b99d3e",
    "a6d272831c8941ca0695176f642a2101c48d1ad45cf86a307afd451448310b93",
    "10c348e60ebebe2ff95f6fbe17dc0957df5ec5adf3d3062acf2dde382d62cb14",
    "14cb5870ea8b2212fa56bcd3c982874e1d9b3ced00b53c752e9ee70ff1ed9bb4",
    "275243269b2253a77ef44215da51f4b91ee1f28318c666ace45fb8d4eb87b830",
    "525e352e27635fa62b9a505d717c8f328c932f7dd9dc5f4b9aa86b5083d225ed",
    "d7f176d8c8a67b5f999e13b230a1ae287c25d92af9707823b7cddd7bd899250a",
    "a6d1a88426cb9821a2423dcf800ff4a74705a3ed5f555f3b1dca2beec7bef2c8",
    "7304f8da66122d37c5e9179c2e031f594c47638aa5a57cf20633bd139e94f216",
    "a7ce3ca4d12f1b7b28fbf4a936af5f285b070c234320e9e3134fb0a99a262497",
    "adff3f74fdb1a2b9244cb0ef728a06e46ee8990050465dc4b304cc9ca78c214a",
    "4a4d07069c1b9fed36f7f1f11bf2bd66c8889dfb2ccd0c7497f503701c17591a",
    "7fbff1f4d17913043901ab2b292f87bb2ad0b4386e88b9de7296d985e2a9d87f",
    "997c459b019e15fc01d6d982366b7403662755e79b1127bba1eddece1c016b27",
    "fcaae811d4d0023755f68642e42332bdac29beca79a2a7e52a72639f6142c553",
    "51519a314a8c421b4f8f3b9148526f53f763ab8426a92998b07a0dbfa2160286",
    "6f8d000a9f8b9e1096b4e93f04e6f8afec089595865a029b90cbbc51c19288d2",
    "91e48cad4b9f09ad76ff391c9d4926651361a9d87342cc4cc85ec856689f39a4",
    "121c1045a8cd0165648560044ab86479651d8764f3d988599821cfcfcb46d5bb",
    "076e25839f5a0b91eab20832c296fc08e47c3f335a123da04f3609a2c70aae81",
    "223bdc48df83421f24ec55dd37fa8f61b756fdc1b2a1a4e3875f4b277e857f33",
    "eac02497e33bf37b803d69721e644c96ed2639704472749294fc1bb73d22ec77",
    "35018cd31b51bc25af68bc8d5f6db5c7dc43bf1f5664af973b5f50d8191b0b6a",
    "09c605221891e212dc4623954579361200e137204b566d966e4aaadb08703bef",
    "445fb1b5c1532ead8ca51b3a9f8bc5ef11dc3fcdddb14452cf54fbb688749ff0",
    "8c32f3baeb6df8722f212e976ac5fa7e98e3c4ec0174fec994dbc3a6c7952ed4",
    "ebeb85ee16d0e3f5fd26790e66e5f8e550b7f9acf59b6f78bc81030c402b6dcc",
    "20e073e1049a2b2a31bfdbba05fae5f2d78a9ca4765e6c87842c58d493446844",
    "4a291c8557026ce30d1b24f600d63deb606145addc818ec86fb5f6092358403b",
    "20ddb1675f21eeb2c0a3470f9930b5e6940f2b51373392d4832df89823deea24",
    "7082a9cc172cab6a24657ef5dbd459ac3f0cf5012a1bca4e8e719cfd971934c9",
    "559f3c76ea3f740daf5c2fd96686bed6714baaedd56f63f5cbb61dd9bc747075",
    "53a3e7030948aff94585b80068db9e559afb3d894f9de72407be9b0a6dd042b7",
    "c7de285e64ed90b671327e517c20863bd0fb179c6c0d3bbb27b59e2c69aa0b04",
    "18d5548055309338e34a57dfaf8a8bd59167c122113347a992dcf6ba5d72f83e",
    "b9651bc0c78c39819dc6021c3396b566276a62d9d32f0f3d64b1989daed5bec5",
    "3844e3406e6158b1b20444bc2d88d79e8085bcdb06fee474be090b2abc86480f",
    "21c0f0e68a37ea06714a996cdd0814f203585a84e82393db92bf377e7ce313d7",
    "e8dd4c0266f239957df6c0d41f7df68ccc8b708ee27801de70bf63d291a95f62",
];
const GAPS: [&str; 14] = [
    "FQ03:authentic-source-version-substitution-journey",
    "FQ04:authentic-source-lowering-mapping",
    "FQ09:authentic-source-policy-selection",
    "FQ10:authentic-source-declared-cross-domain-combination",
    "FQ11:authentic-source-declared-cross-domain-combination",
    "FQ12:production-key-and-checkpoint-custody",
    "FQ13:production-custody-and-client-behavior",
    "FQ17:production-stores-backups-and-other-profiles-audit",
    "FQ20:authentic-production-anchor-custody",
    "FQ21:operational-transport-and-client-recovery",
    "FQ23:operational-remote-head-propagation",
    "FQ23:representative-source-integration",
    "FQ24:authentic-inventory-mapping",
    "FQ24:production-custody",
];
const AREA_PROFILES: [&str; 8] = [
    "temple3.application-contract-closure/v0.1",
    "temple3.representative-composition-formation/v0.1",
    "temple3.transition-lowering-obligations/v0.1",
    "temple3.basis-decision-formation/v0.1",
    "temple3.testimony-formation/v0.1",
    "temple3.basis-consequence-formation/v0.1",
    "temple3.durable-effect-formation/v0.1",
    "temple3.disclosure-grant-formation/v0.1",
];
const AREA_CLUSTERS: [&str; 8] = [
    "formation",
    "representative",
    "lowering",
    "basis",
    "testimony",
    "consequence",
    "effect",
    "disclosure",
];
const AREA_GAPS: [&[&str]; 8] = [
    &["FQ03"],
    &["FQ10", "FQ11"],
    &["FQ04"],
    &["FQ12"],
    &["FQ09"],
    &["FQ17"],
    &["FQ13", "FQ20", "FQ21"],
    &["FQ23", "FQ24"],
];
const CASE_CLUSTERS: [&str; 24] = [
    "formation",
    "formation",
    "import",
    "lowering",
    "basis",
    "basis",
    "basis",
    "inventory",
    "testimony",
    "typed",
    "representative",
    "request",
    "effect",
    "publication",
    "consequence",
    "disclosure",
    "custody",
    "consequence",
    "consequence",
    "transport",
    "transport",
    "typed",
    "disclosure",
    "inventory",
];
fn expected_manifest() -> Vec<(String, String, String, String, String)> {
    let mut v = Vec::new();
    for (i, s, e, w, _k) in CLUSTERS {
        let digest = PINNED_SHA256[v.len()];
        v.push((
            format!("{i}-contract"),
            format!("specification/{s}"),
            "versioned-contract".into(),
            format!("content-sha256:{digest}"),
            digest.into(),
        ));
        let digest = PINNED_SHA256[v.len()];
        v.push((
            format!("{i}-verifier"),
            format!("conformance/{e}"),
            "executable-verifier".into(),
            format!("content-sha256:{digest}"),
            digest.into(),
        ));
        let digest = PINNED_SHA256[v.len()];
        v.push((
            format!("{i}-vector"),
            format!("conformance/vectors/{w}"),
            "canonical-vector".into(),
            format!("content-sha256:{digest}"),
            digest.into(),
        ));
    }
    for (id, path, role) in [
        (
            "closure-model",
            "model/application-contract-closure.mjs",
            "executable-prerequisite",
        ),
        (
            "ledger-v02-model",
            "model/foundation-obligation-ledger-v02.mjs",
            "executable-prerequisite",
        ),
        (
            "af4-recovery-evidence",
            "pilot/evidence/runtime-export-transport-capacity-recovery-v0.2.json",
            "target-laboratory-evidence",
        ),
        (
            "af5-authority-vector",
            "conformance/vectors/representative-authority-closure-v0.5.json",
            "operative-authority-refusal-vector",
        ),
        (
            "historical-run-132",
            "qualification/evidence/run-132-2026-09-09.json",
            "historical-integration-evidence",
        ),
    ] {
        let digest = PINNED_SHA256[v.len()];
        v.push((
            id.into(),
            path.into(),
            role.into(),
            format!("content-sha256:{digest}"),
            digest.into(),
        ));
    }
    v
}
fn bad() -> String {
    "invalid-application-contract-completion-audit-v02".into()
}
fn refs_ok(v: &Value, cluster: &str) -> bool {
    v.as_array()
        .map(|a| {
            a.len() == 3
                && a[0] == format!("{cluster}-contract")
                && a[1] == format!("{cluster}-verifier")
                && a[2] == format!("{cluster}-vector")
        })
        .unwrap_or(false)
}
fn strings_ok(v: &Value, want: &[&str]) -> bool {
    v.as_array()
        .map(|a| a.len() == want.len() && a.iter().zip(want).all(|(x, y)| x == *y))
        .unwrap_or(false)
}
pub fn audit(x: &Value) -> std::result::Result<Value, String> {
    if x.as_object().map(|o| o.len()) != Some(6)
        || ![
            "kind",
            "prerequisites",
            "evidenceManifest",
            "areas",
            "cases",
            "policy",
        ]
        .iter()
        .all(|k| x.get(k).is_some())
        || x["kind"] != "temple3.application-contract-completion-audit/v0.2"
    {
        return Err(bad());
    }
    let manifest = x["evidenceManifest"].as_array().ok_or_else(bad)?;
    let expected = expected_manifest();
    if manifest.len() != expected.len() {
        return Err(bad());
    }
    for (m, (id, path, role, identity, expected_digest)) in manifest.iter().zip(expected.iter()) {
        if m.as_object().map(|o| o.len()) != Some(5)
            || m["id"] != *id
            || m["path"] != *path
            || m["role"] != *role
            || m["declaredIdentity"] != *identity
        {
            return Err(bad());
        }
        let bytes = std::fs::read(path).map_err(|_| bad())?;
        let actual = hex::encode(Sha256::digest(&bytes));
        if actual != *expected_digest || m["sha256"] != actual {
            return Err(bad());
        }
        // The exact expected vector kind is independently bound above. Some retained
        // adversarial JSON contains lone surrogate escapes that serde_json correctly
        // refuses to materialize; byte identity, not lossy parsing, is authoritative here.
    }
    let prereq = x["prerequisites"].as_array().ok_or_else(bad)?;
    let expected_prereq = [
        (
            "application-contract-closure/v0.1",
            "application-contract-closure:sha256:330ccca4874e582d6ecd1a240c2532b8d73c6f5e66fcfacd2e4dc84b484f4a17",
            "closed-local-metadata-obligations",
            "bounded-metadata-prerequisite-not-af1-completion",
        ),
        (
            "selected-formation-surface/v0.1",
            "selected-formation-surface-v0.1:sha256:af2f0ebc9412cba47366d1f700ab9f53e380f088c0636563d19915dde5599129",
            "selected-formation-boundaries-complete",
            "bounded-formation-prerequisite",
        ),
        (
            "foundation-obligation-ledger/v0.2",
            "foundation-obligation-ledger-v0.2:sha256:dee24a1ef27f5a11160605e4a8c7c4dd34f9cfa49655519b18debeaef50a7b80",
            "refused-incomplete-foundation",
            "operative-predecessor-refusal",
        ),
        (
            "representative-composition-signed/v0.2",
            "temple3.representative-composition-signed-vectors/v0.2",
            "passed-bounded-synthetic-profile",
            "bounded-af1-evidence-not-authentic-source",
        ),
        (
            "runtime-export-transport-capacity-recovery/v0.2",
            "temple3.runtime-export-transport-capacity-recovery-evidence/v0.2",
            "passed-target-laboratory",
            "af4-evidence-not-af1-completion",
        ),
        (
            "representative-authority-closure/v0.5",
            "54d932e1d14f2693b762ae997020f2db6be6be73a6ab7ad0a447e3c91bd016ec",
            "refused-operative-authority-and-capacity-incomplete",
            "af5-refusal-not-af1-completion",
        ),
        (
            "stable-source-integration/132",
            "b1490ae30f78dbcfe5e41509c13916cdda954cd2264c07524bac378ef4703809",
            "passed-historical-snapshot-stale-for-v02",
            "historical-evidence-never-current-authority",
        ),
    ];
    if prereq.len() != expected_prereq.len() {
        return Err(bad());
    }
    for (p, (id, identity, status, role)) in prereq.iter().zip(expected_prereq) {
        if p.as_object().map(|o| o.len()) != Some(4)
            || !["id", "identity", "status", "role"]
                .iter()
                .all(|k| p.get(k).is_some())
            || p["id"] != id
            || p["identity"] != identity
            || p["status"] != status
            || p["role"] != role
        {
            return Err(bad());
        }
    }
    let areas = x["areas"].as_array().ok_or_else(bad)?;
    let cases = x["cases"].as_array().ok_or_else(bad)?;
    if areas.len() != 8 || cases.len() != 24 {
        return Err(bad());
    }
    for (i, a) in areas.iter().enumerate() {
        if a.as_object().map(|o| o.len()) != Some(5)
            || a["id"] != format!("F{}", i + 1)
            || a["af1ContractStatus"] != "in-progress"
            || a["profileIdentity"] != AREA_PROFILES[i]
            || !refs_ok(&a["evidenceRefs"], AREA_CLUSTERS[i])
            || !strings_ok(&a["contractGaps"], AREA_GAPS[i])
        {
            return Err(bad());
        }
    }
    let complete = [1, 2, 5, 6, 7, 8, 14, 15, 16, 18, 19, 22];
    for (i, c) in cases.iter().enumerate() {
        let n = i + 1;
        let status = if complete.contains(&n) {
            "selected-complete"
        } else {
            "in-progress"
        };
        let vector_index = CLUSTERS
            .iter()
            .position(|x| x.0 == CASE_CLUSTERS[i])
            .ok_or_else(bad)?;
        let vector_kind = format!("content-sha256:{}", PINNED_SHA256[vector_index * 3 + 2]);
        if c.as_object().map(|o| o.len()) != Some(5)
            || c["id"] != format!("FQ{n:02}")
            || c["af1ContractStatus"] != status
            || c["profileIdentity"] != vector_kind
            || !refs_ok(&c["evidenceRefs"], CASE_CLUSTERS[i])
        {
            return Err(bad());
        }
    }
    let mut got_gaps: Vec<String> = Vec::new();
    for c in cases {
        let row_gaps = c["contractGaps"].as_array().ok_or_else(bad)?;
        for g in row_gaps {
            got_gaps.push(format!(
                "{}:{}",
                c["id"].as_str().ok_or_else(bad)?,
                g.as_str().ok_or_else(bad)?
            ));
        }
    }
    if got_gaps != GAPS {
        return Err(bad());
    }
    let p = &x["policy"];
    if p.as_object().map(|o| o.len()) != Some(15)
        || p["missingUnreadableOrChangedEvidence"] != "refuse"
        || p["missingOrReorderedObligation"] != "refuse"
        || p["substitutedIdentityOrStatus"] != "refuse"
        || p["hiddenDefaults"] != "forbidden"
        || p["forgiveness"]["scope"] != "selected-af1-contract-completion-audit-v0.2"
        || p["af1"] != "in-progress"
        || p["af4"] != "in-progress"
        || p["af5"] != "in-progress"
        || p["af6"] != "not-started"
        || p["historicalVsOperative"] != "post-132-source-is-not-qualified-by-the-132-manifest"
        || p["syntheticEvidence"] != "never-authentic-source-or-operational-authority"
        || p["forgiveness"]["disposition"] != "refuse"
        || p["forgiveness"].as_object().map(|o| o.len()) != Some(2)
        || p["migration"] != "not-started"
        || p["productionAuthority"] != "not-claimed"
        || p["cutover"] != "not-authorized"
        || p["arkSealing"] != "not-authorized"
    {
        return Err(bad());
    }
    let aid = crate::canonical::identify("application-contract-completion-audit-v0.2", x)
        .map_err(|_| bad())?;
    let gaps: Vec<Value> = GAPS.iter().map(|s| json!(s)).collect();
    Ok(
        json!({"kind":"temple3.application-contract-completion-assessment/v0.2","status":"refused-incomplete-selected-af1-contract","auditId":aid,"prerequisites":prereq,"evidenceManifest":manifest,"areas":areas,"cases":cases,"counts":{"areas":"8","cases":"24","af1ContractGaps":GAPS.len().to_string()},"af1ContractGaps":gaps,"readyForSelectedAF1Contract":false,"readyForAF1":false,"readyForAF4":false,"readyForAF5":false,"readyForAF6":false,"readyForAuthenticMigration":false,"refusal":{"code":"selected-af1-contract-required-evidence-incomplete","missing":gaps},"policy":p}),
    )
}
