use async_trait::async_trait;
use std::sync::Arc;

use super::api_implementation::ApiImplementationImpl;
use super::constants;
use super::cookie_and_header;
use super::interfaces::{ApiInterface, RecipeInterface, SessionClaim, SessionClaimValidator};
use super::recipe_implementation::RecipeImplementationImpl;
use super::types::*;
use super::utils::validate_and_normalise_user_input;
use crate::error::SuperTokensError;
use crate::framework::request::BaseRequest;
use crate::framework::response::BaseResponse;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::recipe_module::{APIHandled, HttpMethod, RecipeModule};
use crate::types::config::AppInfo;
use crate::user_context::UserContext;

/// The Session recipe module.
pub struct SessionRecipe {
    recipe_id: String,
    app_info: AppInfo,
    config: NormalisedSessionConfig,
    recipe_implementation: Arc<dyn RecipeInterface>,
    api_implementation: Arc<dyn ApiInterface>,
    claims_added_by_other_recipes: std::sync::Mutex<Vec<Box<dyn SessionClaim>>>,
    claim_validators_added_by_other_recipes: std::sync::Mutex<Vec<Box<dyn SessionClaimValidator>>>,
}

impl SessionRecipe {
    pub fn new(app_info: AppInfo, config: SessionConfig) -> Result<Self, SuperTokensError> {
        let normalised_config = validate_and_normalise_user_input(&app_info, config)?;

        let querier = Querier::get_instance(Some("session".to_string()))?;

        let recipe_impl = Arc::new(RecipeImplementationImpl {
            querier,
            config: normalised_config.clone(),
            app_info: app_info.clone(),
        });

        let api_impl = Arc::new(ApiImplementationImpl);

        Ok(Self {
            recipe_id: "session".to_string(),
            app_info,
            config: normalised_config,
            recipe_implementation: recipe_impl,
            api_implementation: api_impl,
            claims_added_by_other_recipes: std::sync::Mutex::new(Vec::new()),
            claim_validators_added_by_other_recipes: std::sync::Mutex::new(Vec::new()),
        })
    }

    pub fn get_recipe_implementation(&self) -> &Arc<dyn RecipeInterface> {
        &self.recipe_implementation
    }

    pub fn get_api_implementation(&self) -> &Arc<dyn ApiInterface> {
        &self.api_implementation
    }

    pub fn get_config(&self) -> &NormalisedSessionConfig {
        &self.config
    }

    /// Add a claim from another recipe (e.g., email verification adds its claim).
    pub fn add_claim_from_other_recipe(
        &self,
        claim: Box<dyn SessionClaim>,
    ) -> Result<(), SuperTokensError> {
        let mut claims = self.claims_added_by_other_recipes.lock().unwrap();
        // Check for duplicate keys
        let key = claim.get_key().to_string();
        if claims.iter().any(|c| c.get_key() == key) {
            return Err(crate::error::raise_general_exception(format!(
                "Claim with key '{}' already exists",
                key
            )));
        }
        claims.push(claim);
        Ok(())
    }

    /// Add a claim validator from another recipe.
    pub fn add_claim_validator_from_other_recipe(&self, validator: Box<dyn SessionClaimValidator>) {
        let mut validators = self.claim_validators_added_by_other_recipes.lock().unwrap();
        validators.push(validator);
    }
}

#[async_trait]
impl RecipeModule for SessionRecipe {
    fn get_recipe_id(&self) -> &str {
        &self.recipe_id
    }

    fn get_app_info(&self) -> &AppInfo {
        &self.app_info
    }

    fn get_apis_handled(&self) -> Vec<APIHandled> {
        vec![
            APIHandled {
                path_without_api_base_path: NormalisedURLPath::new(constants::SESSION_REFRESH)
                    .unwrap(),
                method: HttpMethod::Post,
                request_id: "session-refresh".to_string(),
                disabled: self.api_implementation.disable_refresh_post(),
            },
            APIHandled {
                path_without_api_base_path: NormalisedURLPath::new(constants::SIGNOUT).unwrap(),
                method: HttpMethod::Post,
                request_id: "session-signout".to_string(),
                disabled: self.api_implementation.disable_signout_post(),
            },
        ]
    }

