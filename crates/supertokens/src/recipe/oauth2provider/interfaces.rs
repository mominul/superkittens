use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Get a login request by challenge.
    async fn get_login_request(
        &self,
        challenge: &str,
        user_context: &mut UserContext,
    ) -> Result<LoginRequest, SuperTokensError>;

    /// Accept a login request.
    async fn accept_login_request(
        &self,
        challenge: &str,
        subject: &str,
        remember: bool,
        remember_for: u64,
        user_context: &mut UserContext,
    ) -> Result<LoginConsentResult, SuperTokensError>;

    /// Reject a login request.
    async fn reject_login_request(
        &self,
        challenge: &str,
        error: &str,
        error_description: &str,
        user_context: &mut UserContext,
    ) -> Result<LoginConsentResult, SuperTokensError>;

    /// Get a consent request by challenge.
    async fn get_consent_request(
        &self,
        challenge: &str,
        user_context: &mut UserContext,
    ) -> Result<ConsentRequest, SuperTokensError>;

    /// Accept a consent request.
    async fn accept_consent_request(
        &self,
        challenge: &str,
        grant_scope: &[String],
        grant_access_token_audience: &[String],
        remember: bool,
        remember_for: u64,
        user_context: &mut UserContext,
    ) -> Result<LoginConsentResult, SuperTokensError>;

    /// Reject a consent request.
    async fn reject_consent_request(
        &self,
        challenge: &str,
        error: &str,
        error_description: &str,
        user_context: &mut UserContext,
    ) -> Result<LoginConsentResult, SuperTokensError>;

    /// Handle the authorization endpoint.
    async fn authorization(
        &self,
        params: serde_json::Value,
        cookies: Option<&str>,
        session: Option<serde_json::Value>,
        user_context: &mut UserContext,
    ) -> Result<AuthorizationResult, SuperTokensError>;

    /// Handle the token exchange endpoint.
    async fn token_exchange(
        &self,
        authorization_header: Option<&str>,
        body: serde_json::Value,
        user_context: &mut UserContext,
    ) -> Result<TokenExchangeResult, SuperTokensError>;

    /// List OAuth2 clients.
    async fn get_oauth2_clients(
        &self,
        page_size: Option<u32>,
        pagination_token: Option<&str>,
        client_name: Option<&str>,
        user_context: &mut UserContext,
    ) -> Result<GetOAuth2ClientsOkResult, SuperTokensError>;

    /// Get a single OAuth2 client by ID.
    async fn get_oauth2_client(
        &self,
        client_id: &str,
        user_context: &mut UserContext,
    ) -> Result<OAuth2ClientResult, SuperTokensError>;

    /// Create a new OAuth2 client.
    async fn create_oauth2_client(
        &self,
        params: serde_json::Value,
        user_context: &mut UserContext,
    ) -> Result<OAuth2ClientResult, SuperTokensError>;

    /// Update an existing OAuth2 client.
    async fn update_oauth2_client(
        &self,
        client_id: &str,
        params: serde_json::Value,
        user_context: &mut UserContext,
    ) -> Result<OAuth2ClientResult, SuperTokensError>;

    /// Delete an OAuth2 client.
    async fn delete_oauth2_client(
        &self,
        client_id: &str,
        user_context: &mut UserContext,
    ) -> Result<DeleteOAuth2ClientResult, SuperTokensError>;

    /// Revoke a token.
    async fn revoke_token(
        &self,
        token: &str,
        client_id: &str,
        client_secret: Option<&str>,
        user_context: &mut UserContext,
    ) -> Result<RevokeTokenResult, SuperTokensError>;

    /// Introspect a token.
    async fn introspect_token(
        &self,
        token: &str,
        scopes: Option<&[String]>,
        user_context: &mut UserContext,
    ) -> Result<IntrospectTokenResult, SuperTokensError>;
}
