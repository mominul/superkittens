use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use super::super::constants::FORM_FIELD_EMAIL_ID;
use super::super::interfaces::{ApiInterface, ApiOptions};
use super::super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::user_context::UserContext;

/// Default API implementation for emailpassword endpoints.
pub struct ApiImplementationImpl;

#[async_trait]
impl ApiInterface for ApiImplementationImpl {
    async fn email_exists_get(
        &self,
        email: &str,
        tenant_id: &str,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<EmailExistsGetResult, SuperTokensError> {
        let path = NormalisedURLPath::new(&format!("/{}/recipe/signup/email/exists", tenant_id))?;
        let mut params = HashMap::new();
        params.insert("email".to_string(), email.to_string());

        let response = api_options
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let exists = response
            .get("exists")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(EmailExistsGetResult::Ok { exists })
    }

    async fn generate_password_reset_token_post(
        &self,
        form_fields: &[FormField],
        tenant_id: &str,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<GeneratePasswordResetTokenPostResult, SuperTokensError> {
        let email = form_fields
            .iter()
            .find(|f| f.id == FORM_FIELD_EMAIL_ID)
            .map(|f| crate::utils::normalise_email(&f.value))
            .ok_or_else(|| crate::error::raise_bad_input_exception("Email field is required"))?;

        // Check if user exists
        let path = NormalisedURLPath::new(&format!("/{}/recipe/signup/email/exists", tenant_id))?;
        let mut params = HashMap::new();
        params.insert("email".to_string(), email.clone());

        let users_response = api_options
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let exists = users_response
            .get("exists")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !exists {
            // Silently succeed even if user doesn't exist (security best practice)
            return Ok(GeneratePasswordResetTokenPostResult::Ok);
        }

        // Get user info to get user_id
        let user_path = NormalisedURLPath::new(&format!("/{}/recipe/user", tenant_id))?;
        let mut user_params = HashMap::new();
        user_params.insert("email".to_string(), email.clone());

        let user_response = api_options
            .querier
            .send_get_request(&user_path, Some(user_params), user_context)
            .await;

        if let Ok(resp) = user_response {
            if let Some(user_id) = resp
                .get("user")
                .and_then(|u| u.get("id"))
                .and_then(|v| v.as_str())
            {
                let token_result = api_options
                    .recipe_implementation
                    .create_reset_password_token(user_id, &email, tenant_id, user_context)
                    .await?;

                if let CreateResetPasswordTokenResult::Ok { token } = token_result {
                    let link = super::super::utils::get_password_reset_link(
                        api_options.app_info,
                        &token,
                        tenant_id,
                    );

                    let template_vars = PasswordResetEmailTemplateVars {
                        user: PasswordResetEmailTemplateVarsUser {
                            id: user_id.to_string(),
                            recipe_user_id: crate::types::user::RecipeUserId::new(
                                user_id.to_string(),
                            ),
                            email: email.clone(),
                        },
                        password_reset_link: link,
                        tenant_id: tenant_id.to_string(),
                    };

                    api_options
                        .email_delivery
                        .send_email(template_vars, user_context)
                        .await?;
                }
            }
        }

        // Always return OK to avoid user enumeration
        Ok(GeneratePasswordResetTokenPostResult::Ok)
    }

    async fn password_reset_post(
        &self,
        form_fields: &[FormField],
        token: &str,
        tenant_id: &str,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<PasswordResetPostResult, SuperTokensError> {
        let consume_result = api_options
            .recipe_implementation
            .consume_password_reset_token(token, tenant_id, user_context)
            .await?;

        let (email, user_id) = match consume_result {
            ConsumePasswordResetTokenResult::Ok { email, user_id } => (email, user_id),
            ConsumePasswordResetTokenResult::PasswordResetTokenInvalid => {
                return Ok(PasswordResetPostResult::PasswordResetTokenInvalid);
            }
        };

        let new_password = form_fields
            .iter()
            .find(|f| f.id == super::super::constants::FORM_FIELD_PASSWORD_ID)
            .map(|f| f.value.clone())
            .ok_or_else(|| crate::error::raise_bad_input_exception("Password field is required"))?;

        let update_result = api_options
            .recipe_implementation
            .update_email_or_password(
                &user_id,
                None,
                Some(&new_password),
                Some(true),
                Some(tenant_id),
                user_context,
            )
            .await?;

        match update_result {
            UpdateEmailOrPasswordResult::Ok => {
                Ok(PasswordResetPostResult::Ok { email, user: None })
            }
            UpdateEmailOrPasswordResult::PasswordPolicyViolation { failure_reason } => {
                Ok(PasswordResetPostResult::PasswordPolicyViolation { failure_reason })
            }
            UpdateEmailOrPasswordResult::UnknownUserId => {
                Ok(PasswordResetPostResult::PasswordResetTokenInvalid)
            }
            _ => Err(crate::error::raise_general_exception(
                "Unexpected result from update_email_or_password during password reset",
            )),
        }
    }

    async fn sign_in_post(
        &self,
        form_fields: &[FormField],
        tenant_id: &str,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<SignInPostResult, SuperTokensError> {
        let email = form_fields
            .iter()
            .find(|f| f.id == FORM_FIELD_EMAIL_ID)
            .map(|f| f.value.clone())
            .ok_or_else(|| crate::error::raise_bad_input_exception("Email field is required"))?;

        let password = form_fields
            .iter()
            .find(|f| f.id == super::super::constants::FORM_FIELD_PASSWORD_ID)
            .map(|f| f.value.clone())
            .ok_or_else(|| crate::error::raise_bad_input_exception("Password field is required"))?;

        let result = api_options
            .recipe_implementation
            .sign_in(
                &email,
                &password,
                tenant_id,
                session,
                should_try_linking_with_session_user,
                user_context,
            )
            .await?;

        match result {
            SignInResult::Ok {
                user,
                recipe_user_id,
            } => {
                let session = create_session_for_user(
                    &user.id,
                    recipe_user_id.get_as_string(),
                    tenant_id,
                    user_context,
                )
                .await?;

                Ok(SignInPostResult::Ok {
                    user: *user,
                    session,
                })
            }
            SignInResult::WrongCredentials => Ok(SignInPostResult::WrongCredentials),
        }
    }

    async fn sign_up_post(
        &self,
        form_fields: &[FormField],
        tenant_id: &str,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<SignUpPostResult, SuperTokensError> {
        let email = form_fields
            .iter()
            .find(|f| f.id == FORM_FIELD_EMAIL_ID)
            .map(|f| f.value.clone())
            .ok_or_else(|| crate::error::raise_bad_input_exception("Email field is required"))?;

        let password = form_fields
            .iter()
            .find(|f| f.id == super::super::constants::FORM_FIELD_PASSWORD_ID)
            .map(|f| f.value.clone())
            .ok_or_else(|| crate::error::raise_bad_input_exception("Password field is required"))?;

        let result = api_options
            .recipe_implementation
            .sign_up(
                &email,
                &password,
                tenant_id,
                session,
                should_try_linking_with_session_user,
                user_context,
            )
            .await?;

        match result {
            SignUpResult::Ok {
                user,
                recipe_user_id,
            } => {
                let session = create_session_for_user(
                    &user.id,
                    recipe_user_id.get_as_string(),
                    tenant_id,
                    user_context,
                )
                .await?;

                Ok(SignUpPostResult::Ok {
                    user: *user,
                    session,
                })
            }
            SignUpResult::EmailAlreadyExists => Ok(SignUpPostResult::EmailAlreadyExists),
        }
    }
}

/// Helper to create a session after successful authentication.
async fn create_session_for_user(
    user_id: &str,
    recipe_user_id: &str,
    tenant_id: &str,
    user_context: &mut UserContext,
) -> Result<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>, SuperTokensError>
{
    let st = crate::Supertokens::get_instance()?;
    let querier = crate::querier::Querier::get_instance(Some("session".into()))?;

    let config = crate::recipe::session::utils::validate_and_normalise_user_input(
        &st.app_info,
        crate::recipe::session::types::SessionConfig {
            cookie_domain: None,
            older_cookie_domain: None,
            cookie_secure: None,
            cookie_same_site: None,
            session_expired_status_code: None,
            anti_csrf: None,
            get_token_transfer_method: None,
            error_handlers: None,
            invalid_claim_status_code: None,
            use_dynamic_access_token_signing_key: None,
            expose_access_token_to_frontend_in_cookie_based_auth: None,
            jwks_refresh_interval_sec: None,
        },
    )?;

    let recipe_impl = Arc::new(
        crate::recipe::session::recipe_implementation::RecipeImplementationImpl {
            querier,
            config,
            app_info: st.app_info.clone(),
        },
    );

    use crate::recipe::session::interfaces::RecipeInterface as SessionRecipeInterface;

    let session = recipe_impl
        .create_new_session(
            user_id,
            &crate::types::user::RecipeUserId::new(recipe_user_id.to_string()),
            Some(serde_json::json!({})),
            Some(serde_json::json!({})),
            None,
            tenant_id,
            user_context,
        )
        .await?;

    Ok(session)
}
