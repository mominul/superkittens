use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::types::user::{RecipeUserId, User};

// ---------------------------------------------------------------------------
// Raw user info from provider
// ---------------------------------------------------------------------------

/// Raw user information returned by a third-party provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawUserInfoFromProvider {
    pub from_id_token_payload: Option<HashMap<String, Value>>,
    pub from_user_info_api: Option<HashMap<String, Value>>,
}

// ---------------------------------------------------------------------------
// Recipe implementation result types
// ---------------------------------------------------------------------------

/// Successful result of sign_in_up.
#[derive(Debug, Clone)]
pub struct SignInUpOkResult {
    pub user: Box<User>,
    pub recipe_user_id: RecipeUserId,
    pub created_new_recipe_user: bool,
    pub oauth_tokens: HashMap<String, Value>,
    pub raw_user_info_from_provider: RawUserInfoFromProvider,
}

/// Result of manually_create_or_update_user.
#[derive(Debug, Clone)]
pub enum ManuallyCreateOrUpdateUserResult {
    Ok {
        user: Box<User>,
        recipe_user_id: RecipeUserId,
        created_new_recipe_user: bool,
    },
    SignInUpNotAllowed {
        reason: String,
    },
    EmailChangeNotAllowed {
        reason: String,
    },
    LinkingToSessionUserFailed {
        reason: String,
    },
}

/// Result of sign_in_up.
#[derive(Debug, Clone)]
pub enum SignInUpResult {
    Ok(SignInUpOkResult),
    NotAllowed { reason: String },
    LinkingToSessionUserFailed { reason: String },
}
