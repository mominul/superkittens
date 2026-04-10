use crate::error::SuperTokensError;
use crate::user_context::UserContext;
use async_trait::async_trait;

/// Trait for SMS delivery services.
///
/// Used by passwordless and other recipes that need to send SMS messages.
#[async_trait]
pub trait SmsDeliveryInterface<I: Send + Sync>: Send + Sync {
    async fn send_sms(&self, input: I, user_context: &UserContext) -> Result<(), SuperTokensError>;
}
