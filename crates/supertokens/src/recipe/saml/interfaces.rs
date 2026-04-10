use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Create or update a SAML client (Service Provider).
    async fn create_or_update_client(
        &self,
        client_id: Option<&str>,
        redirect_uris: Option<&[String]>,
        issuer: Option<&str>,
        acs_url: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateOrUpdateClientResult, SuperTokensError>;

    /// List all SAML clients for a tenant.
    async fn list_clients(
        &self,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ListClientsOkResult, SuperTokensError>;

    /// Remove a SAML client.
    async fn remove_client(
        &self,
        client_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RemoveClientResult, SuperTokensError>;

    /// Create a SAML login request (generate a SAML AuthnRequest).
    async fn create_login_request(
        &self,
        client_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateLoginRequestResult, SuperTokensError>;

    /// Verify a SAML response from the IdP.
    async fn verify_saml_response(
        &self,
        client_id: &str,
        saml_response: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<VerifySAMLResponseResult, SuperTokensError>;
}