    fn is_error_from_this_recipe_based_on_instance(&self, err: &SuperTokensError) -> bool {
        matches!(err, SuperTokensError::Session(_))
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_api_request(
        &self,
        request_id: &str,
        _tenant_id: &str,
        request: &dyn BaseRequest,
        _path: &NormalisedURLPath,
        _method: &str,
        response: &mut dyn BaseResponse,
        user_context: &mut UserContext,
    ) -> Result<Option<()>, SuperTokensError> {
        let api_options = super::interfaces::ApiOptions {
            request,
            recipe_id: self.recipe_id.clone(),
            config: self.config.clone(),
            recipe_implementation: self.recipe_implementation.clone(),
        };

        match request_id {
            "session-refresh" => {
                let session = self
                    .api_implementation
                    .refresh_post(&api_options, user_context)
                    .await?;

                // Attach session to response
                session
                    .attach_to_request_response(
                        request,
                        response,
                        TokenTransferMethod::Cookie, // TODO: determine from request
                        user_context,
                    )
                    .await?;

                crate::utils::send_200_response(serde_json::json!({"status": "OK"}), response);
                Ok(Some(()))
            }
            "session-signout" => {
                // Verify session first
                let session = self
                    .api_implementation
                    .verify_session(&api_options, None, true, false, user_context)
                    .await?;

                if let Some(session) = session {
                    let result = self
                        .api_implementation
                        .signout_post(session.as_ref(), &api_options, user_context)
                        .await?;

                    // Clear session cookies
                    cookie_and_header::clear_session_from_all_token_transfer_methods(
                        response,
                        &self.config,
                        request,
                        user_context,
                    );

                    crate::utils::send_200_response(result, response);
                } else {
                    crate::utils::send_200_response(serde_json::json!({"status": "OK"}), response);
                }
                Ok(Some(()))
            }
            _ => Ok(None),
        }
    }

    async fn handle_error(
        &self,
        request: &dyn BaseRequest,
        err: SuperTokensError,
        response: &mut dyn BaseResponse,
        _user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        match err {
            SuperTokensError::Session(ref session_err) => match session_err {
                crate::error::SessionError::Unauthorized { message } => {
                    (self.config.error_handlers.on_unauthorised)(
                        message.clone(),
                        request,
                        response,
                    );
                    Ok(())
                }
                crate::error::SessionError::TokenTheftDetected {
                    session_handle,
                    user_id,
                } => {
                    (self.config.error_handlers.on_token_theft_detected)(
                        session_handle.clone(),
                        user_id.clone(),
                        request,
                        response,
                    );
                    Ok(())
                }
                crate::error::SessionError::InvalidClaims(errors) => {
                    let claim_errors: Vec<ClaimValidationError> = errors
                        .iter()
                        .map(|e| ClaimValidationError {
                            id: e.id.clone(),
                            reason: e.reason.clone(),
                        })
                        .collect();
                    (self.config.error_handlers.on_invalid_claim)(claim_errors, request, response);
                    Ok(())
                }
                crate::error::SessionError::TryRefreshToken { message } => {
                    crate::utils::send_non_200_response_with_message(
                        message,
                        self.config.session_expired_status_code,
                        response,
                    );
                    Ok(())
                }
                crate::error::SessionError::ClearDuplicateSessionCookies => {
                    crate::utils::send_200_response(serde_json::json!({"status": "OK"}), response);
                    Ok(())
                }
            },
            _ => Err(err),
        }
    }

    fn get_all_cors_headers(&self) -> Vec<String> {
        cookie_and_header::get_cors_allowed_headers()
    }
}
