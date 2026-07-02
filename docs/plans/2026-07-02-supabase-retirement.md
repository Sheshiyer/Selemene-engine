# Supabase Retirement Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Retire Supabase and Discord OAuth by moving database runtime to Railway Postgres, human/admin auth to Cloudflare Zero Trust, raaga audio storage to Cloudflare R2, and runtime documentation/secrets to the new operating model.

**Architecture:** Treat Railway Postgres as the same Postgres schema already defined by root `migrations/`; Supabase is only the source database during migration. Cloudflare Access becomes the sole human/admin identity source by validating CF identity headers/JWTs in Rust middleware, mapping CF `sub` + `email` to local `users`, and mapping CF groups into `user_roles`. Existing internal machine routes remain narrow shared-secret carve-outs for this milestone: Dodo webhook forwarding and raaga clip upserts.

**Tech Stack:** Rust 2021, Axum, SQLx/Postgres, `jsonwebtoken`, `reqwest`, Next.js 16 admin app, Bun TypeScript scripts, Cloudflare Access/Zero Trust, Cloudflare R2 S3-compatible API, Railway Postgres, `pg_dump`, `pg_restore`, `psql`, shell.

---

## Execution Rules

- Use @writing-plans, @executing-plans, @test-driven-development, @systematic-debugging, @verification-before-completion, and @receiving-code-review when implementing.
- Work in a dedicated worktree created before execution. Do not implement directly from a dirty shared workspace.
- Make one small commit after each task passes its verification.
- Do not delete existing user changes. Current scan found unrelated dirty files: `crates/noesis-api/src/handlers/assets.rs` and `crates/noesis-api/tests/assets_generate_contract_test.rs`.
- Keep Option A for internal machine auth: `/internal/billing/events` keeps `X-Forward-Secret`; `/internal/raga/clip` keeps `x-internal-key`; `dodo_reconcile` keeps direct DB + Dodo API access.
- Do not add rollback code. The product decision is surgical replacement and retirement.
- Do not retain `oauth_accounts` compatibility after cutover. Drop the table once code no longer writes it.

## Locked Decisions

- Database migration uses one committed script under `scripts/` with `pg_dump | pg_restore` style passes: schema-only, data-only, critical-tables subset, full, and verification gates.
- Cloudflare identity uses `sub` as primary stable identity and `email` as secondary lookup/display data.
- One IdP only for this milestone.
- CF group names map directly into `user_roles.role`, except `selemene-admin` maps to `platform-admin`.
- Internal service routes are documented carve-outs, not CF-human-auth surfaces.
- `oauth_accounts` is dropped after cutover.
- Local development gets an explicit bypass with `RUST_ENV=development` plus a dev token path that yields platform-admin access.

## Recon Notes

- Rust uses Supabase mainly as ordinary Postgres through `DATABASE_URL`; there is no active Supabase Auth API integration in the Rust runtime.
- Discord OAuth lives in `crates/noesis-api/src/handlers/oauth.rs`, `apps/admin-web/src/lib/discord-oauth.ts`, `apps/admin-web/src/lib/api.ts`, and login/callback UI files.
- `oauth_accounts` lives in `migrations/007_oauth_accounts.sql`, `crates/noesis-data/src/models/oauth_account.rs`, and `crates/noesis-data/src/repositories/oauth_repository.rs`.
- Admin roles/state already exist in `migrations/010_user_roles_account_state.sql`, `crates/noesis-api/src/handlers/admin.rs`, and `crates/noesis-data/src/repositories/admin_repository.rs`.
- Raga storage has an existing mismatch: `migrations/028_raga_clips.sql` requires `suno_prompt` and `r2_key`, but `crates/noesis-api/src/handlers/raga.rs` currently inserts neither.
- `ts-engines/scripts/suno-smoke.ts` and `ts-engines/scripts/suno-bulk-gen.ts` upload to Supabase Storage via `@supabase/supabase-js`.
- The Supabase mirror migration tree is already drifting from root `migrations/`; root migrations are the schema authority.

## Verification Matrix

- Rust unit/route tests: `cargo test -p noesis-api auth_middleware -- --nocapture`, `cargo test -p noesis-api --test user_management_tests -- --test-threads=1`, `cargo test -p noesis-api --test route_inventory_tests`, `cargo test -p noesis-api --test billing_e2e_tests -- --test-threads=1` with `DATABASE_URL` when DB-backed.
- Auth crate: `cargo test -p noesis-auth --features postgres` with `TEST_DATABASE_URL` when testing DB-backed API key behavior.
- Full Rust check: `cargo check --workspace`.
- Admin web: `pnpm --dir apps/admin-web typecheck`, `pnpm --dir apps/admin-web lint`, `pnpm --dir apps/admin-web build`.
- TS engines: `bun --cwd ts-engines test`, `bun --cwd ts-engines run lint`.
- Migration script static checks: `bash -n scripts/supabase-to-railway-migrate.sh`.
- Migration script dry run: `scripts/supabase-to-railway-migrate.sh --dry-run --schema-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"`.
- Cutover smoke: `/health/live` returns 200; `/health/ready` reports DB ok; CF-authenticated `/api/v1/admin/session` returns expected roles.

---

### Task 1: Migration Script Skeleton

**Files:**
- Create: `scripts/supabase-to-railway-migrate.sh`
- Test: `scripts/supabase-to-railway-migrate.sh`

**Step 1: Write the failing static test command**

Run:

```bash
bash -n scripts/supabase-to-railway-migrate.sh
```

Expected: FAIL with `No such file or directory`.

**Step 2: Create the minimal script skeleton**

Create `scripts/supabase-to-railway-migrate.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/supabase-to-railway-migrate.sh --source URL --target URL [mode]

Modes:
  --schema-only       Restore schema only.
  --data-only         Restore data only.
  --critical-tables   Restore only the critical table subset.
  --full              Restore schema and data. Default.

Safety:
  --dry-run           Print commands without executing pg_dump/pg_restore/psql.
  --yes               Required for non-dry-run execution.
  --force             Allow non-empty target database after explicit confirmation.

Examples:
  scripts/supabase-to-railway-migrate.sh --dry-run --schema-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"
  scripts/supabase-to-railway-migrate.sh --yes --full --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"
USAGE
}

SOURCE_URL=""
TARGET_URL=""
MODE="full"
DRY_RUN=0
YES=0
FORCE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --source) SOURCE_URL="${2:-}"; shift 2 ;;
    --target) TARGET_URL="${2:-}"; shift 2 ;;
    --schema-only) MODE="schema-only"; shift ;;
    --data-only) MODE="data-only"; shift ;;
    --critical-tables) MODE="critical-tables"; shift ;;
    --full) MODE="full"; shift ;;
    --dry-run) DRY_RUN=1; shift ;;
    --yes) YES=1; shift ;;
    --force) FORCE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage; exit 2 ;;
  esac
done

if [[ -z "$SOURCE_URL" || -z "$TARGET_URL" ]]; then
  echo "--source and --target are required" >&2
  usage
  exit 2
fi

if [[ "$DRY_RUN" -eq 0 && "$YES" -eq 0 ]]; then
  echo "Refusing to run without --yes. Use --dry-run for planning." >&2
  exit 2
fi

echo "mode=$MODE dry_run=$DRY_RUN force=$FORCE"
echo "source=*** target=***"
```

**Step 3: Run static syntax check**

Run:

```bash
bash -n scripts/supabase-to-railway-migrate.sh
```

Expected: PASS with no output.

**Step 4: Run help output**

Run:

```bash
bash scripts/supabase-to-railway-migrate.sh --help
```

Expected: PASS and prints usage text including `--schema-only`, `--data-only`, `--critical-tables`, and `--full`.

**Step 5: Commit**

```bash
git add scripts/supabase-to-railway-migrate.sh
git commit -m "chore: add Supabase to Railway migration script skeleton"
```

---

### Task 2: Migration Script Safety Gates

**Files:**
- Modify: `scripts/supabase-to-railway-migrate.sh`
- Test: `scripts/supabase-to-railway-migrate.sh`

**Step 1: Write failing safety checks**

Run:

```bash
bash scripts/supabase-to-railway-migrate.sh --source postgres://source --target postgres://target
```

Expected: FAIL with `Refusing to run without --yes`.

Run:

```bash
bash scripts/supabase-to-railway-migrate.sh --dry-run --source postgres://same --target postgres://same
```

Expected now: PASS, but after implementation this must FAIL because source and target are identical.

**Step 2: Add safety helpers**

Add this after argument parsing and before printing mode:

