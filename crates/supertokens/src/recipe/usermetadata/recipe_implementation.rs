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
    async fn get_user_metadata(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<MetadataResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("userId".to_string(), user_id.to_string());

        let path = NormalisedURLPath::new("/recipe/user/metadata")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let metadata = response
            .get("metadata")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        Ok(MetadataResult { metadata })
    }

    async fn update_user_metadata(
        &self,
        user_id: &str,
        metadata_update: &serde_json::Map<String, serde_json::Value>,
        user_context: &mut UserContext,
    ) -> Result<MetadataResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "metadataUpdate": metadata_update,
        });

        let path = NormalisedURLPath::new("/recipe/user/metadata")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let metadata = response
            .get("metadata")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        Ok(MetadataResult { metadata })
    }

    async fn clear_user_metadata(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ClearUserMetadataResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
        });

        let path = NormalisedURLPath::new("/recipe/user/metadata/remove")?;
        self.querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        Ok(ClearUserMetadataResult)
    }
}
