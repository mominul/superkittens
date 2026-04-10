use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

use super::constants::*;
use super::types::{NormalisedSessionConfig, TokenTransferMethod, TokenType};
use crate::framework::request::BaseRequest;
use crate::framework::response::BaseResponse;
use crate::user_context::UserContext;

/// Build the front-token header value.
///
/// Contains user_id, access token expiry, and custom access token payload,
/// base64-encoded as compact JSON.
pub fn build_front_token(
    user_id: &str,
    at_expiry: u64,
    access_token_payload: Option<&serde_json::Value>,
) -> String {
    let obj = serde_json::json!({
        "uid": user_id,
        "ate": at_expiry,
        "up": access_token_payload.unwrap_or(&serde_json::Value::Object(serde_json::Map::new())),
    });
    let json_str = serde_json::to_string(&obj).unwrap_or_default();
    URL_SAFE_NO_PAD.encode(json_str.as_bytes())
}

/// Get the CORS headers needed by the session recipe.
pub fn get_cors_allowed_headers() -> Vec<String> {
    vec![
        ANTI_CSRF_HEADER_KEY.to_string(),
        RID_HEADER_KEY.to_string(),
        AUTHORIZATION_HEADER_KEY.to_string(),
        AUTH_MODE_HEADER_KEY.to_string(),
    ]
}

/// Get a cookie value from the request (URL-decoded).
pub fn get_cookie(request: &dyn BaseRequest, key: &str) -> Option<String> {
    request.get_cookie(key).map(|v| {
        urlencoding::decode(&v)
            .unwrap_or(std::borrow::Cow::Borrowed(&v))
            .into_owned()
    })
}

/// Get a token from the request, either from cookies or headers.
pub fn get_token(
    request: &dyn BaseRequest,
    token_type: TokenType,
    transfer_method: TokenTransferMethod,
) -> Option<String> {
    match transfer_method {
        TokenTransferMethod::Cookie => {
            let key = get_cookie_name_from_token_type(token_type);
            get_cookie(request, key)
        }
        TokenTransferMethod::Header => {
            let auth_header = request.get_header(AUTHORIZATION_HEADER_KEY)?;
            let trimmed = auth_header.trim();
            if trimmed.to_lowercase().starts_with("bearer ") {
                Some(trimmed[7..].to_string())
            } else {
                None
            }
        }
    }
}

/// Get the cookie name for a token type.
pub fn get_cookie_name_from_token_type(token_type: TokenType) -> &'static str {
    match token_type {
        TokenType::Access => ACCESS_TOKEN_COOKIE_KEY,
        TokenType::Refresh => REFRESH_TOKEN_COOKIE_KEY,
    }
}

/// Get the response header name for a token type.
pub fn get_response_header_name_for_token_type(token_type: TokenType) -> &'static str {
    match token_type {
        TokenType::Access => ACCESS_TOKEN_HEADER_KEY,
        TokenType::Refresh => REFRESH_TOKEN_HEADER_KEY,
    }
}

/// Set a header on the response, optionally allowing duplicates.
pub fn set_header(response: &mut dyn BaseResponse, key: &str, value: &str, allow_duplicate: bool) {
    if allow_duplicate {
        if let Some(existing) = response.get_header(key) {
            response.set_header(key, &format!("{}, {}", existing, value));
        } else {
            response.set_header(key, value);
        }
    } else {
        response.set_header(key, value);
    }
}

/// Set a token in the response via the header transfer method.
pub fn set_token_in_header(response: &mut dyn BaseResponse, token_type: TokenType, value: &str) {
    let header_key = get_response_header_name_for_token_type(token_type);
    response.set_header(header_key, value);

    // Add to Access-Control-Expose-Headers
    set_header(response, ACCESS_CONTROL_EXPOSE_HEADERS, header_key, true);
}

/// Set a token on the response (cookie or header).
pub fn set_token(
    response: &mut dyn BaseResponse,
    config: &NormalisedSessionConfig,
    token_type: TokenType,
    value: &str,
    expires: u64,
    transfer_method: TokenTransferMethod,
    request: &dyn BaseRequest,
    user_context: &UserContext,
) {
    match transfer_method {
        TokenTransferMethod::Cookie => {
            set_cookie(
                response,
                config,
                get_cookie_name_from_token_type(token_type),
                value,
                expires,
                token_type,
                request,
                config.cookie_domain.as_deref(),
                user_context,
            );
        }
        TokenTransferMethod::Header => {
            set_token_in_header(response, token_type, value);
        }
    }
}

/// Set the access token in the response, including the front-token header.
pub fn set_access_token_in_response(
    response: &mut dyn BaseResponse,
    access_token: &str,
    front_token: &str,
    config: &NormalisedSessionConfig,
    transfer_method: TokenTransferMethod,
    request: &dyn BaseRequest,
    user_context: &UserContext,
) {
    // Set the front token header
    set_header(response, FRONT_TOKEN_HEADER_SET_KEY, front_token, false);
    set_header(
        response,
        ACCESS_CONTROL_EXPOSE_HEADERS,
        FRONT_TOKEN_HEADER_SET_KEY,
        true,
    );

    // Set access token with 1 year expiry for cookies (actual expiry is in JWT)
    let one_year_ms = crate::constants::ONE_YEAR_IN_MS;
    let expires = crate::utils::get_timestamp_ms() + one_year_ms;

    set_token(
        response,
        config,
        TokenType::Access,
        access_token,
        expires,
        transfer_method,
        request,
        user_context,
    );

    // If configured, also expose access token in header for cookie-based auth
    if config.expose_access_token_to_frontend_in_cookie_based_auth
        && transfer_method == TokenTransferMethod::Cookie
    {
        set_token_in_header(response, TokenType::Access, access_token);
    }
}

