//! Publish-time authorization — the top-level entry point of this crate.
//!
//! Given a computed result and a user intent (publish, share, or export),
//! combine three signals into one [`Authorization`] decision:
//!
//! 1. **Base flag.** `EffectiveRestriction::allow_{publish,share,export}`
//!    — the §11.2 join. A `false` here is unconditional: no fired rule
//!    can upgrade it.
//! 2. **Fired derivative rules.** The most severe rule outcome wins
//!    (`Blocked` > `Watermark` > `Warn` > `Allowed`).
//! 3. **Expiry.** Any contributing tier whose `expiry` has passed forces
//!    an unconditional `Blocked` and surfaces the source in
//!    `expired_sources` so the UI can route the user through re-consent.
//!
//! The fired-rules list is always returned verbatim — even on `Blocked`
//! — so callers can surface every rule message to the user and record
//! them in the audit log required by `specs/license/README.md`.

use arko_core::{
    license::DerivativeAction,
    pipeline::{Computed, EPS_PRESENCE},
    provenance::EffectiveRestriction,
    study::Study,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::fire::{fire_rules, FiredRule};

/// What the caller wants to do with a result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// External publication (report, EPD, marketing material).
    Publish,
    /// Sharing the inventory itself to another workspace.
    Share,
    /// Exporting the raw data as an open format (ecospold2, EPDX, …).
    Export,
}

/// Authorization outcome, ordered by severity.
///
/// `Ord` is deliberately implemented so `max` gives the worst outcome
/// across a set of fired rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Allowed,
    Warn,
    Watermark,
    Blocked,
}

impl Outcome {
    fn severity(self) -> u8 {
        match self {
            Self::Allowed => 0,
            Self::Warn => 1,
            Self::Watermark => 2,
            Self::Blocked => 3,
        }
    }

    /// `true` iff the outcome permits the action (possibly with caveats).
    #[must_use]
    pub fn is_allowed(self) -> bool {
        !matches!(self, Self::Blocked)
    }
}

impl PartialOrd for Outcome {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Outcome {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.severity().cmp(&other.severity())
    }
}

/// The combined decision returned by [`authorize`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Authorization {
    pub intent: Intent,
    pub outcome: Outcome,
    /// Every rule that fired during the walk, in process-index then
    /// rule-declaration order.
    pub fired: Vec<FiredRule>,
    /// Tier sources that contributed to a `Blocked` outcome (de-duplicated,
    /// insertion-ordered). Empty unless `outcome == Blocked`.
    pub blocking_sources: Vec<String>,
    /// Tier sources whose `expiry` had passed at `now` and therefore
    /// required re-consent (de-duplicated, insertion-ordered).
    pub expired_sources: Vec<String>,
}

/// Decide whether `computed` may be used for `intent`, as of `now`.
///
/// `now` is caller-provided rather than read from the system clock so
/// tests are deterministic and a result's provenance `computed_at` can
/// drive replay authorization.
#[must_use]
pub fn authorize(
    intent: Intent,
    study: &Study,
    computed: &Computed,
    now: DateTime<Utc>,
) -> Authorization {
    let mut outcome = Outcome::Allowed;
    let mut blocking: Vec<String> = Vec::new();
    let mut expired: Vec<String> = Vec::new();

    // 1) Base flag.
    if !intent_allowed(&computed.effective_restriction, intent) {
        outcome = Outcome::Blocked;
        for src in &computed.effective_restriction.sources {
            push_unique(&mut blocking, src);
        }
    }

    // 2) Fired derivative rules.
    let fired = fire_rules(study, &computed.scaling);
    for rule in &fired {
        let rule_outcome = action_to_outcome(rule.action);
        if rule_outcome > outcome {
            outcome = rule_outcome;
        }
        if matches!(rule.action, DerivativeAction::Block) {
            push_unique(&mut blocking, &rule.tier_source);
        }
    }

    // 3) Expiries. Any expired contributing tier blocks unconditionally.
    let n = computed.scaling.len().min(study.processes.len());
    for j in 0..n {
        if computed.scaling[j].abs() <= EPS_PRESENCE {
            continue;
        }
        let tier_idx = study.processes[j].license_tier.0 as usize;
        let Some(tier) = study.license_tiers.get(tier_idx) else {
            continue;
        };
        if let Some(exp) = tier.expiry {
            if exp < now {
                outcome = Outcome::Blocked;
                push_unique(&mut expired, &tier.source);
                push_unique(&mut blocking, &tier.source);
            }
        }
    }

    Authorization {
        intent,
        outcome,
        fired,
        blocking_sources: blocking,
        expired_sources: expired,
    }
}

fn intent_allowed(r: &EffectiveRestriction, intent: Intent) -> bool {
    match intent {
        Intent::Publish => r.allow_publish,
        Intent::Share => r.allow_share,
        Intent::Export => r.allow_export,
    }
}

fn action_to_outcome(action: DerivativeAction) -> Outcome {
    match action {
        DerivativeAction::Warn => Outcome::Warn,
        DerivativeAction::Watermark => Outcome::Watermark,
        DerivativeAction::Block => Outcome::Blocked,
    }
}

fn push_unique(v: &mut Vec<String>, s: &str) {
    if !v.iter().any(|x| x == s) {
        v.push(s.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_ordering_matches_severity() {
        assert!(Outcome::Blocked > Outcome::Watermark);
        assert!(Outcome::Watermark > Outcome::Warn);
        assert!(Outcome::Warn > Outcome::Allowed);
    }

    #[test]
    fn outcome_is_allowed_only_blocked_false() {
        assert!(Outcome::Allowed.is_allowed());
        assert!(Outcome::Warn.is_allowed());
        assert!(Outcome::Watermark.is_allowed());
        assert!(!Outcome::Blocked.is_allowed());
    }
}