```bash
redact_url() {
  local url="$1"
  printf '%s\n' "$url" | sed -E 's#(postgres(ql)?://)[^:@/]+(:[^@/]+)?@#\1***:***@#'
}

require_command() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "Required command not found: $name" >&2
    exit 2
  fi
}

validate_postgres_url() {
  local label="$1"
  local url="$2"
  if [[ ! "$url" =~ ^postgres(ql)?:// ]]; then
    echo "$label must start with postgres:// or postgresql://" >&2
    exit 2
  fi
}

validate_postgres_url "--source" "$SOURCE_URL"
validate_postgres_url "--target" "$TARGET_URL"

if [[ "$SOURCE_URL" == "$TARGET_URL" ]]; then
  echo "Refusing to use identical source and target URLs" >&2
  exit 2
fi

require_command pg_dump
require_command pg_restore
require_command psql
```

Replace the final redacted print with:

```bash
echo "source=$(redact_url "$SOURCE_URL")"
echo "target=$(redact_url "$TARGET_URL")"
```

**Step 3: Run safety checks**

Run:

```bash
bash scripts/supabase-to-railway-migrate.sh --dry-run --source postgres://same --target postgres://same
```

Expected: FAIL with `Refusing to use identical source and target URLs`.

Run:

```bash
bash scripts/supabase-to-railway-migrate.sh --dry-run --source mysql://source --target postgres://target
```

Expected: FAIL with `--source must start with postgres:// or postgresql://`.

**Step 4: Run syntax check**

Run:

```bash
bash -n scripts/supabase-to-railway-migrate.sh
```

Expected: PASS.

**Step 5: Commit**

```bash
git add scripts/supabase-to-railway-migrate.sh
git commit -m "chore: add migration script safety gates"
```

---

### Task 3: Migration Script Restore Modes

**Files:**
- Modify: `scripts/supabase-to-railway-migrate.sh`
- Test: `scripts/supabase-to-railway-migrate.sh`

**Step 1: Run dry-run mode before implementation**

Run:

```bash
bash scripts/supabase-to-railway-migrate.sh --dry-run --schema-only --source postgres://u:p@source/db --target postgres://u:p@target/db
```

Expected now: output only shows mode/source/target; no `pg_dump` or `pg_restore` command is printed.

**Step 2: Add command builder**

Add this near the top after flags:

```bash
CRITICAL_TABLES=(
  users
  user_profiles
  api_keys
  user_roles
  user_account_state
  billing_subscriptions
  processed_webhook_events
  usage_logs
  readings
  biofield_sessions
  biofield_measurements
  raga_clips
  reconcile_runs
)
```

Add these helpers after validation helpers:

```bash
run_cmd() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf '+ '
    printf '%q ' "$@"
    printf '\n'
    return 0
  fi
  "$@"
}

run_sql() {
  local sql="$1"
  run_cmd psql "$TARGET_URL" -v ON_ERROR_STOP=1 -c "$sql"
}

restore_schema_only() {
  run_cmd bash -c 'pg_dump --format=custom --no-owner --no-acl --schema-only "$1" | pg_restore --no-owner --no-acl --clean --if-exists --dbname "$2"' _ "$SOURCE_URL" "$TARGET_URL"
}

restore_data_only() {
  run_cmd bash -c 'pg_dump --format=custom --no-owner --no-acl --data-only "$1" | pg_restore --no-owner --no-acl --disable-triggers --dbname "$2"' _ "$SOURCE_URL" "$TARGET_URL"
}

restore_critical_tables() {
  local args=()
  for table in "${CRITICAL_TABLES[@]}"; do
    args+=(--table "public.$table")
  done
  run_cmd pg_dump --format=custom --no-owner --no-acl --data-only "${args[@]}" --file /tmp/noesis-critical-tables.dump "$SOURCE_URL"
  run_cmd pg_restore --no-owner --no-acl --disable-triggers --dbname "$TARGET_URL" /tmp/noesis-critical-tables.dump
}

restore_full() {
  restore_schema_only
  restore_data_only
}
```

Add mode dispatch at the bottom:

```bash
case "$MODE" in
  schema-only) restore_schema_only ;;
  data-only) restore_data_only ;;
  critical-tables) restore_critical_tables ;;
  full) restore_full ;;
  *) echo "Unsupported mode: $MODE" >&2; exit 2 ;;
esac
```

**Step 3: Run dry-run mode checks**

Run:

```bash
bash scripts/supabase-to-railway-migrate.sh --dry-run --schema-only --source postgres://u:p@source/db --target postgres://u:p@target/db
```

Expected: PASS and prints a `pg_dump --format=custom --no-owner --no-acl --schema-only` pipeline to `pg_restore`.

Run:

```bash
bash scripts/supabase-to-railway-migrate.sh --dry-run --critical-tables --source postgres://u:p@source/db --target postgres://u:p@target/db
```

Expected: PASS and prints `--table public.users`, `--table public.user_roles`, and `--table public.raga_clips`.

**Step 4: Run syntax check**

Run:

```bash
bash -n scripts/supabase-to-railway-migrate.sh
```

Expected: PASS.

**Step 5: Commit**

```bash
git add scripts/supabase-to-railway-migrate.sh
git commit -m "chore: add migration restore modes"
```

---

### Task 4: Migration Script Verification Gates

**Files:**
- Modify: `scripts/supabase-to-railway-migrate.sh`
- Test: `scripts/supabase-to-railway-migrate.sh`

**Step 1: Run dry-run before verification exists**

Run:

```bash
bash scripts/supabase-to-railway-migrate.sh --dry-run --full --source postgres://u:p@source/db --target postgres://u:p@target/db
```

Expected now: no row-count or partition verification commands are printed.

**Step 2: Add verification helpers**

Add this after restore helpers:

```bash
verify_target() {
  echo "Verifying target database"
  run_sql "SELECT 1 AS target_ok;"
  run_sql "SELECT ensure_usage_log_partitions(3);"
  run_sql "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name IN ('users', 'api_keys', 'user_roles', 'user_account_state', 'billing_subscriptions', 'usage_logs', 'raga_clips', 'reconcile_runs') ORDER BY table_name;"
  run_sql "SELECT 'users' AS table_name, count(*) FROM users UNION ALL SELECT 'api_keys', count(*) FROM api_keys UNION ALL SELECT 'user_roles', count(*) FROM user_roles UNION ALL SELECT 'user_account_state', count(*) FROM user_account_state UNION ALL SELECT 'raga_clips', count(*) FROM raga_clips;"
  run_sql "SELECT inhrelid::regclass::text AS partition_name FROM pg_inherits WHERE inhparent = 'usage_logs'::regclass ORDER BY 1;"
}
```

Call it after the mode dispatch:

```bash
verify_target
```

**Step 3: Run dry-run verification check**

Run:

```bash
bash scripts/supabase-to-railway-migrate.sh --dry-run --full --source postgres://u:p@source/db --target postgres://u:p@target/db
```

Expected: PASS and prints `SELECT ensure_usage_log_partitions(3);`, critical table counts, and usage partition query.

**Step 4: Run syntax check**

Run:

```bash
bash -n scripts/supabase-to-railway-migrate.sh
```

Expected: PASS.

**Step 5: Commit**

```bash
git add scripts/supabase-to-railway-migrate.sh
git commit -m "chore: verify Railway migration target"
```

---

### Task 5: Cloudflare Auth Types and Config

**Files:**
- Modify: `crates/noesis-api/Cargo.toml`
- Modify: `crates/noesis-api/src/config.rs:13-247`
- Create: `crates/noesis-api/src/cf_access.rs`
- Modify: `crates/noesis-api/src/lib.rs:1-80`
- Test: `crates/noesis-api/src/cf_access.rs`

**Step 1: Write tests for role mapping and dev bypass token parsing**

Create `crates/noesis-api/src/cf_access.rs` with tests first:

```rust
use noesis_auth::AuthUser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CfIdentity {
    pub sub: String,
    pub email: String,
    pub groups: Vec<String>,
}

pub fn map_cf_group_to_role(group: &str) -> String {
    todo!("map CF group to local role")
}

pub fn roles_from_cf_groups(groups: &[String]) -> Vec<String> {
    todo!("map CF groups to user_roles values")
}

pub fn development_auth_user(dev_token: &str, provided: Option<&str>) -> Option<AuthUser> {
    todo!("return platform-admin user when dev token matches")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_selemene_admin_to_platform_admin() {
        assert_eq!(map_cf_group_to_role("selemene-admin"), "platform-admin");
    }

    #[test]
    fn maps_cf_groups_directly_otherwise() {
        assert_eq!(map_cf_group_to_role("support"), "support");
    }

    #[test]
    fn filters_duplicate_empty_and_unsupported_roles() {
        let groups = vec![
            "".to_string(),
            "support".to_string(),
            "support".to_string(),
            "selemene-admin".to_string(),
            "random-group".to_string(),
        ];
        assert_eq!(roles_from_cf_groups(&groups), vec!["platform-admin", "support"]);
    }

    #[test]
    fn development_auth_requires_matching_token() {
        assert!(development_auth_user("dev-secret", Some("wrong")).is_none());
        let user = development_auth_user("dev-secret", Some("dev-secret")).expect("dev user");
        assert_eq!(user.user_id, "00000000-0000-0000-0000-000000000001");
        assert!(user.permissions.contains(&"admin:system:read".to_string()));
        assert_eq!(user.tier, "enterprise");
    }
}
```

