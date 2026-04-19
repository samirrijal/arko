//! Parameters — see `specs/calc/v0.1.md` §5.
//!
//! A study may declare named scalar parameters used in matrix entries.
//! This module defines the **AST** and **storage form**. Evaluation
//! (topological ordering, expression evaluation, partial-derivative
//! computation for §9.3 and §10) lives in a sibling crate
//! (`arko-parameters`) to keep core tiny.

use serde::{Deserialize, Serialize};

/// Stable identifier for a parameter. Used as the variable name in
/// expressions and as the lookup key during evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ParameterId(pub String);

impl From<&str> for ParameterId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for ParameterId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Expression AST. The v0.1 spec permits a restricted arithmetic
/// language (§5.1): `+ - * / ^`, `min max abs sqrt log exp if`, and
/// variable references. No I/O, no loops, no user-defined functions.
///
/// Serde uses the default externally-tagged representation
/// (`{"add": [lhs, rhs]}`, `{"const": 1.0}`, etc.) — internally-tagged
/// `#[serde(tag = "kind")]` is incompatible with tuple variants, and
/// rewriting every variant to struct form purely for wire-format
/// cosmetics would inflate the evaluator match arms with no gain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expression {
    Const(f64),
    Var(ParameterId),

    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    /// Integer-exponent power only, per §5.1.
    Pow(Box<Expression>, i32),

    Min(Box<Expression>, Box<Expression>),
    Max(Box<Expression>, Box<Expression>),
    Abs(Box<Expression>),
    Sqrt(Box<Expression>),
    Log(Box<Expression>),
    Exp(Box<Expression>),

    /// `if(cond, then, else)` — `cond` is truthy iff strictly positive.
    IfPos {
        cond: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },
}

/// A named scalar parameter. See §5.1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub id: ParameterId,

    /// Either the resolved free-variable value OR an expression to evaluate.
    /// A free variable has `expression = Expression::Const(value)`; an
    /// expression-defined parameter references other ids.
    pub expression: Expression,

    /// Required when `expression` is a free variable — i.e., when the
    /// parameter is intended to be user-editable rather than derived.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,

    pub unit: crate::units::Unit,

    /// Resolved dependency ids — populated at construction time by
    /// walking the expression AST. Redundant with `expression` but cached
    /// for cheap cycle detection.
    #[serde(default)]
    pub dependencies: Vec<ParameterId>,
}

impl Parameter {
    /// Construct a free-variable parameter (leaf of the DAG).
    pub fn free(
        id: impl Into<ParameterId>,
        value: f64,
        unit: impl Into<crate::units::Unit>,
    ) -> Self {
        Self {
            id: id.into(),
            expression: Expression::Const(value),
            value: Some(value),
            unit: unit.into(),
            dependencies: Vec::new(),
        }
    }
}
