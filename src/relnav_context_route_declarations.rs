//! Authentic Relationship Navigator admitted context/route declaration boundary.
use crate::{Result, require, shape};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
fn dg(v: &Value) -> Result<String> {
    Ok(hex::encode(Sha256::digest(
        crate::canonical::canonical_json(v)?.as_bytes(),
    )))
}
fn policy() -> Value {
    json!({"kind":"temple3.relnav-context-route-declarations-policy/v0.1","declarations":"authentic-admitted-baseline-structure","writer":"declared-single-authoritative-process-not-executed-custody","recovery":"declared-fail-closed-not-executed-recovery","routes":"declared-authenticated-not-executed-enforcement","retention":"declared-content-addressed-not-runtime-record-export","publication":"declared-bundle-before-receipt-not-observed-finality","runtimeRecords":"not-established","sourceCompleteness":"not-established","exportOrCaptureAuthority":"not-established","semanticMapping":"undetermined","migration":"not-started"})
}
fn discovery() -> Value {
    json!({"catalogHash":"d1e3d40384690bffc320d41ce827a5e824117e4267aeda724178edeedd1ccfc6","intentHash":"5393b0d98ede0caa9edf30a3cf8ec70b05a8595197c9a4cca18fd430ecddaefd","routeStatus":"clarify","observedEvidenceCount":"0","observedEvidenceHash":"37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570","declaredOperationalCapabilities":"0","capabilityHorizonHash":"37b109fc1e9a10d9fcdab68fa9cab60bb9e803809a515fcf4a493b125f1e41bd"})
}
fn authority() -> Value {
    json!({"observation":"verified-baseline-only","semanticMapping":"undetermined","determination":"none","productMutation":"none","runtimeState":"excluded","implicitProductAuthority":"excluded"})
}
fn request() -> Value {
    json!({"requestId":"request-1423-b233-fa16","stoppedRevision":"4","stoppedHash":"ff8448b15a1207e0b4d842bca9ec53d0c72cc786303dbb349e3e54a533de3c8f","activeLeaseAfterStop":false})
}
pub fn audit(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(
        x,
        "kind application artifactHash baselineHash closure selectedDeclarations discovery authority request policy",
    )?;
    require(
        x["kind"] == "temple3.relnav-context-route-declarations/v0.1"
            && x["application"] == "app.relnav"
            && x["artifactHash"]
                == "c77577f4a6eceeeb2a1f73e270aaa19ea1bf2c98355aa359679b39ac36db8de7"
            && x["baselineHash"]
                == "a6d4f7e39faeaf24a758c60d95820f3e5da963095b33978f30d5361c8e915aec"
            && x["policy"] == policy(),
        "invalid-relnav-context-route-declarations",
    )?;
    let c = &x["closure"];
    shape(
        c,
        "protocol subjectIds presentSubjectIds missingSubjectIds factCount factCorpusHash canonicalPresentedFactsHash factHashes assessmentHash contradictions",
    )?;
    require(
        c["protocol"] == "sourcery-imported-admission-subject-closure-assessment/1"
            && c["factCount"] == "54"
            && c["factCorpusHash"]
                == "ee0c4917732e7265344e6baf34dfd3fb7bed3d5fc9d95199b9432dbaf02f6d05"
            && c["canonicalPresentedFactsHash"]
                == "c6edf539144061a3dd8201e8501f82f47dff361a6759e995228306fe861c9887"
            && c["assessmentHash"]
                == "8c7b4ead3b0f5b14d54a4ff231bf6baf0a550bd81a11cd52f81efd098bef7b3b"
            && c["contradictions"] == "20",
        "invalid-relnav-context-route-declarations",
    )?;
    let ss = c["subjectIds"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-relnav-context-route-declarations"))?;
    let hs = c["factHashes"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-relnav-context-route-declarations"))?;
    let ds = x["selectedDeclarations"]
        .as_array()
        .ok_or_else(|| crate::err("invalid-relnav-context-route-declarations"))?;
    require(
        ss.len() == 11
            && c["subjectIds"] == c["presentSubjectIds"]
            && c["missingSubjectIds"] == json!([])
            && dg(&c["subjectIds"])?
                == "f5350abbe3b6f938cd4559f0ce3f65c82319a5920c8a948d0282b6fca17abae7"
            && hs.len() == 54
            && dg(&c["factHashes"])?
                == "509f0c36214445115650124eb9c9d17089af7020e29e0898a4134df6aa10d25c"
            && ds.len() == 30
            && dg(&x["selectedDeclarations"])?
                == "84fcb5be09cd85949e2490c1aa0eaec3a52e044c7e63ee914ccfca56c09b6902",
        "invalid-relnav-context-route-declarations",
    )?;
    for f in ds {
        shape(f, "s p o pol factHash")?;
        let h = f["factHash"].as_str().unwrap_or("");
        require(
            f["pol"] == "is"
                && h.len() == 64
                && h.bytes()
                    .all(|q| q.is_ascii_digit() || (b'a'..=b'f').contains(&q))
                && hs.iter().any(|v| v == h),
            "invalid-relnav-context-route-declarations",
        )?;
    }
    require(
        x["discovery"] == discovery() && x["authority"] == authority() && x["request"] == request(),
        "invalid-relnav-context-route-declarations",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-context-route-declarations-audit/v0.1","status":"qualified-authentic-declaration-boundary","authenticatedRouteDeclarations":true,"explicitSeedDeclaration":true,"singleWriterDeclaration":true,"retentionDeclaration":true,"publicationOrderingDeclaration":true,"failClosedRecoveryDeclaration":true,"executedRouteJourney":false,"runtimeRecords":false,"sourceCompleteness":false,"exportOrCaptureAuthority":false,"operationalCustody":false,"readyForMigration":false,"refusal":{"code":"relnav-context-route-operation-unproven","missing":["executed-authenticated-route-journey","runtime-records","source-head-completeness","export-or-capture-authority","operational-writer-custody"]},"policy":x["policy"]}),
    )
}
