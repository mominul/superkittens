use serde_json::Value;

use super::constants::DEFAULT_TENANT_ID;
use super::errors::SessionError;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::types::user::RecipeUserId;
use crate::user_context::UserContext;

/// Create a new session via the Core API.
pub async fn create_new_session(
    querier: &Querier,
    tenant_id: &str,
    recipe_user_id: &RecipeUserId,
    disable_anti_csrf: bool,
    access_token_payload: Option<Value>,
    session_data_in_database: Option<Value>,
    config: &NormalisedSessionConfig,
    user_context: &mut UserContext,
) -> Result<CreateOrRefreshApiResponse, SuperTokensError> {
    let enable_anti_csrf =
        !disable_anti_csrf && config.anti_csrf_function_or_string == AntiCsrfConfig::ViaToken;

    let body = serde_json::json!({
        "userId": recipe_user_id.get_as_string(),
        "userDataInJWT": access_token_payload.unwrap_or(Value::Object(serde_json::Map::new())),
        "userDataInDatabase": session_data_in_database.unwrap_or(Value::Object(serde_json::Map::new())),
        "useDynamicSigningKey": config.use_dynamic_access_token_signing_key,
        "enableAntiCsrf": enable_anti_csrf,
    });

    let path = NormalisedURLPath::new(&format!("/{}/recipe/session", tenant_id))?;
    let response = querier
        .send_post_request(&path, Some(body), user_context)
        .await?;

    parse_create_or_refresh_response(&response)
}

