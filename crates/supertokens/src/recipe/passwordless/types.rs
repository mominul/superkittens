use serde::{Deserialize, Serialize};

use crate::types::user::{RecipeUserId, User};

// ---------------------------------------------------------------------------
// Device / code types
// ---------------------------------------------------------------------------

/// A single code associated with a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCode {
    #[serde(rename = "codeId")]
    pub code_id: String,
    #[serde(rename = "timeCreated")]
    pub time_created: u64,
    #[serde(rename = "codeLifetime")]
    pub code_life_time: u64,
}

/// A passwordless device with its associated codes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceType {
    #[serde(rename = "preAuthSessionId")]
    pub pre_auth_session_id: String,
    #[serde(rename = "failedCodeInputAttemptCount")]
    pub failed_code_input_attempt_count: u32,
    pub codes: Vec<DeviceCode>,
    pub email: Option<String>,
    #[serde(rename = "phoneNumber")]
    pub phone_number: Option<String>,
}

// ---------------------------------------------------------------------------
// Recipe implementation result types
// ---------------------------------------------------------------------------

/// Result of a successful create_code call.
#[derive(Debug, Clone)]
pub struct CreateCodeOkResult {
    pub pre_auth_session_id: String,
    pub code_id: String,
    pub device_id: String,
    pub user_input_code: Option<String>,
    pub link_code: Option<String>,
    pub code_life_time: u64,
    pub time_created: u64,
}

/// Result of create_new_code_for_device.
#[derive(Debug, Clone)]
pub enum CreateNewCodeForDeviceResult {
    Ok {
        pre_auth_session_id: String,
        code_id: String,
        device_id: String,
        user_input_code: Option<String>,
        link_code: Option<String>,
        code_life_time: u64,
        time_created: u64,
    },
    RestartFlow,
    UserInputCodeAlreadyUsed,
}

/// Result of consume_code.
#[derive(Debug, Clone)]
pub enum ConsumeCodeResult {
    Ok {
        created_new_recipe_user: bool,
        user: Box<User>,
        recipe_user_id: RecipeUserId,
    },
    IncorrectUserInputCode {
        failed_code_input_attempt_count: u32,
        maximum_code_input_attempts: u32,
    },
    ExpiredUserInputCode {
        failed_code_input_attempt_count: u32,
        maximum_code_input_attempts: u32,
    },
    RestartFlow,
    LinkingToSessionUserFailed {
        reason: String,
    },
}

/// Result of check_code.
#[derive(Debug, Clone)]
pub enum CheckCodeResult {
    Ok {
        consumed_device: DeviceType,
    },
    IncorrectUserInputCode {
        failed_code_input_attempt_count: u32,
        maximum_code_input_attempts: u32,
    },
    ExpiredUserInputCode {
        failed_code_input_attempt_count: u32,
        maximum_code_input_attempts: u32,
    },
    RestartFlow,
}

/// Result of update_user.
#[derive(Debug, Clone)]
pub enum UpdateUserResult {
    Ok,
    UnknownUserId,
    EmailAlreadyExists,
    PhoneNumberAlreadyExists,
    EmailChangeNotAllowed { reason: String },
    PhoneNumberChangeNotAllowed { reason: String },
}

/// Result of revoke_all_codes.
#[derive(Debug, Clone)]
pub struct RevokeAllCodesOkResult;

/// Result of revoke_code.
#[derive(Debug, Clone)]
pub struct RevokeCodeOkResult;
