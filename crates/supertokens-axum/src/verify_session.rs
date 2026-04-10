use axum::body::Body;
use axum::response::IntoResponse;
use http::{Request, StatusCode};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::request::AxumRequest;
use crate::response::AxumResponse;
use supertokens::recipe::session::interfaces::SessionContainerInterface;
use supertokens::user_context::UserContext;
use supertokens::Supertokens;

/// Key used to store the session in request extensions.
#[derive(Clone)]
pub struct SessionExtension(pub Arc<dyn SessionContainerInterface>);

/// Configuration for session verification middleware.
#[derive(Clone)]
pub struct VerifySessionConfig {
    /// If true, requests without a valid session get a 401 response.
    /// If false, the session is optional and the handler runs regardless.
    pub session_required: bool,

    /// Whether to check anti-CSRF tokens.
    pub anti_csrf_check: Option<bool>,

    /// Whether to verify the session against the database.
    pub check_database: bool,
}

impl Default for VerifySessionConfig {
    fn default() -> Self {
        Self {
            session_required: true,
            anti_csrf_check: None,
            check_database: false,
        }
    }
}

/// Tower layer that verifies a SuperTokens session before passing the request to the inner service.
///
/// Stores the verified session in request extensions, accessible via `SessionExtension`.
///
/// # Example
/// ```ignore
/// use supertokens_axum::verify_session::{VerifySessionLayer, SessionExtension};
///
/// let app = Router::new()
///     .route("/protected", get(handler))
///     .layer(VerifySessionLayer::new());
///
/// async fn handler(req: Request<Body>) -> impl IntoResponse {
///     let session = req.extensions().get::<SessionExtension>().unwrap();
///     format!("Hello, {}!", session.0.get_user_id())
/// }
/// ```
#[derive(Clone)]
pub struct VerifySessionLayer {
    config: VerifySessionConfig,
}

impl VerifySessionLayer {
    /// Create a new layer that requires a valid session.
    pub fn new() -> Self {
        Self {
            config: VerifySessionConfig::default(),
        }
    }

    /// Create a layer with custom session verification config.
    pub fn with_config(config: VerifySessionConfig) -> Self {
        Self { config }
    }

    /// Create a layer where the session is optional (handler runs even without session).
    pub fn optional() -> Self {
        Self {
            config: VerifySessionConfig {
                session_required: false,
                ..Default::default()
            },
        }
    }
}

impl Default for VerifySessionLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for VerifySessionLayer {
    type Service = VerifySessionMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        VerifySessionMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

/// The session verification middleware service.
#[derive(Clone)]
pub struct VerifySessionMiddleware<S> {
    inner: S,
    config: VerifySessionConfig,
}

impl<S> Service<Request<Body>> for VerifySessionMiddleware<S>
where
    S: Service<Request<Body>, Response = axum::response::Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    type Response = axum::response::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        std::mem::swap(&mut self.inner, &mut inner);
        let config = self.config.clone();

        Box::pin(async move {
            let st = match Supertokens::get_instance() {
                Ok(st) => st,
                Err(_) => {
                    return Ok((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "SuperTokens not initialized",
                    )
                        .into_response());
                }
            };

            // Find the session recipe
            let session_recipe = st
                .recipe_modules
                .iter()
                .find(|r| r.get_recipe_id() == "session");
            if session_recipe.is_none() {
                return Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Session recipe not initialized",
                )
                    .into_response());
            }

            // Extract the access token and anti-CSRF token from the request
            let (parts, body) = req.into_parts();

            let access_token = get_access_token_from_parts(&parts);
            let anti_csrf = parts
                .headers
                .get(supertokens::recipe::session::constants::ANTI_CSRF_HEADER_KEY)
                .and_then(|v| v.to_str().ok())
                .map(String::from);

            // Get the session recipe implementation
            let querier =
                match supertokens::querier::Querier::get_instance(Some("session".to_string())) {
                    Ok(q) => q,
                    Err(_) => {
                        return Ok(
                            (StatusCode::INTERNAL_SERVER_ERROR, "Querier not initialized")
                                .into_response(),
                        );
                    }
                };

            let session_config =
                match supertokens::recipe::session::utils::validate_and_normalise_user_input(
                    &st.app_info,
                    supertokens::recipe::session::types::SessionConfig::default(),
                ) {
                    Ok(c) => c,
                    Err(_) => {
                        return Ok((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to initialize session config",
                        )
                            .into_response());
                    }
                };

            let recipe_impl = Arc::new(
                supertokens::recipe::session::recipe_implementation::RecipeImplementationImpl {
                    querier,
                    config: session_config.clone(),
                    app_info: st.app_info.clone(),
                },
            );

            let mut user_context = UserContext::new();

            let session_result = recipe_impl
                .get_session(
                    access_token.as_deref(),
                    anti_csrf.as_deref(),
                    config.anti_csrf_check,
                    Some(config.session_required),
                    Some(config.check_database),
                    &mut user_context,
                )
                .await;

            match session_result {
                Ok(Some(session)) => {
                    // Session verified — store in extensions and forward
                    let mut req = Request::from_parts(parts, body);
                    req.extensions_mut()
                        .insert(SessionExtension(session.clone()));
                    inner.call(req).await
                }
                Ok(None) => {
                    if config.session_required {
                        // Should not happen since session_required=true should return error
                        Ok((
                            StatusCode::UNAUTHORIZED,
                            "{\"message\":\"Session not found\"}",
                        )
                            .into_response())
                    } else {
                        // Optional session — proceed without session
                        let req = Request::from_parts(parts, body);
                        inner.call(req).await
                    }
                }
                Err(err) => {
                    // Try to handle the error through SuperTokens error handler
                    let axum_req = AxumRequest::new(
                        http::Uri::default(),
                        http::Method::GET,
                        http::HeaderMap::new(),
                        bytes::Bytes::new(),
                    );
                    let mut axum_resp = AxumResponse::new();
                    match st
                        .handle_supertokens_error(&axum_req, err, &mut axum_resp, &mut user_context)
                        .await
                    {
                        Ok(()) => Ok(axum_resp.into_axum_response()),
                        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
                    }
                }
            }
        })
    }
}

/// Extract access token from request parts (cookies or Authorization header).
fn get_access_token_from_parts(parts: &http::request::Parts) -> Option<String> {
    // Try cookies first
    if let Some(cookie_header) = parts.headers.get(http::header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let mut kv = cookie.trim().splitn(2, '=');
                if let (Some(name), Some(value)) = (kv.next(), kv.next()) {
                    if name.trim()
                        == supertokens::recipe::session::constants::ACCESS_TOKEN_COOKIE_KEY
                    {
                        return Some(
                            urlencoding::decode(value.trim())
                                .unwrap_or_default()
                                .into_owned(),
                        );
                    }
                }
            }
        }
    }

    // Fall back to Authorization header
    let auth_header = parts.headers.get(http::header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;
    if auth_str.to_lowercase().starts_with("bearer ") {
        Some(auth_str[7..].to_string())
    } else {
        None
    }
}

use supertokens::recipe::session::interfaces::RecipeInterface;
