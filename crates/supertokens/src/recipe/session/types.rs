use crate::types::user::RecipeUserId;
use serde::{Deserialize, Serialize};

/// Token type: access or refresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Access,
    Refresh,
}

impl TokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Access => "access",
            Self::Refresh => "refresh",
        }
    }
}

/// How tokens are transferred to the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenTransferMethod {
    Cookie,
    Header,
}

impl TokenTransferMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cookie => "cookie",
            Self::Header => "header",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cookie" => Some(Self::Cookie),
            "header" => Some(Self::Header),
            _ => None,
        }
    }
}

/// Token information (token string + expiry + creation time).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub token: String,
    pub expiry: u64,
    #[serde(rename = "createdTime")]
    pub created_time: u64,
}

/// Session object returned by Core API.
#[derive(Debug, Clone)]
pub struct SessionObj {
    pub handle: String,
    pub user_id: String,
    pub recipe_user_id: RecipeUserId,
    pub user_data_in_jwt: serde_json::Value,
    pub tenant_id: String,
}

/// Access token object.
#[derive(Debug, Clone)]
pub struct AccessTokenObj {
    pub token: String,
    pub expiry: u64,
    pub created_time: u64,
}

/// Session information from the Core.
#[derive(Debug, Clone)]
pub struct SessionInformationResult {
    pub session_handle: String,
    pub user_id: String,
    pub recipe_user_id: RecipeUserId,
    pub session_data_in_database: serde_json::Value,
    pub expiry: u64,
    pub custom_claims_in_access_token_payload: serde_json::Value,
    pub time_created: u64,
    pub tenant_id: String,
}

/// Result of regenerating an access token.
#[derive(Debug, Clone)]
pub struct RegenerateAccessTokenOkResult {
    pub session: SessionObj,
    pub access_token: Option<AccessTokenObj>,
}

/// Result of claims validation.
#[derive(Debug, Clone)]
pub struct ClaimsValidationResult {
    pub invalid_claims: Vec<ClaimValidationError>,
    pub access_token_payload_update: Option<serde_json::Value>,
}

/// A single claim validation failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimValidationError {
    pub id: String,
    pub reason: Option<serde_json::Value>,
}

/// Claim validation result for a single validator.
#[derive(Debug, Clone)]
pub struct SingleClaimValidationResult {
    pub is_valid: bool,
    pub reason: Option<serde_json::Value>,
}

/// Request/response info attached to a session.
#[derive(Debug, Clone)]
pub struct ReqResInfo {
    pub transfer_method: TokenTransferMethod,
}

/// All session tokens (dangerous — exposes raw token values).
#[derive(Debug, Clone)]
pub struct GetSessionTokensDangerously {
    pub access_token: String,
    pub access_and_front_token_updated: bool,
    pub refresh_token: Option<String>,
    pub front_token: String,
    pub anti_csrf_token: Option<String>,
}

/// Internal response from Core create/refresh session API.
#[derive(Debug, Clone)]
pub struct CreateOrRefreshApiResponse {
    pub session: SessionObj,
    pub access_token: TokenInfo,
    pub refresh_token: TokenInfo,
    pub anti_csrf_token: Option<String>,
}

/// Internal response from Core get session API.
#[derive(Debug, Clone)]
pub struct GetSessionApiResponse {
    pub session: GetSessionApiResponseSession,
    pub access_token: Option<AccessTokenObj>,
}

#[derive(Debug, Clone)]
pub struct GetSessionApiResponseSession {
    pub handle: String,
    pub user_id: String,
    pub recipe_user_id: RecipeUserId,
    pub user_data_in_jwt: serde_json::Value,
    pub expiry_time: u64,
    pub tenant_id: String,
}

/// Result of get_info_from_access_token.
#[derive(Debug, Clone)]
pub struct AccessTokenInfo {
    pub session_handle: String,
    pub user_id: String,
    pub recipe_user_id: RecipeUserId,
    pub refresh_token_hash1: String,
    pub parent_refresh_token_hash1: Option<String>,
    pub user_data: serde_json::Value,
    pub anti_csrf_token: Option<String>,
    pub expiry_time: u64,
    pub time_created: u64,
    pub tenant_id: String,
}

/// Session config provided by the user.
#[derive(Clone, Default)]
pub struct SessionConfig {
    pub cookie_domain: Option<String>,
    pub older_cookie_domain: Option<String>,
    pub cookie_secure: Option<bool>,
    pub cookie_same_site: Option<String>,
    pub session_expired_status_code: Option<u16>,
    pub anti_csrf: Option<String>,
    pub get_token_transfer_method: Option<GetTokenTransferMethodFn>,
    pub error_handlers: Option<ErrorHandlers>,
    pub invalid_claim_status_code: Option<u16>,
    pub use_dynamic_access_token_signing_key: Option<bool>,
    pub expose_access_token_to_frontend_in_cookie_based_auth: Option<bool>,
    pub jwks_refresh_interval_sec: Option<u64>,
}

/// Normalised (validated) session config.
#[derive(Clone)]
pub struct NormalisedSessionConfig {
    pub refresh_token_path: crate::normalised_url_path::NormalisedURLPath,
    pub cookie_domain: Option<String>,
    pub older_cookie_domain: Option<String>,
    pub cookie_secure: bool,
    pub session_expired_status_code: u16,
    pub invalid_claim_status_code: u16,
    pub anti_csrf_function_or_string: AntiCsrfConfig,
    pub use_dynamic_access_token_signing_key: bool,
    pub expose_access_token_to_frontend_in_cookie_based_auth: bool,
    pub jwks_refresh_interval_sec: u64,
    pub error_handlers: ErrorHandlers,
    pub get_token_transfer_method: GetTokenTransferMethodFn,
    pub get_cookie_same_site: GetCookieSameSiteFn,
}

