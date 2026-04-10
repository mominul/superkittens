use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::types::user::{RecipeUserId, User};
use crate::user_context::UserContext;

/// Default implementation of the Passwordless RecipeInterface.
pub struct RecipeImplementationImpl {
    pub querier: Querier,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn create_code(
        &self,
        email: Option<&str>,
        phone_number: Option<&str>,
        user_input_code: Option<&str>,
        _session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        _should_try_linking_with_session_user: Option<bool>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateCodeOkResult, SuperTokensError> {
        let mut body = serde_json::json!({});

        if let Some(email) = email {
            body["email"] = serde_json::Value::String(email.to_string());
        }
        if let Some(phone_number) = phone_number {
            body["phoneNumber"] = serde_json::Value::String(phone_number.to_string());
        }
        if let Some(user_input_code) = user_input_code {
            body["userInputCode"] = serde_json::Value::String(user_input_code.to_string());
        }

        let path = NormalisedURLPath::new(&format!("/{}/recipe/signinup/code", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(CreateCodeOkResult {
                pre_auth_session_id: response
                    .get("preAuthSessionId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                code_id: response
                    .get("codeId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                device_id: response
                    .get("deviceId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                user_input_code: response
                    .get("userInputCode")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                link_code: response
                    .get("linkCode")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                code_life_time: response
                    .get("codeLifetime")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                time_created: response
                    .get("timeCreated")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            }),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from create_code: {}",
                status
            ))),
        }
    }

    async fn create_new_code_for_device(
        &self,
        device_id: &str,
        user_input_code: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateNewCodeForDeviceResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "deviceId": device_id,
        });

        if let Some(user_input_code) = user_input_code {
            body["userInputCode"] = serde_json::Value::String(user_input_code.to_string());
        }

        let path = NormalisedURLPath::new(&format!("/{}/recipe/signinup/code", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(CreateNewCodeForDeviceResult::Ok {
                pre_auth_session_id: response
                    .get("preAuthSessionId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                code_id: response
                    .get("codeId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                device_id: response
                    .get("deviceId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                user_input_code: response
                    .get("userInputCode")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                link_code: response
                    .get("linkCode")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                code_life_time: response
                    .get("codeLifetime")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                time_created: response
                    .get("timeCreated")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            }),
            "RESTART_FLOW_ERROR" => Ok(CreateNewCodeForDeviceResult::RestartFlow),
            "USER_INPUT_CODE_ALREADY_USED_ERROR" => {
                Ok(CreateNewCodeForDeviceResult::UserInputCodeAlreadyUsed)
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from create_new_code_for_device: {}",
                status
            ))),
        }
    }

    async fn consume_code(
        &self,
        pre_auth_session_id: &str,
        user_input_code: Option<&str>,
        device_id: Option<&str>,
        link_code: Option<&str>,
        _session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        _should_try_linking_with_session_user: Option<bool>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ConsumeCodeResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "preAuthSessionId": pre_auth_session_id,
        });

        if let Some(user_input_code) = user_input_code {
            body["userInputCode"] = serde_json::Value::String(user_input_code.to_string());
        }
        if let Some(device_id) = device_id {
            body["deviceId"] = serde_json::Value::String(device_id.to_string());
        }
        if let Some(link_code) = link_code {
            body["linkCode"] = serde_json::Value::String(link_code.to_string());
        }

        let path = NormalisedURLPath::new(&format!("/{}/recipe/signinup/code/consume", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => {
                let user: User =
                    serde_json::from_value(response.get("user").cloned().unwrap_or_default())?;
                let recipe_user_id = RecipeUserId::new(
                    response
                        .get("recipeUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&user.id)
                        .to_string(),
                );
                let created_new_recipe_user = response
                    .get("createdNewRecipeUser")
                    .or_else(|| response.get("createdNewUser"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(ConsumeCodeResult::Ok {
                    created_new_recipe_user,
                    user: Box::new(user),
                    recipe_user_id,
                })
            }
            "INCORRECT_USER_INPUT_CODE_ERROR" => Ok(ConsumeCodeResult::IncorrectUserInputCode {
                failed_code_input_attempt_count: response
                    .get("failedCodeInputAttemptCount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                maximum_code_input_attempts: response
                    .get("maximumCodeInputAttempts")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
            }),
            "EXPIRED_USER_INPUT_CODE_ERROR" => Ok(ConsumeCodeResult::ExpiredUserInputCode {
                failed_code_input_attempt_count: response
                    .get("failedCodeInputAttemptCount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                maximum_code_input_attempts: response
                    .get("maximumCodeInputAttempts")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
            }),
            "RESTART_FLOW_ERROR" => Ok(ConsumeCodeResult::RestartFlow),
            "LINKING_TO_SESSION_USER_FAILED" => {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Linking to session user failed")
                    .to_string();
                Ok(ConsumeCodeResult::LinkingToSessionUserFailed { reason })
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from consume_code: {}",
                status
            ))),
        }
    }

    async fn check_code(
        &self,
        pre_auth_session_id: &str,
        user_input_code: Option<&str>,
        device_id: Option<&str>,
        link_code: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CheckCodeResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "preAuthSessionId": pre_auth_session_id,
        });

        if let Some(user_input_code) = user_input_code {
            body["userInputCode"] = serde_json::Value::String(user_input_code.to_string());
        }
        if let Some(device_id) = device_id {
            body["deviceId"] = serde_json::Value::String(device_id.to_string());
        }
        if let Some(link_code) = link_code {
            body["linkCode"] = serde_json::Value::String(link_code.to_string());
        }

        let path = NormalisedURLPath::new(&format!("/{}/recipe/signinup/code/check", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => {
                let consumed_device: DeviceType = serde_json::from_value(
                    response.get("consumedDevice").cloned().unwrap_or_default(),
                )?;
                Ok(CheckCodeResult::Ok { consumed_device })
            }
            "INCORRECT_USER_INPUT_CODE_ERROR" => Ok(CheckCodeResult::IncorrectUserInputCode {
                failed_code_input_attempt_count: response
                    .get("failedCodeInputAttemptCount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                maximum_code_input_attempts: response
                    .get("maximumCodeInputAttempts")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
            }),
            "EXPIRED_USER_INPUT_CODE_ERROR" => Ok(CheckCodeResult::ExpiredUserInputCode {
                failed_code_input_attempt_count: response
                    .get("failedCodeInputAttemptCount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                maximum_code_input_attempts: response
                    .get("maximumCodeInputAttempts")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
            }),
            "RESTART_FLOW_ERROR" => Ok(CheckCodeResult::RestartFlow),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from check_code: {}",
                status
            ))),
        }
    }

    async fn update_user(
        &self,
        recipe_user_id: &str,
        email: Option<&str>,
        phone_number: Option<&str>,
        user_context: &mut UserContext,
    ) -> Result<UpdateUserResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "recipeUserId": recipe_user_id,
        });

