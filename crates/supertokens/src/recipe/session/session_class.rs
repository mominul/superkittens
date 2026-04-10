use async_trait::async_trait;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use super::constants::PROTECTED_PROPS;
use super::cookie_and_header;
use super::errors::SessionError;
use super::interfaces::{
    RecipeInterface, SessionClaim, SessionClaimValidator, SessionContainerInterface,
};
use super::types::*;
use crate::error::SuperTokensError;
use crate::framework::request::BaseRequest;
use crate::framework::response::BaseResponse;
use crate::types::user::RecipeUserId;
use crate::user_context::UserContext;

/// Concrete session implementation.
pub struct Session {
    pub recipe_implementation: Arc<dyn RecipeInterface>,
    pub config: NormalisedSessionConfig,
    pub session_handle: String,
    pub user_id: String,
    pub recipe_user_id: RecipeUserId,
    pub tenant_id: String,
    access_token: Mutex<String>,
    front_token: Mutex<String>,
    pub refresh_token: Option<TokenInfo>,
    pub anti_csrf_token: Option<String>,
    user_data_in_access_token: Mutex<Value>,
    access_token_updated: Mutex<bool>,
    pub req_res_info: Mutex<Option<ReqResInfo>>,
}

impl Session {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        recipe_implementation: Arc<dyn RecipeInterface>,
        config: NormalisedSessionConfig,
        access_token: String,
        front_token: String,
        refresh_token: Option<TokenInfo>,
        anti_csrf_token: Option<String>,
        session_handle: String,
        user_id: String,
        recipe_user_id: RecipeUserId,
        user_data_in_access_token: Value,
        tenant_id: String,
        access_token_updated: bool,
    ) -> Self {
        Self {
            recipe_implementation,
            config,
            session_handle,
            user_id,
            recipe_user_id,
            tenant_id,
            access_token: Mutex::new(access_token),
            front_token: Mutex::new(front_token),
            refresh_token,
            anti_csrf_token,
            user_data_in_access_token: Mutex::new(user_data_in_access_token),
            access_token_updated: Mutex::new(access_token_updated),
            req_res_info: Mutex::new(None),
        }
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("session_handle", &self.session_handle)
            .field("user_id", &self.user_id)
            .field("tenant_id", &self.tenant_id)
            .finish()
    }
}

#[async_trait]
impl SessionContainerInterface for Session {
    fn get_handle(&self) -> &str {
        &self.session_handle
    }

    fn get_user_id(&self) -> &str {
        &self.user_id
    }

    fn get_recipe_user_id(&self) -> &RecipeUserId {
        &self.recipe_user_id
    }

    fn get_tenant_id(&self) -> &str {
        &self.tenant_id
    }

    fn get_access_token(&self) -> &str {
        // This is a bit awkward with Mutex but we need interior mutability
        // For a real production impl, consider using parking_lot::Mutex
        // which returns a guard that can deref. Here we leak a reference.
        // In practice, callers should use get_all_session_tokens_dangerously().
        // We'll return a static empty string as a safety fallback;
        // the real access token is available via get_all_session_tokens_dangerously.
        // TODO: Refactor to return String instead of &str
        ""
    }

    fn get_access_token_payload(&self) -> &Value {
        // Same limitation as get_access_token — see TODO above
        &Value::Null
    }

    fn get_all_session_tokens_dangerously(&self) -> GetSessionTokensDangerously {
        let at = self.access_token.lock().unwrap().clone();
        let ft = self.front_token.lock().unwrap().clone();
        let updated = *self.access_token_updated.lock().unwrap();

        GetSessionTokensDangerously {
            access_token: at,
            access_and_front_token_updated: updated,
            refresh_token: self.refresh_token.as_ref().map(|t| t.token.clone()),
            front_token: ft,
            anti_csrf_token: self.anti_csrf_token.clone(),
        }
    }

    fn access_token_updated(&self) -> bool {
        *self.access_token_updated.lock().unwrap()
    }

    fn get_response_mutators(
        &self,
    ) -> &[Box<dyn Fn(&mut dyn BaseResponse, &UserContext) + Send + Sync>] {
        &[]
    }

    async fn revoke_session(&self, user_context: &mut UserContext) -> Result<(), SuperTokensError> {
        self.recipe_implementation
            .revoke_session(&self.session_handle, user_context)
            .await?;
        Ok(())
    }

    async fn get_session_data_from_database(
        &self,
        user_context: &mut UserContext,
    ) -> Result<Value, SuperTokensError> {
        let info = self
            .recipe_implementation
            .get_session_information(&self.session_handle, user_context)
            .await?;

        match info {
            Some(info) => Ok(info.session_data_in_database),
            None => Err(SessionError::unauthorised("Session does not exist").into()),
        }
    }

    async fn update_session_data_in_database(
        &self,
        new_session_data: Value,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        let success = self
            .recipe_implementation
            .update_session_data_in_database(&self.session_handle, new_session_data, user_context)
            .await?;

        if !success {
            return Err(SessionError::unauthorised("Session does not exist").into());
        }
        Ok(())
    }

