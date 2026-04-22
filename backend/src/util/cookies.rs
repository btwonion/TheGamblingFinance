//! `gf_sid` session-cookie helpers: token generation, hashing,
//! Set-Cookie/Clear-Cookie header builders, and a tiny Cookie-header
//! parser that avoids pulling in a third-party cookie crate.
//!
//! The raw token is what lands on the client as the `gf_sid` cookie
//! value; only the SHA-256 hex of that value is persisted to
//! `auth_sessions.token_hash`. Stealing the DB yields no usable cookies.
//!
//! Token encoding: lowercase **hex** (32 random bytes → 64 hex chars).
//! We pick hex over base64url because `base64` is not a dependency of
//! this crate (cookies are opaque to clients so readability does not
//! matter), and hex keeps the dep set minimal.

use axum::http::{HeaderMap, HeaderValue};
use chrono::{DateTime, Utc};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};

/// The cookie name used for the session. Kept in sync with
/// `docs/contracts/openapi.json` (security scheme `sessionCookie`).
pub const COOKIE_NAME: &str = "gf_sid";

/// Number of raw random bytes packed into a session token.
const TOKEN_BYTES: usize = 32;

/// Generate a fresh session token as lowercase hex of 32 random bytes
/// from the OS RNG. Length = 64 chars.
pub fn generate_token() -> String {
    let mut buf = [0u8; TOKEN_BYTES];
    OsRng.fill_bytes(&mut buf);
    to_hex(&buf)
}

/// SHA-256 of `raw` as lowercase hex. This is the value that lands in
/// `auth_sessions.token_hash`; the raw token is only ever in-flight.
pub fn hash_token(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    to_hex(&digest)
}

/// Build a `Set-Cookie` header that installs `gf_sid=<token>` with
/// safe-by-default attributes. `Secure` is only set when `secure` is
/// true so local http dev still works; production always runs behind
/// TLS.
pub fn build_set_cookie(
    token: &str,
    expires: DateTime<Utc>,
    secure: bool,
) -> HeaderValue {
    // Clamp Max-Age to zero if the caller handed us a past instant so
    // we never emit a negative number.
    let max_age = (expires - Utc::now()).num_seconds().max(0);
    // RFC 1123 / IMF-fixdate — what the Expires attribute wants.
    let expires_http = expires.format("%a, %d %b %Y %H:%M:%S GMT");

    let mut s = format!(
        "{name}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}; Expires={expires_http}",
        name = COOKIE_NAME,
    );
    if secure {
        s.push_str("; Secure");
    }

    // HeaderValue only rejects bytes < 0x20 or == 0x7f. Our format
    // string produces printable ASCII only, so this is infallible in
    // practice; fall back to `from_static("")` in the (never-hit)
    // error path rather than panicking.
    HeaderValue::from_str(&s).unwrap_or_else(|_| HeaderValue::from_static(""))
}

/// Build a `Set-Cookie` header that clears the session cookie. Browsers
/// require matching `Path`, `SameSite`, and `Secure` (when set) for the
/// replacement cookie to actually overwrite the original.
pub fn build_clear_cookie(secure: bool) -> HeaderValue {
    let mut s = format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
        name = COOKIE_NAME,
    );
    if secure {
        s.push_str("; Secure");
    }
    HeaderValue::from_str(&s).unwrap_or_else(|_| HeaderValue::from_static(""))
}

/// Extract the `gf_sid` value (if any) from a request's Cookie header.
///
/// We implement this by hand to avoid pulling in a cookie-parsing
/// crate. The parser tolerates:
///   - multiple `Cookie:` headers (we scan them all)
///   - `a=1; b=2; gf_sid=xyz`
///   - arbitrary leading whitespace in each segment
///
/// We return `None` if the cookie is missing or empty. Values are not
/// URL-decoded; we only ever put hex into this cookie so decoding is
/// a no-op.
pub fn read_session_token(headers: &HeaderMap) -> Option<String> {
    for header in headers.get_all(axum::http::header::COOKIE) {
        let Ok(raw) = header.to_str() else { continue };
        for segment in raw.split(';') {
            let segment = segment.trim();
            if let Some(rest) = segment.strip_prefix(COOKIE_NAME) {
                // Accept `gf_sid=<token>` but not `gf_sidX=...`.
                if let Some(value) = rest.strip_prefix('=') {
                    if !value.is_empty() {
                        return Some(value.to_owned());
                    }
                }
            }
        }
    }
    None
}

