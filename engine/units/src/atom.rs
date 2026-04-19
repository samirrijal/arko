//! Atomic units and SI prefixes.
//!
//! An *atom* is a named unit that cannot be decomposed further by the
//! parser. Each atom carries its 7D dimension and its scale factor to
//! the corresponding SI base unit. The parser greedy-matches atoms
//! optionally preceded by a prefix (e.g. `km` = `k` + `m`, `MJ` = `M`
//! + `J`).
//!
//! Coverage at v0.0.1:
//!
//! - SI base atoms (meter, gram, second, mole, kelvin, ampere, candela).
//!   Note mass uses `g` (gram) as the atom with SI-base scale 0.001 kg;
//!   `kg` parses as `k` + `g` with net scale `1.0`.
//! - SI-derived named atoms that appear in LCA data: N, Pa, J, W, Hz,
//!   L (liter), Wh (watt-hour), bar, t (tonne).
//! - Time atoms beyond second: min, h, d, yr.
//! - The dimensionless sentinel `1`.
//!
//! Unicode canonicalization: `μ` (U+03BC) and `µ` (U+00B5, "micro
//! sign") are both accepted as the "micro" prefix.

use crate::dimension::Dimension;

/// A single atomic unit: symbol, dimension, scale to SI base.
#[derive(Debug, Clone, Copy)]
pub struct Atom {
    pub symbol: &'static str,
    pub dimension: Dimension,
    /// Multiplier to convert a value in this atom to the SI base unit
    /// of the same dimension. For example `min` → scale 60.0 (`T`);
    /// `g` → scale 0.001 (`M`, SI base is kg, not g).
    pub scale: f64,
}

/// An SI prefix: symbol and power-of-ten multiplier.
#[derive(Debug, Clone, Copy)]
pub struct Prefix {
    pub symbol: &'static str,
    pub scale: f64,
}

/// The SI prefix table — most-to-least selective so greedy longest-
/// match works against `symbol`.
pub const PREFIXES: &[Prefix] = &[
    // Two-character prefixes MUST appear before their one-character
    // prefixes of the same leading letter so a greedy longest-match
    // parser picks them up first. We list them in descending length.
    Prefix { symbol: "da", scale: 1e1 },
    // One-character prefixes.
    Prefix { symbol: "Y", scale: 1e24 },
    Prefix { symbol: "Z", scale: 1e21 },
    Prefix { symbol: "E", scale: 1e18 },
    Prefix { symbol: "P", scale: 1e15 },
    Prefix { symbol: "T", scale: 1e12 },
    Prefix { symbol: "G", scale: 1e9 },
    Prefix { symbol: "M", scale: 1e6 },
    Prefix { symbol: "k", scale: 1e3 },
    Prefix { symbol: "h", scale: 1e2 },
    Prefix { symbol: "d", scale: 1e-1 },
    Prefix { symbol: "c", scale: 1e-2 },
    Prefix { symbol: "m", scale: 1e-3 },
    Prefix { symbol: "μ", scale: 1e-6 },
    Prefix { symbol: "µ", scale: 1e-6 }, // U+00B5 "micro sign" alias.
    Prefix { symbol: "u", scale: 1e-6 }, // ASCII micro, widely used.
    Prefix { symbol: "n", scale: 1e-9 },
    Prefix { symbol: "p", scale: 1e-12 },
    Prefix { symbol: "f", scale: 1e-15 },
    Prefix { symbol: "a", scale: 1e-18 },
    Prefix { symbol: "z", scale: 1e-21 },
    Prefix { symbol: "y", scale: 1e-24 },
];

