use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Create a new TOTP device for a user.
    async fn create_device(
        &self,
        user_id: &str,
        user_identifier_info: Option<&str>,
        device_name: Option<&str>,
        skew: Option<u32>,
        period: Option<u32>,
        user_context: &mut UserContext,
    ) -> Result<CreateDeviceResult, SuperTokensError>;

    /// Update a TOTP device name.
    async fn update_device(
        &self,
        user_id: &str,
        existing_device_name: &str,
        new_device_name: &str,
        user_context: &mut UserContext,
    ) -> Result<UpdateDeviceResult, SuperTokensError>;

    /// List all TOTP devices for a user.
    async fn list_devices(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ListDevicesOkResult, SuperTokensError>;

    /// Remove a TOTP device.
    async fn remove_device(
        &self,
        user_id: &str,
        device_name: &str,
        user_context: &mut UserContext,
    ) -> Result<RemoveDeviceOkResult, SuperTokensError>;

    /// Verify a TOTP device with a code.
    async fn verify_device(
        &self,
        tenant_id: &str,
        user_id: &str,
        device_name: &str,
        totp: &str,
        user_context: &mut UserContext,
    ) -> Result<VerifyDeviceResult, SuperTokensError>;

    /// Verify a TOTP code for a user.
    async fn verify_totp(
        &self,
        tenant_id: &str,
        user_id: &str,
        totp: &str,
        user_context: &mut UserContext,
    ) -> Result<VerifyTotpResult, SuperTokensError>;
}
