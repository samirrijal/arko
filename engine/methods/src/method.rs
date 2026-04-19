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
        assert!(!m.matches(&ch4_flow(FlowOrigin::NonFossil)));
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
        assert!(m.matches(&ch4_flow(FlowOrigin::NonFossil)));
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
}
