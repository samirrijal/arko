//! Integration tests: build a small solve, run `arko-license::authorize`
//! against it, and check that every branch of the §11 evaluator
//! (base flag, derivative rules, expiry, intent dispatch, preset
//! construction) behaves per spec.

use arko_core::{
    compute,
    license::{DerivativeAction, DerivativeRule, DerivativeTrigger, LicenseTier},
    matrices::{SparseMatrix, SparseVector},
    meta::{FlowMeta, ImpactMeta, LicenseTierRef, ProcessMeta},
    sign::SignConvention,
    study::{MethodRef, Study},
    units::Unit,
};
use arko_license::{authorize, fire_rules, presets, Intent, Outcome};
use arko_solvers_dense::DenseLuSolver;
use chrono::{TimeZone, Utc};
use sprs::TriMat;

/// Build a degenerate 2-process study. The technosphere is the identity
/// so `s = f`, which means whatever `f` the caller passes becomes the
/// scaling vector directly — convenient for driving firing thresholds.
fn identity_study(
    tiers: Vec<LicenseTier>,
    p0_tier: u32,
    p1_tier: u32,
    f: Vec<(usize, f64)>,
) -> Study {
    let mut a = TriMat::new((2, 2));
    a.add_triplet(0, 0, 1.0);
    a.add_triplet(1, 1, 1.0);
    let technosphere: SparseMatrix = a.to_csr();

    let mut b = TriMat::new((1, 2));
    b.add_triplet(0, 0, 1.0);
    b.add_triplet(0, 1, 1.0);
    let biosphere: SparseMatrix = b.to_csr();

    let mut c = TriMat::new((1, 1));
    c.add_triplet(0, 0, 1.0);
    let characterization: SparseMatrix = c.to_csr();

    let (idx, vals): (Vec<_>, Vec<_>) = f.into_iter().unzip();
    let functional_unit = SparseVector::new(2, idx, vals);

    Study {
        technosphere,
        biosphere,
        characterization,
        functional_unit,
        processes: vec![
            ProcessMeta {
                id: "p0".into(),
                name: "p0".into(),
                reference_product: "a".into(),
                reference_unit: Unit::new("kg"),
                allocation: None,
                license_tier: LicenseTierRef(p0_tier),
                geography: None,
            },
            ProcessMeta {
                id: "p1".into(),
                name: "p1".into(),
                reference_product: "b".into(),
                reference_unit: Unit::new("kg"),
                allocation: None,
                license_tier: LicenseTierRef(p1_tier),
                geography: None,
            },
        ],
        flows: vec![FlowMeta {
            id: "f0".into(),
            name: "f0".into(),
            unit: Unit::new("kg"),
            compartment: vec![],
            cas: None,
            origin: Default::default(),
        }],
        impacts: vec![ImpactMeta {
            id: "i0".into(),
            name: "i0".into(),
            unit: Unit::new("kg"),
        }],
        parameters: Vec::new(),
        license_tiers: tiers,
        method: MethodRef {
            id: "m".into(),
            version: "0".into(),
        },
        sign_convention: SignConvention::ProducerPositive,
    }
}

#[test]
fn permissive_study_publishes_freely() {
    let study = identity_study(
        vec![LicenseTier::permissive("custom-user")],
        0,
        0,
        vec![(0, 1.0), (1, 0.5)],
    );
    let computed = compute(&study, &DenseLuSolver).unwrap();
    let auth = authorize(Intent::Publish, &study, &computed, Utc::now());
    assert_eq!(auth.outcome, Outcome::Allowed);
    assert!(auth.fired.is_empty());
    assert!(auth.blocking_sources.is_empty());
    assert!(auth.expired_sources.is_empty());
}

#[test]
fn base_restriction_blocks_export() {
    let mut tier = LicenseTier::permissive("sphera-2024");
    tier.allow_export = false;
    let study = identity_study(vec![tier], 0, 0, vec![(0, 1.0), (1, 0.5)]);
    let computed = compute(&study, &DenseLuSolver).unwrap();
    let auth = authorize(Intent::Export, &study, &computed, Utc::now());
    assert_eq!(auth.outcome, Outcome::Blocked);
    assert!(auth.blocking_sources.iter().any(|s| s == "sphera-2024"));
}

