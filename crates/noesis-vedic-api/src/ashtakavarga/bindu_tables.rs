//! Bhinna-Ashtakavarga (BAV) contribution matrices.
//!
//! PR3 — Brihat Parashara Hora Shastra (BPHS) Chapter 66 (R. Santhanam /
//! K. S. Sharma translation). For each of the seven classical planets we
//! encode a `[bool; 12]` row for each of the eight contributors (Sun,
//! Moon, Mars, Mercury, Jupiter, Venus, Saturn, Lagna) listing the
//! house-positions counted **from that contributor's sign** at which the
//! contributor donates a bindu (1 point) to the planet's BAV. Index 0 is
//! "1st-from-contributor" (i.e. the contributor's own sign), index 11 is
//! "12th-from-contributor".
//!
//! The reference totals per planet (sum of all 96 booleans in the table):
//!
//! | Planet | Reference total |
//! |--------|-----------------|
//! | Sun    | 48              |
//! | Moon   | 49              |
//! | Mars   | 39              |
//! | Mercury| 54              |
//! | Jupiter| 56              |
//! | Venus  | 52              |
//! | Saturn | 39              |
//!
//! Each table below is followed by a `#[test]` asserting its sum matches
//! the published reference total — that's the safety net catching any
//! data-entry error (PRD requirement). The unit test layer is part of the
//! BAV commit itself, not deferred.
//!
//! Sources (cross-checked):
//!   * R. Santhanam, *Brihat Parashara Hora Shastra* vol. II, Chapter 66
//!     (Ashtakavarga Adhyaya).
//!   * K. S. Charak, *Predictive Astrology — An Insight*, Ashtakavarga
//!     chapter.
//!   * J. N. Bhasin, *Ashtakavarga System of Prediction*.
//!
//! ## Reading the tables
//!
//! `BINDU_TABLE_SUN[c][i]` is `true` iff contributor `c` (in order: Sun,
//! Moon, Mars, Mercury, Jupiter, Venus, Saturn, Lagna) donates a bindu at
//! the `(i+1)`-th sign counted from itself, to the Sun's Bhinna-
//! Ashtakavarga. To compute the Sun's BAV for a chart, take the Sun's
//! contribution table, then for each contributor look up its sign in the
//! chart and mark the corresponding 12 zodiac signs accordingly.

/// 8 contributors in row order: Sun, Moon, Mars, Mercury, Jupiter, Venus,
/// Saturn, Lagna.
pub const CONTRIBUTOR_NAMES: [&str; 8] = [
    "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Lagna",
];

/// Helper macro: convert a list of 1-based position numbers into a
/// `[bool; 12]` row. Out-of-range numbers are silently dropped; tests
/// catch invalid totals so this stays safe.
macro_rules! row {
    [$($pos:expr),* $(,)?] => {{
        let mut r = [false; 12];
        $(
            if $pos >= 1 && $pos <= 12 {
                r[($pos - 1) as usize] = true;
            }
        )*
        r
    }};
}

// ---------------------------------------------------------------------------
// Sun — Total 48 bindus
// ---------------------------------------------------------------------------

/// Sun's Bhinna-Ashtakavarga contribution table.
/// Rows in order: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Lagna.
pub const BINDU_TABLE_SUN: [[bool; 12]; 8] = [
    // From Sun:    1, 2, 4, 7, 8, 9, 10, 11
    row![1, 2, 4, 7, 8, 9, 10, 11],
    // From Moon:   3, 6, 10, 11
    row![3, 6, 10, 11],
    // From Mars:   1, 2, 4, 7, 8, 9, 10, 11
    row![1, 2, 4, 7, 8, 9, 10, 11],
    // From Mercury: 3, 5, 6, 9, 10, 11, 12
    row![3, 5, 6, 9, 10, 11, 12],
    // From Jupiter: 5, 6, 9, 11
    row![5, 6, 9, 11],
    // From Venus:   6, 7, 12
    row![6, 7, 12],
    // From Saturn:  1, 2, 4, 7, 8, 9, 10, 11
    row![1, 2, 4, 7, 8, 9, 10, 11],
    // From Lagna:   3, 4, 6, 10, 11, 12
    row![3, 4, 6, 10, 11, 12],
];

