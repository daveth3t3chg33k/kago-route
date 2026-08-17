//! Shared-secret auth middleware for the USSD callback route.
//!
//! Runs as a middleware *around* the handler, so unauthorized requests are
//! rejected before their body is ever read.

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::routes::CALLBACK_KEY_HEADER;

/// Captured state for the middleware: the accepted shared secrets.
/// Empty list = unauthenticated (dev mode; warned at boot).
#[derive(Debug, Clone)]
pub struct CallbackAuth {
    pub secrets: Vec<String>,
}

pub async fn require_callback_key(
    State(auth): State<CallbackAuth>,
    request: Request,
    next: Next,
) -> Response {
    if !auth.secrets.is_empty() {
        let provided = request
            .headers()
            .get(CALLBACK_KEY_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        if !auth.secrets.iter().any(|s| secrets_equal(provided, s)) {
            tracing::warn!("USSD callback rejected: missing or invalid key header");
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }
    next.run(request).await
}

/// Constant-time string comparison for secret values (avoids timing side
/// channels on byte-wise mismatches; length is compared upfront, which is the
/// standard accepted trade-off).
pub fn secrets_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.as_bytes().iter().zip(b.as_bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_equal_basic() {
        assert!(secrets_equal("sekrit", "sekrit"));
        assert!(!secrets_equal("sekrit", "nope"));
        assert!(!secrets_equal("sekrit", "sekri"));
        assert!(!secrets_equal("", "sekrit"));
        assert!(secrets_equal("", ""));
    }
}
