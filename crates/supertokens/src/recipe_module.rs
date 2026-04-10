use async_trait::async_trait;
use regex::Regex;
use std::sync::LazyLock;

use crate::error::SuperTokensError;
use crate::framework::request::BaseRequest;
use crate::framework::response::BaseResponse;
use crate::normalised_url_path::NormalisedURLPath;
use crate::types::config::AppInfo;
use crate::user_context::UserContext;

/// HTTP methods supported by API handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Options,
    Trace,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Delete => "delete",
            Self::Options => "options",
            Self::Trace => "trace",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "get" => Some(Self::Get),
            "post" => Some(Self::Post),
            "put" => Some(Self::Put),
            "delete" => Some(Self::Delete),
            "options" => Some(Self::Options),
            "trace" => Some(Self::Trace),
            _ => None,
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Describes an API endpoint that a recipe can handle.
#[derive(Debug, Clone)]
pub struct APIHandled {
    pub path_without_api_base_path: NormalisedURLPath,
    pub method: HttpMethod,
    pub request_id: String,
    pub disabled: bool,
}

/// Result of route matching — the API id and resolved tenant id.
#[derive(Debug, Clone)]
pub struct ApiIdWithTenantId {
    pub api_id: String,
    pub tenant_id: String,
}

/// Async callback to resolve the tenant ID from the URL path.
pub type GetTenantIdFn = std::sync::Arc<
    dyn Fn(
            String,
            UserContext,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = String> + Send>>
        + Send
        + Sync,
>;

/// Global tenant ID resolver (set once during init).
#[cfg(not(feature = "testing"))]
static GET_TENANT_ID: std::sync::OnceLock<GetTenantIdFn> = std::sync::OnceLock::new();

#[cfg(feature = "testing")]
static GET_TENANT_ID: std::sync::RwLock<Option<GetTenantIdFn>> = std::sync::RwLock::new(None);

#[cfg(not(feature = "testing"))]
pub fn set_get_tenant_id(f: GetTenantIdFn) {
    let _ = GET_TENANT_ID.set(f);
}

#[cfg(feature = "testing")]
pub fn set_get_tenant_id(f: GetTenantIdFn) {
    let mut guard = GET_TENANT_ID
        .write()
        .expect("Failed to acquire write lock on GET_TENANT_ID");
    *guard = Some(f);
}

/// Reset the tenant ID resolver (testing only).
#[cfg(feature = "testing")]
pub fn reset_get_tenant_id() {
    let mut guard = GET_TENANT_ID
        .write()
        .expect("Failed to acquire write lock on GET_TENANT_ID");
    *guard = None;
}

/// The base trait for all recipe modules.
///
/// Each recipe (session, emailpassword, etc.) implements this trait to register
/// its API endpoints, handle requests, and manage errors.
#[async_trait]
pub trait RecipeModule: Send + Sync {
    fn get_recipe_id(&self) -> &str;

    fn get_app_info(&self) -> &AppInfo;

    /// Return the list of API endpoints this recipe handles.
    fn get_apis_handled(&self) -> Vec<APIHandled>;

    /// Check if a given error originated from this recipe.
    fn is_error_from_this_recipe_based_on_instance(&self, err: &SuperTokensError) -> bool;

    /// Handle an API request routed to this recipe.
    #[allow(clippy::too_many_arguments)]
    async fn handle_api_request(
        &self,
        request_id: &str,
        tenant_id: &str,
        request: &dyn BaseRequest,
        path: &NormalisedURLPath,
        method: &str,
        response: &mut dyn BaseResponse,
        user_context: &mut UserContext,
    ) -> Result<Option<()>, SuperTokensError>;

    /// Handle a SuperTokens error from this recipe.
    async fn handle_error(
        &self,
        request: &dyn BaseRequest,
        err: SuperTokensError,
        response: &mut dyn BaseResponse,
        user_context: &mut UserContext,
    ) -> Result<(), SuperTokensError>;

    /// Return additional CORS headers needed by this recipe.
    fn get_all_cors_headers(&self) -> Vec<String>;

    /// Try to match an incoming request to one of this recipe's APIs.
    ///
    /// Returns `Some(ApiIdWithTenantId)` if this recipe can handle the request.
    async fn return_api_id_if_can_handle_request(
        &self,
        path: &NormalisedURLPath,
        method: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<ApiIdWithTenantId>, SuperTokensError> {
        let apis_handled = self.get_apis_handled();
        let base_path_str = self.get_app_info().api_base_path.get_as_string_dangerous();

        // Regex: ^{base_path}(?:/([a-zA-Z0-9-]+))?(/.*)$
        static PATTERN_CACHE: LazyLock<std::sync::Mutex<std::collections::HashMap<String, Regex>>> =
            LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

        let pattern = {
            let mut cache = PATTERN_CACHE.lock().unwrap();
            cache
                .entry(base_path_str.to_string())
                .or_insert_with(|| {
                    let escaped = regex::escape(base_path_str);
                    Regex::new(&format!(r"^{}(?:/([a-zA-Z0-9-]+))?(/.*)$", escaped)).unwrap()
                })
                .clone()
        };

        let path_str = path.get_as_string_dangerous();
        let captures = match pattern.captures(path_str) {
            Some(c) => c,
            None => return Ok(None),
        };

        let tenant_id_from_url = captures.get(1).map(|m| m.as_str().to_string());
        let remaining_path = captures.get(2).map(|m| m.as_str()).unwrap_or("");

        // Resolve tenant_id
        let tenant_id = if let Some(ref tid) = tenant_id_from_url {
            #[cfg(not(feature = "testing"))]
            let get_tid_fn = GET_TENANT_ID.get().cloned();
            #[cfg(feature = "testing")]
            let get_tid_fn = GET_TENANT_ID.read().ok().and_then(|g| g.clone());

            if let Some(get_tid) = get_tid_fn {
                get_tid(tid.clone(), user_context.clone()).await
            } else {
                tid.clone()
            }
        } else {
            "public".to_string()
        };

        let remaining_normalised = NormalisedURLPath::new(remaining_path)?;

        for api in &apis_handled {
            if api.disabled {
                continue;
            }
            if api.path_without_api_base_path.equals(&remaining_normalised)
                && api.method.as_str() == method.to_lowercase()
            {
                return Ok(Some(ApiIdWithTenantId {
                    api_id: api.request_id.clone(),
                    tenant_id,
                }));
            }
        }

        Ok(None)
    }
}
