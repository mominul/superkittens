use async_trait::async_trait;
use std::sync::Arc;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::types::config::AppInfo;
use crate::types::user::{RecipeUserId, User};
use crate::user_context::UserContext;

/// Default implementation of the EmailPassword RecipeInterface.
pub struct RecipeImplementationImpl {
    pub querier: Querier,
    pub config: NormalisedEmailPasswordConfig,
    pub app_info: AppInfo,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn sign_up(
        &self,
        email: &str,
        password: &str,
        tenant_id: &str,
        _session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        _should_try_linking_with_session_user: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<SignUpResult, SuperTokensError> {
        self.create_new_recipe_user(email, password, tenant_id, user_context)
            .await
    }

    async fn create_new_recipe_user(
        &self,
        email: &str,
        password: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<SignUpResult, SuperTokensError> {
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/signup", tenant_id))?;
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
                let user: User =
                    serde_json::from_value(response.get("user").cloned().unwrap_or_default())?;
                let recipe_user_id = RecipeUserId::new(
                    response
                        .get("recipeUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&user.id)
                        .to_string(),
                );
                Ok(SignUpResult::Ok {
                    user: Box::new(user),
                    recipe_user_id,
                })
            }
            "EMAIL_ALREADY_EXISTS_ERROR" => Ok(SignUpResult::EmailAlreadyExists),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from signup: {}",
                status
            ))),
        }
    }

    async fn sign_in(
        &self,
        email: &str,
        password: &str,
        tenant_id: &str,
        _session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        _should_try_linking_with_session_user: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<SignInResult, SuperTokensError> {
        self.verify_credentials(email, password, tenant_id, user_context)
            .await
    }

    async fn verify_credentials(
        &self,
        email: &str,
        password: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<SignInResult, SuperTokensError> {
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/signin", tenant_id))?;
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
                let user: User =
                    serde_json::from_value(response.get("user").cloned().unwrap_or_default())?;
                let recipe_user_id = RecipeUserId::new(
                    response
                        .get("recipeUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&user.id)
                        .to_string(),
                );
                Ok(SignInResult::Ok {
                    user: Box::new(user),
                    recipe_user_id,
                })
            }
            "WRONG_CREDENTIALS_ERROR" => Ok(SignInResult::WrongCredentials),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from signin: {}",
                status
            ))),
        }
    }

    async fn create_reset_password_token(
        &self,
        user_id: &str,
        email: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateResetPasswordTokenResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "email": email,
        });

        let path =
            NormalisedURLPath::new(&format!("/{}/recipe/user/password/reset/token", tenant_id))?;
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
                Ok(CreateResetPasswordTokenResult::Ok { token })
            }
            "UNKNOWN_USER_ID_ERROR" => Ok(CreateResetPasswordTokenResult::UnknownUserId),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from create_reset_password_token: {}",
                status
            ))),
        }
    }

    async fn consume_password_reset_token(
        &self,
        token: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ConsumePasswordResetTokenResult, SuperTokensError> {
        let body = serde_json::json!({
            "token": token,
        });

        let path = NormalisedURLPath::new(&format!(
            "/{}/recipe/user/password/reset/token/consume",
            tenant_id
        ))?;
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
                let email = response
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let user_id = response
                    .get("userId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(ConsumePasswordResetTokenResult::Ok { email, user_id })
            }
            _ => Ok(ConsumePasswordResetTokenResult::PasswordResetTokenInvalid),
        }
    }

    async fn update_email_or_password(
        &self,
        recipe_user_id: &str,
        email: Option<&str>,
        password: Option<&str>,
        apply_password_policy: Option<bool>,
        tenant_id_for_password_policy: Option<&str>,
        user_context: &mut UserContext,
    ) -> Result<UpdateEmailOrPasswordResult, SuperTokensError> {
        // Validate password policy if requested
        if apply_password_policy.unwrap_or(true) {
            if let Some(password_val) = password {
                let tenant = tenant_id_for_password_policy
                    .unwrap_or(crate::recipe::session::constants::DEFAULT_TENANT_ID);

                for field in &self.config.sign_up_feature.form_fields {
                    if field.id == super::constants::FORM_FIELD_PASSWORD_ID {
                        let error =
                            (field.validate)(password_val.to_string(), tenant.to_string()).await?;
                        if let Some(failure_reason) = error {
                            return Ok(UpdateEmailOrPasswordResult::PasswordPolicyViolation {
                                failure_reason,
                            });
                        }
                        break;
                    }
                }
            }
        }

        let mut body = serde_json::json!({
            "recipeUserId": recipe_user_id,
        });

        if let Some(e) = email {
            body["email"] = serde_json::Value::String(e.to_string());
        }
        if let Some(p) = password {
            body["password"] = serde_json::Value::String(p.to_string());
        }

        let path = NormalisedURLPath::new("/recipe/user")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(UpdateEmailOrPasswordResult::Ok),
            "EMAIL_ALREADY_EXISTS_ERROR" => Ok(UpdateEmailOrPasswordResult::EmailAlreadyExists),
            "UNKNOWN_USER_ID_ERROR" => Ok(UpdateEmailOrPasswordResult::UnknownUserId),
            "EMAIL_CHANGE_NOT_ALLOWED_ERROR" => {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Email change not allowed")
                    .to_string();
                Ok(UpdateEmailOrPasswordResult::EmailChangeNotAllowed { reason })
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from update_email_or_password: {}",
                status
            ))),
        }
    }
}