**Step 2: Run tests to verify failure**

Run:

```bash
cargo test -p noesis-api cf_access -- --nocapture
```

Expected: FAIL with `not yet implemented` from the `todo!` calls.

**Step 3: Implement minimal type/config module**

Replace the `todo!` functions with:

```rust
use std::collections::BTreeSet;

const SUPPORTED_ROLES: &[&str] = &["viewer", "support", "admin", "platform-admin"];

pub fn map_cf_group_to_role(group: &str) -> String {
    match group.trim() {
        "selemene-admin" => "platform-admin".to_string(),
        other => other.to_string(),
    }
}

pub fn roles_from_cf_groups(groups: &[String]) -> Vec<String> {
    let supported: BTreeSet<&str> = SUPPORTED_ROLES.iter().copied().collect();
    let mut roles = groups
        .iter()
        .map(|group| map_cf_group_to_role(group))
        .filter(|role| supported.contains(role.as_str()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if roles.is_empty() {
        roles.push("viewer".to_string());
    }

    roles
}

fn permissions_for_roles(roles: &[String]) -> Vec<String> {
    let mut permissions = BTreeSet::from(["basic:access".to_string()]);

    if roles.iter().any(|role| role == "platform-admin" || role == "admin") {
        permissions.extend([
            "admin:users".to_string(),
            "admin:analytics".to_string(),
            "admin:system:read".to_string(),
            "admin:audit:list".to_string(),
            "admin:audit:read".to_string(),
        ]);
    }

    permissions.into_iter().collect()
}

pub fn development_auth_user(dev_token: &str, provided: Option<&str>) -> Option<AuthUser> {
    if dev_token.is_empty() || provided != Some(dev_token) {
        return None;
    }

    let roles = vec!["platform-admin".to_string()];
    Some(AuthUser {
        user_id: "00000000-0000-0000-0000-000000000001".to_string(),
        tier: "enterprise".to_string(),
        permissions: permissions_for_roles(&roles),
        rate_limit: 10_000,
        consciousness_level: 5,
        jti: None,
        token_exp: None,
    })
}
```

Add the module to `crates/noesis-api/src/lib.rs` near the other module declarations:

```rust
pub mod cf_access;
```

Update `crates/noesis-api/src/config.rs` struct with:

```rust
    /// Cloudflare Access issuer URL, usually https://<team>.cloudflareaccess.com
    pub cf_access_issuer: Option<String>,

    /// Cloudflare Access application audience tag.
    pub cf_access_audience: Option<String>,

    /// Local development bypass token. Only honored outside production.
    pub cf_dev_bypass_token: Option<String>,
```

Read env vars in `from_env()` after Dodo envs:

```rust
        let cf_access_issuer = env::var("CF_ACCESS_ISSUER").ok();
        let cf_access_audience = env::var("CF_ACCESS_AUDIENCE").ok();
        let cf_dev_bypass_token = env::var("CF_DEV_BYPASS_TOKEN").ok();
```

Add the fields to the `Ok(Self { ... })` initializer.

**Step 4: Run targeted tests**

Run:

```bash
cargo test -p noesis-api cf_access -- --nocapture
```

Expected: PASS; 4 tests pass.

**Step 5: Commit**

```bash
git add crates/noesis-api/Cargo.toml crates/noesis-api/src/config.rs crates/noesis-api/src/cf_access.rs crates/noesis-api/src/lib.rs
git commit -m "feat: add Cloudflare Access auth primitives"
```

---

### Task 6: Cloudflare JWT Validation

**Files:**
- Modify: `crates/noesis-api/Cargo.toml`
- Modify: `crates/noesis-api/src/cf_access.rs`
- Test: `crates/noesis-api/src/cf_access.rs`

**Step 1: Write failing claim extraction tests**

