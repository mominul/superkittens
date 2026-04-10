use crate::error::SuperTokensError;
use crate::user_context::UserContext;

use super::super::interfaces::{ApiInterface, ApiOptions};
use super::super::types::FormField;
use super::utils::validate_form_fields_or_throw_error;

/// Handle POST /user/password/reset
pub async fn handle_password_reset_api(
    api_implementation: &dyn ApiInterface,
    tenant_id: &str,
    api_options: &ApiOptions<'_>,
    user_context: &mut UserContext,
) -> Result<serde_json::Value, SuperTokensError> {
    let body = api_options.request.json().await.unwrap_or_default();

    let form_fields_raw: Vec<FormField> = body
        .get("formFields")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let token = body
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            crate::error::raise_bad_input_exception("Please provide the password reset token")
        })?
        .to_string();

    let form_fields = validate_form_fields_or_throw_error(
        &api_options
            .config
            .reset_password_using_token_feature
            .form_fields_for_password_reset_form,
        &form_fields_raw,
        tenant_id,
    )
    .await?;

    let result = api_implementation
        .password_reset_post(&form_fields, &token, tenant_id, api_options, user_context)
        .await?;

    match result {
        super::super::types::PasswordResetPostResult::Ok { user, email } => {
            let mut response = serde_json::json!({
                "status": "OK",
            });
            if let Some(u) = user {
                response["user"] = crate::auth_utils::get_backwards_compatible_user_info(&u, None);
            }
            response["email"] = serde_json::Value::String(email);
            Ok(response)
        }
        super::super::types::PasswordResetPostResult::PasswordResetTokenInvalid => {
            Ok(serde_json::json!({
                "status": "RESET_PASSWORD_INVALID_TOKEN_ERROR",
            }))
        }
        super::super::types::PasswordResetPostResult::PasswordPolicyViolation {
            failure_reason,
        } => Ok(serde_json::json!({
            "status": "PASSWORD_POLICY_VIOLATED_ERROR",
            "failureReason": failure_reason,
        })),
        super::super::types::PasswordResetPostResult::GeneralError { message } => {
            Ok(serde_json::json!({
                "status": "GENERAL_ERROR",
                "message": message,
            }))
        }
    }
}
