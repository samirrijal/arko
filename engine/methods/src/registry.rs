//! Method registry — keyed by `(MethodRef.id, MethodRef.version)`.

use crate::method::ImpactMethod;
use arko_core::study::MethodRef;
use std::collections::BTreeMap;
use thiserror::Error;

/// Returned by [`MethodRegistry::lookup`] when a `MethodRef` has no
/// matching entry.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("method `{id}` version `{version}` is not registered")]
pub struct MethodNotFound {
    pub id: String,
    pub version: String,
}

/// A keyed collection of impact methods.
///
/// Uses a `BTreeMap` (not `HashMap`) for deterministic iteration order
/// — relevant for tests, provenance, and the §7.1 determinism
/// contract (snapshotting the full registry should produce a
/// byte-stable blob).
#[derive(Debug, Clone, Default)]
pub struct MethodRegistry {
    methods: BTreeMap<(String, String), ImpactMethod>,
}

impl MethodRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The "batteries-included" registry: populated with the preset
    /// methods Arko ships by default.
    ///
    /// Currently ships:
    /// - `("ipcc-ar6-gwp100", "1")` — **recommended default** for new
    ///   climate-only studies.
    /// - `("ipcc-ar5-gwp100", "1")` — legacy-verification parity for
    ///   historical EPDs authored under AR5 (with climate-carbon
    ///   feedback).
    /// - `("ef-3.1", "1")` — first non-climate preset; ships the 7
    ///   EN 15804+A2 core emission indicators (CC, OD, POCP, AC,
    ///   EU-fw, EU-m, EU-t). Required for shippable EPDs against the
    ///   construction-products PCR.
    /// - `("cml-ia-baseline", "4.8")` — CML-IA baseline (Leiden,
    ///   August 2016), EN 15804+A2-aligned subset (7 categories).
    ///   Legacy-EPD verification + side-by-side with EF 3.1. Note:
    ///   GWP100 values differ from `ipcc-ar5-gwp100` by design —
    ///   CML uses IPCC 2013 *without* climate-carbon feedback.
    #[must_use]
    pub fn standard() -> Self {
        let mut r = Self::new();
        r.register(crate::standard::ipcc_ar6_gwp100());
        r.register(crate::standard::ipcc_ar5_gwp100());
        r.register(crate::ef_31::ef_31());
        r.register(crate::cml_ia::cml_ia());
        r
    }

    /// Insert or replace an entry. Returns the previous value if one
    /// existed (useful for tests that want to assert additive behaviour).
    pub fn register(&mut self, method: ImpactMethod) -> Option<ImpactMethod> {
        let key = (method.id.clone(), method.version.clone());
        self.methods.insert(key, method)
    }

    /// Look up by `MethodRef`. Errors with `MethodNotFound` if the
    /// `(id, version)` pair is absent.
    pub fn lookup(&self, r: &MethodRef) -> Result<&ImpactMethod, MethodNotFound> {
        let key = (r.id.clone(), r.version.clone());
        self.methods.get(&key).ok_or_else(|| MethodNotFound {
            id: r.id.clone(),
            version: r.version.clone(),
        })
    }

    /// All registered methods, deterministically ordered by
    /// `(id, version)`.
    pub fn iter(&self) -> impl Iterator<Item = &ImpactMethod> {
        self.methods.values()
    }

    /// Number of methods in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.methods.len()
    }

    /// `true` iff the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.methods.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::{ImpactCategory, ImpactMethod};

    fn stub_method(id: &str, v: &str) -> ImpactMethod {
        ImpactMethod {
            id: id.into(),
            version: v.into(),
            name: format!("{id} {v}"),
            categories: vec![ImpactCategory {
                id: "x".into(),
                name: "x".into(),
                unit: "kg".into(),
                factors: vec![],
            }],
        }
    }

    #[test]
    fn empty_registry_errors_on_lookup() {
        let r = MethodRegistry::new();
        assert!(r.is_empty());
        let err = r
            .lookup(&MethodRef {
                id: "missing".into(),
                version: "0".into(),
            })
            .unwrap_err();
        assert_eq!(err.id, "missing");
    }

    #[test]
    fn register_then_lookup_succeeds() {
        let mut r = MethodRegistry::new();
        let prev = r.register(stub_method("ipcc-gwp", "ar6"));
        assert!(prev.is_none());
        assert_eq!(r.len(), 1);
        let got = r
            .lookup(&MethodRef {
                id: "ipcc-gwp".into(),
                version: "ar6".into(),
            })
            .unwrap();
        assert_eq!(got.name, "ipcc-gwp ar6");
    }

    #[test]
    fn register_returns_previous_on_duplicate_key() {
        let mut r = MethodRegistry::new();
        r.register(stub_method("ipcc-gwp", "ar6"));
        let mut m2 = stub_method("ipcc-gwp", "ar6");
        m2.name = "override".into();
        let prev = r.register(m2);
        assert_eq!(prev.unwrap().name, "ipcc-gwp ar6");
    }

    #[test]
    fn iter_is_deterministic_alphabetical() {
        let mut r = MethodRegistry::new();
        r.register(stub_method("b", "1"));
        r.register(stub_method("a", "2"));
        r.register(stub_method("a", "1"));
        let ids: Vec<_> = r
            .iter()
            .map(|m| format!("{}-{}", m.id, m.version))
            .collect();
        assert_eq!(ids, vec!["a-1", "a-2", "b-1"]);
    }

    #[test]
    fn standard_registry_has_ipcc_ar6_gwp100() {
        let r = MethodRegistry::standard();
        let m = r
            .lookup(&MethodRef {
                id: "ipcc-ar6-gwp100".into(),
                version: "1".into(),
            })
            .unwrap();
        assert_eq!(m.categories.len(), 1);
        assert_eq!(m.categories[0].id, "gwp100");
    }

    #[test]
    fn standard_registry_has_ipcc_ar5_gwp100() {
        let r = MethodRegistry::standard();
        let m = r
            .lookup(&MethodRef {
                id: "ipcc-ar5-gwp100".into(),
                version: "1".into(),
            })
            .unwrap();
        assert_eq!(m.categories.len(), 1);
        assert_eq!(m.categories[0].id, "gwp100");
    }

    #[test]
    fn standard_registry_ships_ar5_ar6_ef31_and_cml_ia() {
        let r = MethodRegistry::standard();
        assert_eq!(
            r.len(),
            4,
            "standard registry ships AR6 (default) + AR5 (legacy parity, with feedback) + EF 3.1 (EN 15804+A2 core) + CML-IA baseline 4.8 (legacy-EPD verification, GWP without feedback)"
        );
    }

    #[test]
    fn standard_registry_has_ef_31() {
        let r = MethodRegistry::standard();
        let m = r
            .lookup(&MethodRef {
                id: "ef-3.1".into(),
                version: "1".into(),
            })
            .unwrap();
        assert_eq!(
            m.categories.len(),
            7,
            "EF 3.1 V1 ships the 7 EN 15804+A2 core emission indicators"
        );
    }

    #[test]
    fn standard_registry_has_cml_ia_baseline() {
        let r = MethodRegistry::standard();
        let m = r
            .lookup(&MethodRef {
                id: "cml-ia-baseline".into(),
                version: "4.8".into(),
            })
            .unwrap();
        assert_eq!(
            m.categories.len(),
            7,
            "CML-IA baseline V1 ships 7 EN 15804+A2-aligned categories"
        );
        // Spot-check the GWP100 category exists with the expected id.
        let gwp = m.categories.iter().find(|c| c.id == "gwp100");
        assert!(gwp.is_some(), "CML-IA must include `gwp100` category");
        assert_eq!(gwp.unwrap().unit, "kg CO2-eq");
    }
}
