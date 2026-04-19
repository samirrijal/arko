//! Parameter expression evaluator — implements `specs/calc/v0.1.md` §5.
//!
//! Responsibilities:
//! - Extract the dependency set of each `Parameter::expression`.
//! - Topologically sort the dependency DAG; reject cycles with
//!   `EngineError::ParamCycle` per spec §5.2.
//! - Evaluate in dependency order into a `HashMap<ParameterId, f64>`,
//!   rejecting non-finite results with `EngineError::ParamNonfinite`.
//!
//! Intentionally **not** here (v0.1):
//! - Partial-derivative computation (deferred to §9.3 / §10 work).
//! - Parsing expression strings (the AST is constructed programmatically
//!   or via serde; a human-writable parser is a v0.2 feature).
//! - Unit arithmetic across operations — the evaluator is dimensionless;
//!   unit checking belongs in `arko-validation`.

use arko_core::{EngineError, Expression, Parameter, ParameterId};
use std::collections::{HashMap, VecDeque};

/// Evaluate every parameter in topological order. Returns a map from
/// `ParameterId` to the parameter's resolved `f64` value.
pub fn evaluate(params: &[Parameter]) -> Result<HashMap<ParameterId, f64>, EngineError> {
    let mut by_id: HashMap<&ParameterId, &Parameter> = HashMap::with_capacity(params.len());
    for p in params {
        if by_id.insert(&p.id, p).is_some() {
            return Err(EngineError::Internal(format!(
                "duplicate parameter id: {}",
                p.id.0
            )));
        }
    }

    // Build dependency graph: in_degree and adjacency (dep -> [dependents]).
    let mut in_degree: HashMap<ParameterId, usize> =
        params.iter().map(|p| (p.id.clone(), 0_usize)).collect();
    let mut adjacency: HashMap<ParameterId, Vec<ParameterId>> = HashMap::new();

    for p in params {
        let mut deps = Vec::new();
        walk_deps(&p.expression, &mut deps);
        for dep_id in &deps {
            if !by_id.contains_key(dep_id) {
                return Err(EngineError::ParamUnresolved(
                    p.id.0.clone(),
                    dep_id.0.clone(),
                ));
            }
            *in_degree
                .get_mut(&p.id)
                .expect("in_degree seeded for all ids") += 1;
            adjacency
                .entry(dep_id.clone())
                .or_default()
                .push(p.id.clone());
        }
    }

    // Kahn's algorithm — deterministic insertion order for repeatable
    // evaluation (spec §7 determinism contract propagates down to params).
    let mut queue: VecDeque<ParameterId> = params
        .iter()
        .filter(|p| in_degree.get(&p.id).copied().unwrap_or(0) == 0)
        .map(|p| p.id.clone())
        .collect();

    let mut env: HashMap<ParameterId, f64> = HashMap::with_capacity(params.len());
    let mut processed = 0_usize;

    while let Some(id) = queue.pop_front() {
        let p = by_id[&id];
        let v = eval_expr(&p.expression, &env, &p.id)?;
        env.insert(id.clone(), v);
        processed += 1;

        if let Some(succs) = adjacency.remove(&id) {
            for s in succs {
                let d = in_degree
                    .get_mut(&s)
                    .expect("successor must be in in_degree");
                *d -= 1;
                if *d == 0 {
                    queue.push_back(s);
                }
            }
        }
    }

    if processed != params.len() {
        // Pick the lexicographically-first id still with in_degree > 0 as
        // the cycle witness — deterministic for diagnostic stability.
        let mut remaining: Vec<&ParameterId> = in_degree
            .iter()
            .filter(|(_, &d)| d > 0)
            .map(|(id, _)| id)
            .collect();
        remaining.sort_by(|a, b| a.0.cmp(&b.0));
        let witness = remaining
            .first()
            .map_or_else(|| "<unknown>".into(), |id| id.0.clone());
        return Err(EngineError::ParamCycle(witness));
    }

    Ok(env)
}

/// Evaluate a single expression against an already-populated `env`.
/// Public because callers may want to re-evaluate matrix-entry
/// expressions that share the parameter namespace without redoing the
/// full DAG walk.
pub fn eval_expr<S: std::hash::BuildHasher>(
    expr: &Expression,
    env: &HashMap<ParameterId, f64, S>,
    owner_hint: &ParameterId,
) -> Result<f64, EngineError> {
    let v = match expr {
        Expression::Const(c) => *c,
        Expression::Var(id) => *env
            .get(id)
            .ok_or_else(|| EngineError::ParamUnresolved(owner_hint.0.clone(), id.0.clone()))?,
        Expression::Add(l, r) => eval_expr(l, env, owner_hint)? + eval_expr(r, env, owner_hint)?,
        Expression::Sub(l, r) => eval_expr(l, env, owner_hint)? - eval_expr(r, env, owner_hint)?,
        Expression::Mul(l, r) => eval_expr(l, env, owner_hint)? * eval_expr(r, env, owner_hint)?,
        Expression::Div(l, r) => eval_expr(l, env, owner_hint)? / eval_expr(r, env, owner_hint)?,
        Expression::Pow(b, e) => eval_expr(b, env, owner_hint)?.powi(*e),
        Expression::Min(l, r) => eval_expr(l, env, owner_hint)?.min(eval_expr(r, env, owner_hint)?),
        Expression::Max(l, r) => eval_expr(l, env, owner_hint)?.max(eval_expr(r, env, owner_hint)?),
        Expression::Abs(e) => eval_expr(e, env, owner_hint)?.abs(),
        Expression::Sqrt(e) => eval_expr(e, env, owner_hint)?.sqrt(),
        Expression::Log(e) => eval_expr(e, env, owner_hint)?.ln(),
        Expression::Exp(e) => eval_expr(e, env, owner_hint)?.exp(),
        Expression::IfPos {
            cond,
            then_branch,
            else_branch,
        } => {
            if eval_expr(cond, env, owner_hint)? > 0.0 {
                eval_expr(then_branch, env, owner_hint)?
            } else {
                eval_expr(else_branch, env, owner_hint)?
            }
        }
    };

    if !v.is_finite() {
        return Err(EngineError::ParamNonfinite(owner_hint.0.clone()));
    }
    Ok(v)
}

