use async_trait::async_trait;

use super::types::*;
use crate::error::SuperTokensError;
use crate::types::user::RecipeUserId;
use crate::user_context::UserContext;

#[async_trait]
pub trait RecipeInterface: Send + Sync {
    /// Create an email verification token.
    async fn create_email_verification_token(
        &self,
        recipe_user_id: &RecipeUserId,
        email: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateEmailVerificationTokenResult, SuperTokensError>;

    /// Verify an email using a token.
    async fn verify_email_using_token(
        &self,
        token: &str,
        tenant_id: &str,
        attempt_account_linking: bool,
        user_context: &mut UserContext,
    ) -> Result<VerifyEmailUsingTokenResult, SuperTokensError>;

    /// Check if an email is verified.
    async fn is_email_verified(
        &self,
        recipe_user_id: &RecipeUserId,
        email: &str,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError>;

    /// Revoke all verification tokens for a user.
    async fn revoke_email_verification_tokens(
        &self,
        recipe_user_id: &RecipeUserId,
        email: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RevokeEmailVerificationTokensOkResult, SuperTokensError>;

    /// Mark an email as unverified.
    async fn unverify_email(
        &self,
        recipe_user_id: &RecipeUserId,
        email: &str,
        user_context: &mut UserContext,
    ) -> Result<UnverifyEmailOkResult, SuperTokensError>;
}
