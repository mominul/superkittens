use async_trait::async_trait;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::querier::Querier;
use crate::user_context::UserContext;

/// Default implementation of the MultiFactorAuth RecipeInterface.
///
/// Most MFA methods operate on session claims and user metadata.
/// The core logic delegates to session and metadata recipes; these
/// stubs return simple Ok results.
pub struct RecipeImplementationImpl {
    pub querier: Querier,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn get_factors_setup_for_user(
        &self,
        _user_id: &str,
        _user_context: &mut UserContext,
    ) -> Result<GetFactorsSetupForUserOkResult, SuperTokensError> {
        // In a full implementation this would aggregate factor setup info
        // from various recipe implementations (TOTP, WebAuthn, etc.)
        Ok(GetFactorsSetupForUserOkResult {
            factor_ids: Vec::new(),
        })
    }

    async fn get_required_secondary_factors_for_user(
        &self,
        _user_id: &str,
        _user_context: &mut UserContext,
    ) -> Result<GetRequiredSecondaryFactorsOkResult, SuperTokensError> {
        // In a full implementation this reads from user metadata
        Ok(GetRequiredSecondaryFactorsOkResult {
            factor_ids: Vec::new(),
        })
    }

    async fn add_to_required_secondary_factors_for_user(
        &self,
        _user_id: &str,
        _factor_id: &str,
        _user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        // In a full implementation this updates user metadata
        Ok(())
    }

    async fn remove_from_required_secondary_factors_for_user(
        &self,
        _user_id: &str,
        _factor_id: &str,
        _user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        // In a full implementation this updates user metadata
        Ok(())
    }

    async fn mark_factor_as_complete_in_session(
        &self,
        _session_handle: &str,
        _factor_id: &str,
        _user_context: &mut UserContext,
    ) -> Result<MarkFactorAsCompleteOkResult, SuperTokensError> {
        // In a full implementation this updates session claims
        Ok(MarkFactorAsCompleteOkResult)
    }
}
