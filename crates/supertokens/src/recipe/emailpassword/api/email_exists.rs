use crate::error::SuperTokensError;
use crate::user_context::UserContext;

use super::super::interfaces::{ApiInterface, ApiOptions};

/// Handle GET /emailpassword/email/exists
pub async fn handle_email_exists_api(
    api_implementation: &dyn ApiInterface,
    tenant_id: &str,
    api_options: &ApiOptions<'_>,
    user_context: &mut UserContext,
) -> Result<serde_json::Value, SuperTokensError> {
    let email = api_options
        .request
        .get_query_param("email")
        .ok_or_else(|| {
            crate::error::raise_bad_input_exception("Please provide the email as a GET param")
        })?;

    let result = api_implementation
        .email_exists_get(&email, tenant_id, api_options, user_context)
        .await?;

    match result {
        super::super::types::EmailExistsGetResult::Ok { exists } => Ok(serde_json::json!({
            "status": "OK",
            "exists": exists,
        })),
        super::super::types::EmailExistsGetResult::GeneralError { message } => {
            Ok(serde_json::json!({
                "status": "GENERAL_ERROR",
                "message": message,
            }))
        }
    }
}