Add to `cf_access.rs`:

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CfAccessClaims {
    pub sub: String,
    pub email: Option<String>,
    pub aud: serde_json::Value,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub identity_nonce: Option<String>,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

pub fn identity_from_claims(claims: CfAccessClaims) -> Result<CfIdentity, String> {
    todo!("extract required CF identity")
}
```

Add tests:

```rust
    #[test]
    fn extracts_identity_from_claims() {
        let claims = CfAccessClaims {
            sub: "cf-sub-123".to_string(),
            email: Some("USER@Example.COM".to_string()),
            aud: serde_json::json!(["aud"]),
            groups: vec!["support".to_string()],
            identity_nonce: None,
            exp: 4_102_444_800,
            iat: 1,
            iss: "https://team.cloudflareaccess.com".to_string(),
        };

        let identity = identity_from_claims(claims).expect("identity");
        assert_eq!(identity.sub, "cf-sub-123");
        assert_eq!(identity.email, "user@example.com");
        assert_eq!(identity.groups, vec!["support"]);
    }

    #[test]
    fn rejects_claims_without_email() {
        let claims = CfAccessClaims {
            sub: "cf-sub-123".to_string(),
            email: None,
            aud: serde_json::json!(["aud"]),
            groups: vec![],
            identity_nonce: None,
            exp: 4_102_444_800,
            iat: 1,
            iss: "https://team.cloudflareaccess.com".to_string(),
        };

        assert_eq!(identity_from_claims(claims).unwrap_err(), "Cloudflare identity email missing");
    }
```

**Step 2: Run tests to verify failure**

Run:

```bash
cargo test -p noesis-api cf_access -- --nocapture
```

Expected: FAIL on `identity_from_claims` todo.

**Step 3: Add JWT/JWKS dependencies and implementation skeleton**

In `crates/noesis-api/Cargo.toml`, ensure these dependencies exist:

```toml
jsonwebtoken = "9.3"
```

Implement `identity_from_claims`:

```rust
pub fn identity_from_claims(claims: CfAccessClaims) -> Result<CfIdentity, String> {
    let email = claims
        .email
        .map(|email| email.trim().to_ascii_lowercase())
        .filter(|email| !email.is_empty())
        .ok_or_else(|| "Cloudflare identity email missing".to_string())?;

    if claims.sub.trim().is_empty() {
        return Err("Cloudflare identity sub missing".to_string());
    }

    Ok(CfIdentity {
        sub: claims.sub,
        email,
        groups: claims.groups,
    })
}
```

Add a validator stub that will be wired in the next task:

```rust
#[derive(Debug, Clone)]
pub struct CfAccessValidator {
    issuer: String,
    audience: String,
}

impl CfAccessValidator {
    pub fn new(issuer: String, audience: String) -> Self {
        Self { issuer, audience }
    }

    pub async fn validate_token(&self, token: &str) -> Result<CfIdentity, String> {
        if token.trim().is_empty() {
            return Err("Cloudflare Access token missing".to_string());
        }

        Err(format!(
            "Cloudflare Access JWT validation not fully wired for issuer {} and audience {}",
            self.issuer, self.audience
        ))
    }
}
```

**Step 4: Run targeted tests**

Run:

```bash
cargo test -p noesis-api cf_access -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-api/Cargo.toml crates/noesis-api/src/cf_access.rs
git commit -m "feat: parse Cloudflare Access identity claims"
```

---

### Task 7: CF Identity JIT User Resolution

**Files:**
- Modify: `crates/noesis-data/src/repositories/user_repository.rs`
- Modify: `crates/noesis-api/src/cf_access.rs`
- Test: `crates/noesis-api/src/cf_access.rs`

**Step 1: Write failing repository API shape**

Add this test to `cf_access.rs` as a pure unit for the SQL-independent AuthUser conversion:

```rust
pub fn auth_user_from_parts(user_id: uuid::Uuid, tier: &str, consciousness_level: i32, roles: &[String]) -> AuthUser {
    todo!("build AuthUser from local user and mapped roles")
}

#[cfg(test)]
mod auth_user_tests {
    use super::*;

    #[test]
    fn auth_user_from_parts_uses_roles_for_permissions() {
        let user_id = uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let roles = vec!["platform-admin".to_string()];
        let user = auth_user_from_parts(user_id, "premium", 4, &roles);

        assert_eq!(user.user_id, user_id.to_string());
        assert_eq!(user.tier, "premium");
        assert_eq!(user.consciousness_level, 4);
        assert!(user.permissions.contains(&"admin:system:read".to_string()));
    }
}
```

**Step 2: Run test to verify failure**

Run:

```bash
cargo test -p noesis-api auth_user_from_parts -- --nocapture
```

Expected: FAIL on `todo!`.

**Step 3: Implement AuthUser builder and repository method**

Implement in `cf_access.rs`:

```rust
pub fn auth_user_from_parts(user_id: uuid::Uuid, tier: &str, consciousness_level: i32, roles: &[String]) -> AuthUser {
    AuthUser {
        user_id: user_id.to_string(),
        tier: tier.to_string(),
        permissions: permissions_for_roles(roles),
        rate_limit: match tier {
            "enterprise" => 10_000,
            "premium" => 1_000,
            "free" => 60,
            _ => 10,
        },
        consciousness_level: consciousness_level.clamp(0, 5) as u8,
        jti: None,
        token_exp: None,
    }
}
```

Add to `crates/noesis-data/src/repositories/user_repository.rs`:

```rust
    pub async fn find_or_create_cloudflare_user(
        &self,
        email: &str,
        cf_sub: &str,
    ) -> Result<User, Error> {
        let normalized_email = email.trim().to_ascii_lowercase();
        let full_name = normalized_email
            .split('@')
            .next()
            .filter(|name| !name.is_empty())
            .unwrap_or("Cloudflare User");

        let mut tx = self.pool.begin().await?;
        let existing = sqlx::query_as::<_, User>(
            r#"
            SELECT
                id,
                email,
                COALESCE(password_hash, '') AS password_hash,
                COALESCE(full_name, '') AS full_name,
                COALESCE(tier, 'free') AS tier,
                COALESCE(consciousness_level, 0) AS consciousness_level,
                COALESCE(experience_points, 0) AS experience_points,
                created_at,
                updated_at,
                reset_token,
                reset_token_expires_at,
                last_login_at,
                COALESCE(failed_login_attempts, 0) AS failed_login_attempts,
                locked_until
            FROM users
            WHERE lower(btrim(email)) = lower(btrim($1))
            "#,
        )
        .bind(&normalized_email)
        .fetch_optional(&mut *tx)
        .await?;

        let user = if let Some(user) = existing {
            user
        } else {
            sqlx::query_as::<_, User>(
                r#"
                INSERT INTO users (id, email, password_hash, full_name, tier, consciousness_level, created_at, updated_at)
                VALUES ($1, $2, $3, $4, 'free', 0, NOW(), NOW())
                RETURNING *
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(&normalized_email)
            .bind(format!("cf-access:{}", cf_sub))
            .bind(full_name)
            .fetch_one(&mut *tx)
            .await?
        };

        sqlx::query(
            r#"
            INSERT INTO user_account_state (user_id, state)
            VALUES ($1, 'active')
            ON CONFLICT (user_id) DO NOTHING
            "#,
        )
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }
```

**Step 4: Run targeted tests**

Run:

```bash
cargo test -p noesis-api auth_user_from_parts -- --nocapture
```

Expected: PASS.

Run:

```bash
cargo check -p noesis-data
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-data/src/repositories/user_repository.rs crates/noesis-api/src/cf_access.rs
git commit -m "feat: resolve Cloudflare identities to local users"
```

---

### Task 8: CF Role Sync

**Files:**
- Modify: `crates/noesis-data/src/repositories/admin_repository.rs`
- Modify: `crates/noesis-api/src/cf_access.rs`
- Test: `crates/noesis-api/src/cf_access.rs`

**Step 1: Write the failing sync contract test**

Add to `cf_access.rs`:

```rust
pub fn role_values_for_sql(groups: &[String]) -> Vec<String> {
    todo!("return deterministic DB role values")
}

#[cfg(test)]
mod role_sql_tests {
    use super::*;

    #[test]
    fn role_values_for_sql_are_deterministic() {
        let groups = vec!["support".to_string(), "selemene-admin".to_string(), "admin".to_string()];
        assert_eq!(role_values_for_sql(&groups), vec!["admin", "platform-admin", "support"]);
    }
}
```

**Step 2: Run test to verify failure**

Run:

```bash
cargo test -p noesis-api role_values_for_sql -- --nocapture
```

Expected: FAIL on `todo!`.

**Step 3: Implement deterministic role values and DB sync**

Implement in `cf_access.rs`:

```rust
pub fn role_values_for_sql(groups: &[String]) -> Vec<String> {
    roles_from_cf_groups(groups)
}
```

Add to `crates/noesis-data/src/repositories/admin_repository.rs`:

```rust
    pub async fn replace_user_roles_from_cloudflare(
        &self,
        user_id: Uuid,
        roles: &[String],
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        for role in roles {
            sqlx::query(
                r#"
                INSERT INTO user_roles (user_id, role)
                VALUES ($1, $2)
                ON CONFLICT (user_id, role) DO NOTHING
                "#,
            )
            .bind(user_id)
            .bind(role)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
```

**Step 4: Run checks**

Run:

```bash
cargo test -p noesis-api role_values_for_sql -- --nocapture
cargo check -p noesis-data
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-data/src/repositories/admin_repository.rs crates/noesis-api/src/cf_access.rs
git commit -m "feat: sync Cloudflare groups into user roles"
```

---

### Task 9: Wire CF Auth Into Middleware

**Files:**
- Modify: `crates/noesis-api/src/lib.rs:408-430`
- Modify: `crates/noesis-api/src/middleware.rs:81-153`
- Modify: `crates/noesis-api/src/cf_access.rs`
- Test: `crates/noesis-api/src/middleware.rs`

**Step 1: Write failing middleware tests for dev bypass**

Add to the test module in `crates/noesis-api/src/middleware.rs` or create one if absent:

```rust
#[cfg(test)]
mod cf_auth_tests {
    use super::*;

    #[test]
    fn extracts_dev_bypass_header() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-noesis-dev-auth", "dev-secret".parse().unwrap());
        assert_eq!(headers.get("x-noesis-dev-auth").and_then(|v| v.to_str().ok()), Some("dev-secret"));
    }
}
```

**Step 2: Run test to verify baseline**

Run:

```bash
cargo test -p noesis-api cf_auth_tests -- --nocapture
```

Expected: PASS because this only proves header parsing before wiring.

**Step 3: Extend AppState and middleware**

Add to `AppState` in `crates/noesis-api/src/lib.rs`:

```rust
    pub cf_access_validator: Option<Arc<crate::cf_access::CfAccessValidator>>,
    pub cf_dev_bypass_token: Option<String>,
```

Populate these fields in both `build_app_state` and `build_app_state_lazy_db` from `ApiConfig`.

Change the router auth layer to pass `AppState` instead of only `Arc<AuthService>` if needed. The middleware needs repositories and CF config to JIT users. Preferred minimal path:

```rust
.route_layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth_middleware))
```

Update middleware signature:

```rust
pub async fn auth_middleware(
    State(state): State<crate::AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
```

Update existing JWT/API key calls to use `state.auth` instead of `auth`.

Before legacy JWT/API-key checks, add dev bypass only outside production:

```rust
    let is_development = std::env::var("RUST_ENV").as_deref() == Ok("development");
    if is_development {
        if let Some(expected) = state.cf_dev_bypass_token.as_deref() {
            let provided = req.headers().get("x-noesis-dev-auth").and_then(|v| v.to_str().ok());
            if let Some(user) = crate::cf_access::development_auth_user(expected, provided) {
                req.extensions_mut().insert(user);
                return Ok(next.run(req).await);
            }
        }
    }
```

Then add CF token extraction before legacy JWT validation:

```rust
    if let Some(validator) = state.cf_access_validator.as_ref() {
        let cf_token = req
            .headers()
            .get("cf-authorization")
            .or_else(|| req.headers().get("CF_Authorization"))
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if let Some(token) = cf_token {
            match validator.validate_token(token).await {
                Ok(identity) => {
                    let user = state.user_repository.find_or_create_cloudflare_user(&identity.email, &identity.sub).await.map_err(|e| {
                        ErrorMapper::response(
                            StatusCode::UNAUTHORIZED,
                            "UNAUTHORIZED",
                            "Cloudflare identity could not be resolved",
                            Some(serde_json::json!({ "auth_method": "cloudflare", "error": e.to_string() })),
                        )
                    })?;
                    let roles = crate::cf_access::role_values_for_sql(&identity.groups);
                    if let Some(repo) = state.admin_repository.as_ref() {
                        repo.replace_user_roles_from_cloudflare(user.id, &roles).await.map_err(|e| {
                            ErrorMapper::response(
                                StatusCode::UNAUTHORIZED,
                                "UNAUTHORIZED",
                                "Cloudflare roles could not be synchronized",
                                Some(serde_json::json!({ "auth_method": "cloudflare", "error": e.to_string() })),
                            )
                        })?;
                    }
                    req.extensions_mut().insert(crate::cf_access::auth_user_from_parts(user.id, &user.tier, user.consciousness_level, &roles));
                    return Ok(next.run(req).await);
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Cloudflare Access validation failed");
                    return Err(ErrorMapper::response(
                        StatusCode::UNAUTHORIZED,
                        "UNAUTHORIZED",
                        "Invalid Cloudflare Access token",
                        Some(serde_json::json!({ "auth_method": "cloudflare" })),
                    ));
                }
            }
        }
    }
```

Keep legacy JWT/API key validation after CF to avoid breaking existing API key clients until a follow-up explicitly slims `noesis-auth`.

**Step 4: Run checks**

Run:

```bash
cargo check -p noesis-api
cargo test -p noesis-api cf_auth_tests -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-api/src/lib.rs crates/noesis-api/src/middleware.rs crates/noesis-api/src/cf_access.rs
git commit -m "feat: authenticate requests with Cloudflare Access"
```

---

### Task 10: Retire Discord OAuth API Routes

**Files:**
- Modify: `crates/noesis-api/src/lib.rs:699-714`
- Modify: `crates/noesis-api/src/handlers/oauth.rs`
- Modify: `crates/noesis-api/tests/route_inventory_tests.rs`
- Test: `crates/noesis-api/tests/route_inventory_tests.rs`

**Step 1: Update route inventory test first**

Modify `crates/noesis-api/tests/route_inventory_tests.rs` so `/api/v1/auth/discord/authorize` and `/api/v1/auth/discord/callback` are not expected active routes. If the test asserts a fixed count, reduce it accordingly.

**Step 2: Run route test to verify failure**

Run:

```bash
cargo test -p noesis-api --test route_inventory_tests -- --nocapture
```

Expected: FAIL because the implementation still registers Discord routes.

**Step 3: Remove Discord route registration**

In `crates/noesis-api/src/lib.rs`, remove route registrations for:

```rust
/api/v1/auth/discord/authorize
/api/v1/auth/discord/callback
```

Leave `handlers/oauth.rs` in place only if OpenAPI or tests still compile against it during this task. If dead after compile, remove its module export in a later cleanup task.

**Step 4: Run route test and compile**

Run:

```bash
cargo test -p noesis-api --test route_inventory_tests -- --nocapture
cargo check -p noesis-api
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-api/src/lib.rs crates/noesis-api/src/handlers/oauth.rs crates/noesis-api/tests/route_inventory_tests.rs
git commit -m "feat: retire Discord OAuth API routes"
```

---

### Task 11: Deprecate Password Login Issuance

**Files:**
- Modify: `crates/noesis-api/src/handlers/auth.rs:171-266`
- Modify: `crates/noesis-api/tests/user_management_tests.rs:295-409`
- Test: `crates/noesis-api/tests/user_management_tests.rs`

**Step 1: Write failing tests for disabled login**

In `user_management_tests.rs`, update login tests to expect `410 Gone` or `401 Unauthorized` with a stable error code when password login is disabled. Use `410 Gone` for clarity.

Expected assertion shape:

```rust
assert_eq!(status, StatusCode::GONE);
assert_eq!(body["error_code"], "AUTH_RETIRED");
```

**Step 2: Run test to verify failure**

Run:

```bash
cargo test -p noesis-api --test user_management_tests login -- --nocapture
```

Expected: FAIL because `/auth/login` still issues JWTs.

**Step 3: Return explicit retired response**

In `crates/noesis-api/src/handlers/auth.rs`, replace the body of `login` with a minimal retired response:

```rust
pub async fn login(
    State(_state): State<AppState>,
    Json(_payload): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    Ok((
        StatusCode::GONE,
        Json(serde_json::json!({
            "error": "Password login has been retired. Use Cloudflare Access.",
            "error_code": "AUTH_RETIRED"
        })),
    )
        .into_response())
}
```

Do not remove registration/password-reset handlers in this task unless tests force it. They can be retired in a cleanup task after CF auth is proven.

**Step 4: Run tests**

Run:

```bash
cargo test -p noesis-api --test user_management_tests login -- --nocapture
cargo check -p noesis-api
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-api/src/handlers/auth.rs crates/noesis-api/tests/user_management_tests.rs
git commit -m "feat: retire password login issuance"
```

---

### Task 12: Admin Web CF Login Simplification

**Files:**
- Modify: `apps/admin-web/app/(public)/login/login-client.tsx:1-124`
- Modify: `apps/admin-web/src/lib/api.ts:202-226`
- Delete: `apps/admin-web/src/lib/discord-oauth.ts`
- Delete: `apps/admin-web/src/lib/discord-oauth.test.ts`
- Delete: `apps/admin-web/app/(public)/login/discord-callback/page.tsx`
- Delete: `apps/admin-web/app/(public)/login/discord-callback/discord-callback-client.tsx`
- Test: `apps/admin-web/package.json`

**Step 1: Run current typecheck baseline**

Run:

```bash
pnpm --dir apps/admin-web typecheck
```

Expected: PASS before edits or shows unrelated existing errors. If unrelated errors exist, stop and record them before changing files.

**Step 2: Remove Discord API helpers first**

In `apps/admin-web/src/lib/api.ts`, delete:

```ts
export async function getDiscordAuthUrl(redirectUri?: string): Promise<{ url: string }> { ... }
export async function discordCallback(code: string, state?: string, redirectUri?: string): Promise<LoginResponse> { ... }
```

Keep `login` for now only if other tests still import it. It now hits a retired backend path.

**Step 3: Simplify login UI to CF Access message**

Replace `LoginClient` with:

```tsx
"use client";

import { useMemo } from "react";
import { useSearchParams } from "next/navigation";

function normalizeRedirect(rawTarget: string | null): string {
  if (!rawTarget || rawTarget.trim() === "") return "/dashboard";
  if (!rawTarget.startsWith("/")) return "/dashboard";
  if (rawTarget.startsWith("/admin")) {
    const stripped = rawTarget.slice("/admin".length);
    return stripped === "" ? "/dashboard" : stripped;
  }
  return rawTarget;
}

export function LoginClient() {
  const searchParams = useSearchParams();
  const redirectTarget = useMemo(
    () => normalizeRedirect(searchParams.get("redirect")),
    [searchParams]
  );

  return (
    <main className="login-wrap">
      <section className="login-card">
        <h1>Selemene Admin</h1>
        <p>Access is managed by Cloudflare Zero Trust.</p>
        <p>If you can see this page directly, your Access policy may not be applied to the admin route.</p>
        <a className="discord-btn" href={redirectTarget}>Continue to dashboard</a>
      </section>
    </main>
  );
}
```

If `discord-btn` is semantically wrong CSS, rename it only if the stylesheet is in the same file/scope and the change remains small.

**Step 4: Delete callback UI and Discord helper files**

Delete:

```bash
apps/admin-web/src/lib/discord-oauth.ts
apps/admin-web/src/lib/discord-oauth.test.ts
apps/admin-web/app/(public)/login/discord-callback/page.tsx
apps/admin-web/app/(public)/login/discord-callback/discord-callback-client.tsx
```

**Step 5: Run admin checks**

Run:

```bash
pnpm --dir apps/admin-web typecheck
pnpm --dir apps/admin-web lint
```

Expected: PASS and no imports of `discord-oauth`, `getDiscordAuthUrl`, or `discordCallback` remain.

**Step 6: Commit**

```bash
git add apps/admin-web/app/(public)/login/login-client.tsx apps/admin-web/src/lib/api.ts apps/admin-web/src/lib/discord-oauth.ts apps/admin-web/src/lib/discord-oauth.test.ts apps/admin-web/app/(public)/login/discord-callback/page.tsx apps/admin-web/app/(public)/login/discord-callback/discord-callback-client.tsx
git commit -m "feat: simplify admin login for Cloudflare Access"
```

---

### Task 13: Raga API Payload Alignment

**Files:**
- Modify: `crates/noesis-api/src/handlers/raga.rs:138-214`
- Test: `crates/noesis-api/src/handlers/raga.rs`

**Step 1: Write failing request-shape test**

Add a unit test module to `crates/noesis-api/src/handlers/raga.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_request_accepts_r2_metadata() {
        let body = serde_json::json!({
            "melakarta_num": 15,
            "style": "ambient",
            "suno_song_id": "song_123",
            "suno_prompt": "A meditative raga prompt",
            "r2_key": "clips/ambient/15-song_123.mp3",
            "cdn_url": "https://cdn.example.com/clips/ambient/15-song_123.mp3",
            "duration_sec": 45,
            "status": "generated"
        });

        let parsed: UpsertRagaClipRequest = serde_json::from_value(body).expect("request parses");
        assert_eq!(parsed.suno_prompt, "A meditative raga prompt");
        assert_eq!(parsed.r2_key, "clips/ambient/15-song_123.mp3");
    }
}
```

**Step 2: Run test to verify failure**

Run:

```bash
cargo test -p noesis-api upsert_request_accepts_r2_metadata -- --nocapture
```

Expected: FAIL because `suno_prompt` and `r2_key` fields do not exist.

**Step 3: Update request type and SQL**

Change `UpsertRagaClipRequest`:

```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertRagaClipRequest {
    pub melakarta_num: i32,
    pub style: String,
    pub suno_song_id: String,
    pub suno_prompt: String,
    pub r2_key: String,
    pub cdn_url: String,
    pub duration_sec: i32,
    pub status: Option<String>,
}
```

Change status default to match migration:

```rust
let status = body.status.as_deref().unwrap_or("generated");
```

Validate status and style before SQL:

```rust
if !matches!(body.style.as_str(), "ambient" | "meditative" | "cinematic" | "acid") {
    return (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "unsupported style" }))).into_response();
}
if !matches!(status, "pending" | "generated" | "approved" | "rejected" | "regenerate") {
    return (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "unsupported status" }))).into_response();
}
```

Replace insert SQL with:

```rust
let result = sqlx::query(
    r#"
    INSERT INTO raga_clips (melakarta_num, style, suno_song_id, suno_prompt, r2_key, cdn_url, duration_sec, status)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    ON CONFLICT (melakarta_num, style)
    DO UPDATE SET
      suno_song_id = EXCLUDED.suno_song_id,
      suno_prompt  = EXCLUDED.suno_prompt,
      r2_key       = EXCLUDED.r2_key,
      cdn_url      = EXCLUDED.cdn_url,
      duration_sec = EXCLUDED.duration_sec,
      status       = EXCLUDED.status
    "#,
)
.bind(body.melakarta_num)
.bind(&body.style)
.bind(&body.suno_song_id)
.bind(&body.suno_prompt)
.bind(&body.r2_key)
.bind(&body.cdn_url)
.bind(body.duration_sec)
.bind(status)
.execute(pool)
.await;
```

**Step 4: Run tests and check**

Run:

```bash
cargo test -p noesis-api upsert_request_accepts_r2_metadata -- --nocapture
cargo check -p noesis-api
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-api/src/handlers/raga.rs
git commit -m "fix: align raga clip upsert with R2 schema"
```

---

### Task 14: R2 Upload Helper in TS Engines

**Files:**
- Create: `ts-engines/scripts/r2-upload.ts`
- Modify: `ts-engines/package.json:13-17`
- Test: `ts-engines/scripts/r2-upload.ts`

**Step 1: Write helper with testable URL/key logic**

Create `ts-engines/scripts/r2-upload.ts`:

```ts
export interface R2Config {
  accountId: string;
  accessKeyId: string;
  secretAccessKey: string;
  bucket: string;
  publicBaseUrl: string;
}

