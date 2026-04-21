//! Data types for impact methods.
//!
//! An [`ImpactMethod`] is an aggregation of one or more
//! [`ImpactCategory`]s. Each category is one row of the `C` matrix;
//! each `CharacterizationFactor` within a category is one nonzero
//! entry (after resolving the matcher against the study's flows).

use arko_core::meta::{FlowMeta, FlowOrigin};
use serde::{Deserialize, Serialize};

/// A named, versioned impact-assessment method.
///
/// The pair `(id, version)` is the registry key. `name` is a
/// human-readable label shown in UI and audit logs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactMethod {
    pub id: String,
    pub version: String,
    pub name: String,
    /// Ordered list of impact categories. Order is significant: the
    /// index `i` into this vector becomes row `i` of the built `C`
    /// matrix, which in turn becomes the `i`-th entry of the `h`
    /// result vector. Stability of this order across versions is the
    /// responsibility of the method author.
    pub categories: Vec<ImpactCategory>,
}

/// One row of `C`: a named impact category with its characterization
/// factors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactCategory {
    /// Stable short identifier (e.g., `"gwp100"`, `"acidification"`).
    pub id: String,
    /// Human-readable long name.
    pub name: String,
    /// Reference unit of the impact score (e.g., `"kg CO2-eq"`).
    pub unit: String,
    /// Flow-level factors. An empty list is allowed (the category
    /// contributes no factors — useful as a placeholder during method
    /// authorship).
    pub factors: Vec<CharacterizationFactor>,
}

/// One nonzero entry of `C`: a matcher that selects one (or at most
/// one, per specificity tie-breaking) flow, and the factor value to
/// place at that `(category, flow)` cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterizationFactor {
    #[serde(flatten)]
    pub match_on: FactorMatch,
    pub value: f64,
    /// Optional attribution note — cited in audit logs when this
    /// factor contributes to a restricted result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Flow-matching rule.
///
/// `#[serde(tag = "match")]` gives JSON `{"match": "cas", "cas": "124-38-9"}`
/// which is both self-documenting and easy to author by hand.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "match", rename_all = "snake_case")]
pub enum FactorMatch {
    /// Match flows by CAS registry number (e.g., `"124-38-9"` for CO2).
    /// The most reliable cross-database identifier. **Origin-agnostic**
    /// — matches any `FlowOrigin`, including `Unspecified`. Use
    /// [`FactorMatch::CasOrigin`] when the factor value depends on
    /// fossil vs non-fossil provenance.
    Cas { cas: String },
    /// Match by CAS **and** exact flow origin. A flow with
    /// `FlowOrigin::Unspecified` will not match a `CasOrigin` factor —
    /// the missing information surfaces as a
    /// `CMatrixBuild::unmatched_flows` entry rather than silently
    /// inheriting a possibly-wrong factor (e.g., applying the fossil
    /// CH4 GWP to a flow that might actually be biogenic).
    ///
    /// Introduced in v0.0.1 for IPCC AR6 GWP100 CH4, which has distinct
    /// fossil (`29.8`) and non-fossil (`27.0`) values per AR6 WG1
    /// Chapter 7 Table 7.15.
    CasOrigin { cas: String, origin: FlowOrigin },
    /// Match by CAS **and** compartment prefix. Use when the
    /// characterization factor depends on where the substance is
    /// emitted — e.g., EF 3.1 Acidification counts SO2 emissions to air
    /// but not SO2 emissions to water; EF 3.1 Eutrophication splits
    /// nitrogen species by freshwater/marine/terrestrial end compartment.
    ///
    /// Semantics: the flow's CAS must equal `cas` **and** the flow's
    /// `compartment` path must start with the matcher's `compartment`
    /// (prefix match, same as [`Self::NameAndCompartment`]). An empty
    /// `compartment` vector is a prefix of everything — equivalent to
    /// [`Self::Cas`] and therefore redundant; prefer the simpler
    /// variant when compartment does not constrain.
    ///
    /// Origin-agnostic: matches any [`FlowOrigin`], including
    /// `Unspecified`. Substances whose CF depends on **both** origin
    /// and compartment are not expressible in a single matcher; method
    /// authors must pick the axis that matters for the category (for
    /// AR6 CH4 in air-emission contexts, origin; for EF 3.1 tox
    /// categories, compartment) and the other axis is ignored.
    ///
    /// Introduced in Phase 1 for EF 3.1 acidification + eutrophication
    /// + (future) particulate matter / tox categories.
    CasCompartment {
        cas: String,
        compartment: Vec<String>,
    },
    /// Match by stable flow id (used when CAS is absent).
    FlowId { id: String },
    /// Fuzzy match: flow name equals `name` **and** compartment path
    /// starts with `compartment`. Case-sensitive on both.
    NameAndCompartment {
        name: String,
        compartment: Vec<String>,
    },
}

