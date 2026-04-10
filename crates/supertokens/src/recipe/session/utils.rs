use super::types::*;
use crate::error::SuperTokensError;
use crate::framework::response::SameSite;
use crate::normalised_url_path::NormalisedURLPath;
use crate::types::config::AppInfo;
use std::sync::Arc;

/// Validate and normalise user-provided session configuration.
pub fn validate_and_normalise_user_input(
    app_info: &AppInfo,
    config: SessionConfig,
) -> Result<NormalisedSessionConfig, SuperTokensError> {
    let cookie_domain = config.cookie_domain.map(|d| normalise_session_scope(&d));
    let older_cookie_domain = config
        .older_cookie_domain
        .map(|d| normalise_session_scope(&d));
    let cookie_secure = config.cookie_secure.unwrap_or_else(|| {
        let api_domain = app_info.api_domain.get_as_string_dangerous();
        api_domain.starts_with("https://")
    });

    let session_expired_status_code = config.session_expired_status_code.unwrap_or(401);
    let invalid_claim_status_code = config.invalid_claim_status_code.unwrap_or(403);
    let use_dynamic_signing_key = config.use_dynamic_access_token_signing_key.unwrap_or(true);
    let expose_at_to_frontend = config
        .expose_access_token_to_frontend_in_cookie_based_auth
        .unwrap_or(false);
    let jwks_refresh_interval = config.jwks_refresh_interval_sec.unwrap_or(3600); // 1 hour

    let anti_csrf = match config.anti_csrf {
        Some(ref s) => AntiCsrfConfig::parse(s).unwrap_or(AntiCsrfConfig::None),
        None => {
            // Default: VIA_CUSTOM_HEADER for cookie-based, NONE for header-based
            AntiCsrfConfig::ViaCustomHeader
        }
    };

    let refresh_token_path = app_info
        .api_base_path
        .append(&NormalisedURLPath::new(super::constants::SESSION_REFRESH)?);

    let get_cookie_same_site: GetCookieSameSiteFn = match config.cookie_same_site.as_deref() {
        Some("strict") => Arc::new(|_, _| SameSite::Strict),
        Some("lax") => Arc::new(|_, _| SameSite::Lax),
        Some("none") => Arc::new(|_, _| SameSite::None),
        _ => Arc::new(|_, _| SameSite::Lax), // default
    };

    let get_token_transfer_method: GetTokenTransferMethodFn =
        config.get_token_transfer_method.unwrap_or_else(|| {
            Arc::new(|_request, _for_create_new_session, _user_context| {
                // Default: cookie
                TokenTransferMethod::Cookie
            })
        });

    let error_handlers = config.error_handlers.unwrap_or_else(|| ErrorHandlers {
        on_unauthorised: Arc::new(|msg, _req, resp| {
            crate::utils::send_non_200_response_with_message(&msg, 401, resp);
        }),
        on_token_theft_detected: Arc::new(|session_handle, _user_id, _req, resp| {
            crate::utils::send_non_200_response(
                serde_json::json!({
                    "message": "token theft detected",
                    "sessionHandle": session_handle,
                }),
                401,
                resp,
            );
        }),
        on_invalid_claim: Arc::new(|errors, _req, resp| {
            let claim_errors: Vec<_> = errors
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "reason": e.reason,
                    })
                })
                .collect();
            crate::utils::send_non_200_response(
                serde_json::json!({
                    "message": "invalid claims",
                    "claimValidationErrors": claim_errors,
                }),
                403,
                resp,
            );
        }),
    });

    Ok(NormalisedSessionConfig {
        refresh_token_path,
        cookie_domain,
        older_cookie_domain,
        cookie_secure,
        session_expired_status_code,
        invalid_claim_status_code,
        anti_csrf_function_or_string: anti_csrf,
        use_dynamic_access_token_signing_key: use_dynamic_signing_key,
        expose_access_token_to_frontend_in_cookie_based_auth: expose_at_to_frontend,
        jwks_refresh_interval_sec: jwks_refresh_interval,
        error_handlers,
        get_token_transfer_method,
        get_cookie_same_site,
    })
}

/// Normalise a session scope (cookie domain).
fn normalise_session_scope(scope: &str) -> String {
    let mut s = scope.trim().to_lowercase();
    // Remove leading dot
    s = s.trim_start_matches('.').to_string();
    // Remove protocol
    if let Some(idx) = s.find("://") {
        s = s[idx + 3..].to_string();
    }
    // Remove port
    if let Some(idx) = s.find(':') {
        s = s[..idx].to_string();
    }
    // Remove path
    if let Some(idx) = s.find('/') {
        s = s[..idx].to_string();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalise_session_scope() {
        assert_eq!(normalise_session_scope("example.com"), "example.com");
        assert_eq!(normalise_session_scope(".example.com"), "example.com");
        assert_eq!(
            normalise_session_scope("https://example.com:3000/path"),
            "example.com"
        );
        assert_eq!(normalise_session_scope("  EXAMPLE.COM  "), "example.com");
    }
}
