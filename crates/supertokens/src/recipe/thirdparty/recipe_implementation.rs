use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::types::user::{RecipeUserId, User};
use crate::user_context::UserContext;

/// Default implementation of the ThirdParty RecipeInterface.
pub struct RecipeImplementation {
    pub querier: Querier,
}

#[async_trait]
impl RecipeInterface for RecipeImplementation {
    async fn manually_create_or_update_user(
        &self,
        third_party_id: &str,
        third_party_user_id: &str,
        email: &str,
        is_verified: bool,
        _session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        _should_try_linking_with_session_user: Option<bool>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ManuallyCreateOrUpdateUserResult, SuperTokensError> {
        let body = serde_json::json!({
            "thirdPartyId": third_party_id,
            "thirdPartyUserId": third_party_user_id,
            "email": {
                "id": email,
                "isVerified": is_verified,
            },
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/signinup", tenant_id))?;
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
                let created_new_recipe_user = response
                    .get("createdNewRecipeUser")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(ManuallyCreateOrUpdateUserResult::Ok {
                    user: Box::new(user),
                    recipe_user_id,
                    created_new_recipe_user,
                })
            }
            "SIGN_IN_UP_NOT_ALLOWED" => {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Sign in up not allowed")
                    .to_string();
                Ok(ManuallyCreateOrUpdateUserResult::SignInUpNotAllowed { reason })
            }
            "EMAIL_CHANGE_NOT_ALLOWED_ERROR" => {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Email change not allowed")
                    .to_string();
                Ok(ManuallyCreateOrUpdateUserResult::EmailChangeNotAllowed { reason })
            }
            "LINKING_TO_SESSION_USER_FAILED" => {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Linking to session user failed")
                    .to_string();
                Ok(ManuallyCreateOrUpdateUserResult::LinkingToSessionUserFailed { reason })
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from thirdparty signinup: {}",
                status
            ))),
        }
    }

    async fn sign_in_up(
        &self,
        third_party_id: &str,
        third_party_user_id: &str,
        email: &str,
        is_verified: bool,
        oauth_tokens: HashMap<String, Value>,
        raw_user_info_from_provider: RawUserInfoFromProvider,
        session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        should_try_linking_with_session_user: Option<bool>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<SignInUpResult, SuperTokensError> {
        let result = self
            .manually_create_or_update_user(
                third_party_id,
                third_party_user_id,
                email,
                is_verified,
                session,
                should_try_linking_with_session_user,
                tenant_id,
                user_context,
            )
            .await?;

        match result {
            ManuallyCreateOrUpdateUserResult::Ok {
                user,
                recipe_user_id,
                created_new_recipe_user,
            } => Ok(SignInUpResult::Ok(SignInUpOkResult {
                user,
                recipe_user_id,
                created_new_recipe_user,
                oauth_tokens,
                raw_user_info_from_provider,
            })),
            ManuallyCreateOrUpdateUserResult::SignInUpNotAllowed { reason } => {
                Ok(SignInUpResult::NotAllowed { reason })
            }
            ManuallyCreateOrUpdateUserResult::EmailChangeNotAllowed { reason } => {
                Ok(SignInUpResult::NotAllowed { reason })
            }
            ManuallyCreateOrUpdateUserResult::LinkingToSessionUserFailed { reason } => {
                Ok(SignInUpResult::LinkingToSessionUserFailed { reason })
            }
        }
    }

    async fn get_provider(
        &self,
        _third_party_id: &str,
        _client_type: Option<&str>,
        _tenant_id: &str,
        _user_context: &mut UserContext,
    ) -> Result<Option<()>, SuperTokensError> {
        Ok(None)
    }
}
