use async_trait::async_trait;
use std::sync::Arc;

use super::types::*;
use crate::error::SuperTokensError;
use crate::framework::request::BaseRequest;
use crate::querier::Querier;
use crate::types::config::AppInfo;
use crate::user_context::UserContext;

// ---------------------------------------------------------------------------
// RecipeInterface — core business logic
// ---------------------------------------------------------------------------

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Sign up a new user. Handles account linking if a session is provided.
    async fn sign_up(
        &self,
        email: &str,
        password: &str,
        tenant_id: &str,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<SignUpResult, SuperTokensError>;

    /// Create a new recipe user (without account linking logic).
    async fn create_new_recipe_user(
        &self,
        email: &str,
        password: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<SignUpResult, SuperTokensError>;

    /// Sign in an existing user. Handles account linking if a session is provided.
    async fn sign_in(
        &self,
        email: &str,
        password: &str,
        tenant_id: &str,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<SignInResult, SuperTokensError>;

    /// Verify credentials without creating a session.
    async fn verify_credentials(
        &self,
        email: &str,
        password: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<SignInResult, SuperTokensError>;

    /// Create a password reset token for a user.
    async fn create_reset_password_token(
        &self,
        user_id: &str,
        email: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateResetPasswordTokenResult, SuperTokensError>;

    /// Consume a password reset token and return the associated email/user_id.
    async fn consume_password_reset_token(
        &self,
        token: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ConsumePasswordResetTokenResult, SuperTokensError>;

    /// Update a user's email or password.
    async fn update_email_or_password(
        &self,
        recipe_user_id: &str,
        email: Option<&str>,
        password: Option<&str>,
        apply_password_policy: Option<bool>,
        tenant_id_for_password_policy: Option<&str>,
        user_context: &mut UserContext,
    ) -> Result<UpdateEmailOrPasswordResult, SuperTokensError>;
}

// ---------------------------------------------------------------------------
// ApiInterface — HTTP API endpoint handlers
// ---------------------------------------------------------------------------

#[async_trait]
pub trait ApiInterface: Send + Sync {
    /// GET /emailpassword/email/exists
    async fn email_exists_get(
        &self,
        email: &str,
        tenant_id: &str,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<EmailExistsGetResult, SuperTokensError>;

    /// POST /user/password/reset/token
    async fn generate_password_reset_token_post(
        &self,
        form_fields: &[FormField],
        tenant_id: &str,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<GeneratePasswordResetTokenPostResult, SuperTokensError>;

    /// POST /user/password/reset
    async fn password_reset_post(
        &self,
        form_fields: &[FormField],
        token: &str,
        tenant_id: &str,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<PasswordResetPostResult, SuperTokensError>;

    /// POST /signin
    async fn sign_in_post(
        &self,
        form_fields: &[FormField],
        tenant_id: &str,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<SignInPostResult, SuperTokensError>;

    /// POST /signup
    async fn sign_up_post(
        &self,
        form_fields: &[FormField],
        tenant_id: &str,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<SignUpPostResult, SuperTokensError>;
}

// ---------------------------------------------------------------------------
// ApiOptions — context passed to API handlers
// ---------------------------------------------------------------------------

pub struct ApiOptions<'a> {
    pub request: &'a dyn BaseRequest,
    pub recipe_id: String,
    pub config: &'a NormalisedEmailPasswordConfig,
    pub recipe_implementation: Arc<dyn RecipeInterface>,
    pub app_info: &'a AppInfo,
    pub querier: Querier,
    pub email_delivery:
        Arc<dyn crate::ingredients::email_delivery::EmailDeliveryInterface<EmailTemplateVars>>,
}
