use async_trait::async_trait;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::types::user::RecipeUserId;
use crate::user_context::UserContext;

pub struct RecipeImplementationImpl {
    pub querier: Querier,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn get_tenant_id(
        &self,
        tenant_id_from_frontend: &str,
        _user_context: &mut UserContext,
    ) -> Result<String, SuperTokensError> {
        Ok(tenant_id_from_frontend.to_string())
    }

    async fn create_or_update_tenant(
        &self,
        tenant_id: &str,
        config: Option<&TenantConfigCreateOrUpdate>,
        user_context: &mut UserContext,
    ) -> Result<CreateOrUpdateTenantOkResult, SuperTokensError> {
        let mut body = serde_json::json!({ "tenantId": tenant_id });

        if let Some(cfg) = config {
            if let Some(ref core_config) = cfg.core_config {
                body["coreConfig"] = serde_json::Value::Object(core_config.clone());
            }
            if let Some(ref ff) = cfg.first_factors {
                match ff {
                    Some(factors) => body["firstFactors"] = serde_json::json!(factors),
                    None => body["firstFactors"] = serde_json::Value::Null,
                }
            }
            if let Some(ref rsf) = cfg.required_secondary_factors {
                match rsf {
                    Some(factors) => body["requiredSecondaryFactors"] = serde_json::json!(factors),
                    None => body["requiredSecondaryFactors"] = serde_json::Value::Null,
                }
            }
        }

        let path = NormalisedURLPath::new("/recipe/multitenancy/tenant/v2")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let created_new = response
            .get("createdNew")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(CreateOrUpdateTenantOkResult { created_new })
    }

    async fn delete_tenant(
        &self,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<DeleteTenantOkResult, SuperTokensError> {
        let body = serde_json::json!({ "tenantId": tenant_id });
        let path = NormalisedURLPath::new("/recipe/multitenancy/tenant/remove")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let did_exist = response
            .get("didExist")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(DeleteTenantOkResult { did_exist })
    }

    async fn get_tenant(
        &self,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<TenantConfig>, SuperTokensError> {
        let path =
            NormalisedURLPath::new(&format!("/{}/recipe/multitenancy/tenant/v2", tenant_id))?;
        let response = self
            .querier
            .send_get_request(&path, None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        if status == "TENANT_NOT_FOUND_ERROR" {
            return Ok(None);
        }

        Ok(TenantConfig::from_json(&response))
    }

    async fn list_all_tenants(
        &self,
        user_context: &mut UserContext,
    ) -> Result<ListAllTenantsOkResult, SuperTokensError> {
        let path = NormalisedURLPath::new("/recipe/multitenancy/tenant/list/v2")?;
        let response = self
            .querier
            .send_get_request(&path, None, user_context)
            .await?;

        let tenants = response
            .get("tenants")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(TenantConfig::from_json).collect())
            .unwrap_or_default();

        Ok(ListAllTenantsOkResult { tenants })
    }

    async fn create_or_update_third_party_config(
        &self,
        tenant_id: &str,
        config: &ProviderConfig,
        skip_validation: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<CreateOrUpdateThirdPartyConfigOkResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "config": serde_json::to_value(config)?,
        });
        if let Some(skip) = skip_validation {
            body["skipValidation"] = serde_json::Value::Bool(skip);
        }

        let path = NormalisedURLPath::new(&format!(
            "/{}/recipe/multitenancy/config/thirdparty",
            tenant_id
        ))?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let created_new = response
            .get("createdNew")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(CreateOrUpdateThirdPartyConfigOkResult { created_new })
    }

    async fn delete_third_party_config(
        &self,
        tenant_id: &str,
        third_party_id: &str,
        user_context: &mut UserContext,
    ) -> Result<DeleteThirdPartyConfigOkResult, SuperTokensError> {
        let body = serde_json::json!({ "thirdPartyId": third_party_id });
        let path = NormalisedURLPath::new(&format!(
            "/{}/recipe/multitenancy/config/thirdparty/remove",
            tenant_id
        ))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let did_config_exist = response
            .get("didConfigExist")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(DeleteThirdPartyConfigOkResult { did_config_exist })
    }

    async fn associate_user_to_tenant(
        &self,
        tenant_id: &str,
        recipe_user_id: &RecipeUserId,
        user_context: &mut UserContext,
    ) -> Result<AssociateUserToTenantResult, SuperTokensError> {
        let body = serde_json::json!({
            "recipeUserId": recipe_user_id.get_as_string(),
        });
        let path =
            NormalisedURLPath::new(&format!("/{}/recipe/multitenancy/tenant/user", tenant_id))?;
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
                let was_already_associated = response
                    .get("wasAlreadyAssociated")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(AssociateUserToTenantResult::Ok {
                    was_already_associated,
                })
            }
            "UNKNOWN_USER_ID_ERROR" => Ok(AssociateUserToTenantResult::UnknownUserId),
            "EMAIL_ALREADY_EXISTS_ERROR" => Ok(AssociateUserToTenantResult::EmailAlreadyExists),
            "PHONE_NUMBER_ALREADY_EXISTS_ERROR" => {
                Ok(AssociateUserToTenantResult::PhoneNumberAlreadyExists)
            }
            "THIRD_PARTY_USER_ALREADY_EXISTS_ERROR" => {
                Ok(AssociateUserToTenantResult::ThirdPartyUserAlreadyExists)
            }
            "ASSOCIATION_NOT_ALLOWED_ERROR" => {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(AssociateUserToTenantResult::NotAllowed { reason })
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from associate_user_to_tenant: {}",
                status
            ))),
        }
    }

    async fn disassociate_user_from_tenant(
        &self,
        tenant_id: &str,
        recipe_user_id: &RecipeUserId,
        user_context: &mut UserContext,
    ) -> Result<DisassociateUserFromTenantOkResult, SuperTokensError> {
        let body = serde_json::json!({
            "recipeUserId": recipe_user_id.get_as_string(),
        });
        let path = NormalisedURLPath::new(&format!(
            "/{}/recipe/multitenancy/tenant/user/remove",
            tenant_id
        ))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let was_associated = response
            .get("wasAssociated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(DisassociateUserFromTenantOkResult { was_associated })
    }
}
