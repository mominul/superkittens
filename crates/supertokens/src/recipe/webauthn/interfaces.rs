use async_trait::async_trait;
use std::sync::Arc;

use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Generate registration options for WebAuthn.
    async fn register_options(
        &self,
        email: &str,
        rp_id: &str,
        rp_name: &str,
        origin: &str,
        timeout: Option<u64>,
        attestation: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RegisterOptionsResult, SuperTokensError>;

    /// Generate sign-in (authentication) options for WebAuthn.
    async fn sign_in_options(
        &self,
        rp_id: &str,
        origin: &str,
        timeout: Option<u64>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<SignInOptionsResult, SuperTokensError>;

    /// Sign up a new user with WebAuthn credentials.
    async fn sign_up(
        &self,
        webauthn_generated_options_id: &str,
        credential: serde_json::Value,
        tenant_id: &str,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<SignUpResult, SuperTokensError>;

    /// Sign in an existing user with WebAuthn credentials.
    async fn sign_in(
        &self,
        webauthn_generated_options_id: &str,
        credential: serde_json::Value,
        tenant_id: &str,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<SignInResult, SuperTokensError>;

    /// Verify WebAuthn credentials without creating a session.
    async fn verify_credentials(
        &self,
        webauthn_generated_options_id: &str,
        credential: serde_json::Value,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<VerifyCredentialsResult, SuperTokensError>;

    /// List all WebAuthn credentials for a user.
    async fn list_credentials(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ListCredentialsOkResult, SuperTokensError>;

    /// Remove a WebAuthn credential.
    async fn remove_credential(
        &self,
        credential_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RemoveCredentialOkResult, SuperTokensError>;

    /// Generate a recover account token for a user.
    async fn generate_recover_account_token(
        &self,
        user_id: &str,
        email: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<GenerateRecoverAccountTokenResult, SuperTokensError>;

    /// Consume a recover account token.
    async fn consume_recover_account_token(
        &self,
        token: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ConsumeRecoverAccountTokenResult, SuperTokensError>;
}
