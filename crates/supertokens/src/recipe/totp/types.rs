// ---------------------------------------------------------------------------
// TOTP recipe types
// ---------------------------------------------------------------------------

/// A TOTP device registered for a user.
#[derive(Debug, Clone)]
pub struct Device {
    pub name: String,
    pub period: u32,
    pub skew: u32,
    pub verified: bool,
}

/// Result of creating a new TOTP device.
#[derive(Debug, Clone)]
pub struct CreateDeviceOkResult {
    pub device_name: String,
    pub secret: String,
    pub qr_code_string: String,
}

/// Result of verifying a TOTP device.
#[derive(Debug, Clone)]
pub enum VerifyDeviceResult {
    Ok {
        was_already_verified: bool,
    },
    UnknownDevice,
    InvalidTotp {
        current_number_of_failed_attempts: u32,
        max_number_of_failed_attempts: u32,
    },
    LimitReached {
        retry_after_ms: u64,
    },
}

/// Result of verifying a TOTP code.
#[derive(Debug, Clone)]
pub enum VerifyTotpResult {
    Ok,
    UnknownUserId,
    InvalidTotp {
        current_number_of_failed_attempts: u32,
        max_number_of_failed_attempts: u32,
    },
    LimitReached {
        retry_after_ms: u64,
    },
}

/// Result of listing all TOTP devices for a user.
#[derive(Debug, Clone)]
pub struct ListDevicesOkResult {
    pub devices: Vec<Device>,
}

/// Result of removing a TOTP device.
#[derive(Debug, Clone)]
pub struct RemoveDeviceOkResult {
    pub did_device_exist: bool,
}

/// Result of updating a TOTP device name.
#[derive(Debug, Clone)]
pub enum UpdateDeviceResult {
    Ok,
    UnknownDevice,
    DeviceAlreadyExists,
}

/// Result of creating a device (may fail if device already exists or user unknown).
#[derive(Debug, Clone)]
pub enum CreateDeviceResult {
    Ok(CreateDeviceOkResult),
    DeviceAlreadyExists,
    UnknownUserId,
}
