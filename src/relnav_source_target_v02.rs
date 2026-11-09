//! Versioned Relationship Navigator source-target refinement.
use crate::{Result, require, shape};
use serde_json::{Value, json};
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(x, "kind prior structure expectedStructureId")?;
    require(
        x["kind"] == "temple3.relnav-source-target/v0.2",
        "invalid-relnav-source-target-v0.2",
    )?;
    let prior = crate::relnav_source_target::assess(&x["prior"])
        .map_err(|_| crate::err("invalid-relnav-source-target-v0.2"))?;
    let structure = crate::relnav_context_runtime_structure::audit(&x["structure"])?;
    let sid = crate::canonical::identify("relnav-context-runtime-structure", &x["structure"])?;
    require(
        x["expectedStructureId"] == sid,
        "wrong-relnav-context-runtime-structure-anchor",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-source-target-assessment/v0.2","status":"candidate-target-with-refined-static-source-and-runtime-gaps","sourceId":prior["sourceId"],"formationId":prior["formationId"],"structureId":sid,"priorAssessmentKind":prior["kind"],"structureReconciliation":{"preserved":[],"changed":[{"semantic":"actor-context-boundary","source":"unmapped actor expression returns empty string; map completeness and runtime enforcement unverified","target":"unknown or unauthenticated signer explicitly refuses","reason":"target authority is stricter and independently qualified"},{"semantic":"basis-boundary","source":"basis expression projects the last returned manifest layer","target":"command binds exact complete target state identity","reason":"source expression does not prove live-head or completeness equivalence"},{"semantic":"runtime-provider-boundary","source":"missing service or wrong provider protocol explicitly throws; journal.relnav.contexts requested","target":"selected laboratory writer and signed admission","reason":"static fail-closed dependency is preserved as intent but provider execution and writer equivalence are unverified"},{"semantic":"projection-input-boundary","source":"source graph excluded and later durable witness wins a subject/predicate collision","target":"participants-all-state disclosure with retained origin and expression history","reason":"projection filtering and collision behavior are not disclosure, provenance, or custody equivalence"}],"added":[],"removed":[],"removalAssessment":"no-authenticated-source-removal-claim","unsupported":["route-actions-source-structure","provider-runtime-execution","private-actor-map-completeness-and-authority","live-parent-projection-and-route-enforcement","runtime-record-export-and-capture-authority"]},"authority":{"source":"governed-historical-static-source-not-runtime-authority","target":"candidate-profile-not-operative-import","activation":"unavailable"},"readyForMigration":false,"migration":"not-started","refusal":{"code":"relnav-source-runtime-authority-incomplete","missing":["route-actions-source-structure","provider-runtime-execution","runtime-record-export","capture-authority","source-completeness"]},"structure":structure}),
    )
}