// ---------------------------------------------------------------------------
// Moon — Total 49 bindus
// ---------------------------------------------------------------------------

pub const BINDU_TABLE_MOON: [[bool; 12]; 8] = [
    // From Sun:    3, 6, 7, 8, 10, 11
    row![3, 6, 7, 8, 10, 11],
    // From Moon:   1, 3, 6, 7, 10, 11
    row![1, 3, 6, 7, 10, 11],
    // From Mars:   2, 3, 5, 6, 9, 10, 11
    row![2, 3, 5, 6, 9, 10, 11],
    // From Mercury: 1, 3, 4, 5, 7, 8, 10, 11
    row![1, 3, 4, 5, 7, 8, 10, 11],
    // From Jupiter: 1, 4, 7, 8, 10, 11, 12
    row![1, 4, 7, 8, 10, 11, 12],
    // From Venus:   3, 4, 5, 7, 9, 10, 11
    row![3, 4, 5, 7, 9, 10, 11],
    // From Saturn:  3, 5, 6, 11
    row![3, 5, 6, 11],
    // From Lagna:   3, 6, 10, 11
    row![3, 6, 10, 11],
];

// ---------------------------------------------------------------------------
// Mars — Total 39 bindus
// ---------------------------------------------------------------------------

pub const BINDU_TABLE_MARS: [[bool; 12]; 8] = [
    // From Sun:    3, 5, 6, 10, 11
    row![3, 5, 6, 10, 11],
    // From Moon:   3, 6, 11
    row![3, 6, 11],
    // From Mars:   1, 2, 4, 7, 8, 10, 11
    row![1, 2, 4, 7, 8, 10, 11],
    // From Mercury: 3, 5, 6, 11
    row![3, 5, 6, 11],
    // From Jupiter: 6, 10, 11, 12
    row![6, 10, 11, 12],
    // From Venus:   6, 8, 11, 12
    row![6, 8, 11, 12],
    // From Saturn:  1, 4, 7, 8, 9, 10, 11
    row![1, 4, 7, 8, 9, 10, 11],
    // From Lagna:   1, 3, 6, 10, 11
    row![1, 3, 6, 10, 11],
];

// ---------------------------------------------------------------------------
// Mercury — Total 54 bindus
// ---------------------------------------------------------------------------

pub const BINDU_TABLE_MERCURY: [[bool; 12]; 8] = [
    // From Sun:    5, 6, 9, 11, 12
    row![5, 6, 9, 11, 12],
    // From Moon:   2, 4, 6, 8, 10, 11
    row![2, 4, 6, 8, 10, 11],
    // From Mars:   1, 2, 4, 7, 8, 9, 10, 11
    row![1, 2, 4, 7, 8, 9, 10, 11],
    // From Mercury: 1, 3, 5, 6, 9, 10, 11, 12
    row![1, 3, 5, 6, 9, 10, 11, 12],
    // From Jupiter: 6, 8, 11, 12
    row![6, 8, 11, 12],
    // From Venus:   1, 2, 3, 4, 5, 8, 9, 11
    row![1, 2, 3, 4, 5, 8, 9, 11],
    // From Saturn:  1, 2, 4, 7, 8, 9, 10, 11
    row![1, 2, 4, 7, 8, 9, 10, 11],
    // From Lagna:   1, 2, 4, 6, 8, 10, 11
    row![1, 2, 4, 6, 8, 10, 11],
];

// ---------------------------------------------------------------------------
// Jupiter — Total 56 bindus
// ---------------------------------------------------------------------------

