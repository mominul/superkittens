use crate::error::SuperTokensError;
use crate::user_context::UserContext;
use async_trait::async_trait;

/// Trait for email delivery services.
///
/// Each recipe that sends emails (emailpassword, emailverification, passwordless)
/// provides its own input type `I` for the email content.
#[async_trait]
pub trait EmailDeliveryInterface<I: Send + Sync>: Send + Sync {
    async fn send_email(
        &self,
        input: I,
        user_context: &UserContext,
    ) -> Result<(), SuperTokensError>;
}
