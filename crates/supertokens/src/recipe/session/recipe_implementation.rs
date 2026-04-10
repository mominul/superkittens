use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use super::access_token;
use super::constants::PROTECTED_PROPS;
use super::cookie_and_header;
use super::interfaces::{
    RecipeInterface, SessionClaim, SessionClaimValidator, SessionContainerInterface,
};
use super::jwt;
use super::session_class::Session;
use super::session_functions;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::types::config::AppInfo;
use crate::types::user::RecipeUserId;
use crate::user_context::UserContext;

/// Default implementation of the session RecipeInterface.
///
/// Talks to the SuperTokens Core for all session operations.
pub struct RecipeImplementationImpl {
    pub querier: Querier,
    pub config: NormalisedSessionConfig,
    pub app_info: AppInfo,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn create_new_session(
        &self,
        _user_id: &str,
        recipe_user_id: &RecipeUserId,
        access_token_payload: Option<Value>,
        session_data_in_database: Option<Value>,
        disable_anti_csrf: Option<bool>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Arc<dyn SessionContainerInterface>, SuperTokensError> {
        let response = session_functions::create_new_session(
            &self.querier,
            tenant_id,
            recipe_user_id,
            disable_anti_csrf.unwrap_or(false),
            access_token_payload,
            session_data_in_database,
            &self.config,
            user_context,
        )
        .await?;

        let front_token = cookie_and_header::build_front_token(
            &response.session.user_id,
            response.access_token.expiry,
            Some(&response.session.user_data_in_jwt),
        );

        let session = Session::new(
            // This is a chicken-and-egg problem: the Session needs an Arc<RecipeInterface>
            // but we're the RecipeInterface. In practice, the recipe will wrap this in Arc
            // before creating sessions. For now we create a dummy querier instance.
            // The real solution is for the Recipe to inject the Arc<RecipeInterface>.
            Arc::new(RecipeImplementationImpl {
                querier: self.querier.clone(),
                config: self.config.clone(),
                app_info: self.app_info.clone(),
            }),
            self.config.clone(),
            response.access_token.token.clone(),
            front_token,
            Some(response.refresh_token),
            response.anti_csrf_token,
            response.session.handle,
            response.session.user_id,
            response.session.recipe_user_id,
            response.session.user_data_in_jwt,
            response.session.tenant_id,
            true, // access_token_updated = true for new sessions
        );

        Ok(Arc::new(session))
    }

    async fn get_session(
        &self,
        access_token: Option<&str>,
        anti_csrf_token: Option<&str>,
        anti_csrf_check: Option<bool>,
        session_required: Option<bool>,
        check_database: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<Option<Arc<dyn SessionContainerInterface>>, SuperTokensError> {
        let session_required = session_required.unwrap_or(true);
        let check_database = check_database.unwrap_or(false);

        let access_token = match access_token {
            Some(at) if !at.is_empty() => at,
            _ => {
                if session_required {
                    return Err(
                        super::errors::SessionError::unauthorised("No access token found").into(),
                    );
                }
                return Ok(None);
            }
        };

        // Parse JWT
        let jwt_info =
            jwt::parse_jwt_without_signature_verification(access_token).map_err(|e| {
                super::errors::SessionError::try_refresh_token(format!(
                    "Failed to parse JWT: {}",
                    e
                ))
            })?;

        // Try local validation
        let do_anti_csrf = anti_csrf_check.unwrap_or(false);
        let local_result = access_token::get_info_from_access_token(&jwt_info, do_anti_csrf);

        match local_result {
            Ok(info) => {
                // Fast path: valid token, no parent refresh token, don't need to check Core
                if info.parent_refresh_token_hash1.is_none() && !check_database {
                    let front_token = cookie_and_header::build_front_token(
                        &info.user_id,
                        info.expiry_time,
                        Some(&info.user_data),
                    );

                    let session = Session::new(
                        Arc::new(RecipeImplementationImpl {
                            querier: self.querier.clone(),
                            config: self.config.clone(),
                            app_info: self.app_info.clone(),
                        }),
                        self.config.clone(),
                        access_token.to_string(),
                        front_token,
                        None,
                        info.anti_csrf_token,
                        info.session_handle,
                        info.user_id,
                        info.recipe_user_id,
                        info.user_data,
                        info.tenant_id,
                        false, // not updated
                    );

                    return Ok(Some(Arc::new(session)));
                }

                // Need to verify with Core
                let core_response = session_functions::verify_session_with_core(
                    &self.querier,
                    access_token,
                    anti_csrf_token,
                    do_anti_csrf,
                    check_database,
                    user_context,
                )
                .await
                .map_err(SuperTokensError::from)?;

                let new_at = core_response.access_token.as_ref();
                let at_str = new_at.map(|a| a.token.as_str()).unwrap_or(access_token);
                let expiry = new_at
                    .map(|a| a.expiry)
                    .unwrap_or(core_response.session.expiry_time);

                let front_token = cookie_and_header::build_front_token(
                    &core_response.session.user_id,
                    expiry,
                    Some(&core_response.session.user_data_in_jwt),
                );

                let session = Session::new(
                    Arc::new(RecipeImplementationImpl {
                        querier: self.querier.clone(),
                        config: self.config.clone(),
                        app_info: self.app_info.clone(),
                    }),
                    self.config.clone(),
                    at_str.to_string(),
                    front_token,
                    None,
                    None,
                    core_response.session.handle,
                    core_response.session.user_id,
                    core_response.session.recipe_user_id,
                    core_response.session.user_data_in_jwt,
                    core_response.session.tenant_id,
                    new_at.is_some(),
                );

                Ok(Some(Arc::new(session)))
            }
            Err(e) => {
                // Local validation failed — if session is not required, return None
                if !session_required {
                    return Ok(None);
                }
                Err(e.into())
            }
        }
    }

    async fn refresh_session(
        &self,
        refresh_token: &str,
        anti_csrf_token: Option<&str>,
        disable_anti_csrf: bool,
        user_context: &mut UserContext,
    ) -> Result<Arc<dyn SessionContainerInterface>, SuperTokensError> {
        let response = session_functions::refresh_session(
            &self.querier,
            refresh_token,
            anti_csrf_token,
            disable_anti_csrf,
            self.config.use_dynamic_access_token_signing_key,
            &self.config,
            user_context,
        )
        .await?;

        let front_token = cookie_and_header::build_front_token(
            &response.session.user_id,
            response.access_token.expiry,
            Some(&response.session.user_data_in_jwt),
        );

        let session = Session::new(
            Arc::new(RecipeImplementationImpl {
                querier: self.querier.clone(),
                config: self.config.clone(),
                app_info: self.app_info.clone(),
            }),
            self.config.clone(),
            response.access_token.token,
            front_token,
            Some(response.refresh_token),
            response.anti_csrf_token,
            response.session.handle,
            response.session.user_id,
            response.session.recipe_user_id,
            response.session.user_data_in_jwt,
            response.session.tenant_id,
            true,
        );

        Ok(Arc::new(session))
    }

    async fn revoke_session(
        &self,
        session_handle: &str,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError> {
        session_functions::revoke_session(&self.querier, session_handle, user_context).await
    }

    async fn revoke_all_sessions_for_user(
        &self,
        user_id: &str,
        revoke_sessions_for_linked_accounts: bool,
        tenant_id: &str,
        revoke_across_all_tenants: bool,
        user_context: &mut UserContext,
    ) -> Result<Vec<String>, SuperTokensError> {
        session_functions::revoke_all_sessions_for_user(
            &self.querier,
            user_id,
            revoke_sessions_for_linked_accounts,
            Some(tenant_id),
            revoke_across_all_tenants,
            user_context,
        )
        .await
    }

    async fn get_all_session_handles_for_user(
        &self,
        user_id: &str,
        fetch_sessions_for_linked_accounts: bool,
        tenant_id: &str,
        fetch_across_all_tenants: bool,
        user_context: &mut UserContext,
    ) -> Result<Vec<String>, SuperTokensError> {
        session_functions::get_all_session_handles_for_user(
            &self.querier,
            user_id,
            fetch_sessions_for_linked_accounts,
            Some(tenant_id),
            fetch_across_all_tenants,
            user_context,
        )
        .await
    }

    async fn revoke_multiple_sessions(
        &self,
        session_handles: &[String],
        user_context: &mut UserContext,
    ) -> Result<Vec<String>, SuperTokensError> {
        session_functions::revoke_multiple_sessions(&self.querier, session_handles, user_context)
            .await
    }

    async fn get_session_information(
        &self,
        session_handle: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<SessionInformationResult>, SuperTokensError> {
        session_functions::get_session_information(&self.querier, session_handle, user_context)
            .await
    }

    async fn update_session_data_in_database(
        &self,
        session_handle: &str,
        new_session_data: Value,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError> {
        session_functions::update_session_data_in_database(
            &self.querier,
            session_handle,
            new_session_data,
            user_context,
        )
        .await
    }

    async fn merge_into_access_token_payload(
        &self,
        session_handle: &str,
        access_token_payload_update: Value,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError> {
        // Get current session info
        let info = self
            .get_session_information(session_handle, user_context)
            .await?;
        let info = match info {
            Some(i) => i,
            None => return Ok(false),
        };

        // Merge payload
        let mut new_payload = info.custom_claims_in_access_token_payload.clone();
        if let (Value::Object(ref mut target), Value::Object(ref source)) =
            (&mut new_payload, &access_token_payload_update)
        {
            for (key, value) in source {
                if PROTECTED_PROPS.contains(&key.as_str()) {
                    continue;
                }
                if value.is_null() {
                    target.remove(key);
                } else {
                    target.insert(key.clone(), value.clone());
                }
            }
        }

        session_functions::update_access_token_payload(
            &self.querier,
            session_handle,
            new_payload,
            user_context,
        )
        .await
    }

    async fn regenerate_access_token(
        &self,
        access_token: &str,
        new_access_token_payload: Option<Value>,
        user_context: &mut UserContext,
    ) -> Result<Option<RegenerateAccessTokenOkResult>, SuperTokensError> {
        let mut body = serde_json::json!({
            "accessToken": access_token,
        });
        if let Some(payload) = new_access_token_payload {
            body["userDataInJWT"] = payload;
        }

        let path = NormalisedURLPath::new("/recipe/session/regenerate")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if status != "OK" {
            return Ok(None);
        }

        let session = response.get("session").unwrap_or(&Value::Null);
        let at = response.get("accessToken");

        let user_id = session
            .get("userId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        Ok(Some(RegenerateAccessTokenOkResult {
            session: SessionObj {
                handle: session
                    .get("handle")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                user_id: user_id.clone(),
                recipe_user_id: RecipeUserId::new(
                    session
                        .get("recipeUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&user_id),
                ),
                user_data_in_jwt: session.get("userDataInJWT").cloned().unwrap_or_default(),
                tenant_id: session
                    .get("tenantId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("public")
                    .to_string(),
            },
            access_token: at.and_then(|a| {
                Some(AccessTokenObj {
                    token: a.get("token")?.as_str()?.to_string(),
                    expiry: a.get("expiry")?.as_u64()?,
                    created_time: a.get("createdTime")?.as_u64()?,
                })
            }),
        }))
    }

    fn get_global_claim_validators(
        &self,
        _tenant_id: &str,
        _user_id: &str,
        _recipe_user_id: &RecipeUserId,
        claim_validators_added_by_other_recipes: Vec<Box<dyn SessionClaimValidator>>,
        _user_context: &UserContext,
    ) -> Vec<Box<dyn SessionClaimValidator>> {
        claim_validators_added_by_other_recipes
    }

    async fn validate_claims(
        &self,
        _user_id: &str,
        _recipe_user_id: &RecipeUserId,
        access_token_payload: &Value,
        claim_validators: &[Box<dyn SessionClaimValidator>],
        user_context: &mut UserContext,
    ) -> Result<ClaimsValidationResult, SuperTokensError> {
        let mut invalid_claims = Vec::new();
        let ctx = user_context.clone();

        for validator in claim_validators {
            let result = validator.validate(access_token_payload, &ctx).await;
            if !result.is_valid {
                invalid_claims.push(ClaimValidationError {
                    id: validator.get_id().to_string(),
                    reason: result.reason,
                });
            }
        }

        Ok(ClaimsValidationResult {
            invalid_claims,
            access_token_payload_update: None,
        })
    }

    async fn fetch_and_set_claim(
        &self,
        session_handle: &str,
        _claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError> {
        // Get session info, build claim update, merge
        let info = self
            .get_session_information(session_handle, user_context)
            .await?;
        match info {
            Some(_) => {
                // In a full implementation, we'd call claim.fetch_value() here
                // and then merge the result into the payload.
                // For now, this is a stub that returns Ok(true).
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn set_claim_value(
        &self,
        session_handle: &str,
        claim: &dyn SessionClaim,
        value: Value,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError> {
        let info = self
            .get_session_information(session_handle, user_context)
            .await?;
        let info = match info {
            Some(i) => i,
            None => return Ok(false),
        };

        let mut payload = info.custom_claims_in_access_token_payload;
        claim.add_to_payload(&mut payload, value);

        let update = serde_json::json!({
            claim.get_key(): payload.get(claim.get_key()),
        });

        self.merge_into_access_token_payload(session_handle, update, user_context)
            .await
    }

    async fn get_claim_value(
        &self,
        session_handle: &str,
        claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<Option<Value>, SuperTokensError> {
        let info = self
            .get_session_information(session_handle, user_context)
            .await?;

        match info {
            Some(i) => Ok(claim.get_value_from_payload(&i.custom_claims_in_access_token_payload)),
            None => Ok(None),
        }
    }

    async fn remove_claim(
        &self,
        session_handle: &str,
        claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError> {
        let mut payload = Value::Object(serde_json::Map::new());
        claim.remove_from_payload_by_merge(&mut payload);
        self.merge_into_access_token_payload(session_handle, payload, user_context)
            .await
    }
}