export function requireR2Config(env: NodeJS.ProcessEnv = process.env): R2Config {
  const config = {
    accountId: env.R2_ACCOUNT_ID ?? "",
    accessKeyId: env.R2_ACCESS_KEY_ID ?? "",
    secretAccessKey: env.R2_SECRET_ACCESS_KEY ?? "",
    bucket: env.R2_RAGA_CLIPS_BUCKET ?? "selemene-raga-clips",
    publicBaseUrl: env.R2_PUBLIC_BASE_URL ?? "",
  };

  const missing = [
    !config.accountId && "R2_ACCOUNT_ID",
    !config.accessKeyId && "R2_ACCESS_KEY_ID",
    !config.secretAccessKey && "R2_SECRET_ACCESS_KEY",
    !config.publicBaseUrl && "R2_PUBLIC_BASE_URL",
  ].filter(Boolean);

  if (missing.length) {
    throw new Error(`Missing R2 env vars: ${missing.join(", ")}`);
  }

  return config;
}

export function publicR2Url(publicBaseUrl: string, key: string): string {
  return `${publicBaseUrl.replace(/\/+$/, "")}/${key.replace(/^\/+/, "")}`;
}

export async function uploadToR2(key: string, buffer: Buffer, config = requireR2Config()): Promise<string> {
  const endpoint = `https://${config.accountId}.r2.cloudflarestorage.com/${config.bucket}/${key}`;
  const response = await fetch(endpoint, {
    method: "PUT",
    headers: {
      "Content-Type": "audio/mpeg",
      "Cache-Control": "public, max-age=31536000, immutable",
      "Authorization": `Bearer ${config.secretAccessKey}`,
      "X-Access-Key-Id": config.accessKeyId,
    },
    body: buffer,
  });

  if (!response.ok) {
    throw new Error(`R2 upload failed: ${response.status} ${await response.text()}`);
  }

  return publicR2Url(config.publicBaseUrl, key);
}
```

**Step 2: Run typecheck by executing import**

Run:

```bash
bun --cwd ts-engines -e 'import { publicR2Url } from "./scripts/r2-upload.ts"; console.log(publicR2Url("https://cdn.example.com/", "/clips/a.mp3"))'
```

Expected: PASS and prints `https://cdn.example.com/clips/a.mp3`.

