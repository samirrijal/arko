//! Rule-firing walker — evaluates `DerivativeRule`s against a scaling
//! vector. Spec §11.3.
//!
//! For every process `j` with `|s[j]| > EPS_PRESENCE`, we look up its
//! license tier and evaluate every rule's trigger. Firing order is the
//! natural one: outer loop over processes in index order, inner loop
//! over `derivative_rules` in declaration order. This is the same
//! canonicalization used by `pipeline::compute` for the §11.2 join, so
//! the fired-rules list is byte-stable across runs (spec §7.1).

use arko_core::{
    license::{DerivativeAction, DerivativeTrigger},
    pipeline::EPS_PRESENCE,
    study::Study,
};
use serde::{Deserialize, Serialize};

/// A single instance of a `DerivativeRule` firing during a walk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FiredRule {
    /// Process index in `Study::processes`.
    pub process_index: u32,
    /// Source of the tier that owned the rule (e.g., `"ecoinvent-3.11"`).
    pub tier_source: String,
    /// What the rule asks the caller to do.
    pub action: DerivativeAction,
    /// Human-readable message from the rule definition.
    pub message: String,
    /// The scaling factor `s[j]` that caused the trigger to fire.
    pub scaling_factor: f64,
}

/// Evaluate every `DerivativeRule` of every contributing process against
/// `scaling` and return the list of firings.
///
/// `scaling.len()` must equal `study.processes.len()`; processes beyond
/// the end of the shorter of the two are silently skipped so this helper
/// is robust against shape drift during in-tool experimentation.
#[must_use]
pub fn fire_rules(study: &Study, scaling: &[f64]) -> Vec<FiredRule> {
    let mut fired = Vec::new();
    let n = scaling.len().min(study.processes.len());
    for j in 0..n {
        let s_j = scaling[j];
        if s_j.abs() <= EPS_PRESENCE {
            continue;
        }
        let tier_idx = study.processes[j].license_tier.0 as usize;
        let Some(tier) = study.license_tiers.get(tier_idx) else {
            continue;
        };
        for rule in &tier.derivative_rules {
            if trigger_fires(&rule.when, s_j) {
                fired.push(FiredRule {
                    process_index: u32::try_from(j).unwrap_or(u32::MAX),
                    tier_source: tier.source.clone(),
                    action: rule.action,
                    message: rule.message.clone(),
                    scaling_factor: s_j,
                });
            }
        }
    }
    fired
}

fn trigger_fires(trigger: &DerivativeTrigger, scaling: f64) -> bool {
    match trigger {
        DerivativeTrigger::Always => true,
        DerivativeTrigger::ScalingGe { threshold } => scaling.abs() >= *threshold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arko_core::license::{DerivativeRule, DerivativeTrigger};

    #[test]
    fn scaling_ge_fires_at_threshold() {
        let t = DerivativeTrigger::ScalingGe { threshold: 0.5 };
        assert!(trigger_fires(&t, 0.5));
        assert!(trigger_fires(&t, 0.51));
        assert!(!trigger_fires(&t, 0.49));
    }

    #[test]
    fn scaling_ge_uses_absolute_value() {
        let t = DerivativeTrigger::ScalingGe { threshold: 0.5 };
        assert!(trigger_fires(&t, -0.6));
        assert!(!trigger_fires(&t, -0.3));
    }

    #[test]
    fn always_fires_unconditionally() {
        assert!(trigger_fires(&DerivativeTrigger::Always, 0.0));
        assert!(trigger_fires(&DerivativeTrigger::Always, 1e-20));
        assert!(trigger_fires(&DerivativeTrigger::Always, -1e20));
    }

    #[test]
    fn fired_rule_roundtrips_through_json() {
        let fr = FiredRule {
            process_index: 3,
            tier_source: "ecoinvent-3.11".into(),
            action: DerivativeAction::Warn,
            message: "attribution required".into(),
            scaling_factor: 1.25,
        };
        let _ = DerivativeRule {
            when: DerivativeTrigger::ScalingGe { threshold: 0.01 },
            action: DerivativeAction::Warn,
            message: "attribution required".into(),
        };
        let j = serde_json::to_string(&fr).unwrap();
        let back: FiredRule = serde_json::from_str(&j).unwrap();
        assert_eq!(fr, back);
    }
}