impl FactorMatch {
    /// `true` iff `flow` satisfies this matcher.
    #[must_use]
    pub fn matches(&self, flow: &FlowMeta) -> bool {
        match self {
            Self::Cas { cas } => flow.cas.as_deref() == Some(cas.as_str()),
            Self::CasOrigin { cas, origin } => {
                flow.cas.as_deref() == Some(cas.as_str()) && flow.origin == *origin
            }
            Self::CasCompartment { cas, compartment } => {
                flow.cas.as_deref() == Some(cas.as_str())
                    && flow.compartment.len() >= compartment.len()
                    && flow
                        .compartment
                        .iter()
                        .zip(compartment.iter())
                        .all(|(a, b)| a == b)
            }
            Self::FlowId { id } => flow.id == *id,
            Self::NameAndCompartment { name, compartment } => {
                flow.name == *name
                    && flow.compartment.len() >= compartment.len()
                    && flow
                        .compartment
                        .iter()
                        .zip(compartment.iter())
                        .all(|(a, b)| a == b)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arko_core::units::Unit;

    fn co2_flow() -> FlowMeta {
        FlowMeta {
            id: "f0".into(),
            name: "Carbon dioxide".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("124-38-9".into()),
            origin: FlowOrigin::Unspecified,
        }
    }

    fn ch4_flow(origin: FlowOrigin) -> FlowMeta {
        FlowMeta {
            id: "f_ch4".into(),
            name: "Methane".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("74-82-8".into()),
            origin,
        }
    }

    #[test]
    fn cas_matches_when_equal() {
        let m = FactorMatch::Cas {
            cas: "124-38-9".into(),
        };
        assert!(m.matches(&co2_flow()));
    }

    #[test]
    fn cas_rejects_when_flow_has_no_cas() {
        let mut flow = co2_flow();
        flow.cas = None;
        let m = FactorMatch::Cas {
            cas: "124-38-9".into(),
        };
        assert!(!m.matches(&flow));
    }

    #[test]
    fn name_and_compartment_requires_prefix_match() {
        let m = FactorMatch::NameAndCompartment {
            name: "Carbon dioxide".into(),
            compartment: vec!["emission".into(), "air".into()],
        };
        assert!(m.matches(&co2_flow()));

        let m2 = FactorMatch::NameAndCompartment {
            name: "Carbon dioxide".into(),
            compartment: vec!["emission".into(), "water".into()],
        };
        assert!(!m2.matches(&co2_flow()));
    }

    #[test]
    fn name_and_compartment_allows_flow_with_deeper_path() {
        // The matcher's compartment is `["emission", "air"]`; the flow's
        // is `["emission", "air", "urban"]` — the matcher is a prefix.
        let mut flow = co2_flow();
        flow.compartment = vec!["emission".into(), "air".into(), "urban".into()];
        let m = FactorMatch::NameAndCompartment {
            name: "Carbon dioxide".into(),
            compartment: vec!["emission".into(), "air".into()],
        };
        assert!(m.matches(&flow));
    }

    #[test]
    fn flow_id_exact_match() {
        let m = FactorMatch::FlowId { id: "f0".into() };
        assert!(m.matches(&co2_flow()));
        let m2 = FactorMatch::FlowId {
            id: "something-else".into(),
        };
        assert!(!m2.matches(&co2_flow()));
    }

    #[test]
    fn cas_origin_matches_exact_origin() {
        let m = FactorMatch::CasOrigin {
            cas: "74-82-8".into(),
            origin: FlowOrigin::Fossil,
        };
        assert!(m.matches(&ch4_flow(FlowOrigin::Fossil)));
        assert!(!m.matches(&ch4_flow(FlowOrigin::Biogenic)));
        assert!(
            !m.matches(&ch4_flow(FlowOrigin::LandUseChange)),
            "LandUseChange must NOT match a fossil-only factor — exact-match policy applies to all origin variants"
        );
        assert!(
            !m.matches(&ch4_flow(FlowOrigin::Unspecified)),
            "Unspecified must NOT match a fossil-only factor — that is the whole point of the variant"
        );
    }

    #[test]
    fn cas_origin_still_requires_cas_to_match() {
        // Origin alone is not enough: CAS must still agree.
        let m = FactorMatch::CasOrigin {
            cas: "74-82-8".into(),
            origin: FlowOrigin::Fossil,
        };
        // CO2 flow has the wrong CAS even if we claimed it was fossil.
        let mut wrong_cas = co2_flow();
        wrong_cas.origin = FlowOrigin::Fossil;
        assert!(!m.matches(&wrong_cas));
    }

    #[test]
    fn plain_cas_is_origin_agnostic() {
        let m = FactorMatch::Cas {
            cas: "74-82-8".into(),
        };
        assert!(m.matches(&ch4_flow(FlowOrigin::Fossil)));
        assert!(m.matches(&ch4_flow(FlowOrigin::Biogenic)));
        assert!(m.matches(&ch4_flow(FlowOrigin::LandUseChange)));
        assert!(m.matches(&ch4_flow(FlowOrigin::Unspecified)));
    }

    #[test]
    fn cas_origin_round_trips_through_json() {
        let f = CharacterizationFactor {
            match_on: FactorMatch::CasOrigin {
                cas: "74-82-8".into(),
                origin: FlowOrigin::Fossil,
            },
            value: 29.8,
            note: Some("AR6 fossil CH4".into()),
        };
        let s = serde_json::to_string(&f).unwrap();
        assert!(s.contains("\"match\":\"cas_origin\""));
        assert!(s.contains("\"origin\":\"fossil\""));
        let back: CharacterizationFactor = serde_json::from_str(&s).unwrap();
        assert_eq!(f, back);
    }

    #[test]
    fn factor_roundtrips_through_json() {
        let f = CharacterizationFactor {
            match_on: FactorMatch::Cas {
                cas: "124-38-9".into(),
            },
            value: 1.0,
            note: Some("CO2 reference".into()),
        };
        let s = serde_json::to_string(&f).unwrap();
        let back: CharacterizationFactor = serde_json::from_str(&s).unwrap();
        assert_eq!(f, back);
    }

    // ---- CasCompartment ---------------------------------------------

    fn so2_flow(compartment: Vec<&str>) -> FlowMeta {
        FlowMeta {
            id: "f_so2".into(),
            name: "Sulfur dioxide".into(),
            unit: Unit::new("kg"),
            compartment: compartment.into_iter().map(String::from).collect(),
            cas: Some("7446-09-5".into()),
            origin: FlowOrigin::Unspecified,
        }
    }

    #[test]
    fn cas_compartment_matches_exact_compartment() {
        let m = FactorMatch::CasCompartment {
            cas: "7446-09-5".into(),
            compartment: vec!["emission".into(), "air".into()],
        };
        assert!(m.matches(&so2_flow(vec!["emission", "air"])));
    }

    #[test]
    fn cas_compartment_matches_flow_with_deeper_path() {
        // Matcher on "emission/air" must match a flow in
        // "emission/air/urban" — prefix semantics, same as
        // NameAndCompartment. This is the whole point: EF 3.1 air
        // factors need to match both the bare "emission/air" and the
        // urban/rural subcompartment variants that appear in different
        // databases.
        let m = FactorMatch::CasCompartment {
            cas: "7446-09-5".into(),
            compartment: vec!["emission".into(), "air".into()],
        };
        assert!(m.matches(&so2_flow(vec!["emission", "air", "urban"])));
    }

    #[test]
    fn cas_compartment_rejects_different_compartment() {
        // SO2 to water is not an acidification flow — the CF must not
        // match. This is the load-bearing property the variant exists
        // to enforce.
        let m = FactorMatch::CasCompartment {
            cas: "7446-09-5".into(),
            compartment: vec!["emission".into(), "air".into()],
        };
        assert!(!m.matches(&so2_flow(vec!["emission", "water"])));
    }

    #[test]
    fn cas_compartment_rejects_flow_with_shorter_path() {
        // Flow compartment shorter than matcher compartment cannot be
        // a prefix of the matcher; no match.
        let m = FactorMatch::CasCompartment {
            cas: "7446-09-5".into(),
            compartment: vec!["emission".into(), "air".into(), "urban".into()],
        };
        assert!(!m.matches(&so2_flow(vec!["emission", "air"])));
    }

    #[test]
    fn cas_compartment_rejects_when_flow_has_no_cas() {
        let mut flow = so2_flow(vec!["emission", "air"]);
        flow.cas = None;
        let m = FactorMatch::CasCompartment {
            cas: "7446-09-5".into(),
            compartment: vec!["emission".into(), "air".into()],
        };
        assert!(!m.matches(&flow));
    }

    #[test]
    fn cas_compartment_rejects_wrong_cas() {
        // Right compartment, wrong substance. SO2 matcher must not
        // match an NH3 flow even in the same compartment.
        let m = FactorMatch::CasCompartment {
            cas: "7664-41-7".into(), // NH3
            compartment: vec!["emission".into(), "air".into()],
        };
        assert!(!m.matches(&so2_flow(vec!["emission", "air"])));
    }

    #[test]
    fn cas_compartment_is_origin_agnostic() {
        // Origin is explicitly outside this matcher's axis. A flow
        // with any FlowOrigin — fossil, biogenic, LULUC, unspecified
        // — matches on (CAS, compartment) alone.
        let m = FactorMatch::CasCompartment {
            cas: "74-82-8".into(),
            compartment: vec!["emission".into(), "air".into()],
        };
        let mut flow = ch4_flow(FlowOrigin::Fossil);
        flow.compartment = vec!["emission".into(), "air".into()];
        assert!(m.matches(&flow));
        flow.origin = FlowOrigin::Biogenic;
        assert!(m.matches(&flow));
        flow.origin = FlowOrigin::LandUseChange;
        assert!(m.matches(&flow));
        flow.origin = FlowOrigin::Unspecified;
        assert!(m.matches(&flow));
    }

    #[test]
    fn cas_compartment_empty_compartment_matches_any_compartment() {
        // Edge case: compartment = [] is a prefix of everything. The
        // variant reduces to plain CAS matching. Authorship should
        // prefer FactorMatch::Cas in this case; the variant just
        // doesn't blow up if someone writes it.
        let m = FactorMatch::CasCompartment {
            cas: "7446-09-5".into(),
            compartment: vec![],
        };
        assert!(m.matches(&so2_flow(vec!["emission", "air"])));
        assert!(m.matches(&so2_flow(vec!["emission", "water"])));
        assert!(m.matches(&so2_flow(vec![])));
    }

    #[test]
    fn cas_compartment_round_trips_through_json() {
        let f = CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: "7446-09-5".into(),
                compartment: vec!["emission".into(), "air".into()],
            },
            value: 1.31,
            note: Some("EF 3.1 Acidification — SO2 to air".into()),
        };
        let s = serde_json::to_string(&f).unwrap();
        assert!(s.contains("\"match\":\"cas_compartment\""));
        assert!(s.contains("\"cas\":\"7446-09-5\""));
        let back: CharacterizationFactor = serde_json::from_str(&s).unwrap();
        assert_eq!(f, back);
    }
}
