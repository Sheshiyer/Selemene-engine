//! Ephemeris cache key namespace and invalidation scope.
//!
//! Engines that rely on planetary ephemeris data produce cache keys whose
//! results must be invalidated whenever the underlying ephemeris dataset
//! changes (e.g. a data file update, a precision correction, or a new
//! ephemeris epoch).  Non-ephemeris engines (biorhythm, numerology, …)
//! compute purely from birth-date arithmetic and are **not** affected.
//!
//! # Usage
//!
//! ```rust
//! use noesis_cache::ephemeris::{EPHEMERIS_DEPENDENT_PREFIXES, is_ephemeris_dependent};
//!
//! let key = "panchanga:a1b2c3d4";
//! assert!(is_ephemeris_dependent(key));
//!
//! let key2 = "numerology:deadbeef";
//! assert!(!is_ephemeris_dependent(key2));
//! ```

// ---------------------------------------------------------------------------
// Ephemeris-dependent engine cache key prefixes
// ---------------------------------------------------------------------------

/// Cache key prefixes for every engine whose output depends on planetary
/// ephemeris data.  Any cached result whose raw key starts with one of these
/// prefixes **must be invalidated** when the ephemeris dataset changes.
///
/// | Prefix          | Engine          | Why ephemeris-dependent?                        |
/// |-----------------|-----------------|--------------------------------------------------|
/// | `panchanga:`    | panchanga       | Tithi/nakshatra/yoga/karana from solar+lunar pos.|
/// | `vim:`          | vimshottari     | Moon nakshatra determines dasha lord             |
/// | `transits:`     | transits        | Real-time planetary transit positions            |
/// | `vedic-clock:`  | vedic-clock     | TCM/Vedic hour timing derived from planet pos.   |
/// | `biofield:`     | biofield        | Vedic-chart transit overlay on birth chart       |
/// | `hd:`           | human-design    | 64 gates mapped to planetary activations         |
/// | `gk:`           | gene-keys       | Shadow/Gift/Siddhi from planetary gate positions |
pub const EPHEMERIS_DEPENDENT_PREFIXES: &[&str] = &[
    "panchanga:",
    "vim:",
    "transits:",
    "vedic-clock:",
    "biofield:",
    "hd:",
    "gk:",
];

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Returns `true` if the supplied cache key belongs to an ephemeris-dependent
/// engine, meaning it must be invalidated when ephemeris data changes.
///
/// The check is a simple prefix match against [`EPHEMERIS_DEPENDENT_PREFIXES`].
#[inline]
pub fn is_ephemeris_dependent(key: &str) -> bool {
    EPHEMERIS_DEPENDENT_PREFIXES
        .iter()
        .any(|prefix| key.starts_with(prefix))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Representative cache keys produced by each ephemeris-dependent engine,
    // derived from inspecting each engine's `cache_key()` implementation.
    //
    // Format references:
    //   panchanga   : "panchanga:{md5_hex}"   (engine-panchanga/src/lib.rs)
    //   vimshottari : "vim:moon:{lng}:{date}" (engine-vimshottari/src/engine.rs)
    //   transits    : "transits:{sha256_hex}" (engine-transits/src/engine.rs)
    //   vedic-clock : "vedic-clock:h{n}:tz{off}:a{act}:t{tithi}:n{naksh}"
    //                                         (engine-vedic-clock/src/engine.rs)
    //   biofield    : "biofield:vedic:{sha256_hex}"
    //                                         (engine-biofield/src/engine.rs)
    //   human-design: "hd:{date_str}:{time_str}"
    //                                         (engine-human-design/src/engine.rs)
    //   gene-keys   : "gk:gates:{ps}:{pe}:{ds}:{de}"
    //                                         (engine-gene-keys/src/engine.rs)
    const REPRESENTATIVE_EPHEMERIS_KEYS: &[&str] = &[
        // panchanga
        "panchanga:a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
        // vimshottari
        "vim:moon:123.456789:2024-01-15",
        // transits
        "transits:deadbeef1234567890abcdef12345678deadbeef1234567890abcdef12345678",
        // vedic-clock
        "vedic-clock:h6:tz330:aNone:tNone:nNone",
        // biofield (vedic variant)
        "biofield:vedic:cafebabe1234567890abcdef12345678deadbeef1234567890abcdef12345678",
        // human-design
        "hd:1990-01-15:14:30:00",
        // gene-keys
        "gk:gates:1:2:3:4",
    ];

    // Representative keys for non-ephemeris engines that must NOT match.
    //
    // Format references:
    //   numerology       : "numerology:{md5_hex}"     (engine-numerology/src/lib.rs)
    //   nadabrahman      : "nadabrahman:h{n}:d{d}:r{r}:c{c}"
    //                                                  (engine-nadabrahman/src/engine.rs)
    //   face-reading     : "face-reading:mock:seed:{n}"
    //                                                  (engine-face-reading/src/engine.rs)
    //   biofield-capture : "{sha256_hex}" (no prefix)  (engine-biofield-capture/src/lib.rs)
    //   biorhythm        : "{sha256_hex}" (no prefix)  (engine-biorhythm/src/lib.rs)
    //   tarot/i-ching/…  : "{engine_id}:{sha256_hex}" (noesis-bridge/src/lib.rs)
    const REPRESENTATIVE_NON_EPHEMERIS_KEYS: &[&str] = &[
        "numerology:a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
        "nadabrahman:h14:d0:r3:c2",
        "face-reading:mock:seed:42",
        "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
        "tarot:deadbeef1234567890abcdef12345678deadbeef1234567890abcdef12345678",
        "i-ching:deadbeef1234567890abcdef12345678deadbeef1234567890abcdef12345678",
        "enneagram:deadbeef1234567890abcdef12345678deadbeef1234567890abcdef12345678",
        "sacred-geometry:deadbeef1234567890abcdef12345678deadbeef1234567890abcdef12345678",
        "sigil-forge:deadbeef1234567890abcdef12345678deadbeef1234567890abcdef12345678",
    ];

    #[test]
    fn test_all_ephemeris_dependent_keys_are_detected() {
        for key in REPRESENTATIVE_EPHEMERIS_KEYS {
            assert!(
                is_ephemeris_dependent(key),
                "Expected key to be ephemeris-dependent: {key}"
            );
        }
    }

    #[test]
    fn test_non_ephemeris_keys_are_not_detected() {
        for key in REPRESENTATIVE_NON_EPHEMERIS_KEYS {
            assert!(
                !is_ephemeris_dependent(key),
                "Expected key to NOT be ephemeris-dependent: {key}"
            );
        }
    }

    #[test]
    fn test_ephemeris_prefix_list_is_non_empty() {
        assert!(
            !EPHEMERIS_DEPENDENT_PREFIXES.is_empty(),
            "EPHEMERIS_DEPENDENT_PREFIXES must not be empty"
        );
    }

    #[test]
    fn test_every_prefix_ends_with_colon() {
        for prefix in EPHEMERIS_DEPENDENT_PREFIXES {
            assert!(
                prefix.ends_with(':'),
                "Prefix '{prefix}' should end with ':' to avoid false matches"
            );
        }
    }

    #[test]
    fn test_prefix_uniqueness() {
        let mut seen = std::collections::HashSet::new();
        for prefix in EPHEMERIS_DEPENDENT_PREFIXES {
            assert!(seen.insert(*prefix), "Duplicate prefix found: {prefix}");
        }
    }

    #[test]
    fn test_is_ephemeris_dependent_exact_prefix_boundary() {
        // "biofield:" should match, but "biofield-capture:" should NOT because
        // biofield-capture keys use a raw sha256 with no prefix at all.
        // This test guards against over-broad prefix matching.
        assert!(is_ephemeris_dependent("biofield:vedic:abc123"));
        assert!(!is_ephemeris_dependent("biofield-capture:abc123"));
    }

    #[test]
    fn test_panchanga_prefix_matches_engine_key_format() {
        // Mirrors the exact format from engine-panchanga/src/lib.rs:
        //   format!("panchanga:{:x}", md5_hash)
        let key = format!(
            "panchanga:{:x}",
            md5::compute("2024-01-15:12:00:12.972442:77.594562")
        );
        assert!(is_ephemeris_dependent(&key));
    }

    #[test]
    fn test_vimshottari_prefix_matches_engine_key_format() {
        // Mirrors engine-vimshottari/src/engine.rs:
        //   format!("vim:moon:{:.6}:{}", lng, date)
        let key = format!("vim:moon:{:.6}:{}", 135.456789_f64, "2024-01-15");
        assert!(is_ephemeris_dependent(&key));
    }

    #[test]
    fn test_transits_prefix_matches_engine_key_format() {
        // Mirrors engine-transits/src/engine.rs:
        //   format!("transits:{}", sha256_hex)
        let key = "transits:abc123def456abc123def456abc123def456abc123def456abc123def456abc1";
        assert!(is_ephemeris_dependent(key));
    }

    #[test]
    fn test_human_design_prefix_matches_engine_key_format() {
        // Mirrors engine-human-design/src/engine.rs:
        //   format!("hd:{date}:{time}")
        let key = "hd:1990-01-15:14:30:00";
        assert!(is_ephemeris_dependent(key));
    }

    #[test]
    fn test_gene_keys_prefix_matches_engine_key_format() {
        // Mirrors engine-gene-keys/src/engine.rs:
        //   format!("gk:gates:{}:{}:{}:{}", ps, pe, ds, de)
        let key = "gk:gates:11:26:43:10";
        assert!(is_ephemeris_dependent(key));
    }
}
