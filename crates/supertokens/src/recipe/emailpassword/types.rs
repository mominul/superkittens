use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::error::SuperTokensError;
use crate::ingredients::email_delivery::EmailDeliveryInterface;
use crate::types::user::{RecipeUserId, User};

// ---------------------------------------------------------------------------
// Form field types
// ---------------------------------------------------------------------------

/// A form field submitted in an API request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub id: String,
    pub value: String,
}

/// An input form field for recipe configuration.
#[derive(Clone)]
pub struct InputFormField {
    pub id: String,
    pub validate: Option<FormFieldValidator>,
    pub optional: Option<bool>,
}

/// Normalised form field with guaranteed validator.
#[derive(Clone)]
pub struct NormalisedFormField {
    pub id: String,
    pub validate: FormFieldValidator,
    pub optional: bool,
}

/// Validator function for form fields.
/// Returns `Ok(None)` on success, `Ok(Some(error_message))` on validation failure.
pub type FormFieldValidator = Arc<
    dyn Fn(
            String,
            String,
        )
            -> Pin<Box<dyn Future<Output = Result<Option<String>, SuperTokensError>> + Send>>
        + Send
        + Sync,
>;

// ---------------------------------------------------------------------------
// Sign up/in configuration
// ---------------------------------------------------------------------------

/// Input sign-up feature configuration.
pub struct InputSignUpFeature {
    pub form_fields: Option<Vec<InputFormField>>,
}

/// Normalised sign-up feature.
pub struct SignUpFeature {
    pub form_fields: Vec<NormalisedFormField>,
}

/// Normalised sign-in feature.
pub struct SignInFeature {
    pub form_fields: Vec<NormalisedFormField>,
}

/// Normalised reset-password feature.
pub struct ResetPasswordUsingTokenFeature {
    pub form_fields_for_password_reset_form: Vec<NormalisedFormField>,
    pub form_fields_for_generate_token_form: Vec<NormalisedFormField>,
}

// ---------------------------------------------------------------------------
// Email template types
// ---------------------------------------------------------------------------

/// User info in password reset email template.
#[derive(Debug, Clone)]
pub struct PasswordResetEmailTemplateVarsUser {
    pub id: String,
    pub recipe_user_id: RecipeUserId,
    pub email: String,
}

/// Variables for password reset email template.
#[derive(Debug, Clone)]
pub struct PasswordResetEmailTemplateVars {
    pub user: PasswordResetEmailTemplateVarsUser,
    pub password_reset_link: String,
    pub tenant_id: String,
}

/// Email template input type for the email delivery ingredient.
pub type EmailTemplateVars = PasswordResetEmailTemplateVars;

// ---------------------------------------------------------------------------
// Ingredients
// ---------------------------------------------------------------------------

/// Dependency injection container for emailpassword recipe.
pub struct EmailPasswordIngredients {
    pub email_delivery: Arc<dyn EmailDeliveryInterface<EmailTemplateVars>>,
}

// ---------------------------------------------------------------------------
// Recipe implementation result types
// ---------------------------------------------------------------------------

/// Result of sign_up or create_new_recipe_user.
#[derive(Debug, Clone)]
pub enum SignUpResult {
    Ok {
        user: Box<User>,
        recipe_user_id: RecipeUserId,
    },
    EmailAlreadyExists,
}

/// Result of sign_in or verify_credentials.
#[derive(Debug, Clone)]
pub enum SignInResult {
    Ok {
        user: Box<User>,
        recipe_user_id: RecipeUserId,
    },
    WrongCredentials,
}

/// Result of create_reset_password_token.
#[derive(Debug, Clone)]
pub enum CreateResetPasswordTokenResult {
    Ok { token: String },
    UnknownUserId,
}

/// Result of consume_password_reset_token.
#[derive(Debug, Clone)]
pub enum ConsumePasswordResetTokenResult {
    Ok { email: String, user_id: String },
    PasswordResetTokenInvalid,
}

