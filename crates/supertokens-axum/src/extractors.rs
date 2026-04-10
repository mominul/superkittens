use axum::extract::FromRequestParts;
use axum::response::IntoResponse;
use http::request::Parts;
use std::sync::Arc;

use supertokens::recipe::session::interfaces::{RecipeInterface, SessionContainerInterface};
use supertokens::user_context::UserContext;
use supertokens::Supertokens;

use crate::verify_session::SessionExtension;

/// Axum extractor that requires a valid session.
///
/// If the `VerifySessionLayer` middleware is active, this extractor uses the
/// already-verified session from request extensions. Otherwise, it performs
/// session verification inline.
///
/// Returns 401 if no valid session is found.
///
/// # Example
/// ```compile_fail
/// use supertokens_axum::extractors::Session;
///
/// async fn handler(session: Session) -> impl IntoResponse {
///     let user_id = session.get_user_id();
///     format!("Hello, {}!", user_id)
/// }
/// ```
pub struct Session(pub Arc<dyn SessionContainerInterface>);

impl Session {
    pub fn get_user_id(&self) -> &str {
        self.0.get_user_id()
    }

    pub fn get_handle(&self) -> &str {
        self.0.get_handle()
    }

    pub fn get_tenant_id(&self) -> &str {
        self.0.get_tenant_id()
    }

    pub fn inner(&self) -> &Arc<dyn SessionContainerInterface> {
        &self.0
    }
}

impl std::ops::Deref for Session {
    type Target = dyn SessionContainerInterface;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// Error returned when session extraction fails.
pub struct SessionError(pub supertokens::SuperTokensError);

impl IntoResponse for SessionError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self.0 {
            supertokens::SuperTokensError::Session(
                supertokens::error::SessionError::Unauthorized { .. },
            ) => http::StatusCode::UNAUTHORIZED,
            supertokens::SuperTokensError::Session(
                supertokens::error::SessionError::TryRefreshToken { .. },
            ) => http::StatusCode::UNAUTHORIZED,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = serde_json::json!({
            "message": self.0.to_string(),
        });

        (status, axum::Json(body)).into_response()
    }
}

impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
{
    type Rejection = SessionError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // First, check if VerifySessionLayer already verified the session
        if let Some(ext) = parts.extensions.get::<SessionExtension>() {
            return Ok(Session(ext.0.clone()));
        }

        // Otherwise, perform inline session verification
        let st = Supertokens::get_instance().map_err(SessionError)?;

        let access_token = get_access_token_from_parts(parts);
        let anti_csrf = parts
            .headers
            .get(supertokens::recipe::session::constants::ANTI_CSRF_HEADER_KEY)
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let querier = supertokens::querier::Querier::get_instance(Some("session".into()))
            .map_err(SessionError)?;

        let config = supertokens::recipe::session::utils::validate_and_normalise_user_input(
            &st.app_info,
            supertokens::recipe::session::types::SessionConfig::default(),
        )
        .map_err(SessionError)?;

        let recipe_impl = Arc::new(
            supertokens::recipe::session::recipe_implementation::RecipeImplementationImpl {
                querier,
                config,
                app_info: st.app_info.clone(),
            },
        );

        let mut user_context = UserContext::new();

        let session = recipe_impl
            .get_session(
                access_token.as_deref(),
                anti_csrf.as_deref(),
                None,
                Some(true),
                None,
                &mut user_context,
            )
            .await
            .map_err(SessionError)?;

        match session {
            Some(s) => Ok(Session(s)),
            None => Err(SessionError(supertokens::error::SuperTokensError::Session(
                supertokens::error::SessionError::Unauthorized {
                    message: "Session not found".to_string(),
                },
            ))),
        }
    }
}

/// Axum extractor that optionally extracts a session.
///
/// Returns `None` if no valid session is found (does not return an error).
pub struct OptionalSession(pub Option<Arc<dyn SessionContainerInterface>>);

impl<S> FromRequestParts<S> for OptionalSession
where
    S: Send + Sync,
{
    type Rejection = SessionError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Session::from_request_parts(parts, state).await {
            Ok(session) => Ok(OptionalSession(Some(session.0))),
            Err(_) => Ok(OptionalSession(None)),
        }
    }
}

/// Extract access token from request parts (cookies or Authorization header).
fn get_access_token_from_parts(parts: &Parts) -> Option<String> {
    // Try cookie first
    let cookie_header = parts.headers.get(http::header::COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;

    for cookie in cookie_str.split(';') {
        let mut kv = cookie.trim().splitn(2, '=');
        let name = kv.next()?.trim();
        let value = kv.next()?.trim();
        if name == supertokens::recipe::session::constants::ACCESS_TOKEN_COOKIE_KEY {
            return Some(urlencoding::decode(value).unwrap_or_default().into_owned());
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
