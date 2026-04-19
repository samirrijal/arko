//! UCUM-subset parser.
//!
//! The grammar at v0.0.1:
//!
//! ```text
//! unit        := term (operator term)*
//! operator    := '.' | '*' | '/'
//! term        := identifier exponent?
//! exponent    := digits | '^' '-'? digits | digits
//! identifier  := [letter]+       -- resolved against atom table
//! ```
//!
//! Tag handling happens *before* structured parsing: the first ASCII
//! space (or `·` / `*` / `.` / `/` is not a space) splits the input
//! into a unit expression + a *semantic tag* such as `CO2-eq`, `N-eq`,
//! or `PM2.5-eq`. Two units with the same 7D dimension but different
//! tags are **not** commensurable — converting `kg` to `kg CO2-eq`
//! requires a characterization factor, not a scale.

use crate::{
    atom::{find_atom, PREFIXES},
    dimension::Dimension,
};

/// Errors the parser can surface.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum UnitParseError {
    #[error("unexpected character '{ch}' at position {position} in unit `{input}`")]
    UnexpectedChar {
        input: String,
        ch: char,
        position: usize,
    },

    #[error("empty unit expression (input was `{0}`)")]
    Empty(String),

    #[error("unknown unit atom `{atom}` in `{input}`")]
    UnknownAtom { input: String, atom: String },

    #[error("missing exponent digits after `^` in `{0}`")]
    MissingExponent(String),

    #[error("exponent overflow in `{0}`")]
    ExponentOverflow(String),

    #[error("trailing operator `{op}` in `{input}`")]
    TrailingOperator { input: String, op: char },
}

#[derive(Debug, Clone)]
enum Token {
    /// An atomic term with its exponent already folded in.
    Term {
        ident: String,
        exp: i32,
    },
    Op(char),
}

/// Top-level parse entry point. Returns the canonical `(scale, dim,
/// tag)` triple. The scale multiplies a numeric value in the parsed
/// unit to a value in the corresponding SI base unit.
pub fn parse_expression(input: &str) -> Result<ParsedExpression, UnitParseError> {
    // Pre-normalize: middle-dot → '.', collapse whitespace runs.
    let normalized = input.replace('·', ".").replace('\u{00D7}', ".");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Err(UnitParseError::Empty(input.to_string()));
    }

    // Split on the FIRST ASCII space — everything before is the unit
    // expression, everything after is the semantic tag.
    let (expr_part, tag) = match trimmed.find(' ') {
        Some(idx) => {
            let head = &trimmed[..idx];
            let tail = trimmed[idx + 1..].trim();
            let tag_owned = if tail.is_empty() {
                None
            } else {
                Some(tail.split_whitespace().collect::<Vec<_>>().join(" "))
            };
            (head, tag_owned)
        }
        None => (trimmed, None),
    };

    let (scale, dim) = parse_unit_expr(expr_part, input)?;
    Ok(ParsedExpression {
        scale,
        dimension: dim,
        tag,
    })
}

/// Intermediate carrier from parser → ParsedUnit wrapper. Keeps
/// `lib.rs` from needing to know parser internals.
#[derive(Debug, Clone)]
pub struct ParsedExpression {
    pub scale: f64,
    pub dimension: Dimension,
    pub tag: Option<String>,
}

// -------------------------------------------------------------------

fn parse_unit_expr(expr: &str, full_input: &str) -> Result<(f64, Dimension), UnitParseError> {
    if expr.is_empty() {
        return Err(UnitParseError::Empty(full_input.to_string()));
    }
    // Bare "1" is the canonical dimensionless literal. The tokenizer
    // can't represent it as a Term (digits aren't identifier starters),
    // so handle it here before tokenizing.
    if expr.trim() == "1" {
        return Ok((1.0, Dimension::DIMENSIONLESS));
    }
    let tokens = tokenize(expr, full_input)?;

    // Walk tokens: start with sign = +1, toggle for one term after '/'.
    let mut scale = 1.0_f64;
    let mut dim = Dimension::DIMENSIONLESS;
    let mut next_sign: i8 = 1;
    let mut expect_term = true;
    let mut last_op: Option<char> = None;

    for tok in &tokens {
        match tok {
            Token::Term { ident, exp } => {
                if !expect_term {
                    return Err(UnitParseError::UnexpectedChar {
                        input: full_input.to_string(),
                        ch: ident.chars().next().unwrap_or('?'),
                        position: 0,
                    });
                }
                let (atom_scale, atom_dim) = resolve_identifier(ident, full_input)?;
                let eff_exp = i32::from(next_sign) * exp;
                let eff_exp_i8 = i8::try_from(eff_exp)
                    .map_err(|_| UnitParseError::ExponentOverflow(full_input.to_string()))?;
                scale *= atom_scale.powi(eff_exp);
                dim = dim.add(&atom_dim.scale(eff_exp_i8));
                // Reset sign for the next term.
                next_sign = 1;
                expect_term = false;
                last_op = None;
            }
            Token::Op(op) => {
                if expect_term {
                    // Two operators in a row, or leading operator.
                    return Err(UnitParseError::UnexpectedChar {
                        input: full_input.to_string(),
                        ch: *op,
                        position: 0,
                    });
                }
                match op {
                    '/' => next_sign = -1,
                    '.' | '*' => next_sign = 1,
                    other => {
                        return Err(UnitParseError::UnexpectedChar {
                            input: full_input.to_string(),
                            ch: *other,
                            position: 0,
                        });
                    }
                }
                expect_term = true;
                last_op = Some(*op);
            }
        }
    }

    if expect_term {
        if let Some(op) = last_op {
            return Err(UnitParseError::TrailingOperator {
                input: full_input.to_string(),
                op,
            });
        }
        return Err(UnitParseError::Empty(full_input.to_string()));
    }

    Ok((scale, dim))
}

