use serde_json::Value;

use super::errors::SessionError;
use super::jwt::ParsedJwtInfo;
use super::types::AccessTokenInfo;
use crate::types::user::RecipeUserId;
use crate::utils::get_timestamp_ms;

use super::constants::DEFAULT_TENANT_ID;

/// Validate the structure of an access token payload based on its version.
pub fn validate_access_token_structure(payload: &Value, version: u32) -> Result<(), String> {
    if version >= 5 {
        check_str(payload, "sub")?;
        check_number(payload, "exp")?;
        check_number(payload, "iat")?;
        check_str(payload, "sessionHandle")?;
        check_str(payload, "refreshTokenHash1")?;
        check_str(payload, "rsub")?;
    } else if version >= 4 {
        check_str(payload, "sub")?;
        check_number(payload, "exp")?;
        check_number(payload, "iat")?;
        check_str(payload, "sessionHandle")?;
        check_str(payload, "refreshTokenHash1")?;
        check_str(payload, "tId")?;
    } else if version >= 3 {
        check_str(payload, "sub")?;
        check_number(payload, "exp")?;
        check_number(payload, "iat")?;
        check_str(payload, "sessionHandle")?;
        check_str(payload, "refreshTokenHash1")?;
    } else {
        // v2
        check_str(payload, "sessionHandle")?;
        check_str(payload, "userId")?;
        check_str(payload, "refreshTokenHash1")?;
        if payload.get("userData").is_none() {
            return Err("Missing 'userData' in v2 access token".to_string());
        }
        check_number(payload, "expiryTime")?;
        check_number(payload, "timeCreated")?;
    }
    Ok(())
}

/// Extract session info from a parsed (and locally validated) access token.
///
/// This performs local JWT validation using JWKS keys. If validation fails,
/// it raises `SessionError::TryRefreshToken`.
pub fn get_info_from_access_token(
    jwt_info: &ParsedJwtInfo,
    do_anti_csrf_check: bool,
) -> Result<AccessTokenInfo, SessionError> {
    let payload = &jwt_info.payload;
    let version = jwt_info.version;

    // Validate structure
    validate_access_token_structure(payload, version).map_err(|e| {
        SessionError::try_refresh_token(format!("Invalid access token structure: {}", e))
    })?;

    if version >= 3 {
        // v3+ format: standard JWT claims
        let user_id = payload
            .get("sub")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let expiry_time = payload
            .get("exp")
            .and_then(|v| v.as_f64())
            .map(|v| (v * 1000.0) as u64)
            .unwrap_or(0);
        let time_created = payload
            .get("iat")
            .and_then(|v| v.as_f64())
            .map(|v| (v * 1000.0) as u64)
            .unwrap_or(0);

        let session_handle = get_str(payload, "sessionHandle");
        let refresh_token_hash1 = get_str(payload, "refreshTokenHash1");
        let parent_refresh_token_hash1 = payload
            .get("parentRefreshTokenHash1")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let anti_csrf_token = payload
            .get("antiCsrfToken")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let recipe_user_id = payload
            .get("rsub")
            .and_then(|v| v.as_str())
            .unwrap_or(&user_id)
            .to_string();
        let tenant_id = payload
            .get("tId")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_TENANT_ID)
            .to_string();

        // Check anti-CSRF
        if do_anti_csrf_check && anti_csrf_token.is_none() {
            return Err(SessionError::try_refresh_token(
                "Access token does not contain anti-CSRF token but anti-CSRF check is required",
            ));
        }

        // Check expiry
        if expiry_time < get_timestamp_ms() {
            return Err(SessionError::try_refresh_token("Access token expired"));
        }

        Ok(AccessTokenInfo {
            session_handle,
            user_id,
            recipe_user_id: RecipeUserId::new(recipe_user_id),
            refresh_token_hash1,
            parent_refresh_token_hash1,
            user_data: payload.clone(),
            anti_csrf_token,
            expiry_time,
            time_created,
            tenant_id,
        })
    } else {
        // v2 format: custom fields
        let user_id = get_str(payload, "userId");
        let expiry_time = payload
            .get("expiryTime")
            .and_then(|v| v.as_f64())
            .map(|v| v as u64)
            .unwrap_or(0);
        let time_created = payload
            .get("timeCreated")
            .and_then(|v| v.as_f64())
            .map(|v| v as u64)
            .unwrap_or(0);
        let session_handle = get_str(payload, "sessionHandle");
        let refresh_token_hash1 = get_str(payload, "refreshTokenHash1");
        let parent_refresh_token_hash1 = payload
            .get("parentRefreshTokenHash1")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let anti_csrf_token = payload
            .get("antiCsrfToken")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let user_data = payload
            .get("userData")
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));

        if do_anti_csrf_check && anti_csrf_token.is_none() {
            return Err(SessionError::try_refresh_token(
                "Access token does not contain anti-CSRF token but anti-CSRF check is required",
            ));
        }

        if expiry_time < get_timestamp_ms() {
            return Err(SessionError::try_refresh_token("Access token expired"));
        }

        Ok(AccessTokenInfo {
            session_handle,
            user_id: user_id.clone(),
            recipe_user_id: RecipeUserId::new(user_id),
            refresh_token_hash1,
            parent_refresh_token_hash1,
            user_data,
            anti_csrf_token,
            expiry_time,
            time_created,
            tenant_id: DEFAULT_TENANT_ID.to_string(),
        })
    }
}

// Helpers

fn check_str(payload: &Value, key: &str) -> Result<(), String> {
    match payload.get(key) {
        Some(Value::String(_)) => Ok(()),
        _ => Err(format!("Missing or invalid string field '{}'", key)),
    }
}

fn check_number(payload: &Value, key: &str) -> Result<(), String> {
    match payload.get(key) {
        Some(Value::Number(_)) => Ok(()),
        _ => Err(format!("Missing or invalid number field '{}'", key)),
    }
}

fn get_str(payload: &Value, key: &str) -> String {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}
