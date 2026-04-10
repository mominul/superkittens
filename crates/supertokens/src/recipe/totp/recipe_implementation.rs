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

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn create_device(
        &self,
        user_id: &str,
        _user_identifier_info: Option<&str>,
        device_name: Option<&str>,
        skew: Option<u32>,
        period: Option<u32>,
        user_context: &mut UserContext,
    ) -> Result<CreateDeviceResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "userId": user_id,
            "skew": skew.unwrap_or(1),
            "period": period.unwrap_or(30),
        });

        if let Some(name) = device_name {
            body["deviceName"] = serde_json::Value::String(name.to_string());
        }

        let path = NormalisedURLPath::new("/recipe/totp/device")?;
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
                let device_name_val = response
                    .get("deviceName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let secret = response
                    .get("secret")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(CreateDeviceResult::Ok(CreateDeviceOkResult {
                    device_name: device_name_val,
                    secret: secret.clone(),
                    qr_code_string: format!(
                        "otpauth://totp/?secret={}&digits=6&period={}",
                        secret,
                        period.unwrap_or(30)
                    ),
                }))
            }
            "DEVICE_ALREADY_EXISTS_ERROR" => Ok(CreateDeviceResult::DeviceAlreadyExists),
            "UNKNOWN_USER_ID_ERROR" => Ok(CreateDeviceResult::UnknownUserId),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from create_device: {}",
                status
            ))),
        }
    }

    async fn update_device(
        &self,
        user_id: &str,
        existing_device_name: &str,
        new_device_name: &str,
        user_context: &mut UserContext,
    ) -> Result<UpdateDeviceResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "existingDeviceName": existing_device_name,
            "newDeviceName": new_device_name,
        });

        let path = NormalisedURLPath::new("/recipe/totp/device")?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(UpdateDeviceResult::Ok),
            "UNKNOWN_DEVICE_ERROR" => Ok(UpdateDeviceResult::UnknownDevice),
            "DEVICE_ALREADY_EXISTS_ERROR" => Ok(UpdateDeviceResult::DeviceAlreadyExists),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from update_device: {}",
                status
            ))),
        }
    }

    async fn list_devices(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ListDevicesOkResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("userId".to_string(), user_id.to_string());

        let path = NormalisedURLPath::new("/recipe/totp/device/list")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let devices = response
            .get("devices")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|d| Device {
                        name: d
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        period: d.get("period").and_then(|v| v.as_u64()).unwrap_or(30) as u32,
                        skew: d.get("skew").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
                        verified: d.get("verified").and_then(|v| v.as_bool()).unwrap_or(false),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(ListDevicesOkResult { devices })
    }

    async fn remove_device(
        &self,
        user_id: &str,
        device_name: &str,
        user_context: &mut UserContext,
    ) -> Result<RemoveDeviceOkResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "deviceName": device_name,
        });

        let path = NormalisedURLPath::new("/recipe/totp/device/remove")?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let did_device_exist = response
            .get("didDeviceExist")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(RemoveDeviceOkResult { did_device_exist })
    }

    async fn verify_device(
        &self,
        tenant_id: &str,
        user_id: &str,
        device_name: &str,
        totp: &str,
        user_context: &mut UserContext,
    ) -> Result<VerifyDeviceResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "deviceName": device_name,
            "totp": totp,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/totp/device/verify", tenant_id))?;
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
                let was_already_verified = response
                    .get("wasAlreadyVerified")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(VerifyDeviceResult::Ok {
                    was_already_verified,
                })
            }
            "UNKNOWN_DEVICE_ERROR" => Ok(VerifyDeviceResult::UnknownDevice),
            "INVALID_TOTP_ERROR" => {
                let current = response
                    .get("currentNumberOfFailedAttempts")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                let max = response
                    .get("maxNumberOfFailedAttempts")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                Ok(VerifyDeviceResult::InvalidTotp {
                    current_number_of_failed_attempts: current,
                    max_number_of_failed_attempts: max,
                })
            }
            "LIMIT_REACHED_ERROR" => {
                let retry_after_ms = response
                    .get("retryAfterMs")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                Ok(VerifyDeviceResult::LimitReached { retry_after_ms })
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from verify_device: {}",
                status
            ))),
        }
    }

    async fn verify_totp(
        &self,
        tenant_id: &str,
        user_id: &str,
        totp: &str,
        user_context: &mut UserContext,
    ) -> Result<VerifyTotpResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "totp": totp,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/totp/verify", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(VerifyTotpResult::Ok),
            "UNKNOWN_USER_ID_ERROR" => Ok(VerifyTotpResult::UnknownUserId),
            "INVALID_TOTP_ERROR" => {
                let current = response
                    .get("currentNumberOfFailedAttempts")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                let max = response
                    .get("maxNumberOfFailedAttempts")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                Ok(VerifyTotpResult::InvalidTotp {
                    current_number_of_failed_attempts: current,
                    max_number_of_failed_attempts: max,
                })
            }
            "LIMIT_REACHED_ERROR" => {
                let retry_after_ms = response
                    .get("retryAfterMs")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                Ok(VerifyTotpResult::LimitReached { retry_after_ms })
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from verify_totp: {}",
                status
            ))),
        }
    }
}
