use async_trait::async_trait;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::user_context::UserContext;

pub struct RecipeImplementationImpl {
    pub querier: Querier,
}

impl RecipeImplementationImpl {
    fn parse_saml_client(value: &serde_json::Value) -> SAMLClient {
        serde_json::from_value(value.clone()).unwrap_or(SAMLClient {
            client_id: value
                .get("clientId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            redirect_uris: Vec::new(),
            issuer: String::new(),
            acs_url: String::new(),
            metadata: serde_json::Value::Null,
        })
    }
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn create_or_update_client(
        &self,
        client_id: Option<&str>,
        redirect_uris: Option<&[String]>,
        issuer: Option<&str>,
        acs_url: Option<&str>,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateOrUpdateClientResult, SuperTokensError> {
        let mut body = serde_json::json!({});
        if let Some(id) = client_id {
            body["clientId"] = serde_json::Value::String(id.to_string());
        }
        if let Some(uris) = redirect_uris {
            body["redirectUris"] = serde_json::json!(uris);
        }
        if let Some(iss) = issuer {
            body["issuer"] = serde_json::Value::String(iss.to_string());
        }
        if let Some(acs) = acs_url {
            body["acsUrl"] = serde_json::Value::String(acs.to_string());
        }

        let path = NormalisedURLPath::new(&format!("/{}/recipe/saml/client", tenant_id))?;
        let response = self
            .querier
            .send_put_request(&path, Some(body), None, user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(CreateOrUpdateClientResult::Ok {
                client: Self::parse_saml_client(&response),
            }),
            "UNKNOWN_CLIENT_ID_ERROR" => Ok(CreateOrUpdateClientResult::UnknownClientId),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from create_or_update_client: {}",
                status
            ))),
        }
    }

    async fn list_clients(
        &self,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<ListClientsOkResult, SuperTokensError> {
        let path = NormalisedURLPath::new(&format!("/{}/recipe/saml/clients", tenant_id))?;
        let response = self
            .querier
            .send_get_request(&path, None, user_context)
            .await?;

        let clients = response
            .get("clients")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(Self::parse_saml_client).collect())
            .unwrap_or_default();

        Ok(ListClientsOkResult { clients })
    }

    async fn remove_client(
        &self,
        client_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<RemoveClientResult, SuperTokensError> {
        let body = serde_json::json!({
            "clientId": client_id,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/saml/client/remove", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let did_client_exist = response
            .get("didClientExist")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(RemoveClientResult::Ok { did_client_exist })
    }

    async fn create_login_request(
        &self,
        client_id: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<CreateLoginRequestResult, SuperTokensError> {
        let body = serde_json::json!({
            "clientId": client_id,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/saml/login", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(CreateLoginRequestResult::Ok {
                redirect_url: response
                    .get("redirectUrl")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                saml_request: response
                    .get("samlRequest")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),
            "UNKNOWN_CLIENT_ID_ERROR" => Ok(CreateLoginRequestResult::UnknownClientId),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from create_login_request: {}",
                status
            ))),
        }
    }

    async fn verify_saml_response(
        &self,
        client_id: &str,
        saml_response: &str,
        tenant_id: &str,
        user_context: &mut UserContext,
    ) -> Result<VerifySAMLResponseResult, SuperTokensError> {
        let body = serde_json::json!({
            "clientId": client_id,
            "samlResponse": saml_response,
        });

        let path = NormalisedURLPath::new(&format!("/{}/recipe/saml/response/verify", tenant_id))?;
        let response = self
            .querier
            .send_post_request(&path, Some(body), user_context)
            .await?;

        let status = response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        match status {
            "OK" => Ok(VerifySAMLResponseResult::Ok {
                user_id: response
                    .get("userId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                email: response
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                attributes: response
                    .get("attributes")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
            }),
            "INVALID_RESPONSE_ERROR" => {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(VerifySAMLResponseResult::InvalidResponse { reason })
            }
            "UNKNOWN_CLIENT_ID_ERROR" => Ok(VerifySAMLResponseResult::UnknownClientId),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status from verify_saml_response: {}",
                status
            ))),
        }
    }
}
