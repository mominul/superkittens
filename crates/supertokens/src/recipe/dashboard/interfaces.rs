use async_trait::async_trait;

use crate::error::SuperTokensError;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Get the URL for the dashboard bundle (JS/CSS assets).
    async fn get_dashboard_bundle_location(
        &self,
        user_context: &mut UserContext,
    ) -> Result<String, SuperTokensError>;

    /// Check whether the current request should be allowed to access the dashboard.
    async fn should_allow_access(
        &self,
        api_key: Option<&str>,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError>;
}
