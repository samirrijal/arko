//! Study validator — implements the construction-order checks of
//! `specs/calc/v0.1.md` §6.1.
//!
//! The spec order is:
//!   1. Parameter DAG well-formed (no cycles).
//!   2. Parameter evaluation (all finite).
//!   3. Unit consistency — every `Unit` string in the study parses under
//!      `arko-units`. Cross-dimensional compatibility checks between
//!      flows and impact categories are deferred (they need the
//!      characterization matrix's structural role, which `arko-methods`
//!      owns).
//!   4. License-tier compatibility of the graph (deferred — full walk in
//!      `arko-license`; we still do a *structural* check that every
//!      `LicenseTierRef` resolves).
//!   5. Matrix shape consistency.
//!   6. Well-conditioning heuristic (deferred — would require estimating
//!      `cond_1(A)`, which belongs in the solver layer).
//!
//! v0.1 enforces 1, 2, 3 (parse-only), 5, and the structural half of 4.
//! Stage 6 is an explicit no-op so callers see the gap.
//!
//! Calling `validate` before `arko_core::pipeline::compute` is the
//! supported path; `compute` itself does only the *minimal* shape check
//! required not to panic.

use arko_core::{AllocationMode, EngineError, Study};

/// Validate a study. Returns `Ok(())` if every enforced check passes,
/// otherwise returns the first error encountered — order is exactly the
/// §6.1 order so diagnostics are stable.
pub fn validate(study: &Study) -> Result<(), EngineError> {
    check_parameters(study)?;
    check_units(study)?;
    check_license_refs(study)?;
    check_shapes(study)?;
    check_allocations(study)?;
    // (6) well-conditioning heuristic — deferred.
    Ok(())
}

fn check_units(study: &Study) -> Result<(), EngineError> {
    // Parse every unit string under `arko-units`. Any failure to parse
    // surfaces as `E_UNIT_INCOMPATIBLE` with the offending location —
    // the spec's §13.1 error code for this stage.
    for p in &study.processes {
        arko_units::ParsedUnit::parse(p.reference_unit.as_str()).map_err(|e| {
            EngineError::UnitIncompatible(format!(
                "process `{}` reference_unit `{}`: {e}",
                p.id,
                p.reference_unit.as_str()
            ))
        })?;
    }
    for f in &study.flows {
        arko_units::ParsedUnit::parse(f.unit.as_str()).map_err(|e| {
            EngineError::UnitIncompatible(format!(
                "flow `{}` unit `{}`: {e}",
                f.id,
                f.unit.as_str()
            ))
        })?;
    }
    for i in &study.impacts {
        arko_units::ParsedUnit::parse(i.unit.as_str()).map_err(|e| {
            EngineError::UnitIncompatible(format!(
                "impact `{}` unit `{}`: {e}",
                i.id,
                i.unit.as_str()
            ))
        })?;
    }
    Ok(())
}

fn check_parameters(study: &Study) -> Result<(), EngineError> {
    // Delegates to arko-parameters; returns ParamCycle / ParamUnresolved /
    // ParamNonfinite on failure, per §5 / §13.1.
    arko_parameters::evaluate(&study.parameters).map(drop)
}

fn check_license_refs(study: &Study) -> Result<(), EngineError> {
    let n_tiers = study.license_tiers.len();
    for p in &study.processes {
        if (p.license_tier.0 as usize) >= n_tiers {
            return Err(EngineError::Internal(format!(
                "process `{}` has license_tier index {} but only {} tiers defined",
                p.id, p.license_tier.0, n_tiers
            )));
        }
    }
    Ok(())
}

fn check_shapes(study: &Study) -> Result<(), EngineError> {
    let n = study.n_processes();
    let m = study.n_flows();
    let k = study.n_impacts();

    let (a_rows, a_cols) = study.technosphere.shape();
    if a_rows != n || a_cols != n {
        return Err(EngineError::ShapeMismatch(format!(
            "technosphere A is {a_rows}×{a_cols}, expected {n}×{n}"
        )));
    }
    if study.functional_unit.dim() != n {
        return Err(EngineError::ShapeMismatch(format!(
            "functional_unit length {} != technosphere dim {n}",
            study.functional_unit.dim()
        )));
    }
    let (b_rows, b_cols) = study.biosphere.shape();
    if b_rows != m || b_cols != n {
        return Err(EngineError::ShapeMismatch(format!(
            "biosphere B is {b_rows}×{b_cols}, expected {m}×{n}"
        )));
    }
    let (c_rows, c_cols) = study.characterization.shape();
    if c_rows != k || c_cols != m {
        return Err(EngineError::ShapeMismatch(format!(
            "characterization C is {c_rows}×{c_cols}, expected {k}×{m}"
        )));
    }
    Ok(())
}