#[test]
fn base_restriction_blocks_share_but_not_publish() {
    let mut tier = LicenseTier::permissive("ecoinvent-3.11");
    tier.allow_share = false;
    let study = identity_study(vec![tier], 0, 0, vec![(0, 1.0), (1, 0.5)]);
    let computed = compute(&study, &DenseLuSolver).unwrap();

    let share = authorize(Intent::Share, &study, &computed, Utc::now());
    assert_eq!(share.outcome, Outcome::Blocked);

    let publish = authorize(Intent::Publish, &study, &computed, Utc::now());
    assert_eq!(publish.outcome, Outcome::Allowed);
}

#[test]
fn rule_below_threshold_does_not_fire() {
    let mut tier = LicenseTier::permissive("ecoinvent-3.11");
    tier.derivative_rules.push(DerivativeRule {
        when: DerivativeTrigger::ScalingGe { threshold: 2.0 },
        action: DerivativeAction::Warn,
        message: "major contribution".into(),
    });
    let study = identity_study(vec![tier], 0, 0, vec![(0, 1.0), (1, 0.5)]);
    let computed = compute(&study, &DenseLuSolver).unwrap();
    let fired = fire_rules(&study, &computed.scaling);
    assert!(fired.is_empty(), "no process has |s| >= 2.0");
}

#[test]
fn rule_above_threshold_warns_and_returns_messages() {
    let mut tier = LicenseTier::permissive("ecoinvent-3.11");
    tier.derivative_rules.push(DerivativeRule {
        when: DerivativeTrigger::ScalingGe { threshold: 0.01 },
        action: DerivativeAction::Warn,
        message: "attribution required".into(),
    });
    let study = identity_study(vec![tier], 0, 0, vec![(0, 1.0), (1, 0.5)]);
    let computed = compute(&study, &DenseLuSolver).unwrap();
    let auth = authorize(Intent::Publish, &study, &computed, Utc::now());
    assert_eq!(auth.outcome, Outcome::Warn);
    assert_eq!(auth.fired.len(), 2, "both processes fire the same rule");
    assert!(auth.fired.iter().all(|r| r.message == "attribution required"));
    assert_eq!(auth.fired[0].process_index, 0);
    assert_eq!(auth.fired[1].process_index, 1);
    assert!(auth.blocking_sources.is_empty());
}

#[test]
fn block_action_trumps_warn_action() {
    let mut warn_tier = LicenseTier::permissive("ecoinvent-3.11");
    warn_tier.derivative_rules.push(DerivativeRule {
        when: DerivativeTrigger::Always,
        action: DerivativeAction::Warn,
        message: "w".into(),
    });
    let mut block_tier = LicenseTier::permissive("sphera");
    block_tier.derivative_rules.push(DerivativeRule {
        when: DerivativeTrigger::Always,
        action: DerivativeAction::Block,
        message: "b".into(),
    });
    let study = identity_study(
        vec![warn_tier, block_tier],
        0,
        1,
        vec![(0, 1.0), (1, 0.5)],
    );
    let computed = compute(&study, &DenseLuSolver).unwrap();
    let auth = authorize(Intent::Publish, &study, &computed, Utc::now());
    assert_eq!(auth.outcome, Outcome::Blocked);
    assert_eq!(auth.blocking_sources, vec!["sphera".to_string()]);
    assert_eq!(auth.fired.len(), 2);
}

#[test]
fn watermark_sits_between_warn_and_block() {
    let mut tier = LicenseTier::permissive("ecoinvent-3.11");
    tier.derivative_rules.push(DerivativeRule {
        when: DerivativeTrigger::Always,
        action: DerivativeAction::Watermark,
        message: "watermark".into(),
    });
    tier.derivative_rules.push(DerivativeRule {
        when: DerivativeTrigger::Always,
        action: DerivativeAction::Warn,
        message: "warn".into(),
    });
    let study = identity_study(vec![tier], 0, 0, vec![(0, 1.0), (1, 0.5)]);
    let computed = compute(&study, &DenseLuSolver).unwrap();
    let auth = authorize(Intent::Publish, &study, &computed, Utc::now());
    assert_eq!(
        auth.outcome,
        Outcome::Watermark,
        "worst-outcome semantics: Watermark > Warn"
    );
    // Every rule fires for every contributing process → 2 processes × 2 rules = 4.
    assert_eq!(auth.fired.len(), 4);
}

