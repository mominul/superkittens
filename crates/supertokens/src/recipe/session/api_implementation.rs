use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use super::cookie_and_header;
use super::interfaces::{ApiInterface, ApiOptions, SessionContainerInterface};
use super::types::*;
use crate::error::SuperTokensError;
use crate::user_context::UserContext;

/// Default API implementation for session endpoints.
pub struct ApiImplementationImpl;

#[async_trait]
impl ApiInterface for ApiImplementationImpl {
    async fn refresh_post(
        &self,
        api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<Arc<dyn SessionContainerInterface>, SuperTokensError> {
        // Get refresh token from request
        let refresh_token = cookie_and_header::get_token(
            api_options.request,
            TokenType::Refresh,
            TokenTransferMethod::Cookie,
        )
        .or_else(|| {
            cookie_and_header::get_token(
                api_options.request,
                TokenType::Refresh,
                TokenTransferMethod::Header,
            )
        })
        .ok_or_else(|| super::errors::SessionError::unauthorised("No refresh token found"))?;

        let anti_csrf_token = api_options
            .request
            .get_header(super::constants::ANTI_CSRF_HEADER_KEY);

        let session = api_options
            .recipe_implementation
            .refresh_session(
                &refresh_token,
                anti_csrf_token.as_deref(),
                false,
                user_context,
            )
            .await?;

        Ok(session)
    }

    async fn signout_post(
        &self,
        session: &dyn SessionContainerInterface,
        _api_options: &ApiOptions<'_>,
        user_context: &mut UserContext,
    ) -> Result<Value, SuperTokensError> {
        session.revoke_session(user_context).await?;
        Ok(serde_json::json!({ "status": "OK" }))
    }

    async fn verify_session(
        &self,
        api_options: &ApiOptions<'_>,
        anti_csrf_check: Option<bool>,
        session_required: bool,
        check_database: bool,
        user_context: &mut UserContext,
    ) -> Result<Option<Arc<dyn SessionContainerInterface>>, SuperTokensError> {
        let method = api_options.request.method();

        // Skip verification for OPTIONS and TRACE
        if method == "options" || method == "trace" {
            return Ok(None);
        }

        // Get access token from request (try cookie first, then header)
        let access_token = cookie_and_header::get_token(
            api_options.request,
            TokenType::Access,
            TokenTransferMethod::Cookie,
        )
        .or_else(|| {
            cookie_and_header::get_token(
                api_options.request,
                TokenType::Access,
                TokenTransferMethod::Header,
            )
        });

        let anti_csrf_token = api_options
            .request
            .get_header(super::constants::ANTI_CSRF_HEADER_KEY);

        let session = api_options
            .recipe_implementation
            .get_session(
                access_token.as_deref(),
                anti_csrf_token.as_deref(),
                anti_csrf_check,
                Some(session_required),
                Some(check_database),
                user_context,
            )
            .await?;

        Ok(session)
    }
}
