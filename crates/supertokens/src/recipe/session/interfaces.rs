use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use super::types::*;
use crate::error::SuperTokensError;
use crate::framework::request::BaseRequest;
use crate::framework::response::BaseResponse;
use crate::types::user::RecipeUserId;
use crate::user_context::UserContext;

/// The core session recipe interface.
///
/// Default implementation talks to SuperTokens Core. Users can wrap the
/// default with overrides to add custom logic.
#[async_trait]
pub trait RecipeInterface: Send + Sync {
    async fn create_new_session(
        &self,
        user_id: &str,
        recipe_user_id: &RecipeUserId,
        access_token_payload: Option<Value>,
        session_data_in_database: Option<Value>,
        disable_anti_csrf: Option<bool>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Arc<dyn SessionContainerInterface>, SuperTokensError>;

    async fn get_session(
        &self,
        access_token: Option<&str>,
        anti_csrf_token: Option<&str>,
        anti_csrf_check: Option<bool>,
        session_required: Option<bool>,
        check_database: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<Option<Arc<dyn SessionContainerInterface>>, SuperTokensError>;

    async fn refresh_session(
        &self,
        refresh_token: &str,
        anti_csrf_token: Option<&str>,
        disable_anti_csrf: bool,
        user_context: &mut UserContext,
    ) -> Result<Arc<dyn SessionContainerInterface>, SuperTokensError>;

    async fn revoke_session(
        &self,
        session_handle: &str,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError>;

    async fn revoke_all_sessions_for_user(
        &self,
        user_id: &str,
        revoke_sessions_for_linked_accounts: bool,
        tenant_id: &str,
        revoke_across_all_tenants: bool,
        user_context: &mut UserContext,
    ) -> Result<Vec<String>, SuperTokensError>;

    async fn get_all_session_handles_for_user(
        &self,
        user_id: &str,
        fetch_sessions_for_linked_accounts: bool,
        tenant_id: &str,
        fetch_across_all_tenants: bool,
        user_context: &mut UserContext,
    ) -> Result<Vec<String>, SuperTokensError>;

    async fn revoke_multiple_sessions(
        &self,
        session_handles: &[String],
        user_context: &mut UserContext,
    ) -> Result<Vec<String>, SuperTokensError>;

    async fn get_session_information(
        &self,
        session_handle: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<SessionInformationResult>, SuperTokensError>;

    async fn update_session_data_in_database(
        &self,
        session_handle: &str,
        new_session_data: Value,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError>;

    async fn merge_into_access_token_payload(
        &self,
        session_handle: &str,
        access_token_payload_update: Value,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError>;

    async fn regenerate_access_token(
        &self,
        access_token: &str,
        new_access_token_payload: Option<Value>,
        user_context: &mut UserContext,
    ) -> Result<Option<RegenerateAccessTokenOkResult>, SuperTokensError>;

    fn get_global_claim_validators(
        &self,
        tenant_id: &str,
        user_id: &str,
        recipe_user_id: &RecipeUserId,
        claim_validators_added_by_other_recipes: Vec<Box<dyn SessionClaimValidator>>,
        user_context: &UserContext,
    ) -> Vec<Box<dyn SessionClaimValidator>>;

    async fn validate_claims(
        &self,
        user_id: &str,
        recipe_user_id: &RecipeUserId,
        access_token_payload: &Value,
        claim_validators: &[Box<dyn SessionClaimValidator>],
        user_context: &mut UserContext,
    ) -> Result<ClaimsValidationResult, SuperTokensError>;

    async fn fetch_and_set_claim(
        &self,
        session_handle: &str,
        claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError>;

    async fn set_claim_value(
        &self,
        session_handle: &str,
        claim: &dyn SessionClaim,
        value: Value,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError>;

    async fn get_claim_value(
        &self,
        session_handle: &str,
        claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<Option<Value>, SuperTokensError>;

    async fn remove_claim(
        &self,
        session_handle: &str,
        claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError>;
}

/// API-level interface for session endpoints.
#[async_trait]
pub trait ApiInterface: Send + Sync {
    async fn refresh_post(
        &self,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<Arc<dyn SessionContainerInterface>, SuperTokensError>;

    async fn signout_post(
        &self,
        session: &dyn SessionContainerInterface,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<Value, SuperTokensError>;

    #[allow(clippy::too_many_arguments)]
    async fn verify_session(
        &self,
        api_options: &ApiOptions<'_>,
        anti_csrf_check: Option<bool>,
        session_required: bool,
        check_database: bool,
        user_context: &mut UserContext,
    ) -> Result<Option<Arc<dyn SessionContainerInterface>>, SuperTokensError>;

    fn disable_refresh_post(&self) -> bool {
        false
    }

    fn disable_signout_post(&self) -> bool {
        false
    }
}

/// Options passed to API handlers.
pub struct ApiOptions<'a> {
    pub request: &'a dyn BaseRequest,
    pub recipe_id: String,
    pub config: NormalisedSessionConfig,
    pub recipe_implementation: Arc<dyn RecipeInterface>,
}

/// The session container interface — represents an active session.
///
/// Provides methods to read/write session data, access tokens,
/// claims, and to revoke the session.
#[async_trait]
pub trait SessionContainerInterface: Send + Sync {
    fn get_handle(&self) -> &str;
    fn get_user_id(&self) -> &str;
    fn get_recipe_user_id(&self) -> &RecipeUserId;
    fn get_tenant_id(&self) -> &str;
    fn get_access_token(&self) -> &str;
    fn get_access_token_payload(&self) -> &Value;
    fn get_all_session_tokens_dangerously(&self) -> GetSessionTokensDangerously;

    async fn revoke_session(&self, user_context: &mut UserContext) -> Result<(), SuperTokensError>;

    async fn get_session_data_from_database(
        &self,
        user_context: &mut UserContext,
    ) -> Result<Value, SuperTokensError>;

    async fn update_session_data_in_database(
        &self,
        new_session_data: Value,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError>;

    async fn merge_into_access_token_payload(
        &self,
        access_token_payload_update: Value,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError>;

    async fn get_time_created(
        &self,
        user_context: &mut UserContext,
    ) -> Result<u64, SuperTokensError>;

    async fn get_expiry(&self, user_context: &mut UserContext) -> Result<u64, SuperTokensError>;

    async fn fetch_and_set_claim(
        &self,
        claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError>;

    async fn set_claim_value(
        &self,
        claim: &dyn SessionClaim,
        value: Value,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError>;

    async fn get_claim_value(
        &self,
        claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<Option<Value>, SuperTokensError>;

    async fn remove_claim(
        &self,
        claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError>;

    async fn assert_claims(
        &self,
        claim_validators: &[Box<dyn SessionClaimValidator>],
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError>;

    async fn attach_to_request_response(
        &self,
        request: &dyn BaseRequest,
        response: &mut dyn BaseResponse,
        transfer_method: TokenTransferMethod,
        user_context: &UserContext,
    ) -> Result<(), SuperTokensError>;

    /// Whether the access token has been updated since session creation/load.
    fn access_token_updated(&self) -> bool;

    /// Get response mutators to apply.
    fn get_response_mutators(
        &self,
    ) -> &[Box<dyn Fn(&mut dyn BaseResponse, &UserContext) + Send + Sync>];
}

/// A session claim — holds typed data in the access token payload.
pub trait SessionClaim: Send + Sync {
    /// The claim key in the access token payload.
    fn get_key(&self) -> &str;

    /// Add this claim's value to the payload.
    fn add_to_payload(&self, payload: &mut Value, value: Value);

    /// Remove this claim from the payload via merge (set to null).
    fn remove_from_payload_by_merge(&self, payload: &mut Value);

    /// Remove this claim from the payload entirely.
    fn remove_from_payload(&self, payload: &mut Value);

    /// Get the claim's value from the payload.
    fn get_value_from_payload(&self, payload: &Value) -> Option<Value>;

    /// Get the last refetch timestamp for this claim.
    fn get_last_refetch_time(&self, payload: &Value) -> Option<u64>;
}

/// Validates a session claim against the access token payload.
#[async_trait]
pub trait SessionClaimValidator: Send + Sync {
    fn get_id(&self) -> &str;

    fn get_claim(&self) -> Option<&dyn SessionClaim>;

    /// Whether the claim should be refetched before validation.
    fn should_refetch(&self, payload: &Value, user_context: &UserContext) -> bool;

    /// Validate the claim against the payload.
    async fn validate(
        &self,
        payload: &Value,
        user_context: &UserContext,
    ) -> SingleClaimValidationResult;
}
