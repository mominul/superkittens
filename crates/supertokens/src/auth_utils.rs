use crate::types::user::User;

/// Get backwards-compatible user info based on FDI version.
///
/// Different FDI versions expect different response formats:
/// - (>= 1.18 && < 2.0) || >= 3.0: full user object
/// - < 1.18 or (>= 2.0 && < 3.0): single login method object
pub fn get_backwards_compatible_user_info(
    user: &User,
    fdi_version: Option<&str>,
) -> serde_json::Value {
    match fdi_version {
        Some(v) if should_use_full_user_object(v) => user.to_json(),
        _ => {
            // Return the first login method as the user info
            if let Some(lm) = user.login_methods.first() {
                lm.to_json()
            } else {
                user.to_json()
            }
        }
    }
}

fn should_use_full_user_object(fdi_version: &str) -> bool {
    use crate::utils::is_version_gte;
    // (>= 1.18 && < 2.0) || >= 3.0
    (is_version_gte(fdi_version, "1.18") && !is_version_gte(fdi_version, "2.0"))
        || is_version_gte(fdi_version, "3.0")
}
