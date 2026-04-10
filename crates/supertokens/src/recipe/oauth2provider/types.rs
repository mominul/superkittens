use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// OAuth2 Provider types
// ---------------------------------------------------------------------------

/// An OAuth2 client configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Client {
    pub client_id: String,
    #[serde(default)]
    pub client_name: String,
    #[serde(default)]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub grant_types: Vec<String>,
    #[serde(default)]
    pub response_types: Vec<String>,
    #[serde(default)]
    pub token_endpoint_auth_method: String,
    #[serde(default)]
    pub client_uri: String,
    #[serde(default)]
    pub logo_uri: String,
    #[serde(default)]
    pub tos_uri: String,
    #[serde(default)]
    pub policy_uri: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Error response from OAuth2 endpoints.
#[derive(Debug, Clone)]
pub struct ErrorOAuth2Response {
    pub error: String,
    pub error_description: String,
    pub status_code: u16,
}

/// A redirect response from OAuth2 flow endpoints.
#[derive(Debug, Clone)]
pub struct RedirectResponse {
    pub redirect_to: String,
}

/// Token info response from the token exchange endpoint.
#[derive(Debug, Clone)]
pub struct TokenInfoResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: String,
    pub token_type: String,
}

/// An active token from introspection.
#[derive(Debug, Clone)]
pub struct ActiveTokenResponse {
    pub payload: serde_json::Value,
}

/// An inactive token from introspection.
#[derive(Debug, Clone)]
pub struct InactiveTokenResponse;

/// Result of token introspection.
#[derive(Debug, Clone)]
pub enum IntrospectTokenResult {
    Active(ActiveTokenResponse),
    Inactive(InactiveTokenResponse),
}

// ---------------------------------------------------------------------------
// Login/consent request types
// ---------------------------------------------------------------------------

/// An OAuth2 login request.
#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub challenge: String,
    pub client: OAuth2Client,
    pub request_url: String,
    pub requested_scope: Vec<String>,
    pub requested_access_token_audience: Vec<String>,
    pub skip: bool,
    pub subject: String,
    pub oidc_context: serde_json::Value,
}

/// An OAuth2 consent request.
#[derive(Debug, Clone)]
pub struct ConsentRequest {
    pub challenge: String,
    pub client: OAuth2Client,
    pub request_url: String,
    pub requested_scope: Vec<String>,
    pub requested_access_token_audience: Vec<String>,
    pub skip: bool,
    pub subject: String,
}

// ---------------------------------------------------------------------------
// Result types for client CRUD
// ---------------------------------------------------------------------------

/// Result of listing OAuth2 clients.
#[derive(Debug, Clone)]
pub struct GetOAuth2ClientsOkResult {
    pub clients: Vec<OAuth2Client>,
}

/// Result of getting/creating/updating an OAuth2 client.
#[derive(Debug, Clone)]
pub enum OAuth2ClientResult {
    Ok { client: Box<OAuth2Client> },
    Error(ErrorOAuth2Response),
}

/// Result of deleting an OAuth2 client.
#[derive(Debug, Clone)]
pub enum DeleteOAuth2ClientResult {
    Ok,
    Error(ErrorOAuth2Response),
}

/// Result of the authorization endpoint.
#[derive(Debug, Clone)]
pub enum AuthorizationResult {
    Redirect(RedirectResponse),
    Error(ErrorOAuth2Response),
}

/// Result of the token exchange endpoint.
#[derive(Debug, Clone)]
pub enum TokenExchangeResult {
    Ok(TokenInfoResponse),
    Error(ErrorOAuth2Response),
}

/// Result of revoking a token.
#[derive(Debug, Clone)]
pub enum RevokeTokenResult {
    Ok,
    Error(ErrorOAuth2Response),
}

/// Result of accepting/rejecting login or consent.
#[derive(Debug, Clone)]
pub enum LoginConsentResult {
    Redirect(RedirectResponse),
    Error(ErrorOAuth2Response),
}
