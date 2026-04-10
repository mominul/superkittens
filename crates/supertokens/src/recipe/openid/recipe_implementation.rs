use async_trait::async_trait;
use std::sync::Arc;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::recipe::jwt::interfaces::RecipeInterface as JwtRecipeInterface;
use crate::recipe::jwt::types::{CreateJwtResult, GetJWKSResult};
use crate::types::config::AppInfo;
use crate::user_context::UserContext;

pub struct RecipeImplementationImpl {
    pub jwt_recipe_implementation: Arc<dyn JwtRecipeInterface>,
    pub app_info: AppInfo,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn create_jwt(
        &self,
        payload: &serde_json::Map<String, serde_json::Value>,
        validity_seconds: Option<u64>,
        use_static_signing_key: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<CreateJwtResult, SuperTokensError> {
        self.jwt_recipe_implementation
            .create_jwt(
                payload,
                validity_seconds,
                use_static_signing_key,
                user_context,
            )
            .await
    }

    async fn get_jwks(
        &self,
        user_context: &mut UserContext,
    ) -> Result<GetJWKSResult, SuperTokensError> {
        self.jwt_recipe_implementation.get_jwks(user_context).await
    }

    async fn get_open_id_discovery_configuration(
        &self,
        _user_context: &mut UserContext,
    ) -> Result<GetOpenIdDiscoveryConfigurationResult, SuperTokensError> {
        let api_domain = &self.app_info.api_domain;
        let api_base_path = &self.app_info.api_base_path;
        let base_url = format!("{}{}", api_domain, api_base_path);

        Ok(GetOpenIdDiscoveryConfigurationResult {
            issuer: base_url.clone(),
            jwks_uri: format!("{}/jwt/jwks.json", base_url),
            authorization_endpoint: format!("{}/oauth/auth", base_url),
            token_endpoint: format!("{}/oauth/token", base_url),
            userinfo_endpoint: format!("{}/oauth/userinfo", base_url),
            revocation_endpoint: format!("{}/oauth/revoke", base_url),
            token_introspection_endpoint: format!("{}/oauth/introspect", base_url),
            end_session_endpoint: format!("{}/oauth/end-session", base_url),
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec!["RS256".to_string()],
            response_types_supported: vec![
                "code".to_string(),
                "id_token".to_string(),
                "id_token token".to_string(),
            ],
        })
    }
}
