use async_trait::async_trait;

use super::interfaces::RecipeInterface;
use crate::error::SuperTokensError;
use crate::querier::Querier;
use crate::user_context::UserContext;

/// Dashboard CDN bundle URL.
const DASHBOARD_BUNDLE_CDN: &str =
    "https://cdn.jsdelivr.net/gh/nicholasgasior/supertokens-dashboard@latest/build/static";

pub struct RecipeImplementationImpl {
    pub querier: Querier,
    pub api_key: Option<String>,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn get_dashboard_bundle_location(
        &self,
        _user_context: &mut UserContext,
    ) -> Result<String, SuperTokensError> {
        Ok(DASHBOARD_BUNDLE_CDN.to_string())
    }

    async fn should_allow_access(
        &self,
        api_key: Option<&str>,
        _user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError> {
        // If an API key is configured, the request must provide a matching key.
        match &self.api_key {
            Some(configured_key) => match api_key {
                Some(provided_key) => Ok(provided_key == configured_key),
                None => Ok(false),
            },
            // No API key configured means open access (development mode).
            None => Ok(true),
        }
    }
}
