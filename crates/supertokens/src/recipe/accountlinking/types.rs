use crate::types::user::User;

/// Result of get_users query.
#[derive(Debug, Clone)]
pub struct GetUsersResult {
    pub users: Vec<User>,
    pub next_pagination_token: Option<String>,
}

/// Result of can_create_primary_user.
#[derive(Debug, Clone)]
pub enum CanCreatePrimaryUserResult {
    Ok {
        was_already_a_primary_user: bool,
    },
    RecipeUserIdAlreadyLinked {
        primary_user_id: String,
        description: String,
    },
    AccountInfoAlreadyAssociated {
        primary_user_id: String,
        description: String,
    },
}

/// Result of create_primary_user.
#[derive(Debug, Clone)]
pub enum CreatePrimaryUserResult {
    Ok {
        user: Box<User>,
        was_already_a_primary_user: bool,
    },
    RecipeUserIdAlreadyLinked {
        primary_user_id: String,
    },
    AccountInfoAlreadyAssociated {
        primary_user_id: String,
    },
}

/// Result of can_link_accounts.
#[derive(Debug, Clone)]
pub enum CanLinkAccountsResult {
    Ok {
        accounts_already_linked: bool,
    },
    RecipeUserIdAlreadyLinked {
        primary_user_id: String,
        user: Option<Box<User>>,
    },
    AccountInfoAlreadyAssociated {
        primary_user_id: String,
    },
    InputUserNotPrimary,
}

/// Result of link_accounts.
#[derive(Debug, Clone)]
pub enum LinkAccountsResult {
    Ok {
        accounts_already_linked: bool,
        user: Box<User>,
    },
    RecipeUserIdAlreadyLinked {
        primary_user_id: String,
        user: Box<User>,
    },
    AccountInfoAlreadyAssociated {
        primary_user_id: String,
    },
    InputUserNotPrimary,
}

/// Result of unlink_account.
#[derive(Debug, Clone)]
pub struct UnlinkAccountOkResult {
    pub was_recipe_user_deleted: bool,
    pub was_linked: bool,
}

/// Account info input for queries.
#[derive(Debug, Clone, Default)]
pub struct AccountInfoInput {
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub third_party_id: Option<String>,
    pub third_party_user_id: Option<String>,
    pub webauthn_credential_id: Option<String>,
}

/// Decision about whether to auto-link accounts.
#[derive(Debug, Clone)]
pub enum ShouldAutomaticallyLinkDecision {
    NotLink,
    Link { should_require_verification: bool },
}