        if let Some(email) = email {
            body["email"] = serde_json::Value::String(email.to_string());
        }
        if let Some(phone_number) = phone_number {
            body["phoneNumber"] = serde_json::Value::String(phone_number.to_string());
        }

        let path = NormalisedURLPath::new("/recipe/user")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(UpdateUserResult::Ok),
            "UNKNOWN_USER_ID_ERROR" => Ok(UpdateUserResult::UnknownUserId),
            "EMAIL_ALREADY_EXISTS_ERROR" => Ok(UpdateUserResult::EmailAlreadyExists),
            "PHONE_NUMBER_ALREADY_EXISTS_ERROR" => Ok(UpdateUserResult::PhoneNumberAlreadyExists),
            "EMAIL_CHANGE_NOT_ALLOWED_ERROR" => {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Email change not allowed")
                    .to_string();
                Ok(UpdateUserResult::EmailChangeNotAllowed { reason })
            }
            "PHONE_NUMBER_CHANGE_NOT_ALLOWED_ERROR" => {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Phone number change not allowed")
                    .to_string();
                Ok(UpdateUserResult::PhoneNumberChangeNotAllowed { reason })
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from update_user: {}",
                status
            ))),
        }
    }

    async fn revoke_all_codes(
        &self,
        email: Option<&str>,
        phone_number: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RevokeAllCodesOkResult, SuperTokensError> {
        let mut body = serde_json::json!({});

        if let Some(email) = email {
            body["email"] = serde_json::Value::String(email.to_string());
        }
        if let Some(phone_number) = phone_number {
            body["phoneNumber"] = serde_json::Value::String(phone_number.to_string());
        }

        let path = NormalisedURLPath::new(&format!("/{}/recipe/signinup/codes/remove", tenant_id))?;
        self.querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        Ok(RevokeAllCodesOkResult)
    }

    async fn revoke_code(
        &self,
        code_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RevokeCodeOkResult, SuperTokensError> {
        let body = serde_json::json!({
            "codeId": code_id,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/signinup/code/remove", tenant_id))?;
        self.querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        Ok(RevokeCodeOkResult)
    }

    async fn list_codes_by_email(
        &self,
        email: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Vec<DeviceType>, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("email".to_string(), email.to_string());

        self.list_codes(params, tenant_id, user_context).await
    }

    async fn list_codes_by_phone_number(
        &self,
        phone_number: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Vec<DeviceType>, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("phoneNumber".to_string(), phone_number.to_string());

        self.list_codes(params, tenant_id, user_context).await
    }

    async fn list_codes_by_device_id(
        &self,
        device_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<DeviceType>, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("deviceId".to_string(), device_id.to_string());

        let devices = self.list_codes(params, tenant_id, user_context).await?;
        Ok(devices.into_iter().next())
    }

    async fn list_codes_by_pre_auth_session_id(
        &self,
        pre_auth_session_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Option<DeviceType>, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert(
            "preAuthSessionId".to_string(),
            pre_auth_session_id.to_string(),
        );

        let devices = self.list_codes(params, tenant_id, user_context).await?;
        Ok(devices.into_iter().next())
    }
}

impl RecipeImplementationImpl {
    /// Shared helper for all list_codes_by_* methods.
    async fn list_codes(
        &self,
        params: HashMap<String, String>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<Vec<DeviceType>, SuperTokensError> {
        let path = NormalisedURLPath::new(&format!("/{}/recipe/signinup/codes", tenant_id))?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => {
                let devices: Vec<DeviceType> =
                    serde_json::from_value(response.get("devices").cloned().unwrap_or_default())?;
                Ok(devices)
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from list_codes: {}",
                status
            ))),
        }
    }
}