/// Refresh an existing session via the Core API.
pub async fn refresh_session(
    querier: &Querier,
    refresh_token: &str,
    anti_csrf_token: Option<&str>,
    disable_anti_csrf: bool,
    use_dynamic_signing_key: bool,
    config: &NormalisedSessionConfig,
    _user_context: &mut UserContext,
) -> Result<CreateOrRefreshApiResponse, SessionError> {
    let enable_anti_csrf =
        !disable_anti_csrf && config.anti_csrf_function_or_string == AntiCsrfConfig::ViaToken;

    let mut body = serde_json::json!({
        "refreshToken": refresh_token,
        "enableAntiCsrf": enable_anti_csrf,
        "useDynamicSigningKey": use_dynamic_signing_key,
    });

    if let Some(csrf) = anti_csrf_token {
        body["antiCsrfToken"] = Value::String(csrf.to_string());
    }

    let path = NormalisedURLPath::new("/recipe/session/refresh")
        .map_err(|_| SessionError::try_refresh_token("Invalid path"))?;
    let response = querier
        .send_post_request(&path, Some(body), &mut UserContext::new())
        .await
        .map_err(|e| SessionError::try_refresh_token(format!("Core request failed: {}", e)))?;

    let status = response
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match status {
        "OK" => parse_create_or_refresh_response(&response)
            .map_err(|e| SessionError::try_refresh_token(e.to_string())),
        "UNAUTHORISED" => {
            let msg = response
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Session does not exist");
            Err(SessionError::unauthorised(msg))
        }
        "TOKEN_THEFT_DETECTED" => {
            let session = response.get("session").unwrap_or(&Value::Null);
            let user_id = session
                .get("userId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let recipe_user_id = session
                .get("recipeUserId")
                .and_then(|v| v.as_str())
                .unwrap_or(&user_id)
                .to_string();
            let handle = session
                .get("handle")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            Err(SessionError::token_theft_detected(
                user_id,
                RecipeUserId::new(recipe_user_id),
                handle,
            ))
        }
        _ => Err(SessionError::try_refresh_token(format!(
            "Unexpected status from Core: {}",
            status
        ))),
    }
}

/// Verify a session via the Core API (when local validation is insufficient).
pub async fn verify_session_with_core(
    querier: &Querier,
    access_token: &str,
    anti_csrf_token: Option<&str>,
    do_anti_csrf_check: bool,
    check_database: bool,
    user_context: &mut UserContext,
) -> Result<GetSessionApiResponse, SessionError> {
    let mut body = serde_json::json!({
        "accessToken": access_token,
        "doAntiCsrfCheck": do_anti_csrf_check,
        "enableAntiCsrf": do_anti_csrf_check,
        "checkDatabase": check_database,
    });

    if let Some(csrf) = anti_csrf_token {
        body["antiCsrfToken"] = Value::String(csrf.to_string());
    }

    let path = NormalisedURLPath::new("/recipe/session/verify")
        .map_err(|_| SessionError::try_refresh_token("Invalid path"))?;
    let response = querier
        .send_post_request(&path, Some(body), user_context)
        .await
        .map_err(|e| SessionError::try_refresh_token(format!("Core request failed: {}", e)))?;

    let status = response
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match status {
        "OK" => {
            let session = response.get("session").unwrap_or(&Value::Null);
            let access_token_obj = response.get("accessToken").and_then(|at| {
                Some(AccessTokenObj {
                    token: at.get("token")?.as_str()?.to_string(),
                    expiry: at.get("expiry")?.as_u64()?,
                    created_time: at.get("createdTime")?.as_u64()?,
                })
            });

            Ok(GetSessionApiResponse {
                session: GetSessionApiResponseSession {
                    handle: session
                        .get("handle")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    user_id: session
                        .get("userId")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    recipe_user_id: RecipeUserId::new(
                        session
                            .get("recipeUserId")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default(),
                    ),
                    user_data_in_jwt: session.get("userDataInJWT").cloned().unwrap_or_default(),
                    expiry_time: session
                        .get("expiryTime")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    tenant_id: session
                        .get("tenantId")
                        .and_then(|v| v.as_str())
                        .unwrap_or(DEFAULT_TENANT_ID)
                        .to_string(),
                },
                access_token: access_token_obj,
            })
        }
        "UNAUTHORISED" => {
            let msg = response
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Session does not exist");
            Err(SessionError::unauthorised(msg))
        }
        _ => Err(SessionError::try_refresh_token(format!(
            "Unexpected status from Core verify: {}",
            status
        ))),
    }
}

/// Revoke all sessions for a user.
pub async fn revoke_all_sessions_for_user(
    querier: &Querier,
    user_id: &str,
    revoke_sessions_for_linked_accounts: bool,
    tenant_id: Option<&str>,
    revoke_across_all_tenants: bool,
    user_context: &mut UserContext,
) -> Result<Vec<String>, SuperTokensError> {
    let tid = tenant_id.unwrap_or(DEFAULT_TENANT_ID);

    let mut body = serde_json::json!({
        "userId": user_id,
        "revokeSessionsForLinkedAccounts": revoke_sessions_for_linked_accounts,
    });

    let path = if revoke_across_all_tenants {
        NormalisedURLPath::new("/recipe/session/remove")?
    } else {
        body["revokeAcrossAllTenants"] = Value::Bool(false);
        NormalisedURLPath::new(&format!("/{}/recipe/session/remove", tid))?
    };

    let response = querier
        .send_post_request(&path, Some(body), user_context)
        .await?;

    let handles = response
        .get("sessionHandlesRevoked")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(handles)
}

/// Get all session handles for a user.
pub async fn get_all_session_handles_for_user(
    querier: &Querier,
    user_id: &str,
    fetch_sessions_for_linked_accounts: bool,
    tenant_id: Option<&str>,
    fetch_across_all_tenants: bool,
    user_context: &mut UserContext,
) -> Result<Vec<String>, SuperTokensError> {
    let tid = tenant_id.unwrap_or(DEFAULT_TENANT_ID);

    let mut params = std::collections::HashMap::new();
    params.insert("userId".to_string(), user_id.to_string());
    params.insert(
        "fetchSessionsForAllLinkedAccounts".to_string(),
        fetch_sessions_for_linked_accounts.to_string(),
    );

    let path = if fetch_across_all_tenants {
        params.insert("fetchAcrossAllTenants".to_string(), "true".to_string());
        NormalisedURLPath::new("/recipe/session/user")?
    } else {
        NormalisedURLPath::new(&format!("/{}/recipe/session/user", tid))?
    };

    let response = querier
        .send_get_request(&path, Some(params), user_context)
        .await?;

    let handles = response
        .get("sessionHandles")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(handles)
}

/// Revoke a single session.
pub async fn revoke_session(
    querier: &Querier,
    session_handle: &str,
    user_context: &mut UserContext,
) -> Result<bool, SuperTokensError> {
    let body = serde_json::json!({
        "sessionHandles": [session_handle],
    });
    let path = NormalisedURLPath::new("/recipe/session/remove")?;
    let response = querier
        .send_post_request(&path, Some(body), user_context)
        .await?;

    let revoked = response
        .get("sessionHandlesRevoked")
        .and_then(|v| v.as_array())
        .map(|arr| arr.len() == 1)
        .unwrap_or(false);

    Ok(revoked)
}

/// Revoke multiple sessions.
pub async fn revoke_multiple_sessions(
    querier: &Querier,
    session_handles: &[String],
    user_context: &mut UserContext,
) -> Result<Vec<String>, SuperTokensError> {
    let body = serde_json::json!({
        "sessionHandles": session_handles,
    });
    let path = NormalisedURLPath::new("/recipe/session/remove")?;
    let response = querier
        .send_post_request(&path, Some(body), user_context)
        .await?;

    let revoked = response
        .get("sessionHandlesRevoked")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(revoked)
}

/// Get session information from the Core.
pub async fn get_session_information(
    querier: &Querier,
    session_handle: &str,
    user_context: &mut UserContext,
) -> Result<Option<SessionInformationResult>, SuperTokensError> {
    let mut params = std::collections::HashMap::new();
    params.insert("sessionHandle".to_string(), session_handle.to_string());

    let path = NormalisedURLPath::new("/recipe/session")?;
    let response = querier
        .send_get_request(&path, Some(params), user_context)
        .await?;

    let status = response
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if status != "OK" {
        return Ok(None);
    }

    Ok(Some(SessionInformationResult {
        session_handle: response
            .get("sessionHandle")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        user_id: response
            .get("userId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        recipe_user_id: RecipeUserId::new(
            response
                .get("recipeUserId")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
        ),
        session_data_in_database: response
            .get("userDataInDatabase")
            .cloned()
            .unwrap_or_default(),
        expiry: response.get("expiry").and_then(|v| v.as_u64()).unwrap_or(0),
        custom_claims_in_access_token_payload: response
            .get("userDataInJWT")
            .cloned()
            .unwrap_or_default(),
        time_created: response
            .get("timeCreated")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        tenant_id: response
            .get("tenantId")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_TENANT_ID)
            .to_string(),
    }))
}