**Step 3: Replace dependency direction if needed**

This helper uses `fetch` directly. Do not add AWS SDK unless signed R2 upload fails during live testing. If live R2 requires AWS SigV4, replace this helper with `@aws-sdk/client-s3` in the smallest follow-up task.

**Step 4: Run TS engine tests**

Run:

```bash
bun --cwd ts-engines test
```

Expected: PASS.

**Step 5: Commit**

```bash
git add ts-engines/scripts/r2-upload.ts ts-engines/package.json
git commit -m "feat: add R2 upload helper for raga clips"
```

---

### Task 15: Migrate Suno Scripts to R2

**Files:**
- Modify: `ts-engines/scripts/suno-smoke.ts:1-156`
- Modify: `ts-engines/scripts/suno-bulk-gen.ts:1-204`
- Modify: `ts-engines/package.json:13-17`
- Modify: `ts-engines/bun.lock`
- Test: `ts-engines/scripts/suno-smoke.ts`

**Step 1: Replace Supabase imports and env docs**

In both Suno scripts, delete:

```ts
import { createClient } from '@supabase/supabase-js';
```

Add:

```ts
import { uploadToR2 } from './r2-upload';
```

Replace required env comments:

```ts
//   R2_ACCOUNT_ID
//   R2_ACCESS_KEY_ID
//   R2_SECRET_ACCESS_KEY
//   R2_PUBLIC_BASE_URL
//   R2_RAGA_CLIPS_BUCKET     — optional, default selemene-raga-clips
```

**Step 2: Replace upload calls**

In `suno-smoke.ts`, delete `SUPABASE_URL`, `SUPABASE_KEY`, `BUCKET`, and `uploadToSupabase`. Replace env checks:

```ts
if (!INTERNAL_KEY) throw new Error('INTERNAL_SERVICE_KEY env var required');
```

Replace:

```ts
const cdnUrl = await uploadToSupabase(key, buffer);
```

with:

```ts
const cdnUrl = await uploadToR2(key, buffer);
```

Change `upsertClip` signature/body:

```ts
async function upsertClip(melakartaNum: number, style: string, songId: string, sunoPrompt: string, r2Key: string, cdnUrl: string, duration: number) {
  const r = await fetch(`${NOESIS_API}/internal/raga/clip`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'x-internal-key': INTERNAL_KEY },
    body: JSON.stringify({
      melakarta_num: melakartaNum,
      style,
      suno_song_id: songId,
      suno_prompt: sunoPrompt,
      r2_key: r2Key,
      cdn_url: cdnUrl,
      duration_sec: Math.round(duration),
      status: 'generated'
    }),
  });
  if (!r.ok) throw new Error(`upsert clip: ${r.status} ${await r.text()}`);
}
```

Call it with:

```ts
await upsertClip(melakartaNum, style, song.id, prompt.prompt, key, cdnUrl, ready.duration);
```

Make the equivalent changes in `suno-bulk-gen.ts`.

**Step 3: Remove Supabase dependency**

Run:

```bash
bun remove --cwd ts-engines @supabase/supabase-js
```

Expected: `@supabase/supabase-js` removed from `ts-engines/package.json` and `ts-engines/bun.lock` updates.

**Step 4: Run dry-run smoke**

Run:

```bash
SUNO_SMOKE_DRY_RUN=1 bun ts-engines/scripts/suno-smoke.ts 15 ambient 45
```

Expected: PASS and prints prompt without requiring Suno/R2 env vars.

**Step 5: Run TS checks**

Run:

```bash
bun --cwd ts-engines test
bun --cwd ts-engines run lint
```

Expected: PASS.

**Step 6: Commit**

```bash
git add ts-engines/scripts/suno-smoke.ts ts-engines/scripts/suno-bulk-gen.ts ts-engines/scripts/r2-upload.ts ts-engines/package.json ts-engines/bun.lock
git commit -m "feat: migrate raga generation uploads to R2"
```

---

### Task 16: Drop oauth_accounts Runtime Code

