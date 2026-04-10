use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::recipe::jwt::types::{CreateJwtResult, GetJWKSResult};
use crate::user_context::UserContext;

/// OpenID recipe interface — delegates JWT operations to the JWT recipe.
#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Create a JWT (delegated to JWT recipe).
    async fn create_jwt(
        &self,
        payload: &serde_json::Map<String, serde_json::Value>,
        validity_seconds: Option<u64>,
        use_static_signing_key: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<CreateJwtResult, SuperTokensError>;

    /// Get JWKS (delegated to JWT recipe).
    async fn get_jwks(
        &self,
        user_context: &mut UserContext,
    ) -> Result<GetJWKSResult, SuperTokensError>;

    /// Get OpenID discovery configuration.
    async fn get_open_id_discovery_configuration(
        &self,
        user_context: &mut UserContext,
    ) -> Result<GetOpenIdDiscoveryConfigurationResult, SuperTokensError>;
}
