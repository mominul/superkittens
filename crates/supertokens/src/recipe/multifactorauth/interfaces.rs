use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Get the factor IDs that have been set up for a user.
    async fn get_factors_setup_for_user(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<GetFactorsSetupForUserOkResult, SuperTokensError>;

    /// Get the required secondary factor IDs for a user.
    async fn get_required_secondary_factors_for_user(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<GetRequiredSecondaryFactorsOkResult, SuperTokensError>;

    /// Add a factor ID to the required secondary factors for a user.
    async fn add_to_required_secondary_factors_for_user(
        &self,
        user_id: &str,
        factor_id: &str,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError>;

    /// Remove a factor ID from the required secondary factors for a user.
    async fn remove_from_required_secondary_factors_for_user(
        &self,
        user_id: &str,
        factor_id: &str,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError>;

    /// Mark a factor as complete in a session.
    async fn mark_factor_as_complete_in_session(
        &self,
        session_handle: &str,
        factor_id: &str,
        user_context: &mut UserContext,
    ) -> Result<MarkFactorAsCompleteOkResult, SuperTokensError>;
}