**Files:**
- Delete: `crates/noesis-data/src/models/oauth_account.rs`
- Delete: `crates/noesis-data/src/repositories/oauth_repository.rs`
- Modify: `crates/noesis-data/src/models/mod.rs`
- Modify: `crates/noesis-data/src/repositories/mod.rs`
- Modify: `crates/noesis-api/src/lib.rs:408-430`
- Test: `crates/noesis-data`, `crates/noesis-api`

**Step 1: Remove AppState oauth repository references**

Delete field:

```rust
pub oauth_repository: Option<Arc<noesis_data::repositories::oauth_repository::OAuthRepository>>,
```

Remove initialization in `build_app_state` and `build_app_state_lazy_db`.

**Step 2: Delete model and repository module exports**

Remove `oauth_account` from `crates/noesis-data/src/models/mod.rs`.

Remove `oauth_repository` from `crates/noesis-data/src/repositories/mod.rs`.

Delete the two files.

**Step 3: Run compile to find leftover references**

Run:

```bash
cargo check -p noesis-data
cargo check -p noesis-api
```

Expected: FAIL if any `oauth_repository` or `OAuthAccount` references remain. Remove only those direct leftovers.

**Step 4: Run compile again**

Run:

```bash
cargo check -p noesis-data
cargo check -p noesis-api
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/noesis-data/src/models/mod.rs crates/noesis-data/src/repositories/mod.rs crates/noesis-data/src/models/oauth_account.rs crates/noesis-data/src/repositories/oauth_repository.rs crates/noesis-api/src/lib.rs
git commit -m "chore: remove oauth account runtime code"
```

---

### Task 17: Drop oauth_accounts Table Migration

**Files:**
- Create: `migrations/032_drop_oauth_accounts.sql`
- Test: `migrations/032_drop_oauth_accounts.sql`

**Step 1: Create migration with explicit post-cutover guard comment**

Create `migrations/032_drop_oauth_accounts.sql`:

```sql
-- Migration 032: Drop oauth_accounts after Cloudflare Access cutover.
--
-- This is intentionally destructive. Apply only after:
-- 1. Discord OAuth routes are removed from noesis-api.
-- 2. Admin web no longer calls Discord OAuth helpers.
-- 3. Production traffic is protected by Cloudflare Access.
--
-- No DOWN migration is provided because the locked product decision is
-- surgical retirement with no rollback path.

DROP TABLE IF EXISTS oauth_accounts;
```

**Step 2: Run SQL syntax smoke if a local DB is available**

Run:

```bash
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f migrations/032_drop_oauth_accounts.sql
```

Expected: PASS on a disposable/local DB. If no local DB is available, do not run against production; record `NOT RUN: no disposable DATABASE_URL`.

**Step 3: Run grep check**

Run:

```bash
git grep -n "oauth_accounts" -- ':!migrations/032_drop_oauth_accounts.sql'
```

Expected: Only historical docs or older migrations remain. No runtime Rust/TS files should match.

**Step 4: Commit**

```bash
git add migrations/032_drop_oauth_accounts.sql
git commit -m "chore: drop retired oauth accounts table"
```

---

### Task 18: Environment Examples and Config Docs

**Files:**
- Modify: `.env.example`
- Modify: `apps/admin-web/README.md`
- Modify: `DOCKER.md`
- Test: `.env.example`

**Step 1: Update `.env.example`**

Replace the Supabase section:

```dotenv
# === Database Configuration ===
# Production: Railway Postgres. Local: Docker Postgres from docker-compose.yml.
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/noesis
```

Add Cloudflare Access envs:

```dotenv
# === Cloudflare Zero Trust ===
CF_ACCESS_ISSUER=https://your-team.cloudflareaccess.com
CF_ACCESS_AUDIENCE=your-cloudflare-access-audience-tag
# Development only. Honored only when RUST_ENV=development.
CF_DEV_BYPASS_TOKEN=local-dev-platform-admin-token
```

Add R2 envs:

```dotenv
# === Cloudflare R2 for Raaga Clips ===
R2_ACCOUNT_ID=your_cloudflare_account_id
R2_ACCESS_KEY_ID=your_r2_access_key_id
R2_SECRET_ACCESS_KEY=your_r2_secret_access_key
R2_RAGA_CLIPS_BUCKET=selemene-raga-clips
R2_PUBLIC_BASE_URL=https://raga-clips.example.com
```

Remove `SUPABASE_DB_PASSWORD`, `NEXT_PUBLIC_SUPABASE_URL`, and `SUPABASE_SERVICE_ROLE_KEY` if present.

**Step 2: Update admin README**

In `apps/admin-web/README.md`, remove Discord OAuth callback setup and replace with:

```md
## Authentication

Admin access is enforced by Cloudflare Zero Trust in front of the deployed admin surface. The Rust API validates Cloudflare Access identity for protected API calls and maps CF groups into local `user_roles`.

Group mapping:

- `selemene-admin` -> `platform-admin`
- Any supported local role group maps directly: `viewer`, `support`, `admin`, `platform-admin`

Local development can use `RUST_ENV=development` with `CF_DEV_BYPASS_TOKEN` and the `x-noesis-dev-auth` header.
```

**Step 3: Update Docker docs**

In `DOCKER.md`, replace production Supabase wording with Railway Postgres wording. Keep local Docker Postgres docs.

**Step 4: Grep check**

Run:

```bash
git grep -n "SUPABASE_DB_PASSWORD\|NEXT_PUBLIC_SUPABASE_URL\|SUPABASE_SERVICE_ROLE_KEY\|Discord OAuth" -- .env.example apps/admin-web/README.md DOCKER.md
```

Expected: no active setup references. Archival references are not allowed in these three files.

**Step 5: Commit**

```bash
git add .env.example apps/admin-web/README.md DOCKER.md
git commit -m "docs: document Railway Postgres and Cloudflare Access envs"
```

---

### Task 19: Runtime Docs Supabase and Discord Cleanup

**Files:**
- Modify: `README.md`
- Modify: `.github/copilot-instructions.md`
- Modify: `docs/PROJECT_OVERVIEW.md`
- Modify: `docs/deployment/DB_MIGRATION_AUTHORITY.md`
- Modify: `docs/runbooks/discord-oauth-callback-policy.md`
- Test: docs grep

**Step 1: Replace active Supabase architecture references**

In `README.md`, update stack references:

```md
Rust + Axum + Railway Postgres + Railway
```

Replace database table row:

```md
| **Database** | Railway Postgres |
```

Replace `DATABASE_URL` description:

```md
- `DATABASE_URL` — Railway Postgres connection URL (optional locally; required in production)
```

**Step 2: Update Copilot instructions**

In `.github/copilot-instructions.md`, replace Supabase/Discord runtime claims with:

```md
Noesis is deployed on Railway with Railway Postgres for the primary database and Cloudflare Zero Trust for human/admin authentication.
```

**Step 3: Update migration authority docs**

In `docs/deployment/DB_MIGRATION_AUTHORITY.md`, make root `migrations/` the only schema authority and mark `supabase/` mirrors retired.

**Step 4: Retire Discord runbook**

In `docs/runbooks/discord-oauth-callback-policy.md`, replace content with:

