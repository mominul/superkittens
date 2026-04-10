use crate::error::SuperTokensError;
use crate::user_context::UserContext;

use super::super::interfaces::{ApiInterface, ApiOptions};
use super::super::types::FormField;
use super::utils::validate_form_fields_or_throw_error;

/// Handle POST /user/password/reset/token
pub async fn handle_generate_password_reset_token_api(
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

    let form_fields = validate_form_fields_or_throw_error(
        &api_options
            .config
            .reset_password_using_token_feature
            .form_fields_for_generate_token_form,
        &form_fields_raw,
        tenant_id,
    )
    .await?;

    let result = api_implementation
        .generate_password_reset_token_post(&form_fields, tenant_id, api_options, user_context)
        .await?;

    match result {
        super::super::types::GeneratePasswordResetTokenPostResult::Ok => Ok(serde_json::json!({
            "status": "OK",
        })),
        super::super::types::GeneratePasswordResetTokenPostResult::NotAllowed { reason } => {
            Ok(serde_json::json!({
                "status": "PASSWORD_RESET_NOT_ALLOWED",
                "reason": reason,
            }))
        }
        super::super::types::GeneratePasswordResetTokenPostResult::GeneralError { message } => {
            Ok(serde_json::json!({
                "status": "GENERAL_ERROR",
                "message": message,
            }))
        }
    }
}
