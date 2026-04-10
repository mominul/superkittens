use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::types::user::{RecipeUserId, User};
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Get paginated list of users.
    async fn get_users(
        &self,
        tenant_id: &str,
        time_joined_order: &str, // "ASC" or "DESC"
        limit: Option<u64>,
        pagination_token: Option<&str>,
        include_recipe_ids: Option<&[String]>,
        query: Option<&std::collections::HashMap<String, String>>,
        user_context: &mut UserContext,
    ) -> Result<GetUsersResult, SuperTokensError>;

    /// Check if a recipe user can become a primary user.
    async fn can_create_primary_user(
        &self,
        recipe_user_id: &RecipeUserId,
        user_context: &mut UserContext,
    ) -> Result<CanCreatePrimaryUserResult, SuperTokensError>;

    /// Make a recipe user a primary user.
    async fn create_primary_user(
        &self,
        recipe_user_id: &RecipeUserId,
        user_context: &mut UserContext,
    ) -> Result<CreatePrimaryUserResult, SuperTokensError>;

    /// Check if accounts can be linked.
    async fn can_link_accounts(
        &self,
        recipe_user_id: &RecipeUserId,
        primary_user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CanLinkAccountsResult, SuperTokensError>;

    /// Link a recipe user to a primary user.
    async fn link_accounts(
        &self,
        recipe_user_id: &RecipeUserId,
        primary_user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<LinkAccountsResult, SuperTokensError>;

    /// Unlink a recipe user from its primary user.
    async fn unlink_account(
        &self,
        recipe_user_id: &RecipeUserId,
        user_context: &mut UserContext,
    ) -> Result<UnlinkAccountOkResult, SuperTokensError>;

    /// Get a user by ID.
    async fn get_user(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<User>, SuperTokensError>;

    /// List users matching account info.
    async fn list_users_by_account_info(
        &self,
        tenant_id: &str,
        account_info: &AccountInfoInput,
        do_union_of_account_info: bool,
        user_context: &mut UserContext,
    ) -> Result<Vec<User>, SuperTokensError>;

    /// Delete a user.
    async fn delete_user(
        &self,
        user_id: &str,
        remove_all_linked_accounts: bool,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError>;
}
