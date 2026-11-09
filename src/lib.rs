//! Temple3 laboratory protocol. All state advancement recomputes constitutional
//! admission; signatures alone never authorize a transition.
pub mod af6_cross_language_artifact_recognition_v01;
pub mod af6_synthetic_execution_transcript_bridge_v01;
pub mod basis_consequence;
pub mod basis_consequence_profile;
pub mod basis_decision;
pub mod basis_decision_profile;
pub mod canonical;
pub mod commitment;
pub mod coordination;
pub mod decision_request;
pub mod federation;
pub mod generation_publication;
pub mod identity;
pub mod inventory;
pub mod inventory_profile;
pub mod journal;
pub mod kernel;
pub mod migration;
pub mod pilot;
pub mod predicate;
pub mod replica;
pub mod runtime_events;
pub mod source_json;
pub mod typed_composition;
pub mod typed_composition_profile;
pub mod vectors;
pub mod voluntary_expression;
pub mod voluntary_expression_capacity;
pub mod voluntary_expression_profile;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub code: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.code)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
pub(crate) fn err(code: &str) -> Error {
    Error { code: code.into() }
}
pub(crate) fn require(ok: bool, code: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(err(code)) }
}
pub(crate) fn string<'a>(v: &'a Value, code: &str) -> Result<&'a str> {
    v.as_str().ok_or_else(|| err(code))
}
pub(crate) fn array<'a>(v: &'a Value, code: &str) -> Result<&'a Vec<Value>> {
    v.as_array().ok_or_else(|| err(code))
}
pub(crate) fn shape(v: &Value, fields: &str) -> Result<()> {
    let expected: std::collections::BTreeSet<_> = fields.split_whitespace().collect();
    require(
        v.as_object().is_some_and(|m| {
            m.keys()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>()
                == expected
        }),
        "invalid-envelope",
    )
}
pub(crate) fn without(v: &Value, field: &str) -> Result<Value> {
    let mut m = v
        .as_object()
        .ok_or_else(|| err("invalid-envelope"))?
        .clone();
    m.remove(field);
    Ok(Value::Object(m))
}
pub(crate) fn bind(kind: &str, field: &str, mut payload: Value) -> Result<Value> {
    let id = canonical::identify(kind, &payload)?;
    payload
        .as_object_mut()
        .ok_or_else(|| err("invalid-envelope"))?
        .insert(field.into(), id.into());
    Ok(payload)
}
pub(crate) fn decimal(v: &Value) -> Result<num_bigint::BigUint> {
    let s = string(v, "invalid-sequence")?;
    require(
        !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit()) && (s == "0" || !s.starts_with('0')),
        "invalid-sequence",
    )?;
    num_bigint::BigUint::parse_bytes(s.as_bytes(), 10).ok_or_else(|| err("invalid-sequence"))
}
pub mod actor_authority_target;
pub mod actor_source_inventory;
pub mod application_contract_closure;
pub mod application_contract_completion_audit_v02;
pub mod claim_approval_source_target;
pub mod claim_import_application;
pub mod claim_import_application_profile;
pub mod claim_import_replay;
pub mod claim_import_review;
pub mod claim_import_target;
pub mod claim_lifecycle;
pub mod claim_lifecycle_compact_profile;
pub mod claim_lifecycle_host_policy;
pub mod claim_lifecycle_local;
pub mod claim_lifecycle_retention;
pub mod claim_mapping_custody;
pub mod claim_mapping_selection;
pub mod claim_readiness;
pub mod claim_readiness_compact_profile;
pub mod claim_readiness_local;
pub mod claim_readiness_local_profile;
pub mod claim_readiness_profile;
pub mod claim_reconciliation;
pub mod claim_retained_mapping;
pub mod claim_source_inventory;
pub mod exact_money;
pub mod foundation_obligation_ledger;
pub mod foundation_obligation_ledger_v02;
pub mod governed_migration_preflight;
pub mod inventory_operational_authority_v02;
pub mod inventory_source_inventory;
pub mod inventory_source_target;
pub mod relnav_authentic_corpus_capacity;
pub mod relnav_context_route_declarations;
pub mod relnav_context_runtime_structure;
pub mod relnav_governed_authority_refusal_v08;
pub mod relnav_operational_authority_v02;
pub mod relnav_operational_authority_v03;
pub mod relnav_operational_authority_v04;
pub mod relnav_operational_authority_v05;
pub mod relnav_runtime_authority_boundary_v07;
pub mod relnav_runtime_authority_refusal_v06;
pub mod relnav_source_inventory;
pub mod relnav_source_target;
pub mod relnav_source_target_v02;
pub mod relnav_source_target_v03;
pub mod relnav_source_target_v04;
pub mod relnav_source_target_v05;
pub mod relnav_source_target_v06;
pub mod relnav_source_target_v07;
pub mod representative_authority_closure;
pub mod representative_authority_closure_v03;
pub mod representative_authority_closure_v04;
pub mod representative_authority_closure_v05;
pub mod representative_authority_closure_v06;
pub mod representative_authority_closure_v07;
pub mod representative_composition_profile;
pub mod representative_composition_profile_v02;
pub mod representative_formation_closure;
pub mod representative_formation_closure_v02;
pub mod representative_formation_closure_v03;
pub mod runtime_capture_v3_authority_bundle_v01;
pub mod runtime_export_capacity;
pub mod sealed_runtime_events;
pub mod selected_formation_surface;

pub mod source_capsule_custody;
pub mod source_observation_capsule;
