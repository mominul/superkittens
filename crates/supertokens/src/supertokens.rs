use std::sync::{Arc, OnceLock};

use crate::constants::{FDI_KEY_HEADER, RID_KEY_HEADER};
use crate::error::{raise_bad_input_exception, raise_general_exception, SuperTokensError};
use crate::framework::request::BaseRequest;
use crate::framework::response::BaseResponse;
use crate::normalised_url_domain::NormalisedURLDomain;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::recipe_module::RecipeModule;
use crate::types::config::{AppInfo, Host, InputAppInfo, SupertokensConfig};
use crate::user_context::UserContext;
use crate::utils;

static INSTANCE: OnceLock<Arc<Supertokens>> = OnceLock::new();

/// The main SuperTokens SDK singleton.
///
/// Manages recipe modules, middleware routing, CORS headers, and Core communication.
pub struct Supertokens {
    pub app_info: AppInfo,
    pub supertokens_config: SupertokensConfig,
    pub recipe_modules: Vec<Arc<dyn RecipeModule>>,
    pub telemetry: bool,
}

/// Configuration for initializing the SDK.
pub struct SupertokensInit {
    pub app_info: InputAppInfo,
    pub supertokens_config: SupertokensConfig,
    pub recipe_list: Vec<Arc<dyn RecipeModule>>,
    pub telemetry: Option<bool>,
    pub debug: bool,
}

impl Supertokens {
    /// Initialize the SuperTokens SDK singleton.
    ///
    /// Must be called exactly once before any SDK operations.
    pub fn init(config: SupertokensInit) -> Result<(), SuperTokensError> {
        INSTANCE
            .set(Arc::new(Self::create(config)?))
            .map_err(|_| raise_general_exception("SuperTokens has already been initialized"))
    }

    fn create(config: SupertokensInit) -> Result<Self, SuperTokensError> {
        if config.debug {
            crate::logger::enable_debug_logging();
        }

        let app_info = AppInfo::from_input(&config.app_info)?;

        // Parse connection URIs (semicolon-separated)
        let hosts = parse_connection_uri(&config.supertokens_config.connection_uri)?;

        // Initialize the Querier
        Querier::init(
            hosts,
            config.supertokens_config.api_key.clone(),
            config.supertokens_config.network_interceptor.clone(),
            config.supertokens_config.disable_core_call_cache,
        );

        let telemetry = config.telemetry.unwrap_or(true);

        crate::st_log_info!(
            app_name = %app_info.app_name,
            api_domain = %app_info.api_domain,
            "SuperTokens initialized"
        );

        Ok(Self {
            app_info,
            supertokens_config: config.supertokens_config,
            recipe_modules: config.recipe_list,
            telemetry,
        })
    }

