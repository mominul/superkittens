use async_trait::async_trait;
use std::collections::HashMap;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::user_context::UserContext;

pub struct RecipeImplementationImpl {
    pub querier: Querier,
}

impl RecipeImplementationImpl {
    fn parse_error_response(response: &serde_json::Value) -> ErrorOAuth2Response {
        ErrorOAuth2Response {
            error: response
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown_error")
                .to_string(),
            error_description: response
                .get("errorDescription")
                .or_else(|| response.get("error_description"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            status_code: response
                .get("statusCode")
                .and_then(|v| v.as_u64())
                .unwrap_or(400) as u16,
        }
    }

    fn parse_oauth2_client(value: &serde_json::Value) -> OAuth2Client {
        serde_json::from_value(value.clone()).unwrap_or(OAuth2Client {
            client_id: value
                .get("clientId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            client_name: value
                .get("clientName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            client_secret: value
                .get("clientSecret")
                .and_then(|v| v.as_str())
                .map(String::from),
            scope: value
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            redirect_uris: Vec::new(),
            grant_types: Vec::new(),
            response_types: Vec::new(),
            token_endpoint_auth_method: String::new(),
            client_uri: String::new(),
            logo_uri: String::new(),
            tos_uri: String::new(),
            policy_uri: String::new(),
            metadata: serde_json::Value::Null,
        })
    }
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn get_login_request(
        &self,
        challenge: &str,
        user_context: &mut UserContext,
    ) -> Result<LoginRequest, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("loginChallenge".to_string(), challenge.to_string());

        let path = NormalisedURLPath::new("/recipe/oauth/auth/requests/login")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        Ok(LoginRequest {
            challenge: response
                .get("challenge")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            client: Self::parse_oauth2_client(
                response.get("client").unwrap_or(&serde_json::Value::Null),
            ),
            request_url: response
                .get("requestUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            requested_scope: response
                .get("requestedScope")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            requested_access_token_audience: response
                .get("requestedAccessTokenAudience")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            skip: response
                .get("skip")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            subject: response
                .get("subject")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            oidc_context: response
                .get("oidcContext")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        })
    }

    async fn accept_login_request(
        &self,
        challenge: &str,
        subject: &str,
        remember: bool,
        remember_for: u64,
        user_context: &mut UserContext,
    ) -> Result<LoginConsentResult, SuperTokensError> {
        let body = serde_json::json!({
            "loginChallenge": challenge,
            "subject": subject,
            "remember": remember,
            "rememberFor": remember_for,
        });

        let path = NormalisedURLPath::new("/recipe/oauth/auth/requests/login/accept")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(LoginConsentResult::Redirect(RedirectResponse {
                redirect_to: response
                    .get("redirectTo")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })),
            _ => Ok(LoginConsentResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn reject_login_request(
        &self,
        challenge: &str,
        error: &str,
        error_description: &str,
        user_context: &mut UserContext,
    ) -> Result<LoginConsentResult, SuperTokensError> {
        let body = serde_json::json!({
            "loginChallenge": challenge,
            "error": error,
            "errorDescription": error_description,
        });

        let path = NormalisedURLPath::new("/recipe/oauth/auth/requests/login/reject")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(LoginConsentResult::Redirect(RedirectResponse {
                redirect_to: response
                    .get("redirectTo")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })),
            _ => Ok(LoginConsentResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn get_consent_request(
        &self,
        challenge: &str,
        user_context: &mut UserContext,
    ) -> Result<ConsentRequest, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("consentChallenge".to_string(), challenge.to_string());

        let path = NormalisedURLPath::new("/recipe/oauth/auth/requests/consent")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        Ok(ConsentRequest {
            challenge: response
                .get("challenge")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            client: Self::parse_oauth2_client(
                response.get("client").unwrap_or(&serde_json::Value::Null),
            ),
            request_url: response
                .get("requestUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            requested_scope: response
                .get("requestedScope")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            requested_access_token_audience: response
                .get("requestedAccessTokenAudience")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            skip: response
                .get("skip")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            subject: response
                .get("subject")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    async fn accept_consent_request(
        &self,
        challenge: &str,
        grant_scope: &[String],
        grant_access_token_audience: &[String],
        remember: bool,
        remember_for: u64,
        user_context: &mut UserContext,
    ) -> Result<LoginConsentResult, SuperTokensError> {
        let body = serde_json::json!({
            "consentChallenge": challenge,
            "grantScope": grant_scope,
            "grantAccessTokenAudience": grant_access_token_audience,
            "remember": remember,
            "rememberFor": remember_for,
        });

        let path = NormalisedURLPath::new("/recipe/oauth/auth/requests/consent/accept")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(LoginConsentResult::Redirect(RedirectResponse {
                redirect_to: response
                    .get("redirectTo")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })),
            _ => Ok(LoginConsentResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn reject_consent_request(
        &self,
        challenge: &str,
        error: &str,
        error_description: &str,
        user_context: &mut UserContext,
    ) -> Result<LoginConsentResult, SuperTokensError> {
        let body = serde_json::json!({
            "consentChallenge": challenge,
            "error": error,
            "errorDescription": error_description,
        });

        let path = NormalisedURLPath::new("/recipe/oauth/auth/requests/consent/reject")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(LoginConsentResult::Redirect(RedirectResponse {
                redirect_to: response
                    .get("redirectTo")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })),
            _ => Ok(LoginConsentResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn authorization(
        &self,
        params: serde_json::Value,
        cookies: Option<&str>,
        _session: Option<serde_json::Value>,
        user_context: &mut UserContext,
    ) -> Result<AuthorizationResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "params": params,
        });
        if let Some(c) = cookies {
            body["cookies"] = serde_json::Value::String(c.to_string());
        }

        let path = NormalisedURLPath::new("/recipe/oauth/auth")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(AuthorizationResult::Redirect(RedirectResponse {
                redirect_to: response
                    .get("redirectTo")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })),
            _ => Ok(AuthorizationResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn token_exchange(
        &self,
        authorization_header: Option<&str>,
        body: serde_json::Value,
        user_context: &mut UserContext,
    ) -> Result<TokenExchangeResult, SuperTokensError> {
        let mut request_body = serde_json::json!({
            "inputBody": body,
        });
        if let Some(auth) = authorization_header {
            request_body["authorizationHeader"] = serde_json::Value::String(auth.to_string());
        }

        let path = NormalisedURLPath::new("/recipe/oauth/token")?;
        let response = self
            .querier
            .send_post_request(&path, Some(request_body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(TokenExchangeResult::Ok(TokenInfoResponse {
                access_token: response
                    .get("accessToken")
                    .or_else(|| response.get("access_token"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                expires_in: response
                    .get("expiresIn")
                    .or_else(|| response.get("expires_in"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                id_token: response
                    .get("idToken")
                    .or_else(|| response.get("id_token"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                refresh_token: response
                    .get("refreshToken")
                    .or_else(|| response.get("refresh_token"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                scope: response
                    .get("scope")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                token_type: response
                    .get("tokenType")
                    .or_else(|| response.get("token_type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Bearer")
                    .to_string(),
            })),
            _ => Ok(TokenExchangeResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn get_oauth2_clients(
        &self,
        page_size: Option<u32>,
        pagination_token: Option<&str>,
        client_name: Option<&str>,
        user_context: &mut UserContext,
    ) -> Result<GetOAuth2ClientsOkResult, SuperTokensError> {
        let mut params = HashMap::new();
        if let Some(ps) = page_size {
            params.insert("pageSize".to_string(), ps.to_string());
        }
        if let Some(pt) = pagination_token {
            params.insert("paginationToken".to_string(), pt.to_string());
        }
        if let Some(cn) = client_name {
            params.insert("clientName".to_string(), cn.to_string());
        }

        let path = NormalisedURLPath::new("/recipe/oauth/clients/list")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let clients = response
            .get("clients")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(Self::parse_oauth2_client).collect())
            .unwrap_or_default();

        Ok(GetOAuth2ClientsOkResult { clients })
    }

    async fn get_oauth2_client(
        &self,
        client_id: &str,
        user_context: &mut UserContext,
    ) -> Result<OAuth2ClientResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("clientId".to_string(), client_id.to_string());

        let path = NormalisedURLPath::new("/recipe/oauth/client")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(OAuth2ClientResult::Ok {
                client: Box::new(Self::parse_oauth2_client(&response)),
            }),
            _ => Ok(OAuth2ClientResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn create_oauth2_client(
        &self,
        params: serde_json::Value,
        user_context: &mut UserContext,
    ) -> Result<OAuth2ClientResult, SuperTokensError> {
        let path = NormalisedURLPath::new("/recipe/oauth/client")?;
        let response = self
            .querier
            .send_post_request(&path, Some(params), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(OAuth2ClientResult::Ok {
                client: Box::new(Self::parse_oauth2_client(&response)),
            }),
            _ => Ok(OAuth2ClientResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn update_oauth2_client(
        &self,
        client_id: &str,
        params: serde_json::Value,
        user_context: &mut UserContext,
    ) -> Result<OAuth2ClientResult, SuperTokensError> {
        let mut body = params;
        body["clientId"] = serde_json::Value::String(client_id.to_string());

        let path = NormalisedURLPath::new("/recipe/oauth/client")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(OAuth2ClientResult::Ok {
                client: Box::new(Self::parse_oauth2_client(&response)),
            }),
            _ => Ok(OAuth2ClientResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn delete_oauth2_client(
        &self,
        client_id: &str,
        user_context: &mut UserContext,
    ) -> Result<DeleteOAuth2ClientResult, SuperTokensError> {
        let body = serde_json::json!({
            "clientId": client_id,
        });

        let path = NormalisedURLPath::new("/recipe/oauth/client/remove")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(DeleteOAuth2ClientResult::Ok),
            _ => Ok(DeleteOAuth2ClientResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn revoke_token(
        &self,
        token: &str,
        client_id: &str,
        client_secret: Option<&str>,
        user_context: &mut UserContext,
    ) -> Result<RevokeTokenResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "token": token,
            "clientId": client_id,
        });
        if let Some(secret) = client_secret {
            body["clientSecret"] = serde_json::Value::String(secret.to_string());
        }

        let path = NormalisedURLPath::new("/recipe/oauth/token/revoke")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("OK");

        match status {
            "OK" => Ok(RevokeTokenResult::Ok),
            _ => Ok(RevokeTokenResult::Error(Self::parse_error_response(
                &response,
            ))),
        }
    }

    async fn introspect_token(
        &self,
        token: &str,
        scopes: Option<&[String]>,
        user_context: &mut UserContext,
    ) -> Result<IntrospectTokenResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "token": token,
        });
        if let Some(s) = scopes {
            body["scope"] = serde_json::json!(s);
        }

        let path = NormalisedURLPath::new("/recipe/oauth/introspect")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let active = response
            .get("active")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if active {
            Ok(IntrospectTokenResult::Active(ActiveTokenResponse {
                payload: response,
            }))
        } else {
            Ok(IntrospectTokenResult::Inactive(InactiveTokenResponse))
        }
    }
}
