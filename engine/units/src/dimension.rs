//! The 7-dimensional SI base vector.
//!
//! Every unit expression reduces to a 7-tuple of small integer
//! exponents over the ISO 80000 base dimensions:
//!
//! | axis | name                | SI base unit |
//! |------|---------------------|--------------|
//! | `L`  | length              | m            |
//! | `M`  | mass                | kg           |
//! | `T`  | time                | s            |
//! | `N`  | amount of substance | mol          |
//! | `Θ`  | temperature         | K            |
//! | `I`  | electric current    | A            |
//! | `J`  | luminous intensity  | cd           |
//!
//! Exponents are `i8` — plenty of headroom (−128..127) for the
//! `m2/s3` / `kg/(m.s2)` compounds that occur in physics, far more
//! than LCA ever needs.

use serde::{Deserialize, Serialize};

/// A 7-vector of SI base-dimension exponents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Dimension {
    pub length: i8,
    pub mass: i8,
    pub time: i8,
    pub amount: i8,
    pub temperature: i8,
    pub current: i8,
    pub luminosity: i8,
}

impl Dimension {
    /// The dimensionless scalar `1`.
    pub const DIMENSIONLESS: Self = Self {
        length: 0,
        mass: 0,
        time: 0,
        amount: 0,
        temperature: 0,
        current: 0,
        luminosity: 0,
    };

    /// `true` iff every exponent is zero.
    #[must_use]
    pub fn is_dimensionless(&self) -> bool {
        *self == Self::DIMENSIONLESS
    }

    /// Componentwise sum — used when multiplying two units.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            length: self.length + other.length,
            mass: self.mass + other.mass,
            time: self.time + other.time,
            amount: self.amount + other.amount,
            temperature: self.temperature + other.temperature,
            current: self.current + other.current,
            luminosity: self.luminosity + other.luminosity,
        }
    }

    /// Componentwise difference — used when dividing two units.
    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            length: self.length - other.length,
            mass: self.mass - other.mass,
            time: self.time - other.time,
            amount: self.amount - other.amount,
            temperature: self.temperature - other.temperature,
            current: self.current - other.current,
            luminosity: self.luminosity - other.luminosity,
        }
    }

    /// Multiply every exponent by a scalar. Used by `u^k`. `const` so it
    /// can appear in the `ATOMS` table (see `atom.rs`).
    #[must_use]
    pub const fn scale(&self, k: i8) -> Self {
        Self {
            length: self.length * k,
            mass: self.mass * k,
            time: self.time * k,
            amount: self.amount * k,
            temperature: self.temperature * k,
            current: self.current * k,
            luminosity: self.luminosity * k,
        }
    }

    // --- base-dimension constructors --------------------------------

    pub const fn length() -> Self {
        Self {
            length: 1,
            ..Self::DIMENSIONLESS
        }
    }
    pub const fn mass() -> Self {
        Self {
            mass: 1,
            ..Self::DIMENSIONLESS
        }
    }
    pub const fn time() -> Self {
        Self {
            time: 1,
            ..Self::DIMENSIONLESS
        }
    }
    pub const fn amount() -> Self {
        Self {
            amount: 1,
            ..Self::DIMENSIONLESS
        }
    }
    pub const fn temperature() -> Self {
        Self {
            temperature: 1,
            ..Self::DIMENSIONLESS
        }
    }
    pub const fn current() -> Self {
        Self {
            current: 1,
            ..Self::DIMENSIONLESS
        }
    }
    pub const fn luminosity() -> Self {
        Self {
            luminosity: 1,
            ..Self::DIMENSIONLESS
        }
    }
}

impl std::fmt::Display for Dimension {
    /// Emit in canonical order `L^a M^b T^c N^d Θ^e I^f J^g`,
    /// omitting axes with exponent 0. Returns `"1"` for the
    /// dimensionless unit.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_dimensionless() {
            return f.write_str("1");
        }
        let mut first = true;
        let mut write = |f: &mut std::fmt::Formatter<'_>,
                         sym: &str,
                         exp: i8|
         -> std::fmt::Result {
            if exp == 0 {
                return Ok(());
            }
            if !first {
                f.write_str(" ")?;
            }
            first = false;
            if exp == 1 {
                f.write_str(sym)
            } else {
                write!(f, "{sym}^{exp}")
            }
        };
        write(f, "L", self.length)?;
        write(f, "M", self.mass)?;
        write(f, "T", self.time)?;
        write(f, "N", self.amount)?;
        write(f, "Θ", self.temperature)?;
        write(f, "I", self.current)?;
        write(f, "J", self.luminosity)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensionless_is_default() {
        assert_eq!(Dimension::default(), Dimension::DIMENSIONLESS);
        assert!(Dimension::default().is_dimensionless());
    }

    #[test]
    fn add_matches_componentwise() {
        let l = Dimension::length();
        let t = Dimension::time();
        let lt = l.add(&t);
        assert_eq!(lt.length, 1);
        assert_eq!(lt.time, 1);
        assert_eq!(lt.mass, 0);
    }

    #[test]
    fn sub_cancels() {
        let l = Dimension::length();
        let zero = l.sub(&l);
        assert!(zero.is_dimensionless());
    }

    #[test]
    fn scale_multiplies_every_axis() {
        let v = Dimension::length().scale(3);
        assert_eq!(v.length, 3);
        assert_eq!(v.mass, 0);
    }

    #[test]
    fn display_dimensionless() {
        assert_eq!(format!("{}", Dimension::DIMENSIONLESS), "1");
    }

    #[test]
    fn display_compound() {
        // velocity: L T^-1
        let v = Dimension::length().add(&Dimension::time().scale(-1));
        assert_eq!(format!("{v}"), "L T^-1");
    }
}
