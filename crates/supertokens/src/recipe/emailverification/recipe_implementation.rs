use async_trait::async_trait;
use std::collections::HashMap;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::types::user::RecipeUserId;
use crate::user_context::UserContext;

pub struct RecipeImplementationImpl {
    pub querier: Querier,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn create_email_verification_token(
        &self,
        recipe_user_id: &RecipeUserId,
        email: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateEmailVerificationTokenResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": recipe_user_id.get_as_string(),
            "email": email,
        });

        let path =
            NormalisedURLPath::new(&format!("/{}/recipe/user/email/verify/token", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => {
                let token = response
                    .get("token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(CreateEmailVerificationTokenResult::Ok { token })
            }
            "EMAIL_ALREADY_VERIFIED_ERROR" => {
                Ok(CreateEmailVerificationTokenResult::EmailAlreadyVerified)
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn verify_email_using_token(
        &self,
        token: &str,
        tenant_id: &str,
        _attempt_account_linking: bool,
        user_context: &mut UserContext,
    ) -> Result<VerifyEmailUsingTokenResult, SuperTokensError> {
        let body = serde_json::json!({
            "method": "token",
            "token": token,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/user/email/verify", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => {
                let user_id = response
                    .get("userId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let email = response
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(VerifyEmailUsingTokenResult::Ok {
                    user: EmailVerificationUser {
                        recipe_user_id: RecipeUserId::new(user_id),
                        email,
                    },
                })
            }
            _ => Ok(VerifyEmailUsingTokenResult::InvalidToken),
        }
    }

    async fn is_email_verified(
        &self,
        recipe_user_id: &RecipeUserId,
        email: &str,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert(
            "userId".to_string(),
            recipe_user_id.get_as_string().to_string(),
        );
        params.insert("email".to_string(), email.to_string());

        let path = NormalisedURLPath::new("/recipe/user/email/verify")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        Ok(response
            .get("isVerified")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }

    async fn revoke_email_verification_tokens(
        &self,
        recipe_user_id: &RecipeUserId,
        email: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RevokeEmailVerificationTokensOkResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": recipe_user_id.get_as_string(),
            "email": email,
        });

        let path = NormalisedURLPath::new(&format!(
            "/{}/recipe/user/email/verify/token/remove",
            tenant_id
        ))?;
        self.querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        Ok(RevokeEmailVerificationTokensOkResult)
    }

    async fn unverify_email(
        &self,
        recipe_user_id: &RecipeUserId,
        email: &str,
        user_context: &mut UserContext,
    ) -> Result<UnverifyEmailOkResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": recipe_user_id.get_as_string(),
            "email": email,
        });

        let path = NormalisedURLPath::new("/recipe/user/email/verify/remove")?;
        self.querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        Ok(UnverifyEmailOkResult)
    }
}
