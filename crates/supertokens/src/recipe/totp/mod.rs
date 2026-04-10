pub mod interfaces;
pub mod recipe_implementation;
pub mod types;

use std::sync::Arc;

use crate::querier::Querier;

/// Register TOTP's factor-setup check with the MFA recipe.
/// Should be called via `PostSTInitCallbacks` after init.
pub fn register_mfa_factor_setup_callback() {
    use crate::recipe::multifactorauth::recipe_implementation::{
        add_func_to_get_factors_setup_for_user, GetFactorsSetupForUserFn,
    };

    let func: GetFactorsSetupForUserFn = Arc::new(move |user_id: String| {
        Box::pin(async move {
            let querier = Querier::get_instance(Some("totp".to_string()))?;
            let mut user_context = crate::user_context::UserContext::new();

            let mut params = std::collections::HashMap::new();
            params.insert("userId".to_string(), user_id);

            let path = crate::normalised_url_path::NormalisedURLPath::new(
                "/recipe/totp/device/list",
            )?;
            let response = querier
                .send_get_request(&path, Some(params), &mut user_context)
                .await?;

            let has_verified_device = response
                .get("devices")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter().any(|d| {
                        d.get("verified")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if has_verified_device {
                Ok(vec!["totp".to_string()])
            } else {
                Ok(vec![])
            }
        })
    });

    add_func_to_get_factors_setup_for_user(func);
}
