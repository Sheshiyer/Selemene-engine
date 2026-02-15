#!/usr/bin/env python3
"""
Generate 28 new API keys for Noesis Engine across tiers.

Outputs:
  1. SQL to delete ALL existing keys
  2. SQL to insert 28 new keys (hashed)
  3. A plaintext table of keys for the admin to save

Tier distribution:
  - 2 enterprise  (admin, all engines + admin perms)
  - 6 premium     (all engines, no admin)
  - 10 standard   (core engines only)
  - 10 free       (panchanga + numerology only)

Usage:
  python3 scripts/generate_api_keys.py
  # Then run the SQL output against your Supabase/Postgres DB
"""

import hashlib
import secrets
import string
import json
import sys
from datetime import datetime

ADMIN_USER_ID = "4242ec93-a2e7-4383-958f-cc7d1f089808"

def generate_key(length=32):
    """Generate a random alphanumeric key with nk_ prefix."""
    chars = string.ascii_letters + string.digits
    raw = ''.join(secrets.choice(chars) for _ in range(length))
    return f"nk_{raw}"

def sha256_hex(text):
    return hashlib.sha256(text.encode()).hexdigest()

# ── Tier definitions ──────────────────────────────────────────────
ALL_ENGINE_PERMS = [
    "basic:access",
    "panchanga:read", "panchanga:batch",
    "numerology:read",
    "biorhythm:read",
    "human-design:read",
    "gene-keys:read",
    "vimshottari:read",
    "vedic-clock:read",
    "biofield:read",
    "face-reading:read",
    "nadabrahman:read",
    "transits:read",
    "tarot:read",
    "i-ching:read",
    "enneagram:read",
    "sacred-geometry:read",
    "sigil-forge:read",
]

TIERS = {
    "enterprise": {
        "permissions": ALL_ENGINE_PERMS + ["admin:users", "admin:analytics"],
        "consciousness_level": 5,
        "rate_limit": 10000,
        "count": 2,
        "label": "All engines + admin:users + admin:analytics",
    },
    "premium": {
        "permissions": ALL_ENGINE_PERMS,
        "consciousness_level": 3,
        "rate_limit": 1000,
        "count": 6,
        "label": "All 16 engines (no admin)",
    },
    "standard": {
        "permissions": [
            "basic:access",
            "panchanga:read",
            "numerology:read",
            "biorhythm:read",
            "vedic-clock:read",
            "biofield:read",
            "nadabrahman:read",
            "face-reading:read",
            "tarot:read",
            "i-ching:read",
            "enneagram:read",
            "sacred-geometry:read",
        ],
        "consciousness_level": 1,
        "rate_limit": 500,
        "count": 10,
        "label": "12 core engines (no Vedic API engines)",
    },
    "free": {
        "permissions": [
            "basic:access",
            "panchanga:read",
            "numerology:read",
        ],
        "consciousness_level": 0,
        "rate_limit": 100,
        "count": 10,
        "label": "panchanga + numerology only",
    },
}

# ── Generate keys ─────────────────────────────────────────────────
keys = []
key_num = 0
for tier_name in ["enterprise", "premium", "standard", "free"]:
    spec = TIERS[tier_name]
    for i in range(spec["count"]):
        key_num += 1
        raw_key = generate_key()
        key_hash = sha256_hex(raw_key)
        keys.append({
            "num": key_num,
            "tier": tier_name,
            "raw_key": raw_key,
            "key_hash": key_hash,
            "permissions": spec["permissions"],
            "consciousness_level": spec["consciousness_level"],
            "rate_limit": spec["rate_limit"],
            "label": spec["label"],
        })

# ── Output: Step 1 — DELETE SQL ───────────────────────────────────
print("=" * 80)
print("  STEP 1: DELETE ALL EXISTING API KEYS")
print("  Run this SQL in Supabase SQL Editor first")
print("=" * 80)
print()
print("-- Delete all existing API keys for the admin user")
print(f"DELETE FROM api_keys WHERE user_id = '{ADMIN_USER_ID}';")
print()

# ── Output: Step 2 — INSERT SQL ───────────────────────────────────
print("=" * 80)
print("  STEP 2: INSERT NEW API KEYS")
print("  Run this SQL in Supabase SQL Editor after Step 1")
print("=" * 80)
print()
print(f"-- Insert {len(keys)} new API keys for admin user {ADMIN_USER_ID}")
print(f"-- Generated: {datetime.utcnow().isoformat()}Z")
print()

for k in keys:
    perms_json = json.dumps(k["permissions"])
    print(f"-- Key #{k['num']} [{k['tier']}]")
    print(f"INSERT INTO api_keys (key_hash, user_id, tier, permissions, consciousness_level, rate_limit, is_active)")
    print(f"VALUES ('{k['key_hash']}', '{ADMIN_USER_ID}', '{k['tier']}', '{perms_json}', {k['consciousness_level']}, {k['rate_limit']}, true);")
    print()

# ── Output: Step 3 — Plaintext Key Table ──────────────────────────
print("=" * 80)
print("  STEP 3: SAVE THIS TABLE — KEYS CANNOT BE RECOVERED")
print("=" * 80)
print()

# Markdown table
print("| # | Tier | Key | Permissions | Rate Limit |")
print("|---|------|-----|-------------|------------|")
for k in keys:
    print(f"| {k['num']} | {k['tier']} | `{k['raw_key']}` | {k['label']} | {k['rate_limit']}/hr |")

print()
print("=" * 80)
print("  TOTAL: {} keys ({} enterprise, {} premium, {} standard, {} free)".format(
    len(keys),
    TIERS["enterprise"]["count"],
    TIERS["premium"]["count"],
    TIERS["standard"]["count"],
    TIERS["free"]["count"],
))
print("=" * 80)