/// Result of update_email_or_password.
#[derive(Debug, Clone)]
pub enum UpdateEmailOrPasswordResult {
    Ok,
    EmailAlreadyExists,
    UnknownUserId,
    EmailChangeNotAllowed { reason: String },
    PasswordPolicyViolation { failure_reason: String },
}

// ---------------------------------------------------------------------------
// API result types
// ---------------------------------------------------------------------------

/// Result of email_exists_get API.
#[derive(Debug, Clone, Serialize)]
pub enum EmailExistsGetResult {
    Ok { exists: bool },
    GeneralError { message: String },
}

/// Result of generate_password_reset_token_post API.
#[derive(Debug, Clone, Serialize)]
pub enum GeneratePasswordResetTokenPostResult {
    Ok,
    NotAllowed { reason: String },
    GeneralError { message: String },
}

/// Result of password_reset_post API.
#[derive(Debug, Clone)]
pub enum PasswordResetPostResult {
    Ok { email: String, user: Option<User> },
    PasswordResetTokenInvalid,
    PasswordPolicyViolation { failure_reason: String },
    GeneralError { message: String },
}

/// Result of sign_in_post API.
#[derive(Clone)]
pub enum SignInPostResult {
    Ok {
        user: User,
        session: Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>,
    },
    WrongCredentials,
    NotAllowed {
        reason: String,
    },
    GeneralError {
        message: String,
    },
}

/// Result of sign_up_post API.
#[derive(Clone)]
pub enum SignUpPostResult {
    Ok {
        user: User,
        session: Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>,
    },
    EmailAlreadyExists,
    NotAllowed {
        reason: String,
    },
    GeneralError {
        message: String,
    },
}

// ---------------------------------------------------------------------------
// Normalised config
// ---------------------------------------------------------------------------

/// Override functions for recipe and API interfaces.
pub struct OverrideConfig {
    pub functions: Option<
        Box<
            dyn Fn(
                    Arc<dyn super::interfaces::RecipeInterface>,
                ) -> Arc<dyn super::interfaces::RecipeInterface>
                + Send
                + Sync,
        >,
    >,
    pub apis: Option<
        Box<
            dyn Fn(
                    Arc<dyn super::interfaces::ApiInterface>,
                ) -> Arc<dyn super::interfaces::ApiInterface>
                + Send
                + Sync,
        >,
    >,
}

/// Input configuration for emailpassword recipe.
pub struct EmailPasswordConfig {
    pub sign_up_feature: Option<InputSignUpFeature>,
    pub override_: Option<OverrideConfig>,
}

/// Normalised configuration for emailpassword recipe.
pub struct NormalisedEmailPasswordConfig {
    pub sign_up_feature: SignUpFeature,
    pub sign_in_feature: SignInFeature,
    pub reset_password_using_token_feature: ResetPasswordUsingTokenFeature,
    pub override_: OverrideConfig,
}

// Implement Debug manually since we have function pointers
impl std::fmt::Debug for SignInPostResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok { user, .. } => f
                .debug_struct("Ok")
                .field("user", user)
                .field("session", &"<session>")
                .finish(),
            Self::WrongCredentials => write!(f, "WrongCredentials"),
            Self::NotAllowed { reason } => f
                .debug_struct("NotAllowed")
                .field("reason", reason)
                .finish(),
            Self::GeneralError { message } => f
                .debug_struct("GeneralError")
                .field("message", message)
                .finish(),
        }
    }
}

impl std::fmt::Debug for SignUpPostResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok { user, .. } => f
                .debug_struct("Ok")
                .field("user", user)
                .field("session", &"<session>")
                .finish(),
            Self::EmailAlreadyExists => write!(f, "EmailAlreadyExists"),
            Self::NotAllowed { reason } => f
                .debug_struct("NotAllowed")
                .field("reason", reason)
                .finish(),
            Self::GeneralError { message } => f
                .debug_struct("GeneralError")
                .field("message", message)
                .finish(),
        }
    }
}