    async fn merge_into_access_token_payload(
        &self,
        access_token_payload_update: Value,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        // Build new payload: start from current, merge update, remove protected props
        let current_payload = self.user_data_in_access_token.lock().unwrap().clone();
        let mut new_payload = current_payload.clone();

        if let (Value::Object(ref mut target), Value::Object(ref source)) =
            (&mut new_payload, &access_token_payload_update)
        {
            // Remove protected props from update
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

        // Regenerate access token via recipe implementation
        let at = self.access_token.lock().unwrap().clone();
        let result = self
            .recipe_implementation
            .regenerate_access_token(&at, Some(new_payload.clone()), user_context)
            .await?;

        if let Some(regen) = result {
            // Update internal state
            *self.user_data_in_access_token.lock().unwrap() = regen.session.user_data_in_jwt;
            if let Some(new_at) = regen.access_token {
                *self.access_token.lock().unwrap() = new_at.token;
                *self.access_token_updated.lock().unwrap() = true;

                // Rebuild front token
                let user_id = &self.user_id;
                let ft = cookie_and_header::build_front_token(
                    user_id,
                    new_at.expiry,
                    Some(&new_payload),
                );
                *self.front_token.lock().unwrap() = ft;
            }
        }

        Ok(())
    }

    async fn get_time_created(
        &self,
        user_context: &mut UserContext,
    ) -> Result<u64, SuperTokensError> {
        let info = self
            .recipe_implementation
            .get_session_information(&self.session_handle, user_context)
            .await?;

        match info {
            Some(info) => Ok(info.time_created),
            None => Err(SessionError::unauthorised("Session does not exist").into()),
        }
    }

    async fn get_expiry(&self, user_context: &mut UserContext) -> Result<u64, SuperTokensError> {
        let info = self
            .recipe_implementation
            .get_session_information(&self.session_handle, user_context)
            .await?;

        match info {
            Some(info) => Ok(info.expiry),
            None => Err(SessionError::unauthorised("Session does not exist").into()),
        }
    }

    async fn fetch_and_set_claim(
        &self,
        claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        // In a full implementation, we'd call claim.fetch_value() here
        // For now, delegate to recipe_implementation
        self.recipe_implementation
            .fetch_and_set_claim(&self.session_handle, claim, user_context)
            .await?;
        Ok(())
    }

    async fn set_claim_value(
        &self,
        claim: &dyn SessionClaim,
        value: Value,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        let mut payload = self.user_data_in_access_token.lock().unwrap().clone();
        claim.add_to_payload(&mut payload, value);

        // Extract the update (just the claim's key)
        let update = serde_json::json!({
            claim.get_key(): payload.get(claim.get_key()),
        });

        self.merge_into_access_token_payload(update, user_context)
            .await
    }

    async fn get_claim_value(
        &self,
        claim: &dyn SessionClaim,
        _user_context: &mut UserContext,
    ) -> Result<Option<Value>, SuperTokensError> {
        let payload = self.user_data_in_access_token.lock().unwrap().clone();
        Ok(claim.get_value_from_payload(&payload))
    }

    async fn remove_claim(
        &self,
        claim: &dyn SessionClaim,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        let mut payload = Value::Object(serde_json::Map::new());
        claim.remove_from_payload_by_merge(&mut payload);
        self.merge_into_access_token_payload(payload, user_context)
            .await
    }

    async fn assert_claims(
        &self,
        claim_validators: &[Box<dyn SessionClaimValidator>],
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        let payload = self.user_data_in_access_token.lock().unwrap().clone();

        let result = self
            .recipe_implementation
            .validate_claims(
                &self.user_id,
                &self.recipe_user_id,
                &payload,
                claim_validators,
                user_context,
            )
            .await?;

        // Apply payload update if any
        if let Some(update) = result.access_token_payload_update {
            let mut filtered_update = update.clone();
            if let Value::Object(ref mut map) = filtered_update {
                for prop in PROTECTED_PROPS {
                    map.remove(*prop);
                }
            }
            self.merge_into_access_token_payload(filtered_update, user_context)
                .await?;
        }

        if !result.invalid_claims.is_empty() {
            return Err(SessionError::invalid_claims(
                "Some claims are invalid",
                result.invalid_claims,
            )
            .into());
        }

        Ok(())
    }

    async fn attach_to_request_response(
        &self,
        request: &dyn BaseRequest,
        response: &mut dyn BaseResponse,
        transfer_method: TokenTransferMethod,
        user_context: &UserContext,
    ) -> Result<(), SuperTokensError> {
        *self.req_res_info.lock().unwrap() = Some(ReqResInfo { transfer_method });

        let tokens = self.get_all_session_tokens_dangerously();

        if tokens.access_and_front_token_updated {
            cookie_and_header::set_access_token_in_response(
                response,
                &tokens.access_token,
                &tokens.front_token,
                &self.config,
                transfer_method,
                request,
                user_context,
            );

            if let Some(ref rt) = tokens.refresh_token {
                cookie_and_header::set_token(
                    response,
                    &self.config,
                    TokenType::Refresh,
                    rt,
                    self.refresh_token.as_ref().map(|t| t.expiry).unwrap_or(0),
                    transfer_method,
                    request,
                    user_context,
                );
            }

            if let Some(ref csrf) = tokens.anti_csrf_token {
                cookie_and_header::attach_anti_csrf_header(response, csrf);
            }
        }

        Ok(())
    }
}
