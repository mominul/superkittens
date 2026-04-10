use crate::types::user::{RecipeUserId, User};

// ---------------------------------------------------------------------------
// WebAuthn recipe types
// ---------------------------------------------------------------------------

/// Relying Party information for WebAuthn registration.
#[derive(Debug, Clone)]
pub struct RelyingParty {
    pub id: String,
    pub name: String,
}

/// User information for WebAuthn registration.
#[derive(Debug, Clone)]
pub struct WebAuthnUser {
    pub id: String,
    pub name: String,
    pub display_name: String,
}

/// Result of generating registration options.
#[derive(Debug, Clone)]
pub struct RegisterOptionsOkResult {
    pub webauthn_generated_options_id: String,
    pub rp: RelyingParty,
    pub user: WebAuthnUser,
    pub challenge: String,
    pub timeout: u64,
    pub exclude_credentials: Vec<CredentialDescriptor>,
    pub attestation: String,
    pub pub_key_cred_params: serde_json::Value,
    pub authenticator_selection: serde_json::Value,
}

/// A credential descriptor used in registration/authentication.
#[derive(Debug, Clone)]
pub struct CredentialDescriptor {
    pub id: String,
    pub r#type: String,
    pub transports: Vec<String>,
}

/// Result of generating sign-in (authentication) options.
#[derive(Debug, Clone)]
pub struct SignInOptionsOkResult {
    pub webauthn_generated_options_id: String,
    pub challenge: String,
    pub timeout: u64,
    pub rp_id: String,
    pub allow_credentials: Vec<CredentialDescriptor>,
    pub user_verification: String,
}

/// Result of register_options.
#[derive(Debug, Clone)]
pub enum RegisterOptionsResult {
    Ok(Box<RegisterOptionsOkResult>),
    RelyingPartyIdMismatch,
    InvalidGeneratedOptions,
    EmailAlreadyExists,
}

/// Result of sign_in_options.
#[derive(Debug, Clone)]
pub enum SignInOptionsResult {
    Ok(SignInOptionsOkResult),
    RelyingPartyIdMismatch,
    InvalidGeneratedOptions,
}

/// Result of sign-up.
#[derive(Debug, Clone)]
pub enum SignUpResult {
    Ok {
        user: Box<User>,
        recipe_user_id: RecipeUserId,
    },
    InvalidCredentials,
    InvalidAuthenticator {
        reason: String,
    },
    EmailAlreadyExists,
    WebAuthnGeneratedOptionsNotFound,
    InvalidWebAuthnGeneratedOptions,
}

/// Result of sign-in.
#[derive(Debug, Clone)]
pub enum SignInResult {
    Ok {
        user: Box<User>,
        recipe_user_id: RecipeUserId,
    },
    InvalidCredentials,
    WebAuthnGeneratedOptionsNotFound,
    InvalidWebAuthnGeneratedOptions,
}

/// A WebAuthn credential.
#[derive(Debug, Clone)]
pub struct Credential {
    pub id: String,
    pub relying_party_id: String,
    pub created_at: u64,
}

/// Result of listing credentials.
#[derive(Debug, Clone)]
pub struct ListCredentialsOkResult {
    pub credentials: Vec<Credential>,
}

/// Result of removing a credential.
#[derive(Debug, Clone)]
pub struct RemoveCredentialOkResult;

/// Result of verifying credentials.
#[derive(Debug, Clone)]
pub enum VerifyCredentialsResult {
    Ok,
    InvalidCredentials,
    WebAuthnGeneratedOptionsNotFound,
    InvalidWebAuthnGeneratedOptions,
}

/// Result of generating a recover account token.
#[derive(Debug, Clone)]
pub enum GenerateRecoverAccountTokenResult {
    Ok { token: String },
    UnknownUserId,
}

/// Result of consuming a recover account token.
#[derive(Debug, Clone)]
pub enum ConsumeRecoverAccountTokenResult {
    Ok { email: String, user_id: String },
    InvalidToken,
}
