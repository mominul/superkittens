use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

// ---------------------------------------------------------------------------
// RecipeInterface — core business logic
// ---------------------------------------------------------------------------

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Manually create or update a third-party user. Handles account linking
    /// if a session is provided.
    async fn manually_create_or_update_user(
        &self,
        third_party_id: &str,
        third_party_user_id: &str,
        email: &str,
        is_verified: bool,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ManuallyCreateOrUpdateUserResult, SuperTokensError>;

    /// Sign in or sign up a third-party user. Delegates to
    /// `manually_create_or_update_user` with additional OAuth token
    /// and raw provider info context.
    async fn sign_in_up(
        &self,
        third_party_id: &str,
        third_party_user_id: &str,
        email: &str,
        is_verified: bool,
        oauth_tokens: HashMap<String, Value>,
        raw_user_info_from_provider: RawUserInfoFromProvider,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<SignInUpResult, SuperTokensError>;

    /// Get a provider configuration by third-party ID and optional client type.
    /// Returns `Ok(None)` when no matching provider is found.
    async fn get_provider(
        &self,
        third_party_id: &str,
        client_type: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<()>, SuperTokensError>;
}
