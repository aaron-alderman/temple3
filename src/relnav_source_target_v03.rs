//! Relationship Navigator authentic-declaration target refinement v0.3.
use crate::{Result, require, shape};
use serde_json::{Value, json};
pub fn assess(x: &Value) -> Result<Value> {
    crate::canonical::canonical_json(x)?;
    shape(x, "kind prior declarations expectedDeclarationsId")?;
    require(
        x["kind"] == "temple3.relnav-source-target/v0.3",
        "invalid-relnav-source-target-v0.3",
    )?;
    let prior = crate::relnav_source_target_v02::assess(&x["prior"])?;
    let declarations = crate::relnav_context_route_declarations::audit(&x["declarations"])?;
    let id = crate::canonical::identify("relnav-context-route-declarations", &x["declarations"])?;
    require(
        x["expectedDeclarationsId"] == id,
        "wrong-relnav-context-route-declarations-anchor",
    )?;
    Ok(
        json!({"kind":"temple3.relnav-source-target-assessment/v0.3","status":"candidate-target-with-authentic-declarations-and-operational-gaps","sourceId":prior["sourceId"],"formationId":prior["formationId"],"structureId":prior["structureId"],"declarationsId":id,"priorAssessmentKind":prior["kind"],"declarationReconciliation":{"preserved":[{"semantic":"authenticated-route-intent","source":"commit and view routes declare authenticated access","target":"signed actor admission and authenticated participant inspection","qualification":"candidate-preservation-not-executed-enforcement-equivalence"},{"semantic":"explicit-formation-intent","source":"Context journal bootstrap declares explicitSeedOnly","target":"formation is a separate explicit operation with no implicit open default","qualification":"candidate-preservation-not-source-formation-equivalence"},{"semantic":"retained-recovery-intent","source":"content-addressed retention, bundle-before-receipt publication and fail-closed linear recovery declared","target":"retained laboratory writer and recovery qualification","qualification":"candidate-preservation-not-operational-custody-equivalence"}],"changed":[{"semantic":"disclosure-scope","source":"route text declares signed-in participant own private Context and authorized closure","target":"participants-all-state selected voluntary-expression disclosure","reason":"target policy differs and source runtime delivery is unverified"},{"semantic":"writer-custody","source":"singleAuthoritativeProcessV1 declared","target":"selected laboratory writer with no source-writer transfer","reason":"a declaration does not establish custody identity or authority transfer"},{"semantic":"post-command-delivery","source":"recordAnswer declares reload-authorized-route-after-completion","target":"policy-first loopback response and explicit retained-outcome recovery","reason":"delivery mechanisms are intentionally different and neither proves source execution"}],"added":[],"removed":[],"removalAssessment":"no-authenticated-source-removal-claim","unsupported":["executed-authenticated-context-routes","authentic-runtime-record-export-and-capture","current-source-head-completeness","operational-single-writer-custody-and-recovery","private-actor-map-completeness-and-authority","source-to-target-record-corpus-fit"]},"authority":{"source":"verified-admitted-declarations-not-operative-runtime-authority","target":"candidate-profile-not-operative-import","activation":"unavailable"},"readyForMigration":false,"migration":"not-started","refusal":{"code":"relnav-operational-source-journey-incomplete","missing":["executed-route-journey","runtime-record-export","capture-authority","source-head-completeness","operational-writer-custody","authentic-corpus-fit"]},"declarations":declarations}),
    )
}
