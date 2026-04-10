use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Create a JWT.
    async fn create_jwt(
        &self,
        payload: &serde_json::Map<String, serde_json::Value>,
        validity_seconds: Option<u64>,
        use_static_signing_key: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<CreateJwtResult, SuperTokensError>;

    /// Get the JWKS (JSON Web Key Set).
    async fn get_jwks(
        &self,
        user_context: &mut UserContext,
    ) -> Result<GetJWKSResult, SuperTokensError>;
}
