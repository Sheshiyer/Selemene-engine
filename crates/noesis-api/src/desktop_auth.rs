//! Browser-based desktop sign-in: pure helpers for the PKCE loopback flow.
//!
//! Flow (RFC 8252 + RFC 7636):
//! 1. The desktop app opens `GET /api/v1/desktop/authorize` in the system
//!    browser. Cloudflare Access authenticates the person; the handler issues
//!    a single-use code bound to the PKCE challenge and loopback redirect.
//! 2. The browser is redirected to `http://127.0.0.1:<port>/callback`.
//! 3. The app exchanges the code plus its PKCE verifier at
//!    `POST /api/v1/desktop/token` for a short-lived Selemene JWT and a
//!    rotating refresh token.
//!
//! Desktop tokens carry only `basic:access`: a desktop session can run engines
//! and read its own data, never reach admin routes, and never mint new codes
//! (authorize accepts browser identities only).

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Authorization codes live for one minute.
pub const AUTH_CODE_TTL_SECS: i64 = 60;
/// Refresh tokens live for 30 days and rotate on every use.
pub const REFRESH_TOKEN_TTL_DAYS: i64 = 30;
/// The only permission a desktop access token carries.
pub const DESKTOP_PERMISSIONS: &[&str] = &["basic:access"];

const LOOPBACK_PREFIX: &str = "http://127.0.0.1:";
const CALLBACK_PATH: &str = "/callback";

/// Validate a loopback redirect: exactly `http://127.0.0.1:<port>/callback`
/// with an unprivileged port, and nothing else (no userinfo, query, fragment,
/// other hosts or paths).
pub fn validate_loopback_redirect(uri: &str) -> Result<(), &'static str> {
    let rest = uri
        .strip_prefix(LOOPBACK_PREFIX)
        .ok_or("redirect_uri must start with http://127.0.0.1:")?;
    let port = rest
        .strip_suffix(CALLBACK_PATH)
        .ok_or("redirect_uri path must be exactly /callback")?;
    if port.is_empty() || port.len() > 5 || !port.bytes().all(|b| b.is_ascii_digit()) {
        return Err("redirect_uri port must be numeric");
    }
    match port.parse::<u32>() {
        Ok(p) if (1024..=65535).contains(&p) => Ok(()),
        _ => Err("redirect_uri port must be between 1024 and 65535"),
    }
}

fn is_unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~')
}

/// RFC 7636 §4.1: 43–128 unreserved characters.
pub fn valid_code_verifier(verifier: &str) -> bool {
    (43..=128).contains(&verifier.len()) && verifier.bytes().all(is_unreserved)
}

/// An S256 challenge is base64url (no padding) of a SHA-256 digest: 43 chars.
pub fn valid_code_challenge(challenge: &str) -> bool {
    challenge.len() == 43
        && challenge
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Opaque client state echoed back on the redirect.
pub fn valid_state(state: &str) -> bool {
    (16..=128).contains(&state.len()) && state.bytes().all(is_unreserved)
}

/// The S256 challenge for a verifier.
pub fn s256_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Constant-time PKCE check: does `verifier` produce `challenge`?
pub fn pkce_matches(verifier: &str, challenge: &str) -> bool {
    if !valid_code_verifier(verifier) {
        return false;
    }
    let computed = s256_challenge(verifier);
    computed.len() == challenge.len()
        && computed
            .bytes()
            .zip(challenge.bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

/// 256 bits of OS randomness, base64url without padding (43 chars).
pub fn new_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Storage form of a code or refresh token.
pub fn hash_secret(secret: &str) -> String {
    format!("{:x}", Sha256::digest(secret.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 7636 Appendix B test vector.
    const RFC_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const RFC_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    #[test]
    fn s256_matches_rfc_7636_vector() {
        assert_eq!(s256_challenge(RFC_VERIFIER), RFC_CHALLENGE);
        assert!(pkce_matches(RFC_VERIFIER, RFC_CHALLENGE));
        assert!(valid_code_challenge(RFC_CHALLENGE));
    }

    #[test]
    fn pkce_rejects_wrong_or_malformed_verifier() {
        assert!(!pkce_matches(
            "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXx",
            RFC_CHALLENGE
        ));
        assert!(!pkce_matches("short", RFC_CHALLENGE));
        assert!(!pkce_matches(&"a".repeat(129), RFC_CHALLENGE));
        assert!(!pkce_matches(
            &format!("{}!", &RFC_VERIFIER[..43]),
            RFC_CHALLENGE
        ));
    }

    #[test]
    fn loopback_redirect_accepts_only_127_0_0_1_callback() {
        assert!(validate_loopback_redirect("http://127.0.0.1:53682/callback").is_ok());
        for bad in [
            "https://127.0.0.1:53682/callback",
            "http://localhost:53682/callback",
            "http://127.0.0.1.evil.example:53682/callback",
            "http://127.0.0.1:53682/callback?x=1",
            "http://127.0.0.1:53682/callback#x",
            "http://127.0.0.1:53682/other",
            "http://127.0.0.1:53682/callback/",
            "http://user@127.0.0.1:53682/callback",
            "http://127.0.0.1:80/callback",
            "http://127.0.0.1:70000/callback",
            "http://127.0.0.1:/callback",
            "http://127.0.0.1:+8080/callback",
            "http://127.0.0.1/callback",
        ] {
            assert!(
                validate_loopback_redirect(bad).is_err(),
                "{bad} should be rejected"
            );
        }
    }

    #[test]
    fn state_and_challenge_shapes() {
        assert!(valid_state("abcdefghijklmnop"));
        assert!(!valid_state("short"));
        assert!(!valid_state("has spaces in it ok"));
        assert!(!valid_code_challenge("too-short"));
        assert!(!valid_code_challenge(&format!("{}=", &RFC_CHALLENGE[..42])));
    }

    #[test]
    fn secrets_are_random_and_hashed() {
        let a = new_secret();
        let b = new_secret();
        assert_eq!(a.len(), 43);
        assert_ne!(a, b);
        assert_eq!(hash_secret(&a).len(), 64);
        assert_ne!(hash_secret(&a), a);
    }
}
