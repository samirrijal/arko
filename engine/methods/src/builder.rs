//! Build the sparse `C` matrix (k × m) from a method + flow list.
//!
//! The construction enforces a single invariant that surprises users
//! who come from matrix-multiply frameworks: **at most one factor per
//! (category, flow) cell.** If two factors in the same category both
//! match the same flow, that is a method-authorship bug, not a silent
//! "summed" value — we surface it as `CMatrixError::DuplicateMatch`.

use crate::method::{FactorMatch, ImpactMethod};
use arko_core::{
    matrices::SparseMatrix,
    meta::{FlowMeta, ImpactMeta},
    units::Unit,
};
use sprs::TriMat;
use thiserror::Error;

/// Errors returned by [`build_c_matrix`].
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CMatrixError {
    /// Two characterization factors in the same category matched the
    /// same flow. The cell `(category, flow)` is ambiguous.
    #[error(
        "duplicate match: category `{category}` (row {row}) assigned factors \
         from `{first}` and `{second}` to flow `{flow}` (column {col})"
    )]
    DuplicateMatch {
        category: String,
        row: usize,
        flow: String,
        col: usize,
        first: String,
        second: String,
    },

    /// A characterization factor was non-finite (NaN or ±infinity).
    #[error("non-finite factor: category `{category}`, flow `{flow}`, value {value}")]
    NonFiniteFactor {
        category: String,
        flow: String,
        value: f64,
    },
}

/// The successful output of [`build_c_matrix`].
#[derive(Debug, Clone, PartialEq)]
pub struct CMatrixBuild {
    /// The `k × m` sparse characterization matrix.
    pub matrix: SparseMatrix,
    /// One `ImpactMeta` per category, in the same order as
    /// `method.categories` (so `impacts[i]` is the label of row `i`).
    pub impacts: Vec<ImpactMeta>,
    /// Flows that were never matched by any factor in any category.
    /// Not an error — these flows simply don't contribute to any
    /// impact under this method — but surfaced so the caller can
    /// display them to the user and optionally prompt for additional
    /// factors.
    pub unmatched_flows: Vec<String>,
}

/// Build the `C` matrix for `method` against `flows`.
///
/// Determinism: triplets are sorted by `(row, col)` before handing to
/// sprs, so the resulting `CsMat` is byte-stable (spec §7.2).
pub fn build_c_matrix(
    method: &ImpactMethod,
    flows: &[FlowMeta],
) -> Result<CMatrixBuild, CMatrixError> {
    let k = method.categories.len();
    let m = flows.len();

    let mut triplets: Vec<(usize, usize, f64)> = Vec::new();
    let mut impacts = Vec::with_capacity(k);
    let mut matched_any: Vec<bool> = vec![false; m];

    for (row_idx, cat) in method.categories.iter().enumerate() {
        // Which flow did we assign into this row? Detect duplicates.
        let mut first_hit: Vec<Option<String>> = vec![None; m];

        for factor in &cat.factors {
            if !factor.value.is_finite() {
                return Err(CMatrixError::NonFiniteFactor {
                    category: cat.id.clone(),
                    flow: matcher_label(&factor.match_on),
                    value: factor.value,
                });
            }
            for (col_idx, flow) in flows.iter().enumerate() {
                if !factor.match_on.matches(flow) {
                    continue;
                }
                matched_any[col_idx] = true;
                let label = matcher_label(&factor.match_on);
                if let Some(prev_label) = &first_hit[col_idx] {
                    return Err(CMatrixError::DuplicateMatch {
                        category: cat.id.clone(),
                        row: row_idx,
                        flow: flow.id.clone(),
                        col: col_idx,
                        first: prev_label.clone(),
                        second: label,
                    });
                }
                first_hit[col_idx] = Some(label);
                triplets.push((row_idx, col_idx, factor.value));
            }
        }

        impacts.push(ImpactMeta {
            id: cat.id.clone(),
            name: cat.name.clone(),
            unit: Unit::new(&cat.unit),
        });
    }

    triplets.sort_by_key(|a| (a.0, a.1));
    let mut t = TriMat::new((k, m));
    for (i, j, v) in triplets {
        t.add_triplet(i, j, v);
    }

    let unmatched_flows: Vec<String> = flows
        .iter()
        .enumerate()
        .filter(|(j, _)| !matched_any[*j])
        .map(|(_, f)| f.id.clone())
        .collect();

    Ok(CMatrixBuild {
        matrix: t.to_csr(),
        impacts,
        unmatched_flows,
    })
}