fn check_allocations(study: &Study) -> Result<(), EngineError> {
    for p in &study.processes {
        let Some(alloc) = &p.allocation else { continue };
        if alloc.mode == AllocationMode::User {
            let sum: f64 = alloc.user_factors.iter().sum();
            if (sum - 1.0).abs() > 1e-9 {
                return Err(EngineError::AllocationInvalid { sum });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arko_core::{
        Allocation, AllocationMode, FlowMeta, ImpactMeta, LicenseTier, LicenseTierRef,
        MethodRef, ProcessMeta, SignConvention, SparseMatrix, SparseVector, Unit,
    };
    use sprs::TriMat;

    fn build_minimal_study() -> Study {
        let mut a = TriMat::new((1, 1));
        a.add_triplet(0, 0, 1.0);
        let technosphere: SparseMatrix = a.to_csr();
        let biosphere: SparseMatrix = TriMat::<f64>::new((1, 1)).to_csr();
        let characterization: SparseMatrix = TriMat::<f64>::new((1, 1)).to_csr();

        Study {
            technosphere,
            biosphere,
            characterization,
            functional_unit: SparseVector::new(1, vec![0], vec![1.0]),
            processes: vec![ProcessMeta {
                id: "p".into(),
                name: "p".into(),
                reference_product: "p".into(),
                reference_unit: Unit::new("kg"),
                allocation: None,
                license_tier: LicenseTierRef(0),
                geography: None,
            }],
            flows: vec![FlowMeta {
                id: "f".into(),
                name: "f".into(),
                unit: Unit::new("kg"),
                compartment: vec![],
                cas: None,
                origin: Default::default(),
            }],
            impacts: vec![ImpactMeta {
                id: "i".into(),
                name: "i".into(),
                unit: Unit::new("kg"),
            }],
            parameters: vec![],
            license_tiers: vec![LicenseTier::permissive("custom-user")],
            method: MethodRef {
                id: "m".into(),
                version: "0".into(),
            },
            sign_convention: SignConvention::ProducerPositive,
        }
    }

    #[test]
    fn minimal_study_is_valid() {
        validate(&build_minimal_study()).unwrap();
    }

    #[test]
    fn wrong_shape_rejected() {
        let mut s = build_minimal_study();
        // Force A to 1x2 (not square).
        let mut a = TriMat::new((1, 2));
        a.add_triplet(0, 0, 1.0);
        s.technosphere = a.to_csr();
        let err = validate(&s).unwrap_err();
        assert_eq!(err.code(), "E_SHAPE_MISMATCH");
    }

    #[test]
    fn bad_tier_ref_rejected() {
        let mut s = build_minimal_study();
        s.processes[0].license_tier = LicenseTierRef(42);
        let err = validate(&s).unwrap_err();
        assert_eq!(err.code(), "E_INTERNAL");
    }

    #[test]
    fn unparseable_flow_unit_rejected() {
        let mut s = build_minimal_study();
        s.flows[0].unit = Unit::new("banana");
        let err = validate(&s).unwrap_err();
        assert_eq!(err.code(), "E_UNIT_INCOMPATIBLE");
    }

    #[test]
    fn unparseable_process_reference_unit_rejected() {
        let mut s = build_minimal_study();
        s.processes[0].reference_unit = Unit::new("banana");
        let err = validate(&s).unwrap_err();
        assert_eq!(err.code(), "E_UNIT_INCOMPATIBLE");
    }

    #[test]
    fn tagged_impact_unit_is_accepted() {
        // kg CO2-eq carries a semantic tag but still parses cleanly.
        let mut s = build_minimal_study();
        s.impacts[0].unit = Unit::new("kg CO2-eq");
        validate(&s).unwrap();
    }

    #[test]
    fn user_allocation_must_sum_to_one() {
        let mut s = build_minimal_study();
        s.processes[0].allocation = Some(Allocation {
            mode: AllocationMode::User,
            user_factors: vec![0.3, 0.3], // sums to 0.6
        });
        let err = validate(&s).unwrap_err();
        assert_eq!(err.code(), "E_ALLOCATION_INVALID");
    }
}
