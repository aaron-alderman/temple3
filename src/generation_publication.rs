//! Independently derived inventory contract manifests and anchored publication chains.
use serde_json::{Value, json};
type R<T> = Result<T, String>;
fn need(ok: bool, c: &str) -> R<()> {
    if ok { Ok(()) } else { Err(c.into()) }
}
fn exact(v: &Value, fields: &str) -> R<()> {
    crate::shape(v, fields).map_err(|_| "invalid-publication-shape".into())
}
fn canonical(v: &Value) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|e| e.code)
}
fn id(kind: &str, v: &Value) -> R<String> {
    crate::canonical::identify(kind, v).map_err(|e| e.code)
}
pub fn config(c: &Value) -> R<()> {
    canonical(c)?;
    exact(
        c,
        "kind instance sourceKey publicationKey derivation forgiveness retention disclosure amendment",
    )?;
    need(
        c["kind"] == "temple3.publication-config/v0.1"
            && c["instance"].as_str().is_some_and(|s| {
                s.len() == 32
                    && s.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }),
        "invalid-publication-config",
    )?;
    crate::identity::key_id(&c["sourceKey"]).map_err(|e| e.code)?;
    crate::identity::key_id(&c["publicationKey"]).map_err(|e| e.code)?;
    need(
        c["sourceKey"] != c["publicationKey"],
        "shared-publication-authority",
    )?;
    need(
        c["derivation"] == "inventory-contract-manifest/v0.1"
            && c["retention"] == "retain-all-generation-and-publication-evidence"
            && c["disclosure"] == "public-manifests-and-source-evidence"
            && c["amendment"] == "fixed-stream-policy",
        "invalid-publication-config",
    )?;
    need(
        c["forgiveness"]
            == json!({"disposition":"refuse","scope":"generation-and-publication-history"}),
        "invalid-publication-config",
    )
}
fn stream_id(c: &Value) -> R<String> {
    config(c)?;
    id("publication-stream", c)
}
fn meaning(v: &Value) -> R<()> {
    canonical(v)?;
    exact(v, "descriptor genesis expectedTempleId")?;
    crate::inventory_profile::recognize(&v["genesis"], &v["descriptor"], &v["expectedTempleId"])
        .map_err(|e| e.code)?;
    Ok(())
}
fn generation_payload(c: &Value, sequence: &Value, previous: &Value, value: &Value) -> R<Value> {
    let stream = stream_id(c)?;
    meaning(value)?;
    let mut p = json!({"kind":"temple3.semantic-generation/v0.1","streamId":stream,"sequence":sequence,"previousGenerationId":previous,"meaning":value});
    p["generationId"] = json!(id("semantic-generation", &p)?);
    Ok(p)
}
pub fn generations(c: &Value, sources: &Value, anchor: &Value) -> R<()> {
    config(c)?;
    let bytes = canonical(sources)?;
    let gs = sources.as_array().ok_or("generation-chain-bound")?;
    need(
        gs.len() <= 32 && bytes.len() <= 4194304,
        "generation-chain-bound",
    )?;
    need(
        anchor.as_str().is_some_and(|a| !a.is_empty()),
        "missing-generation-anchor",
    )?;
    let mut head = json!("none");
    for (index, s) in gs.iter().enumerate() {
        need(
            crate::identity::verify_envelope(s, "attestation", &c["sourceKey"]),
            "invalid-generation-signature",
        )?;
        exact(
            &s["payload"],
            "kind streamId sequence previousGenerationId meaning generationId",
        )?;
        let expected = generation_payload(
            c,
            &json!((index + 1).to_string()),
            &head,
            &s["payload"]["meaning"],
        )?;
        need(s["payload"] == expected, "invalid-generation-chain")?;
        head = expected["generationId"].clone();
    }
    need(head == *anchor, "generation-anchor-mismatch")
}
fn commands() -> Value {
    fn input(name: &str, ty: &str) -> Value {
        json!({"name":name,"type":ty})
    }
    json!([
    {"op":"draft","inputs":[input("source","account-reference"),input("destination","account-reference"),input("quantity","positive-integer-1..999999999"),input("unit","selected-unit")],"authority":"source-owner","guards":["existing-distinct-accounts","matching-unit","transfer-capacity"],"effects":["append-draft","balances-unchanged"]},
    {"op":"correct","inputs":[input("transferId","transfer-reference"),input("quantity","positive-integer-1..999999999")],"authority":"transfer-author","guards":["existing-draft","changed-quantity"],"effects":["replace-draft-quantity","balances-unchanged"]},
    {"op":"cancel","inputs":[input("transferId","transfer-reference")],"authority":"transfer-author","guards":["existing-draft"],"effects":["retain-cancelled-transfer","balances-unchanged"]},
    {"op":"post","inputs":[input("transferId","transfer-reference")],"authority":"selected-reviewer","guards":["existing-draft","account-balance-bounds","selected-stock-policy"],"effects":["retain-posted-transfer","debit-source-credit-destination","recompute-all-balances"]}
    ])
}
pub fn derive(c: &Value, g: &Value) -> R<Value> {
    config(c)?;
    need(
        crate::identity::verify_envelope(g, "attestation", &c["sourceKey"]),
        "invalid-generation-signature",
    )?;
    let p = &g["payload"];
    exact(
        p,
        "kind streamId sequence previousGenerationId meaning generationId",
    )?;
    need(
        p["sequence"].as_str().is_some_and(|s| {
            !s.is_empty()
                && s.len() <= 2
                && s.as_bytes()[0] != b'0'
                && s.bytes().all(|b| b.is_ascii_digit())
                && s.parse::<u32>().is_ok_and(|n| n <= 32)
        }) && p["previousGenerationId"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "invalid-generation-chain",
    )?;
    need(
        *p == generation_payload(c, &p["sequence"], &p["previousGenerationId"], &p["meaning"])?,
        "invalid-generation-chain",
    )?;
    let f = &p["meaning"]["descriptor"]["formation"];
    let mut a = json!({"kind":"temple3.inventory-contract-manifest/v0.1","derivation":c["derivation"],"streamId":p["streamId"],"generationId":p["generationId"],"templeId":p["meaning"]["expectedTempleId"],"meaningId":id("application-meaning",&p["meaning"])?,"policy":f["policy"],"accounts":f["accounts"],
 "bounds":{"accounts":"2..8","transfers":"0..8","sequence":"0..1024","stateBytes":"16384","integer":"-999999999..999999999","arithmetic":"exact-integer"},
 "commonGuards":["exact-canonical-shapes","participant-actor","sequence-capacity","fixed-policy-and-accounts","single-atomic-state-replacement"],
 "equations":{"account":"opening+posted-incoming-posted-outgoing","aggregate":"sum-balances=sum-openings","includedStatus":"posted","draftReservation":"none"},"commands":commands()});
    need(canonical(&a)?.len() <= 16384, "artifact-bound")?;
    a["artifactId"] = json!(id("contract-artifact", &a)?);
    Ok(a)
}
pub fn artifact_valid(c: &Value, g: &Value, a: &Value) -> bool {
    derive(c, g).is_ok_and(|expected| expected == *a)
}
pub fn publications(
    c: &Value,
    sources: &Value,
    source_anchor: &Value,
    records: &Value,
    anchor: &Value,
) -> R<()> {
    config(c)?;
    generations(c, sources, source_anchor)?;
    let bytes = canonical(records)?;
    let ps = records.as_array().ok_or("publication-chain-bound")?;
    need(
        ps.len() <= 32 && bytes.len() <= 4194304,
        "publication-chain-bound",
    )?;
    need(
        anchor.as_str().is_some_and(|a| !a.is_empty()),
        "missing-publication-anchor",
    )?;
    let mut head = json!("none");
    let mut seen = Vec::new();
    for (index, p) in ps.iter().enumerate() {
        need(
            crate::identity::verify_envelope(p, "attestation", &c["publicationKey"]),
            "invalid-publication-signature",
        )?;
        let v = &p["payload"];
        exact(
            v,
            "kind streamId sequence previousPublicationId generationId artifact publicationId",
        )?;
        need(
            v["kind"] == "temple3.generation-publication/v0.1"
                && v["streamId"] == stream_id(c)?
                && v["sequence"].as_str() == Some((index + 1).to_string().as_str())
                && v["previousPublicationId"] == head
                && !seen.contains(&v["generationId"]),
            "invalid-publication-chain",
        )?;
        let g = sources
            .as_array()
            .unwrap()
            .iter()
            .find(|g| g["payload"]["generationId"] == v["generationId"]);
        need(
            g.is_some_and(|g| artifact_valid(c, g, &v["artifact"])),
            "invalid-derived-artifact",
        )?;
        let mut payload = v.clone();
        payload.as_object_mut().unwrap().remove("publicationId");
        need(
            v["publicationId"] == id("generation-publication", &payload)?,
            "invalid-publication-chain",
        )?;
        seen.push(v["generationId"].clone());
        head = v["publicationId"].clone();
    }
    need(head == *anchor, "publication-anchor-mismatch")
}
pub fn resolve(
    c: &Value,
    sources: &Value,
    source_anchor: &Value,
    records: &Value,
    anchor: &Value,
) -> R<Value> {
    publications(c, sources, source_anchor, records, anchor)?;
    let selected = records
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["payload"]["generationId"] == *source_anchor);
    Ok(match selected {
        Some(p) => {
            json!({"status":"current","generationId":source_anchor,"publicationId":p["payload"]["publicationId"],"artifact":p["payload"]["artifact"]})
        }
        None => json!({"status":"pending","generationId":source_anchor}),
    })
}
