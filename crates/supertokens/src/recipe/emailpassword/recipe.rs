use async_trait::async_trait;
use std::sync::Arc;

use super::api;
use super::constants::*;
use super::interfaces::{ApiInterface, ApiOptions, RecipeInterface};
use super::recipe_implementation::RecipeImplementationImpl;
use super::types::*;
use super::utils::validate_and_normalise_user_input;

use crate::error::{EmailPasswordError, SuperTokensError};
use crate::framework::request::BaseRequest;
use crate::framework::response::BaseResponse;
use crate::ingredients::email_delivery::EmailDeliveryInterface;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::recipe_module::{APIHandled, HttpMethod, RecipeModule};
use crate::types::config::AppInfo;
use crate::user_context::UserContext;

/// The EmailPassword recipe module.
pub struct EmailPasswordRecipe {
    pub recipe_id: String,
    pub app_info: AppInfo,
    pub config: NormalisedEmailPasswordConfig,
    pub recipe_implementation: Arc<dyn RecipeInterface>,
    pub api_implementation: Arc<dyn ApiInterface>,
    pub email_delivery: Arc<dyn EmailDeliveryInterface<EmailTemplateVars>>,
}

impl EmailPasswordRecipe {
    /// Initialize the EmailPassword recipe.
    pub fn new(
        app_info: AppInfo,
        config: EmailPasswordConfig,
        email_delivery: Arc<dyn EmailDeliveryInterface<EmailTemplateVars>>,
    ) -> Result<Self, SuperTokensError> {
        let normalised_config = validate_and_normalise_user_input(&app_info, config)?;

        let querier = Querier::get_instance(Some("emailpassword".into()))?;

        let recipe_impl: Arc<dyn RecipeInterface> = Arc::new(RecipeImplementationImpl {
            querier: querier.clone(),
            config: NormalisedEmailPasswordConfig {
                sign_up_feature: SignUpFeature {
                    form_fields: normalised_config.sign_up_feature.form_fields.clone(),
                },
                sign_in_feature: SignInFeature {
                    form_fields: normalised_config.sign_in_feature.form_fields.clone(),
                },
                reset_password_using_token_feature: ResetPasswordUsingTokenFeature {
                    form_fields_for_password_reset_form: normalised_config
                        .reset_password_using_token_feature
                        .form_fields_for_password_reset_form
                        .clone(),
                    form_fields_for_generate_token_form: normalised_config
                        .reset_password_using_token_feature
                        .form_fields_for_generate_token_form
                        .clone(),
                },
                override_: OverrideConfig {
                    functions: None,
                    apis: None,
                },
            },
            app_info: app_info.clone(),
        });

        // Apply override if provided
        let recipe_impl = if let Some(ref override_fn) = normalised_config.override_.functions {
            override_fn(recipe_impl)
        } else {
            recipe_impl
        };

        let api_impl: Arc<dyn ApiInterface> = Arc::new(api::implementation::ApiImplementationImpl);

        let api_impl = if let Some(ref override_fn) = normalised_config.override_.apis {
            override_fn(api_impl)
        } else {
            api_impl
        };

        Ok(Self {
            recipe_id: "emailpassword".to_string(),
            app_info,
            config: normalised_config,
            recipe_implementation: recipe_impl,
            api_implementation: api_impl,
            email_delivery,
        })
    }
}

#[async_trait]
impl RecipeModule for EmailPasswordRecipe {
    fn get_recipe_id(&self) -> &str {
        &self.recipe_id
    }

    fn get_app_info(&self) -> &AppInfo {
        &self.app_info
    }

