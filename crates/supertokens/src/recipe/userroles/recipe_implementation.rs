use async_trait::async_trait;
use std::collections::HashMap;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::user_context::UserContext;

pub struct RecipeImplementationImpl {
    pub querier: Querier,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn add_role_to_user(
        &self,
        user_id: &str,
        role: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<AddRoleToUserResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "role": role,
        });
        let path = NormalisedURLPath::new(&format!("/{}/recipe/user/role", tenant_id))?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => {
                let already = response
                    .get("didUserAlreadyHaveRole")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(AddRoleToUserResult::Ok {
                    did_user_already_have_role: already,
                })
            }
            "UNKNOWN_ROLE_ERROR" => Ok(AddRoleToUserResult::UnknownRole),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn remove_user_role(
        &self,
        user_id: &str,
        role: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RemoveUserRoleResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "role": role,
        });
        let path = NormalisedURLPath::new(&format!("/{}/recipe/user/role/remove", tenant_id))?;
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
                let had = response
                    .get("didUserHaveRole")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(RemoveUserRoleResult::Ok {
                    did_user_have_role: had,
                })
            }
            "UNKNOWN_ROLE_ERROR" => Ok(RemoveUserRoleResult::UnknownRole),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn get_roles_for_user(
        &self,
        user_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<GetRolesForUserOkResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("userId".to_string(), user_id.to_string());

        let path = NormalisedURLPath::new(&format!("/{}/recipe/user/roles", tenant_id))?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let roles = response
            .get("roles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(GetRolesForUserOkResult { roles })
    }

    async fn get_users_that_have_role(
        &self,
        role: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<GetUsersThatHaveRoleResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("role".to_string(), role.to_string());

        let path = NormalisedURLPath::new(&format!("/{}/recipe/role/users", tenant_id))?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => {
                let users = response
                    .get("users")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|s| s.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(GetUsersThatHaveRoleResult::Ok { users })
            }
            "UNKNOWN_ROLE_ERROR" => Ok(GetUsersThatHaveRoleResult::UnknownRole),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn create_new_role_or_add_permissions(
        &self,
        role: &str,
        permissions: &[String],
        user_context: &mut UserContext,
    ) -> Result<CreateNewRoleOrAddPermissionsOkResult, SuperTokensError> {
        let body = serde_json::json!({
            "role": role,
            "permissions": permissions,
        });
        let path = NormalisedURLPath::new("/recipe/role")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let created = response
            .get("createdNewRole")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(CreateNewRoleOrAddPermissionsOkResult {
            created_new_role: created,
        })
    }

    async fn get_permissions_for_role(
        &self,
        role: &str,
        user_context: &mut UserContext,
    ) -> Result<GetPermissionsForRoleResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("role".to_string(), role.to_string());

        let path = NormalisedURLPath::new("/recipe/role/permissions")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => {
                let permissions = response
                    .get("permissions")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|s| s.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(GetPermissionsForRoleResult::Ok { permissions })
            }
            "UNKNOWN_ROLE_ERROR" => Ok(GetPermissionsForRoleResult::UnknownRole),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn remove_permissions_from_role(
        &self,
        role: &str,
        permissions: &[String],
        user_context: &mut UserContext,
    ) -> Result<RemovePermissionsFromRoleResult, SuperTokensError> {
        let body = serde_json::json!({
            "role": role,
            "permissions": permissions,
        });
        let path = NormalisedURLPath::new("/recipe/role/permissions/remove")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(RemovePermissionsFromRoleResult::Ok),
            "UNKNOWN_ROLE_ERROR" => Ok(RemovePermissionsFromRoleResult::UnknownRole),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn get_roles_that_have_permission(
        &self,
        permission: &str,
        user_context: &mut UserContext,
    ) -> Result<GetRolesThatHavePermissionOkResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("permission".to_string(), permission.to_string());

        let path = NormalisedURLPath::new("/recipe/permission/roles")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let roles = response
            .get("roles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(GetRolesThatHavePermissionOkResult { roles })
    }

    async fn delete_role(
        &self,
        role: &str,
        user_context: &mut UserContext,
    ) -> Result<DeleteRoleOkResult, SuperTokensError> {
        let body = serde_json::json!({ "role": role });
        let path = NormalisedURLPath::new("/recipe/role/remove")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let existed = response
            .get("didRoleExist")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(DeleteRoleOkResult {
            did_role_exist: existed,
        })
    }

    async fn get_all_roles(
        &self,
        user_context: &mut UserContext,
    ) -> Result<GetAllRolesOkResult, SuperTokensError> {
        let path = NormalisedURLPath::new("/recipe/roles")?;
        let response = self
            .querier
            .send_get_request(&path, None, user_context)
            .await?;

        let roles = response
            .get("roles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(GetAllRolesOkResult { roles })
    }
}
