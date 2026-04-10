use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    async fn add_role_to_user(
        &self,
        user_id: &str,
        role: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<AddRoleToUserResult, SuperTokensError>;

    async fn remove_user_role(
        &self,
        user_id: &str,
        role: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RemoveUserRoleResult, SuperTokensError>;

    async fn get_roles_for_user(
        &self,
        user_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<GetRolesForUserOkResult, SuperTokensError>;

    async fn get_users_that_have_role(
        &self,
        role: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<GetUsersThatHaveRoleResult, SuperTokensError>;

    async fn create_new_role_or_add_permissions(
        &self,
        role: &str,
        permissions: &[String],
        user_context: &mut UserContext,
    ) -> Result<CreateNewRoleOrAddPermissionsOkResult, SuperTokensError>;

    async fn get_permissions_for_role(
        &self,
        role: &str,
        user_context: &mut UserContext,
    ) -> Result<GetPermissionsForRoleResult, SuperTokensError>;

    async fn remove_permissions_from_role(
        &self,
        role: &str,
        permissions: &[String],
        user_context: &mut UserContext,
    ) -> Result<RemovePermissionsFromRoleResult, SuperTokensError>;

    async fn get_roles_that_have_permission(
        &self,
        permission: &str,
        user_context: &mut UserContext,
    ) -> Result<GetRolesThatHavePermissionOkResult, SuperTokensError>;

    async fn delete_role(
        &self,
        role: &str,
        user_context: &mut UserContext,
    ) -> Result<DeleteRoleOkResult, SuperTokensError>;

    async fn get_all_roles(
        &self,
        user_context: &mut UserContext,
    ) -> Result<GetAllRolesOkResult, SuperTokensError>;
}