    fn get_apis_handled(&self) -> Vec<APIHandled> {
        vec![
            APIHandled {
                path_without_api_base_path: NormalisedURLPath::new(SIGNUP).unwrap(),
                method: HttpMethod::Post,
                request_id: "emailpassword-signup".to_string(),
                disabled: false,
            },
            APIHandled {
                path_without_api_base_path: NormalisedURLPath::new(SIGNIN).unwrap(),
                method: HttpMethod::Post,
                request_id: "emailpassword-signin".to_string(),
                disabled: false,
            },
            APIHandled {
                path_without_api_base_path: NormalisedURLPath::new(USER_PASSWORD_RESET_TOKEN)
                    .unwrap(),
                method: HttpMethod::Post,
                request_id: "emailpassword-generate-password-reset-token".to_string(),
                disabled: false,
            },
            APIHandled {
                path_without_api_base_path: NormalisedURLPath::new(USER_PASSWORD_RESET).unwrap(),
                method: HttpMethod::Post,
                request_id: "emailpassword-password-reset".to_string(),
                disabled: false,
            },
            APIHandled {
                path_without_api_base_path: NormalisedURLPath::new(SIGNUP_EMAIL_EXISTS).unwrap(),
                method: HttpMethod::Get,
                request_id: "emailpassword-email-exists".to_string(),
                disabled: false,
            },
            APIHandled {
                path_without_api_base_path: NormalisedURLPath::new(SIGNUP_EMAIL_EXISTS_OLD)
                    .unwrap(),
                method: HttpMethod::Get,
                request_id: "emailpassword-email-exists".to_string(),
                disabled: false,
            },
        ]
    }

    fn is_error_from_this_recipe_based_on_instance(&self, err: &SuperTokensError) -> bool {
        matches!(err, SuperTokensError::EmailPassword(_))
    }

    async fn handle_api_request(
        &self,
        request_id: &str,
        tenant_id: &str,
        request: &dyn BaseRequest,
        _path: &NormalisedURLPath,
        _method: &str,
        response: &mut dyn BaseResponse,
        user_context: &mut UserContext,
    ) -> Result<Option<()>, SuperTokensError> {
        let querier = Querier::get_instance(Some("emailpassword".into()))?;

        let api_options = ApiOptions {
            request,
            recipe_id: self.recipe_id.clone(),
            config: &self.config,
            recipe_implementation: self.recipe_implementation.clone(),
            app_info: &self.app_info,
            querier,
            email_delivery: self.email_delivery.clone(),
        };

        let result = match request_id {
            "emailpassword-signup" => {
                api::signup::handle_sign_up_api(
                    self.api_implementation.as_ref(),
                    tenant_id,
                    &api_options,
                    user_context,
                )
                .await?
            }
            "emailpassword-signin" => {
                api::signin::handle_sign_in_api(
                    self.api_implementation.as_ref(),
                    tenant_id,
                    &api_options,
                    user_context,
                )
                .await?
            }
            "emailpassword-email-exists" => {
                api::email_exists::handle_email_exists_api(
                    self.api_implementation.as_ref(),
                    tenant_id,
                    &api_options,
                    user_context,
                )
                .await?
            }
            "emailpassword-generate-password-reset-token" => {
                api::generate_password_reset_token::handle_generate_password_reset_token_api(
                    self.api_implementation.as_ref(),
                    tenant_id,
                    &api_options,
                    user_context,
                )
                .await?
            }
            "emailpassword-password-reset" => {
                api::password_reset::handle_password_reset_api(
                    self.api_implementation.as_ref(),
                    tenant_id,
                    &api_options,
                    user_context,
                )
                .await?
            }
            _ => {
                return Err(crate::error::raise_general_exception(format!(
                    "Unknown request_id: {}",
                    request_id
                )));
            }
        };

        response.set_json_content(result);
        Ok(Some(()))
    }

    async fn handle_error(
        &self,
        _request: &dyn BaseRequest,
        err: SuperTokensError,
        response: &mut dyn BaseResponse,
        _user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        match &err {
            SuperTokensError::EmailPassword(EmailPasswordError::FieldError {
                message: _,
                form_fields,
            }) => {
                let json_fields: Vec<serde_json::Value> = form_fields
                    .iter()
                    .map(|(id, error)| {
                        serde_json::json!({
                            "id": id,
                            "error": error,
                        })
                    })
                    .collect();

                response.set_status_code(200);
                response.set_json_content(serde_json::json!({
                    "status": "FIELD_ERROR",
                    "formFields": json_fields,
                }));
                Ok(())
            }
            _ => Err(err),
        }
    }

    fn get_all_cors_headers(&self) -> Vec<String> {
        Vec::new()
    }
}
