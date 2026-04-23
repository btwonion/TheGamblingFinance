//! Per-IP rate limiter for login attempts.
//!
//! Quota: 5 requests per 12 seconds per IP, with a burst of 5. That
//! means ~5 attempts back-to-back before the client is throttled, and
//! then one fresh attempt every 12 seconds — enough to cover "typo,
//! fat-fingered password, try again" but meaningfully slow for a
//! brute-forcer. Combined with the 1-second synthetic delay inside
//! the login handler itself (see `routes::auth`), the wall-clock cost
//! per attempt is bounded below.
//!
//! The `tower_governor` layer returns its own 429 response shape on
//! overflow; it is NOT the `ErrorEnvelope` JSON the rest of the app
//! uses. We accept that mismatch deliberately: the prompt's
//! "identical error text" rule applies to the login handler's
//! wrong-password / disabled / no-user paths, not to the transport-
//! level 429. Swapping to a custom `error_handler` closure to emit
//! our envelope is a follow-up if the frontend starts switching on
//! response codes.

use std::sync::Arc;

use governor::middleware::NoOpMiddleware;
use tower_governor::{
    governor::{GovernorConfig, GovernorConfigBuilder},
    key_extractor::PeerIpKeyExtractor,
    GovernorLayer,
};

/// Build the login rate-limit layer. The returned `GovernorLayer`
/// is `Clone`, so call sites can reuse it across routes; typical
/// usage is
/// `.route("/login", post(login).route_layer(login_rate_limit_layer()))`.
///
/// `PeerIpKeyExtractor` keys on the socket peer IP — adequate for a
/// home-poker app on a single VPS. A CDN-fronted deployment would
/// want `SmartIpKeyExtractor` reading `X-Forwarded-For`.
pub fn login_rate_limit_layer() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware> {
    // 5 requests per 12 seconds, burst 5. `.finish()` returns `None`
    // only if burst or period is zero; neither is, so `.expect(...)`
    // is unreachable.
    let config: GovernorConfig<PeerIpKeyExtractor, NoOpMiddleware> =
        GovernorConfigBuilder::default()
            .per_second(12)
            .burst_size(5)
            .finish()
            .expect("login rate-limit config: non-zero period and burst");

    GovernorLayer {
        config: Arc::new(config),
    }
}
