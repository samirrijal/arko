//! Standard-library presets for common LCA database licenses.
//!
//! **Disclaimer.** These are **conservative defaults** derived from
//! publicly-summarized license terms. They are *not* legal advice, and
//! they do not reproduce any database vendor's EULA. Operators MUST
//! verify against their signed license agreement before relying on
//! these presets for a specific deployment. The audit-log requirement
//! from `specs/license/README.md` applies to every publish action —
//! presets do not exempt the operator from it.
//!
//! Each preset returns a plain `LicenseTier`; callers should push it
//! onto their `Study::license_tiers` vector and point
//! `ProcessMeta::license_tier` at its index.

use arko_core::license::{DerivativeAction, DerivativeRule, DerivativeTrigger, LicenseTier};

/// Ecoinvent v3.x family — conservative default.
///
/// Publishable within the licensed workspace; inventory share and raw
/// export are blocked at the base-flag level. A `Warn` derivative rule
/// fires for any process contributing `≥ 1%` scaling so downstream
/// reports attach attribution as required by the ecoinvent EULA.
#[must_use]
pub fn ecoinvent_v3(version: &str) -> LicenseTier {
    LicenseTier {
        source: format!("ecoinvent-{version}"),
        allow_publish: true,
        allow_share: false,
        allow_export: false,
        derivative_rules: vec![DerivativeRule {
            when: DerivativeTrigger::ScalingGe { threshold: 0.01 },
            action: DerivativeAction::Warn,
            message: format!(
                "Result includes material contribution from ecoinvent-{version}; \
                 attribution required per EULA, derivative sharing restricted"
            ),
        }],
        expiry: None,
    }
}

/// Sphera / GaBi commercial datasets — strict default.
///
/// All three base flags are `false`; any contribution triggers a
/// `Block` rule. This matches the common Sphera commercial-license
/// default where external publication requires a re-licensing step.
#[must_use]
pub fn sphera_strict(version: &str) -> LicenseTier {
    LicenseTier {
        source: format!("sphera-{version}"),
        allow_publish: false,
        allow_share: false,
        allow_export: false,
        derivative_rules: vec![DerivativeRule {
            when: DerivativeTrigger::Always,
            action: DerivativeAction::Block,
            message: format!(
                "Sphera / GaBi dataset version {version} — external publication \
                 requires re-licensing with the vendor"
            ),
        }],
        expiry: None,
    }
}

/// Open / CC-BY-style dataset — fully permissive with an attribution `Warn`.
///
/// Use for datasets published under CC-BY-4.0 or similar open licenses
/// that require attribution but permit publication, sharing, and export.
#[must_use]
pub fn open_cc_by(source: &str) -> LicenseTier {
    LicenseTier {
        source: source.to_owned(),
        allow_publish: true,
        allow_share: true,
        allow_export: true,
        derivative_rules: vec![DerivativeRule {
            when: DerivativeTrigger::ScalingGe { threshold: 0.01 },
            action: DerivativeAction::Warn,
            message: format!("Attribution required (CC-BY 4.0): {source}"),
        }],
        expiry: None,
    }
}

/// User-authored / custom process — no third-party licensing obligations.
///
/// Thin wrapper around `LicenseTier::permissive` for discoverability:
/// new users finding `presets::*` should see this alongside the
/// commercial presets.
#[must_use]
pub fn custom_user() -> LicenseTier {
    LicenseTier::permissive("custom-user")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecoinvent_v3_has_warn_rule_with_threshold() {
        let t = ecoinvent_v3("3.11");
        assert_eq!(t.source, "ecoinvent-3.11");
        assert!(t.allow_publish);
        assert!(!t.allow_share);
        assert!(!t.allow_export);
        assert_eq!(t.derivative_rules.len(), 1);
        assert!(matches!(
            t.derivative_rules[0].action,
            DerivativeAction::Warn
        ));
        assert!(matches!(
            t.derivative_rules[0].when,
            DerivativeTrigger::ScalingGe { threshold } if (threshold - 0.01).abs() < 1e-12
        ));
    }

    #[test]
    fn sphera_strict_blocks_everything() {
        let t = sphera_strict("2024.1");
        assert!(!t.allow_publish);
        assert!(!t.allow_share);
        assert!(!t.allow_export);
        assert!(matches!(
            t.derivative_rules[0].action,
            DerivativeAction::Block
        ));
    }

    #[test]
    fn open_cc_by_is_fully_permissive() {
        let t = open_cc_by("openlca-2.0");
        assert!(t.allow_publish && t.allow_share && t.allow_export);
    }

    #[test]
    fn custom_user_has_no_rules() {
        let t = custom_user();
        assert_eq!(t.source, "custom-user");
        assert!(t.derivative_rules.is_empty());
    }
}
