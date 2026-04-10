use async_trait::async_trait;
use std::collections::HashMap;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::types::user::{RecipeUserId, User};
use crate::user_context::UserContext;

pub struct RecipeImplementationImpl {
    pub querier: Querier,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn get_users(
        &self,
        tenant_id: &str,
        time_joined_order: &str,
        limit: Option<u64>,
        pagination_token: Option<&str>,
        include_recipe_ids: Option<&[String]>,
        query: Option<&HashMap<String, String>>,
        user_context: &mut UserContext,
    ) -> Result<GetUsersResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("timeJoinedOrder".to_string(), time_joined_order.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        if let Some(t) = pagination_token {
            params.insert("paginationToken".to_string(), t.to_string());
        }
        if let Some(ids) = include_recipe_ids {
            params.insert("includeRecipeIds".to_string(), ids.join(","));
        }
        if let Some(q) = query {
            params.extend(q.clone());
        }

        let path = NormalisedURLPath::new(&format!("/{}/users", tenant_id))?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let users: Vec<User> = response
            .get("users")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|u| serde_json::from_value(u.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        let next_pagination_token = response
            .get("nextPaginationToken")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(GetUsersResult {
            users,
            next_pagination_token,
        })
    }

    async fn can_create_primary_user(
        &self,
        recipe_user_id: &RecipeUserId,
        user_context: &mut UserContext,
    ) -> Result<CanCreatePrimaryUserResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert(
            "recipeUserId".to_string(),
            recipe_user_id.get_as_string().to_string(),
        );

        let path = NormalisedURLPath::new("/recipe/accountlinking/user/primary/check")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => {
                let was_already = response
                    .get("wasAlreadyAPrimaryUser")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(CanCreatePrimaryUserResult::Ok {
                    was_already_a_primary_user: was_already,
                })
            }
            "RECIPE_USER_ID_ALREADY_LINKED_WITH_PRIMARY_USER_ID_ERROR" => {
                Ok(CanCreatePrimaryUserResult::RecipeUserIdAlreadyLinked {
                    primary_user_id: response
                        .get("primaryUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: response
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
            }
            "ACCOUNT_INFO_ALREADY_ASSOCIATED_WITH_ANOTHER_PRIMARY_USER_ID_ERROR" => {
                Ok(CanCreatePrimaryUserResult::AccountInfoAlreadyAssociated {
                    primary_user_id: response
                        .get("primaryUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: response
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn create_primary_user(
        &self,
        recipe_user_id: &RecipeUserId,
        user_context: &mut UserContext,
    ) -> Result<CreatePrimaryUserResult, SuperTokensError> {
        let body = serde_json::json!({
            "recipeUserId": recipe_user_id.get_as_string(),
        });

        let path = NormalisedURLPath::new("/recipe/accountlinking/user/primary")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => {
                let user: User =
                    serde_json::from_value(response.get("user").cloned().unwrap_or_default())?;
                let was_already = response
                    .get("wasAlreadyAPrimaryUser")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(CreatePrimaryUserResult::Ok {
                    user: Box::new(user),
                    was_already_a_primary_user: was_already,
                })
            }
            "RECIPE_USER_ID_ALREADY_LINKED_WITH_PRIMARY_USER_ID_ERROR" => {
                Ok(CreatePrimaryUserResult::RecipeUserIdAlreadyLinked {
                    primary_user_id: response
                        .get("primaryUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
            }
            "ACCOUNT_INFO_ALREADY_ASSOCIATED_WITH_ANOTHER_PRIMARY_USER_ID_ERROR" => {
                Ok(CreatePrimaryUserResult::AccountInfoAlreadyAssociated {
                    primary_user_id: response
                        .get("primaryUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn can_link_accounts(
        &self,
        recipe_user_id: &RecipeUserId,
        primary_user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CanLinkAccountsResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert(
            "recipeUserId".to_string(),
            recipe_user_id.get_as_string().to_string(),
        );
        params.insert("primaryUserId".to_string(), primary_user_id.to_string());

        let path = NormalisedURLPath::new("/recipe/accountlinking/user/link/check")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => {
                let already = response
                    .get("accountsAlreadyLinked")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(CanLinkAccountsResult::Ok {
                    accounts_already_linked: already,
                })
            }
            "RECIPE_USER_ID_ALREADY_LINKED_WITH_ANOTHER_PRIMARY_USER_ID_ERROR" => {
                let user = response
                    .get("user")
                    .and_then(|v| serde_json::from_value::<User>(v.clone()).ok())
                    .map(Box::new);
                Ok(CanLinkAccountsResult::RecipeUserIdAlreadyLinked {
                    primary_user_id: response
                        .get("primaryUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    user,
                })
            }
            "ACCOUNT_INFO_ALREADY_ASSOCIATED_WITH_ANOTHER_PRIMARY_USER_ID_ERROR" => {
                Ok(CanLinkAccountsResult::AccountInfoAlreadyAssociated {
                    primary_user_id: response
                        .get("primaryUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
            }
            "INPUT_USER_IS_NOT_A_PRIMARY_USER" => Ok(CanLinkAccountsResult::InputUserNotPrimary),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn link_accounts(
        &self,
        recipe_user_id: &RecipeUserId,
        primary_user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<LinkAccountsResult, SuperTokensError> {
        let body = serde_json::json!({
            "recipeUserId": recipe_user_id.get_as_string(),
            "primaryUserId": primary_user_id,
        });

        let path = NormalisedURLPath::new("/recipe/accountlinking/user/link")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => {
                let user: User =
                    serde_json::from_value(response.get("user").cloned().unwrap_or_default())?;
                let already = response
                    .get("accountsAlreadyLinked")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(LinkAccountsResult::Ok {
                    accounts_already_linked: already,
                    user: Box::new(user),
                })
            }
            "RECIPE_USER_ID_ALREADY_LINKED_WITH_ANOTHER_PRIMARY_USER_ID_ERROR" => {
                let user: User =
                    serde_json::from_value(response.get("user").cloned().unwrap_or_default())?;
                Ok(LinkAccountsResult::RecipeUserIdAlreadyLinked {
                    primary_user_id: response
                        .get("primaryUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    user: Box::new(user),
                })
            }
            "ACCOUNT_INFO_ALREADY_ASSOCIATED_WITH_ANOTHER_PRIMARY_USER_ID_ERROR" => {
                Ok(LinkAccountsResult::AccountInfoAlreadyAssociated {
                    primary_user_id: response
                        .get("primaryUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
            }
            "INPUT_USER_IS_NOT_A_PRIMARY_USER" => Ok(LinkAccountsResult::InputUserNotPrimary),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn unlink_account(
        &self,
        recipe_user_id: &RecipeUserId,
        user_context: &mut UserContext,
    ) -> Result<UnlinkAccountOkResult, SuperTokensError> {
        let body = serde_json::json!({
            "recipeUserId": recipe_user_id.get_as_string(),
        });

        let path = NormalisedURLPath::new("/recipe/accountlinking/user/unlink")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        Ok(UnlinkAccountOkResult {
            was_recipe_user_deleted: response
                .get("wasRecipeUserDeleted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            was_linked: response
                .get("wasLinked")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }

    async fn get_user(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<User>, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("userId".to_string(), user_id.to_string());

        let path = NormalisedURLPath::new("/user/id")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        if status == "UNKNOWN_USER_ID_ERROR" {
            return Ok(None);
        }

        let user = response
            .get("user")
            .and_then(|v| serde_json::from_value::<User>(v.clone()).ok());

        Ok(user)
    }

    async fn list_users_by_account_info(
        &self,
        tenant_id: &str,
        account_info: &AccountInfoInput,
        do_union_of_account_info: bool,
        user_context: &mut UserContext,
    ) -> Result<Vec<User>, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert(
            "doUnionOfAccountInfo".to_string(),
            do_union_of_account_info.to_string(),
        );
        if let Some(ref email) = account_info.email {
            params.insert("email".to_string(), email.clone());
        }
        if let Some(ref phone) = account_info.phone_number {
            params.insert("phoneNumber".to_string(), phone.clone());
        }
        if let Some(ref tp_id) = account_info.third_party_id {
            params.insert("thirdPartyId".to_string(), tp_id.clone());
        }
        if let Some(ref tp_uid) = account_info.third_party_user_id {
            params.insert("thirdPartyUserId".to_string(), tp_uid.clone());
        }

        let path = NormalisedURLPath::new(&format!("/{}/users/by-accountinfo", tenant_id))?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let users: Vec<User> = response
            .get("users")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|u| serde_json::from_value(u.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(users)
    }

    async fn delete_user(
        &self,
        user_id: &str,
        remove_all_linked_accounts: bool,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "removeAllLinkedAccounts": remove_all_linked_accounts,
        });

        let path = NormalisedURLPath::new("/user/remove")?;
        self.querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        Ok(())
    }
}
