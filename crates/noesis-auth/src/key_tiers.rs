//! Server-owned product entitlements. Administrator roles are deliberately separate.
use serde::Serialize;
use std::collections::BTreeSet;

/// Reviewed runtime catalog, pinned rather than granting new engines automatically.
pub const KEY_ENGINE_IDS: [&str; 19] = [
    "biofield",
    "biofield-capture",
    "biorhythm",
    "enneagram",
    "face-reading",
    "financial-biosensor",
    "gene-keys",
    "human-design",
    "i-ching",
    "nadabrahman",
    "numerology",
    "panchanga",
    "raaga",
    "sacred-geometry",
    "sigil-forge",
    "tarot",
    "transits",
    "vedic-clock",
    "vimshottari",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyTier {
    Basic,
    Pro,
    Enterprise,
}

#[derive(Clone, Debug, Serialize)]
pub struct KeyEntitlements {
    pub permissions: Vec<String>,
    pub consciousness_level: u8,
    pub rate_limit: u32,
}

impl KeyTier {
    pub const ALL: [Self; 3] = [Self::Basic, Self::Pro, Self::Enterprise];

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "basic" | "free" => Some(Self::Basic),
            "pro" | "premium" => Some(Self::Pro),
            "enterprise" => Some(Self::Enterprise),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Basic => "free",
            Self::Pro => "premium",
            Self::Enterprise => "enterprise",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Pro => "Pro",
            Self::Enterprise => "Enterprise",
        }
    }

    pub fn entitlements(self) -> KeyEntitlements {
        let engines: &[&str] = match self {
            Self::Basic => &["panchanga", "numerology"],
            _ => &KEY_ENGINE_IDS,
        };
        let mut permissions: BTreeSet<String> =
            engines.iter().map(|id| format!("{id}:read")).collect();
        permissions.insert("basic:access".into());
        let (consciousness_level, rate_limit) = match self {
            Self::Basic => (0, 60),
            Self::Pro => (3, 1000),
            Self::Enterprise => (5, 10000),
        };
        KeyEntitlements {
            permissions: permissions.into_iter().collect(),
            consciousness_level,
            rate_limit,
        }
    }

    /// Legacy optional fields can narrow a tier, never grant more than its bundle.
    pub fn restrict(
        self,
        permissions: Option<Vec<String>>,
        level: Option<i32>,
        rate: Option<i32>,
    ) -> Result<KeyEntitlements, &'static str> {
        let mut e = self.entitlements();
        if let Some(permissions) = permissions {
            if permissions.iter().any(|p| !e.permissions.contains(p)) {
                return Err("Requested permissions exceed the selected tier");
            }
            let mut narrowed: BTreeSet<_> = permissions.into_iter().collect();
            narrowed.insert("basic:access".into());
            e.permissions = narrowed.into_iter().collect();
        }
        if let Some(level) = level {
            if level < 0 || level > i32::from(e.consciousness_level) {
                return Err("Requested access level exceeds the selected tier");
            }
            e.consciousness_level = level as u8;
        }
        if let Some(rate) = rate {
            // Zero disables throttling and therefore is not a restriction.
            if rate <= 0 || rate as u32 > e.rate_limit {
                return Err("Requested rate limit exceeds the selected tier");
            }
            e.rate_limit = rate as u32;
        }
        Ok(e)
    }

    /// Preserve explicitly scoped service keys; fully managed bundles follow the tier.
    pub fn transition(
        self,
        managed: bool,
        permissions: &[String],
        level: i32,
        rate: i32,
    ) -> KeyEntitlements {
        let mut target = self.entitlements();
        let engine_permissions: BTreeSet<_> = permissions
            .iter()
            .filter(|p| !p.starts_with("admin:"))
            .cloned()
            .collect();
        if !managed {
            target
                .permissions
                .retain(|p| p == "basic:access" || engine_permissions.contains(p));
            target.consciousness_level =
                level.clamp(0, i32::from(target.consciousness_level)) as u8;
            if rate > 0 {
                target.rate_limit = target.rate_limit.min(rate as u32);
            }
        }
        target.permissions.extend(
            permissions
                .iter()
                .filter(|p| p.starts_with("admin:"))
                .cloned(),
        );
        target.permissions.sort();
        target.permissions.dedup();
        target
    }
}