fn tokenize(input: &str, full_input: &str) -> Result<Vec<Token>, UnitParseError> {
    let bytes: Vec<char> = input.chars().collect();
    let mut tokens: Vec<Token> = Vec::new();
    let mut i: usize = 0;

    while i < bytes.len() {
        let c = bytes[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '.' || c == '*' || c == '/' {
            tokens.push(Token::Op(c));
            i += 1;
            continue;
        }
        if is_ident_start(c) {
            let start = i;
            while i < bytes.len() && is_ident_continue(bytes[i]) {
                i += 1;
            }
            let ident: String = bytes[start..i].iter().collect();

            // Attached exponent, if any.
            let mut exp: i32 = 1;
            if i < bytes.len() && bytes[i] == '^' {
                i += 1;
                let sign: i32 = if i < bytes.len() && bytes[i] == '-' {
                    i += 1;
                    -1
                } else if i < bytes.len() && bytes[i] == '+' {
                    i += 1;
                    1
                } else {
                    1
                };
                let digits_start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i == digits_start {
                    return Err(UnitParseError::MissingExponent(full_input.to_string()));
                }
                let raw: String = bytes[digits_start..i].iter().collect();
                let num: i32 = raw
                    .parse()
                    .map_err(|_| UnitParseError::ExponentOverflow(full_input.to_string()))?;
                exp = sign * num;
            } else if i < bytes.len() && bytes[i].is_ascii_digit() {
                // Attached implicit-positive exponent `m2`.
                let digits_start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let raw: String = bytes[digits_start..i].iter().collect();
                let num: i32 = raw
                    .parse()
                    .map_err(|_| UnitParseError::ExponentOverflow(full_input.to_string()))?;
                exp = num;
            }

            tokens.push(Token::Term { ident, exp });
            continue;
        }

        return Err(UnitParseError::UnexpectedChar {
            input: full_input.to_string(),
            ch: c,
            position: i,
        });
    }

    Ok(tokens)
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || matches!(c, 'μ' | 'µ')
}

fn is_ident_continue(c: char) -> bool {
    // Identifiers run of letters only. Digits attach as a separate
    // exponent token even without an explicit `^`.
    c.is_ascii_alphabetic() || matches!(c, 'μ' | 'µ')
}

/// Resolve a single bare identifier (no exponent, no operator) to
/// `(scale, dimension)`. Tries an exact atom match first; on miss,
/// peels off every known SI prefix (longest first) and looks up the
/// remainder.
fn resolve_identifier(ident: &str, full_input: &str) -> Result<(f64, Dimension), UnitParseError> {
    if let Some(a) = find_atom(ident) {
        return Ok((a.scale, a.dimension));
    }
    for prefix in PREFIXES {
        if let Some(remainder) = ident.strip_prefix(prefix.symbol) {
            if remainder.is_empty() {
                continue;
            }
            if let Some(a) = find_atom(remainder) {
                return Ok((prefix.scale * a.scale, a.dimension));
            }
        }
    }
    Err(UnitParseError::UnknownAtom {
        input: full_input.to_string(),
        atom: ident.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_meter_is_length_scale_1() {
        let p = parse_expression("m").unwrap();
        assert_eq!(p.dimension, Dimension::length());
        assert!((p.scale - 1.0).abs() < 1e-18);
        assert_eq!(p.tag, None);
    }

    #[test]
    fn parse_kilogram_is_mass_scale_1() {
        let p = parse_expression("kg").unwrap();
        assert_eq!(p.dimension, Dimension::mass());
        // k * g = 1e3 * 1e-3 = 1.0 — `kg` is the SI base of mass.
        assert!((p.scale - 1.0).abs() < 1e-15);
    }

    #[test]
    fn parse_mj_is_energy_scale_1e6() {
        let p = parse_expression("MJ").unwrap();
        assert_eq!(
            p.dimension,
            Dimension {
                length: 2,
                mass: 1,
                time: -2,
                ..Dimension::DIMENSIONLESS
            }
        );
        assert!((p.scale - 1e6).abs() < 1e-3);
    }

    #[test]
    fn parse_kwh_is_3_6e6_joules() {
        let p = parse_expression("kWh").unwrap();
        assert!((p.scale - 3.6e6).abs() < 1e-3);
    }

    #[test]
    fn parse_m_exp_3_is_volume() {
        let p = parse_expression("m3").unwrap();
        assert_eq!(p.dimension, Dimension::length().scale(3));
    }

    #[test]
    fn parse_m_caret_minus_2_is_inverse_area() {
        let p = parse_expression("m^-2").unwrap();
        assert_eq!(p.dimension, Dimension::length().scale(-2));
    }

    #[test]
    fn parse_velocity_with_slash() {
        let p = parse_expression("m/s").unwrap();
        assert_eq!(
            p.dimension,
            Dimension::length().add(&Dimension::time().scale(-1))
        );
        assert!((p.scale - 1.0).abs() < 1e-18);
    }

    #[test]
    fn parse_dot_multiplication() {
        let p = parse_expression("N.m").unwrap();
        // N = kg·m·s−2; times m = kg·m2·s−2 = J.
        assert_eq!(
            p.dimension,
            Dimension {
                length: 2,
                mass: 1,
                time: -2,
                ..Dimension::DIMENSIONLESS
            }
        );
    }

    #[test]
    fn parse_tonne_km_is_mass_times_length() {
        let p = parse_expression("t.km").unwrap();
        assert_eq!(p.dimension, Dimension::mass().add(&Dimension::length()));
        assert!((p.scale - 1e6).abs() < 1e-3); // 1 t = 1000 kg, 1 km = 1000 m.
    }

    #[test]
    fn parse_tag_is_extracted_after_first_space() {
        let p = parse_expression("kg CO2-eq").unwrap();
        assert_eq!(p.dimension, Dimension::mass());
        assert_eq!(p.tag.as_deref(), Some("CO2-eq"));
    }

    #[test]
    fn parse_unknown_atom_is_err() {
        let err = parse_expression("banana").unwrap_err();
        assert!(matches!(err, UnitParseError::UnknownAtom { .. }));
    }

    #[test]
    fn parse_empty_is_err() {
        assert!(matches!(
            parse_expression("").unwrap_err(),
            UnitParseError::Empty(_)
        ));
    }

    #[test]
    fn parse_trailing_slash_is_err() {
        let err = parse_expression("m/").unwrap_err();
        assert!(matches!(err, UnitParseError::TrailingOperator { .. }));
    }

    #[test]
    fn parse_double_slash_is_err() {
        let err = parse_expression("m//s").unwrap_err();
        assert!(matches!(err, UnitParseError::UnexpectedChar { .. }));
    }

    #[test]
    fn parse_unicode_middle_dot_as_multiplication() {
        let p = parse_expression("kg·m").unwrap();
        assert_eq!(p.dimension, Dimension::mass().add(&Dimension::length()));
    }

    #[test]
    fn parse_dimensionless_is_one() {
        let p = parse_expression("1").unwrap();
        assert_eq!(p.dimension, Dimension::DIMENSIONLESS);
        assert!((p.scale - 1.0).abs() < 1e-18);
    }

    #[test]
    fn left_associative_division() {
        // a/b/c = a · b^-1 · c^-1. Use m/s/s → m·s^-2.
        let p = parse_expression("m/s/s").unwrap();
        assert_eq!(
            p.dimension,
            Dimension::length().add(&Dimension::time().scale(-2))
        );
    }

    #[test]
    fn division_then_multiplication_does_not_escape_denominator_early() {
        // UCUM spec: `m/s.kg` = (m/s)·kg = m·kg/s  (NOT m/(s·kg)).
        let p = parse_expression("m/s.kg").unwrap();
        let expected = Dimension::length()
            .add(&Dimension::time().scale(-1))
            .add(&Dimension::mass());
        assert_eq!(p.dimension, expected);
    }
}