/// Update session data in the database.
pub async fn update_session_data_in_database(
    querier: &Querier,
    session_handle: &str,
    new_session_data: Value,
    user_context: &mut UserContext,
) -> Result<bool, SuperTokensError> {
    let body = serde_json::json!({
        "sessionHandle": session_handle,
        "userDataInDatabase": new_session_data,
    });
    let path = NormalisedURLPath::new("/recipe/session/data")?;
    let response = querier
        .send_put_request(&path, Some(body), None, user_context)
        .await?;

    let status = response
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Ok(status != "UNAUTHORISED")
}

/// Update access token payload in the Core.
pub async fn update_access_token_payload(
    querier: &Querier,
    session_handle: &str,
    new_payload: Value,
    user_context: &mut UserContext,
) -> Result<bool, SuperTokensError> {
    let body = serde_json::json!({
        "sessionHandle": session_handle,
        "userDataInJWT": new_payload,
    });
    let path = NormalisedURLPath::new("/recipe/jwt/data")?;
    let response = querier
        .send_put_request(&path, Some(body), None, user_context)
        .await?;

    let status = response
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Ok(status != "UNAUTHORISED")
}

// ---------- Helpers ----------

fn parse_create_or_refresh_response(
    response: &Value,
) -> Result<CreateOrRefreshApiResponse, SuperTokensError> {
    let session = response.get("session").unwrap_or(&Value::Null);
    let at = response.get("accessToken").unwrap_or(&Value::Null);
    let rt = response.get("refreshToken").unwrap_or(&Value::Null);

    let user_id = session
        .get("userId")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let recipe_user_id_str = session
        .get("recipeUserId")
        .and_then(|v| v.as_str())
        .unwrap_or(&user_id);

    Ok(CreateOrRefreshApiResponse {
        session: SessionObj {
            handle: session
                .get("handle")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            user_id: user_id.clone(),
            recipe_user_id: RecipeUserId::new(recipe_user_id_str),
            user_data_in_jwt: session.get("userDataInJWT").cloned().unwrap_or_default(),
            tenant_id: session
                .get("tenantId")
                .and_then(|v| v.as_str())
                .unwrap_or(DEFAULT_TENANT_ID)
                .to_string(),
        },
        access_token: TokenInfo {
            token: at
                .get("token")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            expiry: at.get("expiry").and_then(|v| v.as_u64()).unwrap_or(0),
            created_time: at.get("createdTime").and_then(|v| v.as_u64()).unwrap_or(0),
        },
        refresh_token: TokenInfo {
            token: rt
                .get("token")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            expiry: rt.get("expiry").and_then(|v| v.as_u64()).unwrap_or(0),
            created_time: rt.get("createdTime").and_then(|v| v.as_u64()).unwrap_or(0),
        },
        anti_csrf_token: response
            .get("antiCsrfToken")
            .and_then(|v| v.as_str())
            .map(String::from),
    })
}