/// Short human-readable label for a matcher — used in error messages.
fn matcher_label(m: &FactorMatch) -> String {
    match m {
        FactorMatch::Cas { cas } => format!("CAS {cas}"),
        FactorMatch::CasOrigin { cas, origin } => format!("CAS {cas} ({origin:?})"),
        FactorMatch::CasCompartment { cas, compartment } => {
            format!("CAS {cas} in {compartment:?}")
        }
        FactorMatch::FlowId { id } => format!("id {id}"),
        FactorMatch::NameAndCompartment { name, compartment } => {
            format!("\"{name}\" in {compartment:?}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::{CharacterizationFactor, ImpactCategory, ImpactMethod};

    fn co2_factor(value: f64) -> CharacterizationFactor {
        CharacterizationFactor {
            match_on: FactorMatch::Cas {
                cas: "124-38-9".into(),
            },
            value,
            note: None,
        }
    }

    fn ch4_factor(value: f64) -> CharacterizationFactor {
        CharacterizationFactor {
            match_on: FactorMatch::Cas {
                cas: "74-82-8".into(),
            },
            value,
            note: None,
        }
    }

    fn flow(id: &str, cas: Option<&str>) -> FlowMeta {
        FlowMeta {
            id: id.into(),
            name: id.into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: cas.map(String::from),
            origin: arko_core::meta::FlowOrigin::Unspecified,
        }
    }

    fn single_cat_method(factors: Vec<CharacterizationFactor>) -> ImpactMethod {
        ImpactMethod {
            id: "m".into(),
            version: "0".into(),
            name: "m".into(),
            categories: vec![ImpactCategory {
                id: "gwp100".into(),
                name: "GWP100".into(),
                unit: "kg CO2-eq".into(),
                factors,
            }],
        }
    }

    #[test]
    fn single_category_single_flow_produces_1x1_matrix() {
        let method = single_cat_method(vec![co2_factor(1.0)]);
        let flows = vec![flow("f0", Some("124-38-9"))];
        let b = build_c_matrix(&method, &flows).unwrap();
        assert_eq!(b.matrix.shape(), (1, 1));
        assert_eq!(b.impacts.len(), 1);
        assert_eq!(b.impacts[0].id, "gwp100");
        assert!(b.unmatched_flows.is_empty());
    }

    #[test]
    fn unmatched_flows_surface_in_build_result() {
        let method = single_cat_method(vec![co2_factor(1.0)]);
        let flows = vec![
            flow("co2", Some("124-38-9")),
            flow("water", Some("7732-18-5")), // not in method
        ];
        let b = build_c_matrix(&method, &flows).unwrap();
        assert_eq!(b.matrix.shape(), (1, 2));
        assert_eq!(b.unmatched_flows, vec!["water".to_string()]);
    }

    #[test]
    fn two_factors_matching_same_flow_is_duplicate_error() {
        // Author bug: two CAS entries for the same molecule.
        let method = single_cat_method(vec![co2_factor(1.0), co2_factor(2.0)]);
        let flows = vec![flow("co2", Some("124-38-9"))];
        let err = build_c_matrix(&method, &flows).unwrap_err();
        assert!(matches!(err, CMatrixError::DuplicateMatch { .. }));
    }

    #[test]
    fn non_finite_factor_is_rejected() {
        let method = single_cat_method(vec![co2_factor(f64::NAN)]);
        let flows = vec![flow("co2", Some("124-38-9"))];
        let err = build_c_matrix(&method, &flows).unwrap_err();
        assert!(matches!(err, CMatrixError::NonFiniteFactor { .. }));
    }

    #[test]
    fn same_cas_in_different_categories_is_fine() {
        // Duplicate detection is per-category, not global.
        let method = ImpactMethod {
            id: "m".into(),
            version: "0".into(),
            name: "m".into(),
            categories: vec![
                ImpactCategory {
                    id: "gwp100".into(),
                    name: "GWP100".into(),
                    unit: "kg CO2-eq".into(),
                    // Builder test uses the plain-Cas (origin-agnostic) factor
                    // deliberately — this unit exercises the duplicate-detection
                    // machinery, not the AR6 fossil/non-fossil split.
                    factors: vec![co2_factor(1.0), ch4_factor(29.8)],
                },
                ImpactCategory {
                    id: "gwp20".into(),
                    name: "GWP20".into(),
                    unit: "kg CO2-eq".into(),
                    factors: vec![co2_factor(1.0), ch4_factor(81.2)],
                },
            ],
        };
        let flows = vec![flow("co2", Some("124-38-9")), flow("ch4", Some("74-82-8"))];
        let b = build_c_matrix(&method, &flows).unwrap();
        assert_eq!(b.matrix.shape(), (2, 2));
        assert_eq!(b.impacts.len(), 2);
    }

    #[test]
    fn flow_without_cas_falls_back_to_flow_id_match() {
        let method = single_cat_method(vec![CharacterizationFactor {
            match_on: FactorMatch::FlowId {
                id: "custom-ghg".into(),
            },
            value: 100.0,
            note: None,
        }]);
        let flows = vec![flow("custom-ghg", None)];
        let b = build_c_matrix(&method, &flows).unwrap();
        assert_eq!(b.matrix.shape(), (1, 1));
        assert!(b.unmatched_flows.is_empty());
    }

    // ---- CasCompartment interaction with builder --------------------

    fn so2_factor_air(value: f64) -> CharacterizationFactor {
        CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: "7446-09-5".into(),
                compartment: vec!["emission".into(), "air".into()],
            },
            value,
            note: None,
        }
    }

    fn so2_factor_water(value: f64) -> CharacterizationFactor {
        CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: "7446-09-5".into(),
                compartment: vec!["emission".into(), "water".into()],
            },
            value,
            note: None,
        }
    }

    fn so2_flow(compartment: Vec<&str>) -> FlowMeta {
        FlowMeta {
            id: "f_so2".into(),
            name: "SO2".into(),
            unit: Unit::new("kg"),
            compartment: compartment.into_iter().map(String::from).collect(),
            cas: Some("7446-09-5".into()),
            origin: arko_core::meta::FlowOrigin::Unspecified,
        }
    }

    #[test]
    fn cas_compartment_factor_routes_flow_into_matrix() {
        let method = single_cat_method(vec![so2_factor_air(1.31)]);
        let flows = vec![so2_flow(vec!["emission", "air"])];
        let b = build_c_matrix(&method, &flows).unwrap();
        assert_eq!(b.matrix.shape(), (1, 1));
        assert!(b.unmatched_flows.is_empty());
    }

    #[test]
    fn disjoint_cas_compartment_factors_coexist_in_one_category() {
        // Two factors, same CAS, different compartments — one flow
        // each. Neither duplicates the other; both land.
        let method = single_cat_method(vec![so2_factor_air(1.31), so2_factor_water(0.0)]);
        let mut air = so2_flow(vec!["emission", "air"]);
        air.id = "f_so2_air".into();
        let mut water = so2_flow(vec!["emission", "water"]);
        water.id = "f_so2_water".into();
        let flows = vec![air, water];
        let b = build_c_matrix(&method, &flows).unwrap();
        assert_eq!(b.matrix.shape(), (1, 2));
        assert!(b.unmatched_flows.is_empty());
    }

    #[test]
    fn cas_and_cas_compartment_matching_same_flow_is_duplicate_error() {
        // Authorship bug: a plain Cas factor and a CasCompartment
        // factor both match the same SO2-to-air flow. The builder
        // must hard-fail rather than silently pick one or sum the
        // values.
        let cas_factor = CharacterizationFactor {
            match_on: FactorMatch::Cas {
                cas: "7446-09-5".into(),
            },
            value: 1.0,
            note: None,
        };
        let method = single_cat_method(vec![cas_factor, so2_factor_air(1.31)]);
        let flows = vec![so2_flow(vec!["emission", "air"])];
        let err = build_c_matrix(&method, &flows).unwrap_err();
        assert!(matches!(err, CMatrixError::DuplicateMatch { .. }));
    }
}