/// Walk an expression tree and collect every `ParameterId` referenced
/// via `Expression::Var`. Duplicates are preserved — callers that want
/// a unique set should dedupe after calling.
pub fn walk_deps(expr: &Expression, out: &mut Vec<ParameterId>) {
    match expr {
        Expression::Const(_) => {}
        Expression::Var(id) => out.push(id.clone()),
        Expression::Add(l, r)
        | Expression::Sub(l, r)
        | Expression::Mul(l, r)
        | Expression::Div(l, r)
        | Expression::Min(l, r)
        | Expression::Max(l, r) => {
            walk_deps(l, out);
            walk_deps(r, out);
        }
        Expression::Pow(b, _) => walk_deps(b, out),
        Expression::Abs(e) | Expression::Sqrt(e) | Expression::Log(e) | Expression::Exp(e) => {
            walk_deps(e, out);
        }
        Expression::IfPos {
            cond,
            then_branch,
            else_branch,
        } => {
            walk_deps(cond, out);
            walk_deps(then_branch, out);
            walk_deps(else_branch, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arko_core::Unit;

    fn pf(id: &str, v: f64) -> Parameter {
        Parameter::free(id, v, Unit::dimensionless())
    }

    fn pe(id: &str, expr: Expression) -> Parameter {
        Parameter {
            id: id.into(),
            expression: expr,
            value: None,
            unit: Unit::dimensionless(),
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn free_parameters_evaluate_to_their_constant() {
        let params = vec![pf("a", 3.0), pf("b", 4.0)];
        let env = evaluate(&params).unwrap();
        assert_eq!(env[&ParameterId::from("a")], 3.0);
        assert_eq!(env[&ParameterId::from("b")], 4.0);
    }

    #[test]
    fn derived_parameter_uses_upstream_values() {
        // c = a + b, where a=3, b=4 → c=7
        let params = vec![
            pf("a", 3.0),
            pf("b", 4.0),
            pe(
                "c",
                Expression::Add(
                    Box::new(Expression::Var("a".into())),
                    Box::new(Expression::Var("b".into())),
                ),
            ),
        ];
        let env = evaluate(&params).unwrap();
        assert_eq!(env[&ParameterId::from("c")], 7.0);
    }

    #[test]
    fn cycle_is_rejected() {
        // a = b, b = a
        let params = vec![
            pe("a", Expression::Var("b".into())),
            pe("b", Expression::Var("a".into())),
        ];
        let err = evaluate(&params).unwrap_err();
        assert_eq!(err.code(), "E_PARAM_CYCLE");
    }

    #[test]
    fn unresolved_reference_is_rejected() {
        let params = vec![pe("a", Expression::Var("ghost".into()))];
        let err = evaluate(&params).unwrap_err();
        assert_eq!(err.code(), "E_PARAM_UNRESOLVED");
    }

    #[test]
    fn sqrt_of_negative_is_nonfinite() {
        // a = sqrt(-1) → NaN → rejected
        let params = vec![pe("a", Expression::Sqrt(Box::new(Expression::Const(-1.0))))];
        let err = evaluate(&params).unwrap_err();
        assert_eq!(err.code(), "E_PARAM_NONFINITE");
    }

    #[test]
    fn divide_by_zero_is_nonfinite() {
        let params = vec![pe(
            "a",
            Expression::Div(
                Box::new(Expression::Const(1.0)),
                Box::new(Expression::Const(0.0)),
            ),
        )];
        let err = evaluate(&params).unwrap_err();
        assert_eq!(err.code(), "E_PARAM_NONFINITE");
    }

    #[test]
    fn if_pos_branches_correctly() {
        // a = if(1, 10, 20) → 10; b = if(-1, 10, 20) → 20
        let params = vec![
            pe(
                "a",
                Expression::IfPos {
                    cond: Box::new(Expression::Const(1.0)),
                    then_branch: Box::new(Expression::Const(10.0)),
                    else_branch: Box::new(Expression::Const(20.0)),
                },
            ),
            pe(
                "b",
                Expression::IfPos {
                    cond: Box::new(Expression::Const(-1.0)),
                    then_branch: Box::new(Expression::Const(10.0)),
                    else_branch: Box::new(Expression::Const(20.0)),
                },
            ),
        ];
        let env = evaluate(&params).unwrap();
        assert_eq!(env[&ParameterId::from("a")], 10.0);
        assert_eq!(env[&ParameterId::from("b")], 20.0);
    }

    #[test]
    fn duplicate_id_rejected() {
        let params = vec![pf("a", 1.0), pf("a", 2.0)];
        let err = evaluate(&params).unwrap_err();
        assert_eq!(err.code(), "E_INTERNAL");
    }
}