    /// Get the singleton instance.
    pub fn get_instance() -> Result<&'static Arc<Supertokens>, SuperTokensError> {
        INSTANCE.get().ok_or_else(|| {
            raise_general_exception("SuperTokens not initialized. Call Supertokens::init() first.")
        })
    }

    /// Collect all CORS headers needed by registered recipes.
    pub fn get_all_cors_headers(&self) -> Vec<String> {
        let mut headers = vec![RID_KEY_HEADER.to_string(), FDI_KEY_HEADER.to_string()];
        for recipe in &self.recipe_modules {
            for h in recipe.get_all_cors_headers() {
                if !headers.contains(&h) {
                    headers.push(h);
                }
            }
        }
        headers
    }

    /// Main middleware: route an incoming request to the appropriate recipe handler.
    ///
    /// Returns `Ok(true)` if the request was handled, `Ok(false)` if no recipe matched.
    pub async fn middleware(
        &self,
        request: &dyn BaseRequest,
        response: &mut dyn BaseResponse,
        user_context: &mut UserContext,
    ) -> Result<bool, SuperTokensError> {
        let request_path = request.get_path();
        let path = NormalisedURLPath::new(&format!(
            "{}{}",
            self.app_info.api_gateway_path.get_as_string_dangerous(),
            request_path
        ))?;

        // Check if path starts with api_base_path
        if !path.startswith(&self.app_info.api_base_path) {
            return Ok(false);
        }

        let method = utils::normalise_http_method(&request.method());

        // Step 1: Try routing by Recipe ID (RID) header
        let rid = utils::get_rid_from_header(request);
        let rid = rid.as_deref().filter(|r| *r != "anti-csrf");

        if let Some(rid) = rid {
            // Find recipes matching this RID
            let matching_recipes: Vec<&Arc<dyn RecipeModule>> = self
                .recipe_modules
                .iter()
                .filter(|r| {
                    let recipe_id = r.get_recipe_id();
                    recipe_id == rid || self.rid_matches_combined(rid, recipe_id)
                })
                .collect();

            if !matching_recipes.is_empty() {
                // Try to find a handler in matching recipes
                if let Some(handled) = self
                    .try_handle_with_recipes(
                        &matching_recipes,
                        &path,
                        &method,
                        request,
                        response,
                        user_context,
                    )
                    .await?
                {
                    return Ok(handled);
                }
            }
        }

        // Step 2: Fallback — try all recipes without RID filter
        let all_recipes: Vec<&Arc<dyn RecipeModule>> = self.recipe_modules.iter().collect();
        if let Some(handled) = self
            .try_handle_with_recipes(
                &all_recipes,
                &path,
                &method,
                request,
                response,
                user_context,
            )
            .await?
        {
            return Ok(handled);
        }

        Ok(false)
    }

    /// Handle a SuperTokens error by delegating to the appropriate recipe.
    pub async fn handle_supertokens_error(
        &self,
        request: &dyn BaseRequest,
        err: SuperTokensError,
        response: &mut dyn BaseResponse,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError> {
        crate::st_log_error!(error = %err, "SuperTokens error");

        match &err {
            SuperTokensError::General { .. } => return Err(err),
            SuperTokensError::BadInput { message } => {
                utils::send_non_200_response_with_message(message, 400, response);
                return Ok(());
            }
            SuperTokensError::Plugin { message } => {
                utils::send_non_200_response_with_message(message, 400, response);
                return Ok(());
            }
            _ => {}
        }

        // Check each recipe for error ownership
        for recipe in &self.recipe_modules {
            if recipe.is_error_from_this_recipe_based_on_instance(&err) {
                return recipe
                    .handle_error(request, err, response, user_context)
                    .await;
            }
        }

        Err(err)
    }

    /// Get the user count from the Core.
    pub async fn get_user_count(
        &self,
        include_recipe_ids: Option<&[&str]>,
        tenant_id: Option<&str>,
        user_context: &mut UserContext,
    ) -> Result<u64, SuperTokensError> {
        let querier = Querier::get_instance(None)?;
        let tid = tenant_id.unwrap_or("public");
        let path = NormalisedURLPath::new(&format!("/{}/recipe/user/count", tid))?;

        let mut params = std::collections::HashMap::new();
        if let Some(ids) = include_recipe_ids {
            params.insert("includeRecipeIds".to_string(), ids.join(","));
        }
        if tenant_id.is_none() {
            params.insert("includeAllTenants".to_string(), "true".to_string());
        }

        let response = querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        response
            .get("count")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| raise_general_exception("Invalid response from Core: missing count"))
    }

    /// Check if a recipe is initialized.
    pub fn is_recipe_initialized(&self, recipe_id: &str) -> bool {
        self.recipe_modules
            .iter()
            .any(|r| r.get_recipe_id() == recipe_id)
    }

    // ---------- Private helpers ----------

    /// Check if a combined RID (like "thirdpartyemailpassword") matches a recipe.
    fn rid_matches_combined(&self, rid: &str, recipe_id: &str) -> bool {
        match rid {
            "thirdpartyemailpassword" => recipe_id == "thirdparty" || recipe_id == "emailpassword",
            "thirdpartypasswordless" => recipe_id == "thirdparty" || recipe_id == "passwordless",
            _ => false,
        }
    }

    /// Try to handle a request with a list of candidate recipes.
    async fn try_handle_with_recipes(
        &self,
        recipes: &[&Arc<dyn RecipeModule>],
        path: &NormalisedURLPath,
        method: &str,
        request: &dyn BaseRequest,
        response: &mut dyn BaseResponse,
        user_context: &mut UserContext,
    ) -> Result<Option<bool>, SuperTokensError> {
        let mut matched_recipe: Option<(&Arc<dyn RecipeModule>, String, String)> = None;

        for recipe in recipes {
            if let Some(api_id_with_tenant) = recipe
                .return_api_id_if_can_handle_request(path, method, user_context)
                .await?
            {
                if matched_recipe.is_some() {
                    return Err(raise_bad_input_exception(
                        "Multiple recipes can handle the same API request. This should not happen.",
                    ));
                }
                matched_recipe = Some((
                    recipe,
                    api_id_with_tenant.api_id,
                    api_id_with_tenant.tenant_id,
                ));
            }
        }

        if let Some((recipe, api_id, tenant_id)) = matched_recipe {
            recipe
                .handle_api_request(
                    &api_id,
                    &tenant_id,
                    request,
                    path,
                    method,
                    response,
                    user_context,
                )
                .await?;
            return Ok(Some(true));
        }

        Ok(None)
    }
}

/// Parse a semicolon-separated connection URI into a list of Hosts.
fn parse_connection_uri(uri: &str) -> Result<Vec<Host>, SuperTokensError> {
    let mut hosts = Vec::new();
    for part in uri.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let domain = NormalisedURLDomain::new(part)?;
        let base_path = NormalisedURLPath::new(part)?;
        hosts.push(Host { domain, base_path });
    }
    if hosts.is_empty() {
        return Err(raise_general_exception(
            "Please provide at least one SuperTokens Core connection URI",
        ));
    }
    Ok(hosts)
}

/// Helper: extract the BaseRequest from user_context (if set by middleware).
pub fn get_request_from_user_context(user_context: &UserContext) -> Option<Arc<dyn BaseRequest>> {
    user_context
        .get::<Arc<dyn BaseRequest>>(crate::user_context::internal_keys::REQUEST)
        .cloned()
}

/// Helper: check if a recipe is initialized (static convenience).
pub fn is_recipe_initialized(recipe_id: &str) -> Result<bool, SuperTokensError> {
    Ok(Supertokens::get_instance()?.is_recipe_initialized(recipe_id))
}
