use async_trait::async_trait;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::user_context::UserContext;

/// A function that checks whether a user has set up a particular factor.
/// Takes `(user_id)` and returns a list of factor IDs the user has set up (e.g. `["totp"]`).
pub type GetFactorsSetupForUserFn = Arc<
    dyn Fn(
            String,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, SuperTokensError>> + Send>>
        + Send
        + Sync,
>;

/// Global registry of factor-setup check functions.
/// Populated by other recipes (e.g. TOTP) via PostSTInitCallbacks.
static FACTOR_SETUP_FUNCS: Mutex<Vec<GetFactorsSetupForUserFn>> = Mutex::new(Vec::new());

/// Register a function that checks if a user has set up a factor.
/// Called by recipes like TOTP during post-init.
pub fn add_func_to_get_factors_setup_for_user(func: GetFactorsSetupForUserFn) {
    if let Ok(mut funcs) = FACTOR_SETUP_FUNCS.lock() {
        funcs.push(func);
    }
}

/// Clear all registered factor-setup functions (testing only).
#[cfg(feature = "testing")]
pub fn reset_factor_setup_funcs() {
    if let Ok(mut funcs) = FACTOR_SETUP_FUNCS.lock() {
        funcs.clear();
    }
}

/// Default implementation of the MultiFactorAuth RecipeInterface.
///
/// Required secondary factors are stored in user metadata under
/// `_supertokens.requiredSecondaryFactors`, matching the Python SDK.
pub struct RecipeImplementationImpl {
    pub querier: Querier,
}

impl RecipeImplementationImpl {
    /// Helper: read user metadata from Core.
    async fn get_user_metadata(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<serde_json::Map<String, serde_json::Value>, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("userId".to_string(), user_id.to_string());

        let path = NormalisedURLPath::new("/recipe/user/metadata")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        Ok(response
            .get("metadata")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default())
    }

    /// Helper: update user metadata in Core.
    async fn update_user_metadata(
        &self,
        user_id: &str,
        metadata_update: &serde_json::Map<String, serde_json::Value>,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "metadataUpdate": metadata_update,
        });

        let path = NormalisedURLPath::new("/recipe/user/metadata")?;
        self.querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        Ok(())
    }

    /// Helper: read the `_supertokens.requiredSecondaryFactors` array from metadata.
    fn extract_required_factors(
        metadata: &serde_json::Map<String, serde_json::Value>,
    ) -> Vec<String> {
        metadata
            .get("_supertokens")
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("requiredSecondaryFactors"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn get_factors_setup_for_user(
        &self,
        user_id: &str,
        _user_context: &mut UserContext,
    ) -> Result<GetFactorsSetupForUserOkResult, SuperTokensError> {
        let funcs = {
            let guard = FACTOR_SETUP_FUNCS
                .lock()
                .map_err(|_| crate::error::raise_general_exception("factor setup funcs lock poisoned"))?;
            guard.clone()
        };

        let mut factor_ids: Vec<String> = Vec::new();
        for func in funcs {
            let result = func(user_id.to_string()).await?;
            for factor_id in result {
                if !factor_ids.contains(&factor_id) {
                    factor_ids.push(factor_id);
                }
            }
        }

        Ok(GetFactorsSetupForUserOkResult { factor_ids })
    }

    async fn get_required_secondary_factors_for_user(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<GetRequiredSecondaryFactorsOkResult, SuperTokensError> {
        let metadata = self.get_user_metadata(user_id, user_context).await?;
        let factor_ids = Self::extract_required_factors(&metadata);
        Ok(GetRequiredSecondaryFactorsOkResult { factor_ids })
    }

    async fn add_to_required_secondary_factors_for_user(
        &self,
        user_id: &str,
        factor_id: &str,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        let metadata = self.get_user_metadata(user_id, user_context).await?;
        let mut factors = Self::extract_required_factors(&metadata);

        if !factors.contains(&factor_id.to_string()) {
            factors.push(factor_id.to_string());

            let mut update = serde_json::Map::new();
            let mut st_obj = serde_json::Map::new();
            st_obj.insert(
                "requiredSecondaryFactors".to_string(),
                serde_json::json!(factors),
            );
            update.insert("_supertokens".to_string(), serde_json::json!(st_obj));

            self.update_user_metadata(user_id, &update, user_context)
                .await?;
        }

        Ok(())
    }

    async fn remove_from_required_secondary_factors_for_user(
        &self,
        user_id: &str,
        factor_id: &str,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        let metadata = self.get_user_metadata(user_id, user_context).await?;
        let factors = Self::extract_required_factors(&metadata);

        if factors.contains(&factor_id.to_string()) {
            let new_factors: Vec<String> = factors
                .into_iter()
                .filter(|f| f != factor_id)
                .collect();

            let mut update = serde_json::Map::new();
            let mut st_obj = serde_json::Map::new();
            st_obj.insert(
                "requiredSecondaryFactors".to_string(),
                serde_json::json!(new_factors),
            );
            update.insert("_supertokens".to_string(), serde_json::json!(st_obj));

            self.update_user_metadata(user_id, &update, user_context)
                .await?;
        }

        Ok(())
    }

    async fn mark_factor_as_complete_in_session(
        &self,
        _session_handle: &str,
        _factor_id: &str,
        _user_context: &mut UserContext,
    ) -> Result<MarkFactorAsCompleteOkResult, SuperTokensError> {
        // In a full implementation this updates session claims
        Ok(MarkFactorAsCompleteOkResult)
    }
}