/// Anti-CSRF configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AntiCsrfConfig {
    ViaToken,
    ViaCustomHeader,
    None,
}

impl AntiCsrfConfig {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ViaToken => "VIA_TOKEN",
            Self::ViaCustomHeader => "VIA_CUSTOM_HEADER",
            Self::None => "NONE",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "VIA_TOKEN" => Some(Self::ViaToken),
            "VIA_CUSTOM_HEADER" => Some(Self::ViaCustomHeader),
            "NONE" => Some(Self::None),
            _ => None,
        }
    }
}

/// Callback to determine token transfer method.
pub type GetTokenTransferMethodFn = std::sync::Arc<
    dyn Fn(
            &dyn crate::framework::request::BaseRequest,
            bool,
            &crate::user_context::UserContext,
        ) -> TokenTransferMethod
        + Send
        + Sync,
>;

/// Callback to determine cookie SameSite attribute.
pub type GetCookieSameSiteFn = std::sync::Arc<
    dyn Fn(
            &dyn crate::framework::request::BaseRequest,
            &crate::user_context::UserContext,
        ) -> crate::framework::response::SameSite
        + Send
        + Sync,
>;

/// A response mutator: a callback that modifies the HTTP response.
pub type ResponseMutator = Box<
    dyn Fn(&mut dyn crate::framework::response::BaseResponse, &crate::user_context::UserContext)
        + Send
        + Sync,
>;

/// Error handler callbacks.
#[derive(Clone)]
pub struct ErrorHandlers {
    pub on_unauthorised: std::sync::Arc<
        dyn Fn(
                String,
                &dyn crate::framework::request::BaseRequest,
                &mut dyn crate::framework::response::BaseResponse,
            ) + Send
            + Sync,
    >,
    pub on_token_theft_detected: std::sync::Arc<
        dyn Fn(
                String,
                String,
                &dyn crate::framework::request::BaseRequest,
                &mut dyn crate::framework::response::BaseResponse,
            ) + Send
            + Sync,
    >,
    pub on_invalid_claim: std::sync::Arc<
        dyn Fn(
                Vec<ClaimValidationError>,
                &dyn crate::framework::request::BaseRequest,
                &mut dyn crate::framework::response::BaseResponse,
            ) + Send
            + Sync,
    >,
}

impl std::fmt::Debug for NormalisedSessionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NormalisedSessionConfig")
            .field("cookie_domain", &self.cookie_domain)
            .field("cookie_secure", &self.cookie_secure)
            .field(
                "session_expired_status_code",
                &self.session_expired_status_code,
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_transfer_method_parse_cookie() {
        assert_eq!(
            TokenTransferMethod::parse("cookie"),
            Some(TokenTransferMethod::Cookie)
        );
    }

    #[test]
    fn test_token_transfer_method_parse_header() {
        assert_eq!(
            TokenTransferMethod::parse("header"),
            Some(TokenTransferMethod::Header)
        );
    }

    #[test]
    fn test_token_transfer_method_parse_case_insensitive() {
        assert_eq!(
            TokenTransferMethod::parse("COOKIE"),
            Some(TokenTransferMethod::Cookie)
        );
        assert_eq!(
            TokenTransferMethod::parse("Header"),
            Some(TokenTransferMethod::Header)
        );
    }

    #[test]
    fn test_token_transfer_method_parse_invalid() {
        assert_eq!(TokenTransferMethod::parse("bearer"), None);
        assert_eq!(TokenTransferMethod::parse(""), None);
    }

    #[test]
    fn test_token_type_as_str() {
        assert_eq!(TokenType::Access.as_str(), "access");
        assert_eq!(TokenType::Refresh.as_str(), "refresh");
    }

    #[test]
    fn test_anti_csrf_config_parse_and_as_str() {
        assert_eq!(
            AntiCsrfConfig::parse("VIA_TOKEN"),
            Some(AntiCsrfConfig::ViaToken)
        );
        assert_eq!(
            AntiCsrfConfig::parse("via_custom_header"),
            Some(AntiCsrfConfig::ViaCustomHeader)
        );
        assert_eq!(AntiCsrfConfig::parse("none"), Some(AntiCsrfConfig::None));
        assert_eq!(AntiCsrfConfig::parse("invalid"), None);

        assert_eq!(AntiCsrfConfig::ViaToken.as_str(), "VIA_TOKEN");
        assert_eq!(
            AntiCsrfConfig::ViaCustomHeader.as_str(),
            "VIA_CUSTOM_HEADER"
        );
        assert_eq!(AntiCsrfConfig::None.as_str(), "NONE");
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert!(config.cookie_domain.is_none());
        assert!(config.cookie_secure.is_none());
        assert!(config.cookie_same_site.is_none());
        assert!(config.session_expired_status_code.is_none());
        assert!(config.anti_csrf.is_none());
        assert!(config.get_token_transfer_method.is_none());
        assert!(config.error_handlers.is_none());
        assert!(config
            .expose_access_token_to_frontend_in_cookie_based_auth
            .is_none());
    }
}