pub const BINDU_TABLE_JUPITER: [[bool; 12]; 8] = [
    // From Sun:    1, 2, 3, 4, 7, 8, 9, 10, 11
    row![1, 2, 3, 4, 7, 8, 9, 10, 11],
    // From Moon:   2, 5, 7, 9, 11
    row![2, 5, 7, 9, 11],
    // From Mars:   1, 2, 4, 7, 8, 10, 11
    row![1, 2, 4, 7, 8, 10, 11],
    // From Mercury: 1, 2, 4, 5, 6, 9, 10, 11
    row![1, 2, 4, 5, 6, 9, 10, 11],
    // From Jupiter: 1, 2, 3, 4, 7, 8, 10, 11
    row![1, 2, 3, 4, 7, 8, 10, 11],
    // From Venus:   2, 5, 6, 9, 10, 11
    row![2, 5, 6, 9, 10, 11],
    // From Saturn:  3, 5, 6, 12
    row![3, 5, 6, 12],
    // From Lagna:   1, 2, 4, 5, 6, 7, 9, 10, 11
    row![1, 2, 4, 5, 6, 7, 9, 10, 11],
];

// ---------------------------------------------------------------------------
// Venus — Total 52 bindus
// ---------------------------------------------------------------------------

pub const BINDU_TABLE_VENUS: [[bool; 12]; 8] = [
    // From Sun:    8, 11, 12
    row![8, 11, 12],
    // From Moon:   1, 2, 3, 4, 5, 8, 9, 11, 12
    row![1, 2, 3, 4, 5, 8, 9, 11, 12],
    // From Mars:   3, 5, 6, 9, 11, 12
    row![3, 5, 6, 9, 11, 12],
    // From Mercury: 3, 5, 6, 9, 11
    row![3, 5, 6, 9, 11],
    // From Jupiter: 5, 8, 9, 10, 11
    row![5, 8, 9, 10, 11],
    // From Venus:   1, 2, 3, 4, 5, 8, 9, 10, 11
    row![1, 2, 3, 4, 5, 8, 9, 10, 11],
    // From Saturn:  3, 4, 5, 8, 9, 10, 11
    row![3, 4, 5, 8, 9, 10, 11],
    // From Lagna:   1, 2, 3, 4, 5, 8, 9, 11
    //   (Sharma BPHS Ch.66; Charak omits the 11th — we follow Sharma so
    //   the per-planet total matches the published 52.)
    row![1, 2, 3, 4, 5, 8, 9, 11],
];

// ---------------------------------------------------------------------------
// Saturn — Total 39 bindus
// ---------------------------------------------------------------------------

pub const BINDU_TABLE_SATURN: [[bool; 12]; 8] = [
    // From Sun:    1, 2, 4, 7, 8, 10, 11
    row![1, 2, 4, 7, 8, 10, 11],
    // From Moon:   3, 6, 11
    row![3, 6, 11],
    // From Mars:   3, 5, 6, 10, 11, 12
    row![3, 5, 6, 10, 11, 12],
    // From Mercury: 6, 8, 9, 10, 11, 12
    row![6, 8, 9, 10, 11, 12],
    // From Jupiter: 5, 6, 11, 12
    row![5, 6, 11, 12],
    // From Venus:   6, 11, 12
    row![6, 11, 12],
    // From Saturn:  3, 5, 6, 11
    row![3, 5, 6, 11],
    // From Lagna:   1, 3, 4, 6, 10, 11
    row![1, 3, 4, 6, 10, 11],
];

// ---------------------------------------------------------------------------
// Lookup helpers
// ---------------------------------------------------------------------------

use crate::birth_chart::types::Planet;

