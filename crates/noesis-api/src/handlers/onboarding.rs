//! OpenClaw onboarding handler
//!
//! Provides endpoints for generating invite codes and onboarding instructions
//! to connect OpenClaw agents to Selemene Engine.

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension,
};
use noesis_auth::AuthUser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::AppState;

/// In-memory store for invite codes (in production, use database)
static INVITE_CODES: std::sync::LazyLock<RwLock<HashMap<String, InviteCode>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

/// Get gateway configuration from environment
fn get_gateway_config() -> (String, String) {
    let gateway_url = std::env::var("GATEWAY_URL")
        .unwrap_or_else(|_| "wss://selemene.tryambakam.space/gateway".to_string());
    let gateway_token =
        std::env::var("GATEWAY_TOKEN").unwrap_or_else(|_| "your-gateway-token".to_string());
    (gateway_url, gateway_token)
}

/// Invite code data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteCode {
    pub code: String,
    pub user_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub gateway_url: Option<String>,
    pub gateway_token: Option<String>,
}

/// Generate a new onboarding invite
#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    #[serde(default)]
    pub expires_hours: Option<u32>,
}

/// Generate a new invite code
pub async fn create_invite(
    State(_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    axum::extract::Json(req): axum::extract::Json<CreateInviteRequest>,
) -> impl IntoResponse {
    // Get user ID from auth
    let user_id = auth_user.user_id.clone();

    // Generate unique invite code
    let code = format!(
        "selemene_{}",
        Uuid::new_v4().to_string().replace("-", "")[..16].to_string()
    );

    // Calculate expiration (default 24 hours)
    let expires_hours = req.expires_hours.unwrap_or(24);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let expires_at = now + (expires_hours as u64 * 3600);

    // Get gateway config from environment
    let (gateway_url, gateway_token) = get_gateway_config();

    let invite = InviteCode {
        code: code.clone(),
        user_id: user_id.clone(),
        created_at: now,
        expires_at,
        gateway_url: Some(gateway_url.clone()),
        gateway_token: Some(gateway_token.clone()),
    };

    // Store invite code
    let mut codes = INVITE_CODES.write().unwrap();
    codes.insert(code.clone(), invite);

    tracing::info!("Generated onboarding invite for user: {}", user_id);

    axum::Json(serde_json::json!({
        "invite_code": code,
        "expires_in_hours": expires_hours,
        "onboarding_url": format!("/api/v1/onboarding/{}/openclaw.txt", code)
    }))
}

/// Get onboarding instructions for an invite code
pub async fn get_onboarding(
    Path(code): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Validate invite code
    let codes = INVITE_CODES.read().unwrap();

    let invite = codes
        .get(&code)
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "Invite code not found"))?;

    // Check expiration
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now > invite.expires_at {
        return Err((axum::http::StatusCode::GONE, "Invite code has expired"));
    }

    // Get gateway info from environment
    let (base_url, gateway_token) = get_gateway_config();

    // Generate onboarding text similar to Paperclip
    let onboarding_text = format_onboarding_text(&code, &base_url, &gateway_token, &invite.user_id);

    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/plain")],
        onboarding_text,
    ))
}

/// Generate the onboarding instructions text
fn format_onboarding_text(
    invite_code: &str,
    gateway_url: &str,
    token: &str,
    _user_id: &str,
) -> String {
    // Helper to avoid unused variable warnings
    let _ = gateway_url;

    let text = format!(
        r#"You're invited to join Selemene Engine as an OpenClaw Gateway.

The URLs you should try are:
- http://127.0.0.1:3100/api/invites/{}/onboarding.txt
- http://host.docker.internal:3100/api/invites/{}/onboarding.txt

Connectivity guidance:
If you are running on a different machine than Paperclip, Paperclip must be reachable at one of the hostnames used above.
Verify the hostname works from your runtime with: GET <base-url>/health

If none are reachable, ask your user to add a reachable hostname in Paperclip, restart, and retry.
Suggested command:
- pnpm paperclipai allowed-hostname <host>
Then verify with: curl -fsS <base-url>/api/health

For your "agentName", just use your own agent name (OpenClaw or whatever your name is) and send it directly.
You also need to send a gateway URL that Paperclip can access. Check your config and include that URL.
You MUST test Paperclip-to-gateway reachability, call: http://127.0.0.1:3100/api/invites/{}/test-resolution?url=<urlencoded-gateway-url> (using the hostname that worked above). Do not assume your 172.x is necessarily reachable from Paperclip. Test it.

---

This onboarding flow is for OpenClaw Gateway.
When you submit the join request, use:
- `adapterType: "openclaw_gateway"`
- `agentDefaultsPayload.url` as your `ws://` or `wss://` gateway URL
- `agentDefaultsPayload.headers["x-openclaw-token"]` with your gateway token
- (legacy accepted) `agentDefaultsPayload.headers["x-openclaw-auth"]`
- Keep device auth enabled (recommended). If `devicePrivateKeyPem` is omitted, Paperclip will generate and persist one during join so pairing approvals remain stable.
- Only use `disableDeviceAuth=true` for special environments where pairing cannot be completed.

Do NOT use `/v1/responses` or `/hooks/*` in this join flow.

---

## Selemene Engine OpenClaw Configuration

### Gateway Details
- Gateway URL: {}
- Gateway Token: {}

### Your Invite Code
{}

### Quick Test Commands

# Test connectivity to gateway:
curl -fsS https://selemene.tryambakam.space/health

# Test list engines (with API key):
curl -s https://selemene.tryambakam.space/api/v1/engines \
  -H "X-API-Key: nk_your_api_key_here"

# Test engine calculation:
curl -s -X POST https://selemene.tryambakam.space/api/v1/engines/vimshottari/calculate \
  -H "X-API-Key: nk_your_api_key_here" \
  -H "Content-Type: application/json" \
  -d '{{"birth_data": {{"date": "1990-06-15", "time": "14:30:00", "latitude": 28.6139, "longitude": 77.2090, "timezone": "Asia/Kolkata"}}}}'

### Available Engines (16)
- vimshottari, human-design, gene-keys, transits, vedic-clock, panchanga
- biofield, biorhythm, face-reading, numerology, nadabrahman
- enneagram, i-ching, sacred-geometry, sigil-forge, tarot

### Available Workflows (6)
- birth-blueprint, creative-expression, daily-practice
- decision-support, full-spectrum, self-inquiry

### Environment Setup
Set these in your agent configuration:
  NOESIS_API_KEY=nk_your_api_key_here
  SELEMENE_BASE_URL=https://selemene.tryambakam.space

---
Generated by Selemene Engine (https://selemene.tryambakam.space)
"#,
        invite_code, invite_code, invite_code, gateway_url, token, invite_code
    );

    text
}

/// List all active invite codes (admin only)
pub async fn list_invites() -> impl IntoResponse {
    let codes = INVITE_CODES.read().unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let active: Vec<_> = codes
        .values()
        .filter(|invite| invite.expires_at > now)
        .map(|invite| {
            serde_json::json!({
                "code": invite.code,
                "user_id": invite.user_id,
                "created_at": invite.created_at,
                "expires_at": invite.expires_at,
            })
        })
        .collect();

    axum::Json(serde_json::json!({ "invites": active }))
}
