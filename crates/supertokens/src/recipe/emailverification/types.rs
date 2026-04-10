use crate::types::user::RecipeUserId;

/// User info associated with email verification.
#[derive(Debug, Clone)]
pub struct EmailVerificationUser {
    pub recipe_user_id: RecipeUserId,
    pub email: String,
}

/// Variables for verification email template.
#[derive(Debug, Clone)]
pub struct VerificationEmailTemplateVars {
    pub user: VerificationEmailTemplateVarsUser,
    pub email_verify_link: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct VerificationEmailTemplateVarsUser {
    pub id: String,
    pub recipe_user_id: RecipeUserId,
    pub email: String,
}

/// Email template input type alias.
pub type EmailTemplateVars = VerificationEmailTemplateVars;

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum CreateEmailVerificationTokenResult {
    Ok { token: String },
    EmailAlreadyVerified,
}

#[derive(Debug, Clone)]
pub enum VerifyEmailUsingTokenResult {
    Ok { user: EmailVerificationUser },
    InvalidToken,
}

#[derive(Debug, Clone)]
pub struct RevokeEmailVerificationTokensOkResult;

#[derive(Debug, Clone)]
pub struct UnverifyEmailOkResult;
