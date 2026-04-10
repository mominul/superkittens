use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::types::user::RecipeUserId;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Resolve the tenant ID from the frontend-provided value.
    async fn get_tenant_id(
        &self,
        tenant_id_from_frontend: &str,
        user_context: &mut UserContext,
    ) -> Result<String, SuperTokensError>;

    /// Create or update a tenant.
    async fn create_or_update_tenant(
        &self,
        tenant_id: &str,
        config: Option<&TenantConfigCreateOrUpdate>,
        user_context: &mut UserContext,
    ) -> Result<CreateOrUpdateTenantOkResult, SuperTokensError>;

    /// Delete a tenant.
    async fn delete_tenant(
        &self,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<DeleteTenantOkResult, SuperTokensError>;

    /// Get a tenant's configuration.
    async fn get_tenant(
        &self,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<TenantConfig>, SuperTokensError>;

    /// List all tenants.
    async fn list_all_tenants(
        &self,
        user_context: &mut UserContext,
    ) -> Result<ListAllTenantsOkResult, SuperTokensError>;

    /// Create or update a third-party provider config for a tenant.
    async fn create_or_update_third_party_config(
        &self,
        tenant_id: &str,
        config: &ProviderConfig,
        skip_validation: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<CreateOrUpdateThirdPartyConfigOkResult, SuperTokensError>;

    /// Delete a third-party provider config from a tenant.
    async fn delete_third_party_config(
        &self,
        tenant_id: &str,
        third_party_id: &str,
        user_context: &mut UserContext,
    ) -> Result<DeleteThirdPartyConfigOkResult, SuperTokensError>;

    /// Associate a user with a tenant.
    async fn associate_user_to_tenant(
        &self,
        tenant_id: &str,
        recipe_user_id: &RecipeUserId,
        user_context: &mut UserContext,
    ) -> Result<AssociateUserToTenantResult, SuperTokensError>;

    /// Disassociate a user from a tenant.
    async fn disassociate_user_from_tenant(
        &self,
        tenant_id: &str,
        recipe_user_id: &RecipeUserId,
        user_context: &mut UserContext,
    ) -> Result<DisassociateUserFromTenantOkResult, SuperTokensError>;
}