/// The atom table. Listed longest-symbol first where a short symbol
/// could be mistaken for a prefix (e.g. `kWh` — parser must try `Wh`
/// as an atom before peeling off `W` + `h`).
pub const ATOMS: &[Atom] = &[
    // Dimensionless.
    Atom {
        symbol: "1",
        dimension: Dimension::DIMENSIONLESS,
        scale: 1.0,
    },
    // --- time ----------------------------------------------------
    Atom {
        symbol: "min",
        dimension: Dimension::time(),
        scale: 60.0,
    },
    Atom {
        symbol: "yr",
        dimension: Dimension::time(),
        scale: 365.25 * 86_400.0,
    },
    Atom {
        symbol: "a", // annum — UCUM uses `a` for year.
        dimension: Dimension::time(),
        scale: 365.25 * 86_400.0,
    },
    Atom {
        symbol: "h",
        dimension: Dimension::time(),
        scale: 3_600.0,
    },
    Atom {
        symbol: "d",
        dimension: Dimension::time(),
        scale: 86_400.0,
    },
    Atom {
        symbol: "s",
        dimension: Dimension::time(),
        scale: 1.0,
    },
    // --- energy / power (named derived) --------------------------
    Atom {
        symbol: "Wh",
        // W·h = J/s · s = J, but we write it out: kg·m2·s−2·s·s−1·s = kg·m2·s−2.
        // Scale vs SI base J = 1: Wh = 3600 J.
        dimension: Dimension {
            length: 2,
            mass: 1,
            time: -2,
            ..Dimension::DIMENSIONLESS
        },
        scale: 3_600.0,
    },
    Atom {
        symbol: "eV",
        dimension: Dimension {
            length: 2,
            mass: 1,
            time: -2,
            ..Dimension::DIMENSIONLESS
        },
        scale: 1.602_176_634e-19,
    },
    Atom {
        // Joule — energy: kg·m2·s−2.
        symbol: "J",
        dimension: Dimension {
            length: 2,
            mass: 1,
            time: -2,
            ..Dimension::DIMENSIONLESS
        },
        scale: 1.0,
    },
    Atom {
        // Watt — power: kg·m2·s−3.
        symbol: "W",
        dimension: Dimension {
            length: 2,
            mass: 1,
            time: -3,
            ..Dimension::DIMENSIONLESS
        },
        scale: 1.0,
    },
    // --- force / pressure ----------------------------------------
    Atom {
        // Newton: kg·m·s−2.
        symbol: "N",
        dimension: Dimension {
            length: 1,
            mass: 1,
            time: -2,
            ..Dimension::DIMENSIONLESS
        },
        scale: 1.0,
    },
    Atom {
        // Pascal: kg·m−1·s−2.
        symbol: "Pa",
        dimension: Dimension {
            length: -1,
            mass: 1,
            time: -2,
            ..Dimension::DIMENSIONLESS
        },
        scale: 1.0,
    },
    Atom {
        symbol: "bar",
        dimension: Dimension {
            length: -1,
            mass: 1,
            time: -2,
            ..Dimension::DIMENSIONLESS
        },
        scale: 1e5,
    },
    // --- frequency -----------------------------------------------
    Atom {
        symbol: "Hz",
        dimension: Dimension::time().scale(-1),
        scale: 1.0,
    },
    // --- volume (L = dm3) ----------------------------------------
    Atom {
        symbol: "L",
        dimension: Dimension::length().scale(3),
        scale: 1e-3,
    },
    Atom {
        symbol: "l", // lowercase alias.
        dimension: Dimension::length().scale(3),
        scale: 1e-3,
    },
    // --- mass ----------------------------------------------------
    // UCUM's canonical mass atom is `g` (gram). `kg` is parsed as
    // `k` + `g`, net scale 1.0. `t` (tonne) is 1000 kg.
    Atom {
        symbol: "t",
        dimension: Dimension::mass(),
        scale: 1e3,
    },
    Atom {
        symbol: "g",
        dimension: Dimension::mass(),
        scale: 1e-3,
    },
    // --- base atoms ----------------------------------------------
    Atom {
        symbol: "m",
        dimension: Dimension::length(),
        scale: 1.0,
    },
    Atom {
        symbol: "mol",
        dimension: Dimension::amount(),
        scale: 1.0,
    },
    Atom {
        symbol: "K",
        dimension: Dimension::temperature(),
        scale: 1.0,
    },
    Atom {
        symbol: "A",
        dimension: Dimension::current(),
        scale: 1.0,
    },
    Atom {
        symbol: "cd",
        dimension: Dimension::luminosity(),
        scale: 1.0,
    },
];

/// Look up an atom by exact symbol match. Returns `None` on miss.
///
/// The table above is intentionally ordered longest-symbol-first for
/// atoms whose short form could otherwise be misidentified (e.g., we
/// check `min` before `m`, `Wh` before `W`). Prefix-stripping is the
/// parser's job, not ours — this function does not peel prefixes.
#[must_use]
pub fn find_atom(symbol: &str) -> Option<&'static Atom> {
    ATOMS.iter().find(|a| a.symbol == symbol)
}

/// Look up a prefix by exact symbol match.
#[must_use]
pub fn find_prefix(symbol: &str) -> Option<&'static Prefix> {
    PREFIXES.iter().find(|p| p.symbol == symbol)
}

/// Iterator over all atoms — handy for external tooling that wants
/// to enumerate the known unit set (e.g., documentation generators).
pub fn all_atoms() -> impl Iterator<Item = &'static Atom> {
    ATOMS.iter()
}

/// Iterator over all SI prefixes.
pub fn all_prefixes() -> impl Iterator<Item = &'static Prefix> {
    PREFIXES.iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_atom_hits_base() {
        assert_eq!(find_atom("m").unwrap().symbol, "m");
        assert_eq!(find_atom("s").unwrap().symbol, "s");
        assert_eq!(find_atom("mol").unwrap().symbol, "mol");
    }

    #[test]
    fn find_atom_miss_is_none() {
        assert!(find_atom("nope").is_none());
    }

    #[test]
    fn find_prefix_resolves_micro_aliases() {
        assert_eq!(find_prefix("μ").unwrap().scale, 1e-6);
        assert_eq!(find_prefix("µ").unwrap().scale, 1e-6);
        assert_eq!(find_prefix("u").unwrap().scale, 1e-6);
    }

    #[test]
    fn gram_is_the_mass_atom_not_kilogram() {
        // Spec choice: `kg` is prefix-plus-atom, not a distinct atom.
        assert!(find_atom("kg").is_none());
        let g = find_atom("g").unwrap();
        assert_eq!(g.dimension, Dimension::mass());
        assert!((g.scale - 1e-3).abs() < 1e-18);
    }

    #[test]
    fn tonne_scales_to_1000_kg() {
        let t = find_atom("t").unwrap();
        assert_eq!(t.dimension, Dimension::mass());
        // tonne to SI base (kg): scale should equal 1000.
        assert!((t.scale - 1e3).abs() < 1e-12);
    }

    #[test]
    fn liter_scales_to_cubic_meters() {
        let l = find_atom("L").unwrap();
        assert_eq!(l.dimension, Dimension::length().scale(3));
        assert!((l.scale - 1e-3).abs() < 1e-18);
    }

    #[test]
    fn watt_hour_is_3600_joules() {
        let wh = find_atom("Wh").unwrap();
        let j = find_atom("J").unwrap();
        assert_eq!(wh.dimension, j.dimension);
        assert!((wh.scale - 3_600.0).abs() < 1e-9);
    }
}
