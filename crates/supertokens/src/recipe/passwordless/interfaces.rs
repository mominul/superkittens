use async_trait::async_trait;
use std::sync::Arc;

use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

// ---------------------------------------------------------------------------
// RecipeInterface — core business logic for passwordless
// ---------------------------------------------------------------------------

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Create a new passwordless code for a user identified by email or phone number.
    async fn create_code(
        &self,
        email: Option<&str>,
        phone_number: Option<&str>,
        user_input_code: Option<&str>,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateCodeOkResult, SuperTokensError>;

    /// Create a new code for an existing device (resend flow).
    async fn create_new_code_for_device(
        &self,
        device_id: &str,
        user_input_code: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateNewCodeForDeviceResult, SuperTokensError>;

    /// Consume a code to complete sign in/up.
    async fn consume_code(
        &self,
        pre_auth_session_id: &str,
        user_input_code: Option<&str>,
        device_id: Option<&str>,
        link_code: Option<&str>,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ConsumeCodeResult, SuperTokensError>;

    /// Check a code without consuming it.
    async fn check_code(
        &self,
        pre_auth_session_id: &str,
        user_input_code: Option<&str>,
        device_id: Option<&str>,
        link_code: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CheckCodeResult, SuperTokensError>;

    /// Update a user's email or phone number.
    async fn update_user(
        &self,
        recipe_user_id: &str,
        email: Option<&str>,
        phone_number: Option<&str>,
        user_context: &mut UserContext,
    ) -> Result<UpdateUserResult, SuperTokensError>;

    /// Revoke all codes for a user identified by email or phone number.
    async fn revoke_all_codes(
        &self,
        email: Option<&str>,
        phone_number: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RevokeAllCodesOkResult, SuperTokensError>;

    /// Revoke a specific code by its ID.
    async fn revoke_code(
        &self,
        code_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RevokeCodeOkResult, SuperTokensError>;

    /// List all codes/devices for a given email.
    async fn list_codes_by_email(
        &self,
        email: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Vec<DeviceType>, SuperTokensError>;

    /// List all codes/devices for a given phone number.
    async fn list_codes_by_phone_number(
        &self,
        phone_number: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Vec<DeviceType>, SuperTokensError>;

    /// List codes/devices for a given device ID.
    async fn list_codes_by_device_id(
        &self,
        device_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<DeviceType>, SuperTokensError>;

    /// List codes/devices for a given pre-auth session ID.
    async fn list_codes_by_pre_auth_session_id(
        &self,
        pre_auth_session_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<DeviceType>, SuperTokensError>;
}
