use crate::error::SuperTokensError;
use crate::user_context::UserContext;

use super::super::interfaces::{ApiInterface, ApiOptions};
use super::super::types::FormField;
use super::utils::validate_form_fields_or_throw_error;

/// Handle POST /signin
pub async fn handle_sign_in_api(
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
        &api_options.config.sign_in_feature.form_fields,
        &form_fields_raw,
        tenant_id,
    )
    .await?;

    let result = api_implementation
        .sign_in_post(
            &form_fields,
            tenant_id,
            None,
            None,
            api_options,
            user_context,
        )
        .await?;

    match result {
        super::super::types::SignInPostResult::Ok { user, .. } => Ok(serde_json::json!({
            "status": "OK",
            "user": crate::auth_utils::get_backwards_compatible_user_info(
                &user,
                None,
            ),
        })),
        super::super::types::SignInPostResult::WrongCredentials => Ok(serde_json::json!({
            "status": "WRONG_CREDENTIALS_ERROR",
        })),
        super::super::types::SignInPostResult::NotAllowed { reason } => Ok(serde_json::json!({
            "status": "SIGN_IN_NOT_ALLOWED",
            "reason": reason,
        })),
        super::super::types::SignInPostResult::GeneralError { message } => Ok(serde_json::json!({
            "status": "GENERAL_ERROR",
            "message": message,
        })),
    }
}
