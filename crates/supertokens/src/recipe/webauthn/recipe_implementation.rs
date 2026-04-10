use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::types::user::{RecipeUserId, User};
use crate::user_context::UserContext;

pub struct RecipeImplementationImpl {
    pub querier: Querier,
}

impl RecipeImplementationImpl {
    fn parse_credential_descriptors(value: &serde_json::Value) -> Vec<CredentialDescriptor> {
        value
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|c| CredentialDescriptor {
                        id: c
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        r#type: c
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("public-key")
                            .to_string(),
                        transports: c
                            .get("transports")
                            .and_then(|v| v.as_array())
                            .map(|a| {
                                a.iter()
                                    .filter_map(|s| s.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn register_options(
        &self,
        email: &str,
        rp_id: &str,
        rp_name: &str,
        origin: &str,
        timeout: Option<u64>,
        attestation: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RegisterOptionsResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "email": email,
            "relyingPartyId": rp_id,
            "relyingPartyName": rp_name,
            "origin": origin,
        });
        if let Some(t) = timeout {
            body["timeout"] = serde_json::json!(t);
        }
        if let Some(a) = attestation {
            body["attestation"] = serde_json::Value::String(a.to_string());
        }

        let path =
            NormalisedURLPath::new(&format!("/{}/recipe/webauthn/options/register", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(RegisterOptionsResult::Ok(Box::new(
                RegisterOptionsOkResult {
                    webauthn_generated_options_id: response
                        .get("webauthnGeneratedOptionsId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    rp: RelyingParty {
                        id: response
                            .get("rp")
                            .and_then(|v| v.get("id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(rp_id)
                            .to_string(),
                        name: response
                            .get("rp")
                            .and_then(|v| v.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(rp_name)
                            .to_string(),
                    },
                    user: WebAuthnUser {
                        id: response
                            .get("user")
                            .and_then(|v| v.get("id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        name: response
                            .get("user")
                            .and_then(|v| v.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        display_name: response
                            .get("user")
                            .and_then(|v| v.get("displayName"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    },
                    challenge: response
                        .get("challenge")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    timeout: response
                        .get("timeout")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(60000),
                    exclude_credentials: Self::parse_credential_descriptors(
                        response
                            .get("excludeCredentials")
                            .unwrap_or(&serde_json::Value::Null),
                    ),
                    attestation: response
                        .get("attestation")
                        .and_then(|v| v.as_str())
                        .unwrap_or("none")
                        .to_string(),
                    pub_key_cred_params: response
                        .get("pubKeyCredParams")
                        .cloned()
                        .unwrap_or(serde_json::Value::Array(vec![])),
                    authenticator_selection: response
                        .get("authenticatorSelection")
                        .cloned()
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                },
            ))),
            "RELYING_PARTY_ID_MISMATCH_ERROR" => Ok(RegisterOptionsResult::RelyingPartyIdMismatch),
            "INVALID_GENERATED_OPTIONS_ERROR" => Ok(RegisterOptionsResult::InvalidGeneratedOptions),
            "EMAIL_ALREADY_EXISTS_ERROR" => Ok(RegisterOptionsResult::EmailAlreadyExists),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from register_options: {}",
                status
            ))),
        }
    }

    async fn sign_in_options(
        &self,
        rp_id: &str,
        origin: &str,
        timeout: Option<u64>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<SignInOptionsResult, SuperTokensError> {
        let mut body = serde_json::json!({
            "relyingPartyId": rp_id,
            "origin": origin,
        });
        if let Some(t) = timeout {
            body["timeout"] = serde_json::json!(t);
        }

        let path =
            NormalisedURLPath::new(&format!("/{}/recipe/webauthn/options/signin", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(SignInOptionsResult::Ok(SignInOptionsOkResult {
                webauthn_generated_options_id: response
                    .get("webauthnGeneratedOptionsId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                challenge: response
                    .get("challenge")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                timeout: response
                    .get("timeout")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(60000),
                rp_id: response
                    .get("rpId")
                    .and_then(|v| v.as_str())
                    .unwrap_or(rp_id)
                    .to_string(),
                allow_credentials: Self::parse_credential_descriptors(
                    response
                        .get("allowCredentials")
                        .unwrap_or(&serde_json::Value::Null),
                ),
                user_verification: response
                    .get("userVerification")
                    .and_then(|v| v.as_str())
                    .unwrap_or("preferred")
                    .to_string(),
            })),
            "RELYING_PARTY_ID_MISMATCH_ERROR" => Ok(SignInOptionsResult::RelyingPartyIdMismatch),
            "INVALID_GENERATED_OPTIONS_ERROR" => Ok(SignInOptionsResult::InvalidGeneratedOptions),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from sign_in_options: {}",
                status
            ))),
        }
    }

    async fn sign_up(
        &self,
        webauthn_generated_options_id: &str,
        credential: serde_json::Value,
        tenant_id: &str,
        _session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        _should_try_linking_with_session_user: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<SignUpResult, SuperTokensError> {
        let body = serde_json::json!({
            "webauthnGeneratedOptionsId": webauthn_generated_options_id,
            "credential": credential,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/webauthn/signup", tenant_id))?;
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
                let user: User =
                    serde_json::from_value(response.get("user").cloned().unwrap_or_default())?;
                let recipe_user_id = RecipeUserId::new(
                    response
                        .get("recipeUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&user.id)
                        .to_string(),
                );
                Ok(SignUpResult::Ok {
                    user: Box::new(user),
                    recipe_user_id,
                })
            }
            "INVALID_CREDENTIALS_ERROR" => Ok(SignUpResult::InvalidCredentials),
            "INVALID_AUTHENTICATOR_ERROR" => {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(SignUpResult::InvalidAuthenticator { reason })
            }
            "EMAIL_ALREADY_EXISTS_ERROR" => Ok(SignUpResult::EmailAlreadyExists),
            "GENERATED_OPTIONS_NOT_FOUND_ERROR" => {
                Ok(SignUpResult::WebAuthnGeneratedOptionsNotFound)
            }
            "INVALID_GENERATED_OPTIONS_ERROR" => Ok(SignUpResult::InvalidWebAuthnGeneratedOptions),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from webauthn signup: {}",
                status
            ))),
        }
    }

    async fn sign_in(
        &self,
        webauthn_generated_options_id: &str,
        credential: serde_json::Value,
        tenant_id: &str,
        _session: Option<Arc<dyn crate::recipe::session::interfaces::SessionContainerInterface>>,
        _should_try_linking_with_session_user: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<SignInResult, SuperTokensError> {
        let body = serde_json::json!({
            "webauthnGeneratedOptionsId": webauthn_generated_options_id,
            "credential": credential,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/webauthn/signin", tenant_id))?;
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
                let user: User =
                    serde_json::from_value(response.get("user").cloned().unwrap_or_default())?;
                let recipe_user_id = RecipeUserId::new(
                    response
                        .get("recipeUserId")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&user.id)
                        .to_string(),
                );
                Ok(SignInResult::Ok {
                    user: Box::new(user),
                    recipe_user_id,
                })
            }
            "INVALID_CREDENTIALS_ERROR" => Ok(SignInResult::InvalidCredentials),
            "GENERATED_OPTIONS_NOT_FOUND_ERROR" => {
                Ok(SignInResult::WebAuthnGeneratedOptionsNotFound)
            }
            "INVALID_GENERATED_OPTIONS_ERROR" => Ok(SignInResult::InvalidWebAuthnGeneratedOptions),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from webauthn signin: {}",
                status
            ))),
        }
    }

    async fn verify_credentials(
        &self,
        webauthn_generated_options_id: &str,
        credential: serde_json::Value,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<VerifyCredentialsResult, SuperTokensError> {
        let body = serde_json::json!({
            "webauthnGeneratedOptionsId": webauthn_generated_options_id,
            "credential": credential,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/webauthn/signin", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(VerifyCredentialsResult::Ok),
            "INVALID_CREDENTIALS_ERROR" => Ok(VerifyCredentialsResult::InvalidCredentials),
            "GENERATED_OPTIONS_NOT_FOUND_ERROR" => {
                Ok(VerifyCredentialsResult::WebAuthnGeneratedOptionsNotFound)
            }
            "INVALID_GENERATED_OPTIONS_ERROR" => {
                Ok(VerifyCredentialsResult::InvalidWebAuthnGeneratedOptions)
            }
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from webauthn verify_credentials: {}",
                status
            ))),
        }
    }

    async fn list_credentials(
        &self,
        user_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ListCredentialsOkResult, SuperTokensError> {
        let mut params = HashMap::new();
        params.insert("userId".to_string(), user_id.to_string());

        let path = NormalisedURLPath::new("/recipe/webauthn/credentials")?;
        let response = self
            .querier
            .send_get_request(&path, Some(params), user_context)
            .await?;

        let credentials = response
            .get("credentials")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|c| Credential {
                        id: c
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        relying_party_id: c
                            .get("relyingPartyId")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        created_at: c.get("createdAt").and_then(|v| v.as_u64()).unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(ListCredentialsOkResult { credentials })
    }

    async fn remove_credential(
        &self,
        credential_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RemoveCredentialOkResult, SuperTokensError> {
        let body = serde_json::json!({
            "credentialId": credential_id,
        });

        let path = NormalisedURLPath::new("/recipe/webauthn/credentials/remove")?;
        self.querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        Ok(RemoveCredentialOkResult)
    }

    async fn generate_recover_account_token(
        &self,
        user_id: &str,
        email: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<GenerateRecoverAccountTokenResult, SuperTokensError> {
        let body = serde_json::json!({
            "userId": user_id,
            "email": email,
        });

        let path = NormalisedURLPath::new(&format!(
            "/{}/recipe/webauthn/user/recover/token",
            tenant_id
        ))?;
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
                let token = response
                    .get("token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(GenerateRecoverAccountTokenResult::Ok { token })
            }
            "UNKNOWN_USER_ID_ERROR" => Ok(GenerateRecoverAccountTokenResult::UnknownUserId),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from generate_recover_account_token: {}",
                status
            ))),
        }
    }

    async fn consume_recover_account_token(
        &self,
        token: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ConsumeRecoverAccountTokenResult, SuperTokensError> {
        let body = serde_json::json!({
            "token": token,
        });

        let path = NormalisedURLPath::new(&format!(
            "/{}/recipe/webauthn/user/recover/token/consume",
            tenant_id
        ))?;
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
                let email = response
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let user_id = response
                    .get("userId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(ConsumeRecoverAccountTokenResult::Ok { email, user_id })
            }
            _ => Ok(ConsumeRecoverAccountTokenResult::InvalidToken),
        }
    }
}
