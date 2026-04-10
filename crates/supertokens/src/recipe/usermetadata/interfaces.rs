use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Get metadata for a user.
    async fn get_user_metadata(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<MetadataResult, SuperTokensError>;

    /// Update metadata for a user (merge).
    async fn update_user_metadata(
        &self,
        user_id: &str,
        metadata_update: &serde_json::Map<String, serde_json::Value>,
        user_context: &mut UserContext,
    ) -> Result<MetadataResult, SuperTokensError>;

    /// Clear all metadata for a user.
    async fn clear_user_metadata(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ClearUserMetadataResult, SuperTokensError>;
}