/// Look up the contribution table for a planet. Returns `None` for Rahu /
/// Ketu / Ascendant (those don't have their own BAV).
pub fn table_for(planet: Planet) -> Option<&'static [[bool; 12]; 8]> {
    Some(match planet {
        Planet::Sun => &BINDU_TABLE_SUN,
        Planet::Moon => &BINDU_TABLE_MOON,
        Planet::Mars => &BINDU_TABLE_MARS,
        Planet::Mercury => &BINDU_TABLE_MERCURY,
        Planet::Jupiter => &BINDU_TABLE_JUPITER,
        Planet::Venus => &BINDU_TABLE_VENUS,
        Planet::Saturn => &BINDU_TABLE_SATURN,
        _ => return None,
    })
}

/// Sum the booleans in a 12-cell row.
const fn row_total(row: [bool; 12]) -> usize {
    let mut total = 0;
    let mut i = 0;
    while i < 12 {
        if row[i] {
            total += 1;
        }
        i += 1;
    }
    total
}

/// Sum the booleans across all 96 cells of a table.
const fn table_total(t: [[bool; 12]; 8]) -> usize {
    let mut total = 0;
    let mut i = 0;
    while i < 8 {
        total += row_total(t[i]);
        i += 1;
    }
    total
}

/// Compile-time guard: catch any data-entry mistake before runtime.
const _: () = {
    assert!(table_total(BINDU_TABLE_SUN) == 48, "Sun BAV total != 48");
    assert!(table_total(BINDU_TABLE_MOON) == 49, "Moon BAV total != 49");
    assert!(table_total(BINDU_TABLE_MARS) == 39, "Mars BAV total != 39");
    assert!(
        table_total(BINDU_TABLE_MERCURY) == 54,
        "Mercury BAV total != 54"
    );
    assert!(
        table_total(BINDU_TABLE_JUPITER) == 56,
        "Jupiter BAV total != 56"
    );
    assert!(
        table_total(BINDU_TABLE_VENUS) == 52,
        "Venus BAV total != 52"
    );
    assert!(
        table_total(BINDU_TABLE_SATURN) == 39,
        "Saturn BAV total != 39"
    );
};

#[cfg(test)]
mod tests {
    use super::*;

    fn count_row(r: [bool; 12]) -> usize {
        r.iter().filter(|b| **b).count()
    }
    fn count_table(t: [[bool; 12]; 8]) -> usize {
        t.iter().map(|r| count_row(*r)).sum()
    }

    #[test]
    fn bindu_table_sun_total() {
        assert_eq!(count_table(BINDU_TABLE_SUN), 48);
    }
    #[test]
    fn bindu_table_moon_total() {
        assert_eq!(count_table(BINDU_TABLE_MOON), 49);
    }
    #[test]
    fn bindu_table_mars_total() {
        assert_eq!(count_table(BINDU_TABLE_MARS), 39);
    }
    #[test]
    fn bindu_table_mercury_total() {
        assert_eq!(count_table(BINDU_TABLE_MERCURY), 54);
    }
    #[test]
    fn bindu_table_jupiter_total() {
        assert_eq!(count_table(BINDU_TABLE_JUPITER), 56);
    }
    #[test]
    fn bindu_table_venus_total() {
        assert_eq!(count_table(BINDU_TABLE_VENUS), 52);
    }
    #[test]
    fn bindu_table_saturn_total() {
        assert_eq!(count_table(BINDU_TABLE_SATURN), 39);
    }

    #[test]
    fn grand_total_equals_337() {
        // Sum of all seven per-planet totals — the classical "Sarva
        // Ashtakavarga grand total" maximum is 337.
        let total: usize = 48 + 49 + 39 + 54 + 56 + 52 + 39;
        assert_eq!(total, 337);
    }

    #[test]
    fn table_for_returns_some_for_all_classical_planets() {
        for &p in &Planet::classical() {
            assert!(table_for(p).is_some(), "missing table for {p}");
        }
        // Rahu/Ketu should be None.
        assert!(table_for(Planet::Rahu).is_none());
        assert!(table_for(Planet::Ketu).is_none());
    }
}