/// Clear all session tokens from the response.
pub fn clear_session(
    response: &mut dyn BaseResponse,
    config: &NormalisedSessionConfig,
    transfer_method: TokenTransferMethod,
    request: &dyn BaseRequest,
    user_context: &UserContext,
) {
    // Set empty tokens with 0 expiry
    set_token(
        response,
        config,
        TokenType::Access,
        "",
        0,
        transfer_method,
        request,
        user_context,
    );
    set_token(
        response,
        config,
        TokenType::Refresh,
        "",
        0,
        transfer_method,
        request,
        user_context,
    );

    // Remove anti-CSRF header
    response.remove_header(ANTI_CSRF_HEADER_KEY);

    // Set front token to "remove"
    set_header(response, FRONT_TOKEN_HEADER_SET_KEY, "remove", false);
    set_header(
        response,
        ACCESS_CONTROL_EXPOSE_HEADERS,
        FRONT_TOKEN_HEADER_SET_KEY,
        true,
    );
}

/// Clear session from all transfer methods (both cookie and header).
pub fn clear_session_from_all_token_transfer_methods(
    response: &mut dyn BaseResponse,
    config: &NormalisedSessionConfig,
    request: &dyn BaseRequest,
    user_context: &UserContext,
) {
    clear_session(
        response,
        config,
        TokenTransferMethod::Cookie,
        request,
        user_context,
    );
    clear_session(
        response,
        config,
        TokenTransferMethod::Header,
        request,
        user_context,
    );
}

/// Set the anti-CSRF token header.
pub fn attach_anti_csrf_header(response: &mut dyn BaseResponse, value: &str) {
    response.set_header(ANTI_CSRF_HEADER_KEY, value);
    set_header(
        response,
        ACCESS_CONTROL_EXPOSE_HEADERS,
        ANTI_CSRF_HEADER_KEY,
        true,
    );
}

/// Get the auth mode from the request header.
pub fn get_auth_mode_from_header(request: &dyn BaseRequest) -> Option<String> {
    request.get_header(AUTH_MODE_HEADER_KEY)
}

// Internal helper for setting cookies

fn set_cookie(
    response: &mut dyn BaseResponse,
    config: &NormalisedSessionConfig,
    key: &str,
    value: &str,
    expires: u64,
    token_type: TokenType,
    request: &dyn BaseRequest,
    domain: Option<&str>,
    user_context: &UserContext,
) {
    let encoded_value = urlencoding::encode(value).into_owned();
    let same_site = (config.get_cookie_same_site)(request, user_context);
    let path = match token_type {
        TokenType::Access => "/",
        TokenType::Refresh => config.refresh_token_path.get_as_string_dangerous(),
    };

    response.set_cookie(
        key,
        &encoded_value,
        expires,
        path,
        domain,
        config.cookie_secure,
        true, // httponly
        same_site,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    #[test]
    fn test_build_front_token_basic() {
        let token = build_front_token("user123", 1000, None);
        let decoded = URL_SAFE_NO_PAD.decode(token.as_bytes()).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(json["uid"], "user123");
        assert_eq!(json["ate"], 1000);
        assert_eq!(json["up"], serde_json::json!({}));
    }

    #[test]
    fn test_build_front_token_with_payload() {
        let payload = serde_json::json!({"role": "admin"});
        let token = build_front_token("u1", 5000, Some(&payload));
        let decoded = URL_SAFE_NO_PAD.decode(token.as_bytes()).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(json["uid"], "u1");
        assert_eq!(json["ate"], 5000);
        assert_eq!(json["up"]["role"], "admin");
    }

    #[test]
    fn test_get_cors_allowed_headers() {
        let headers = get_cors_allowed_headers();
        assert_eq!(headers.len(), 4);
        assert!(headers.contains(&ANTI_CSRF_HEADER_KEY.to_string()));
        assert!(headers.contains(&RID_HEADER_KEY.to_string()));
        assert!(headers.contains(&AUTHORIZATION_HEADER_KEY.to_string()));
        assert!(headers.contains(&AUTH_MODE_HEADER_KEY.to_string()));
    }

    #[test]
    fn test_get_cookie_name_from_token_type_access() {
        assert_eq!(
            get_cookie_name_from_token_type(TokenType::Access),
            ACCESS_TOKEN_COOKIE_KEY
        );
    }

    #[test]
    fn test_get_cookie_name_from_token_type_refresh() {
        assert_eq!(
            get_cookie_name_from_token_type(TokenType::Refresh),
            REFRESH_TOKEN_COOKIE_KEY
        );
    }

    #[test]
    fn test_get_response_header_name_for_token_type() {
        assert_eq!(
            get_response_header_name_for_token_type(TokenType::Access),
            ACCESS_TOKEN_HEADER_KEY
        );
        assert_eq!(
            get_response_header_name_for_token_type(TokenType::Refresh),
            REFRESH_TOKEN_HEADER_KEY
        );
    }
}