```md
# Discord OAuth Callback Policy

Discord OAuth is retired. Human/admin authentication now uses Cloudflare Zero Trust.

Historical context only: older releases used Discord OAuth callback routes under `/api/v1/auth/discord/*`. Do not configure new Discord OAuth clients for this project.
```

**Step 5: Run grep check**

Run:

```bash
git grep -i -n "supabase\|discord oauth" -- README.md .github/copilot-instructions.md docs/PROJECT_OVERVIEW.md docs/deployment/DB_MIGRATION_AUTHORITY.md docs/runbooks/discord-oauth-callback-policy.md
```

Expected: only explicit historical/retired references remain; no active setup instructions remain.

**Step 6: Commit**

```bash
git add README.md .github/copilot-instructions.md docs/PROJECT_OVERVIEW.md docs/deployment/DB_MIGRATION_AUTHORITY.md docs/runbooks/discord-oauth-callback-policy.md
git commit -m "docs: retire Supabase and Discord OAuth references"
```

---

### Task 20: Supabase Directory and Drift Script Archive/Delete

**Files:**
- Delete: `supabase/README.md`
- Delete: `supabase/config.toml`
- Delete: `supabase/migrations/*`
- Delete: `scripts/check_migration_drift.sh`
- Test: repo file list

**Step 1: Confirm root migrations are present**

Run:

```bash
git ls-files 'migrations/*.sql' | wc -l
```

Expected: at least `31` after adding `032_drop_oauth_accounts.sql`; if count differs, list files and confirm before deleting Supabase mirrors.

**Step 2: Delete Supabase mirror and drift script**

Delete:

```bash
supabase/
scripts/check_migration_drift.sh
```

Use non-destructive editor/file deletion in the implementation environment. Do not use `git reset` or `git checkout`.

**Step 3: Verify no tracked Supabase directory remains**

Run:

```bash
git ls-files 'supabase/**' 'scripts/check_migration_drift.sh'
```

Expected: no output.

**Step 4: Commit**

```bash
git add -A supabase scripts/check_migration_drift.sh
git commit -m "chore: remove retired Supabase migration mirror"
```

---

### Task 21: Cutover Runbook

**Files:**
- Create: `docs/runbooks/supabase-retirement-cutover.md`
- Test: `docs/runbooks/supabase-retirement-cutover.md`

**Step 1: Create cutover runbook**

Create `docs/runbooks/supabase-retirement-cutover.md`:

```md
# Supabase Retirement Cutover Runbook

## Preflight

- Railway Postgres is provisioned and reachable.
- Cloudflare Access application protects admin-web and API routes that require human/admin auth.
- CF groups exist: `selemene-admin`, plus optional direct local roles `viewer`, `support`, `admin`, `platform-admin`.
- R2 bucket exists: `selemene-raga-clips`.
- Runtime secrets are set: `DATABASE_URL`, `CF_ACCESS_ISSUER`, `CF_ACCESS_AUDIENCE`, Dodo envs, `INTERNAL_SERVICE_KEY`, and R2 envs for TS generation jobs.

## Database Migration

1. Take final Supabase backup.
2. Dry-run schema pass:
   `scripts/supabase-to-railway-migrate.sh --dry-run --schema-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"`
3. Run schema pass:
   `scripts/supabase-to-railway-migrate.sh --yes --schema-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"`
4. Run data pass:
   `scripts/supabase-to-railway-migrate.sh --yes --data-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"`
5. Verify row counts and partitions from script output.

## Deploy

1. Deploy API with `DATABASE_URL` pointing to Railway Postgres and CF validation envs set.
2. Deploy admin-web behind Cloudflare Access.
3. Deploy TS engine script changes for R2 upload.

## Smoke Tests

- `curl -fsS https://selemene.tryambakam.space/health/live`
- `curl -fsS https://selemene.tryambakam.space/health/ready`
- CF-authenticated admin session returns platform-admin for `selemene-admin` group.
- Dodo webhook forward path still accepts valid `X-Forward-Secret`.
- Raga generation uploads to R2 and `/internal/raga/clip` persists the row.

## Cleanup

- Unset Supabase secrets from Railway, local shells, and CI.
- Remove Discord OAuth app credentials from secret stores.
- Confirm `oauth_accounts` has been dropped.
```

**Step 2: Grep for required smoke checks**

Run:

```bash
git grep -n "health/ready\|selemene-admin\|X-Forward-Secret\|R2" docs/runbooks/supabase-retirement-cutover.md
```

Expected: all four concepts appear.

**Step 3: Commit**

```bash
git add docs/runbooks/supabase-retirement-cutover.md
git commit -m "docs: add Supabase retirement cutover runbook"
```

---

### Task 22: Full Verification Pass

**Files:**
- No edits expected
- Test: full repo checks

**Step 1: Run Rust checks**

Run:

```bash
cargo check --workspace
```

Expected: PASS.

Run:

```bash
cargo test -p noesis-api --test route_inventory_tests -- --nocapture
cargo test -p noesis-api --test user_management_tests -- --test-threads=1
```

Expected: PASS.

**Step 2: Run admin checks**

Run:

```bash
pnpm --dir apps/admin-web typecheck
pnpm --dir apps/admin-web lint
```

Expected: PASS.

**Step 3: Run TS engine checks**

Run:

```bash
bun --cwd ts-engines test
bun --cwd ts-engines run lint
```

Expected: PASS.

**Step 4: Run migration script checks**

Run:

```bash
bash -n scripts/supabase-to-railway-migrate.sh
bash scripts/supabase-to-railway-migrate.sh --dry-run --full --source postgres://u:p@source/db --target postgres://u:p@target/db
```

Expected: PASS; dry-run prints schema/data restore commands and verification SQL.

**Step 5: Run active-reference grep**

Run:

```bash
git grep -i -n "NEXT_PUBLIC_SUPABASE_URL\|SUPABASE_SERVICE_ROLE_KEY\|SUPABASE_DB_PASSWORD\|/auth/discord\|discordCallback\|getDiscordAuthUrl\|oauth_accounts" -- ':!migrations/007_oauth_accounts.sql' ':!migrations/032_drop_oauth_accounts.sql' ':!docs/runbooks/discord-oauth-callback-policy.md'
```

Expected: no runtime references. If matches remain in archival docs, explicitly classify them as historical or remove them.

**Step 6: Commit verification-only docs if needed**

If no files changed, do not commit. If verification notes were added to a runbook, commit:

```bash
git add docs/runbooks/supabase-retirement-cutover.md
git commit -m "docs: record Supabase retirement verification"
```

---

### Task 23: Live Cutover Execution

**Files:**
- No code edits expected
- Test: live environment

**Step 1: Confirm final backup exists**

Run the approved Supabase backup/export procedure. Record backup artifact path in the deployment log. Do not proceed without a backup artifact.

Expected: backup path or dashboard artifact ID is recorded.

**Step 2: Run schema then data migration**

Run:

```bash
scripts/supabase-to-railway-migrate.sh --yes --schema-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"
scripts/supabase-to-railway-migrate.sh --yes --data-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"
```

Expected: both passes complete and verification SQL passes.

**Step 3: Set production env vars**

Set in Railway/API environment:

```dotenv
DATABASE_URL=<Railway Postgres URL>
CF_ACCESS_ISSUER=<Cloudflare Access issuer>
CF_ACCESS_AUDIENCE=<Cloudflare Access audience>
DODO_PAYMENTS_API_KEY=<existing>
DODO_PAYMENTS_WEBHOOK_KEY=<existing>
DODO_INTERNAL_FORWARD_SECRET=<existing>
INTERNAL_SERVICE_KEY=<existing>
```

Unset Supabase and Discord envs.

Expected: deploy environment contains no active Supabase or Discord OAuth secrets.

**Step 4: Deploy API/admin/TS jobs**

Run the project’s normal deploy command or CI deployment. Do not invent a new deploy path in this task.

Expected: deploy completes.

**Step 5: Smoke live health**

Run:

```bash
curl -fsS https://selemene.tryambakam.space/health/live
curl -fsS https://selemene.tryambakam.space/health/ready
```

Expected: both return success; ready reports DB ok.

**Step 6: Smoke live CF admin auth**

Use a browser or curl with Cloudflare Access token to call:

```bash
curl -fsS -H "CF_Authorization: $CF_ACCESS_TOKEN" https://selemene.tryambakam.space/api/v1/admin/session
```

Expected: returns current user, tier, and `platform-admin` role for a user in CF group `selemene-admin`.

**Step 7: Smoke internal carve-outs**

Run billing webhook test with a known safe fixture against non-production or Dodo test mode first. Run raga dry-run/upload in a controlled single clip:

```bash
SUNO_SMOKE_DRY_RUN=1 bun ts-engines/scripts/suno-smoke.ts 15 ambient 45
```

Expected: dry-run passes. Live R2 upload test should be run only when Suno and R2 credentials are confirmed.

**Step 8: Record cutover result**

Update deployment log or runbook with timestamp, DB row-count verification, CF smoke result, billing carve-out result, and raga result.

**Step 9: Commit any documentation-only result updates**

```bash
git add docs/runbooks/supabase-retirement-cutover.md
git commit -m "docs: record Supabase retirement cutover result"
```

---

## Final Done Criteria

- `scripts/supabase-to-railway-migrate.sh` exists, is committed, and supports schema-only, data-only, critical-tables, full, dry-run, safety checks, and verification SQL.
- Railway Postgres is the production `DATABASE_URL` and `/health/ready` reports DB ok.
- Cloudflare Access identity is accepted for protected user/admin routes using `sub`, `email`, and groups.
- CF group `selemene-admin` produces local role `platform-admin`.
- `RUST_ENV=development` plus `x-noesis-dev-auth` supports local platform-admin bypass.
- Discord OAuth routes are removed or retired and admin-web no longer starts Discord OAuth flows.
- No runtime code writes `oauth_accounts`; final migration drops the table.
- Raga generation uploads to R2 and persists `suno_prompt`, `r2_key`, `cdn_url`, and `generated` status.
- Billing webhook and raga internal routes continue to work with existing shared-secret carve-outs.
- Supabase mirror directory and migration drift script are removed or clearly archived as historical.
- Runtime docs/env examples reference Railway Postgres, CF Zero Trust, and R2, not Supabase/Discord OAuth.
- Full verification matrix passes or any skipped live checks are explicitly recorded with reason.