#[test]
fn expired_tier_blocks_even_when_all_flags_permissive() {
    let mut tier = LicenseTier::permissive("ecoinvent-3.11");
    tier.expiry = Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap());
    let study = identity_study(vec![tier], 0, 0, vec![(0, 1.0), (1, 0.5)]);
    let computed = compute(&study, &DenseLuSolver).unwrap();

    // `now` is well after expiry.
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let auth = authorize(Intent::Publish, &study, &computed, now);
    assert_eq!(auth.outcome, Outcome::Blocked);
    assert_eq!(auth.expired_sources, vec!["ecoinvent-3.11".to_string()]);
    assert!(auth
        .blocking_sources
        .iter()
        .any(|s| s == "ecoinvent-3.11"));
}

#[test]
fn expiry_check_uses_caller_provided_now_not_system_clock() {
    let mut tier = LicenseTier::permissive("ecoinvent-3.11");
    let expiry = Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap();
    tier.expiry = Some(expiry);
    let study = identity_study(vec![tier], 0, 0, vec![(0, 1.0), (1, 0.5)]);
    let computed = compute(&study, &DenseLuSolver).unwrap();

    // Before expiry — allowed.
    let before = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(
        authorize(Intent::Publish, &study, &computed, before).outcome,
        Outcome::Allowed
    );

    // After expiry — blocked.
    let after = Utc.with_ymd_and_hms(2031, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(
        authorize(Intent::Publish, &study, &computed, after).outcome,
        Outcome::Blocked
    );
}

#[test]
fn zero_contribution_process_is_ignored() {
    // Only process 0 contributes; process 1 has s=0 because f=[1, 0].
    let mut tier = LicenseTier::permissive("sphera");
    tier.derivative_rules.push(DerivativeRule {
        when: DerivativeTrigger::Always,
        action: DerivativeAction::Block,
        message: "would block if it fired".into(),
    });
    let mut permissive = LicenseTier::permissive("custom-user");
    permissive.derivative_rules = Vec::new();
    // Assign the blocking tier to p1, which has zero scaling.
    let study = identity_study(vec![permissive, tier], 0, 1, vec![(0, 1.0)]);
    let computed = compute(&study, &DenseLuSolver).unwrap();
    let auth = authorize(Intent::Publish, &study, &computed, Utc::now());
    assert_eq!(
        auth.outcome,
        Outcome::Allowed,
        "p1 is below EPS_PRESENCE so its rule must not fire"
    );
    assert!(auth.fired.is_empty());
}

#[test]
fn ecoinvent_preset_end_to_end_warns_on_real_solve() {
    let tier = presets::ecoinvent_v3("3.11");
    let study = identity_study(vec![tier], 0, 0, vec![(0, 1.0), (1, 0.5)]);
    let computed = compute(&study, &DenseLuSolver).unwrap();

    // Publish is allowed (base flag) but should Warn because both
    // processes exceed the 1% threshold.
    let auth = authorize(Intent::Publish, &study, &computed, Utc::now());
    assert_eq!(auth.outcome, Outcome::Warn);

    // Export is blocked at the base-flag level.
    let auth_export = authorize(Intent::Export, &study, &computed, Utc::now());
    assert_eq!(auth_export.outcome, Outcome::Blocked);
}

#[test]
fn sphera_preset_blocks_every_intent() {
    let tier = presets::sphera_strict("2024.1");
    let study = identity_study(vec![tier], 0, 0, vec![(0, 1.0), (1, 0.5)]);
    let computed = compute(&study, &DenseLuSolver).unwrap();
    for intent in [Intent::Publish, Intent::Share, Intent::Export] {
        let auth = authorize(intent, &study, &computed, Utc::now());
        assert_eq!(auth.outcome, Outcome::Blocked, "intent = {intent:?}");
    }
}

#[test]
fn authorization_roundtrips_through_json() {
    let study = identity_study(
        vec![presets::ecoinvent_v3("3.11")],
        0,
        0,
        vec![(0, 1.0), (1, 0.5)],
    );
    let computed = compute(&study, &DenseLuSolver).unwrap();
    let auth = authorize(Intent::Publish, &study, &computed, Utc::now());

    let j = serde_json::to_string(&auth).unwrap();
    let back: arko_license::Authorization = serde_json::from_str(&j).unwrap();
    assert_eq!(auth, back);
}