/// Lowercase hex encoder. Avoids pulling in the `hex` crate.
fn to_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{header, HeaderMap, HeaderValue};

    #[test]
    fn generate_token_is_64_hex_chars() {
        let t = generate_token();
        assert_eq!(t.len(), 64);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn generate_token_is_not_deterministic() {
        assert_ne!(generate_token(), generate_token());
    }

    #[test]
    fn hash_token_is_deterministic_and_64_hex_chars() {
        let a = hash_token("abc");
        let b = hash_token("abc");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        // SHA-256 of "abc" is a well-known vector.
        assert_eq!(
            a,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn hash_token_differs_for_different_inputs() {
        assert_ne!(hash_token("abc"), hash_token("abd"));
    }

    #[test]
    fn read_session_token_reads_lone_cookie() {
        let mut h = HeaderMap::new();
        h.insert(header::COOKIE, HeaderValue::from_static("gf_sid=deadbeef"));
        assert_eq!(read_session_token(&h).as_deref(), Some("deadbeef"));
    }

    #[test]
    fn read_session_token_reads_middle_cookie() {
        let mut h = HeaderMap::new();
        h.insert(
            header::COOKIE,
            HeaderValue::from_static("foo=1; gf_sid=xyz; bar=2"),
        );
        assert_eq!(read_session_token(&h).as_deref(), Some("xyz"));
    }

    #[test]
    fn read_session_token_tolerates_whitespace() {
        let mut h = HeaderMap::new();
        h.insert(
            header::COOKIE,
            HeaderValue::from_static("foo=1;   gf_sid=abc  "),
        );
        assert_eq!(read_session_token(&h).as_deref(), Some("abc"));
    }

    #[test]
    fn read_session_token_returns_none_when_absent() {
        let mut h = HeaderMap::new();
        h.insert(header::COOKIE, HeaderValue::from_static("foo=1; bar=2"));
        assert_eq!(read_session_token(&h), None);
    }

    #[test]
    fn read_session_token_returns_none_without_cookie_header() {
        let h = HeaderMap::new();
        assert_eq!(read_session_token(&h), None);
    }

    #[test]
    fn read_session_token_ignores_prefix_collisions() {
        // "gf_sidx" must NOT match "gf_sid".
        let mut h = HeaderMap::new();
        h.insert(header::COOKIE, HeaderValue::from_static("gf_sidx=nope"));
        assert_eq!(read_session_token(&h), None);
    }

    #[test]
    fn build_set_cookie_contains_attrs() {
        let expires = Utc::now() + chrono::Duration::days(30);
        let v = build_set_cookie("tkn", expires, true);
        let s = v.to_str().unwrap();
        assert!(s.starts_with("gf_sid=tkn;"));
        assert!(s.contains("HttpOnly"));
        assert!(s.contains("SameSite=Lax"));
        assert!(s.contains("Secure"));
        assert!(s.contains("Max-Age="));
        assert!(s.contains("Expires="));
    }

    #[test]
    fn build_set_cookie_without_secure() {
        let expires = Utc::now() + chrono::Duration::days(30);
        let v = build_set_cookie("tkn", expires, false);
        assert!(!v.to_str().unwrap().contains("Secure"));
    }

    #[test]
    fn build_clear_cookie_is_zero_max_age() {
        let v = build_clear_cookie(false);
        let s = v.to_str().unwrap();
        assert!(s.contains("gf_sid=;"));
        assert!(s.contains("Max-Age=0"));
        assert!(!s.contains("Secure"));
    }
}
