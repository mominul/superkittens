use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// SAML recipe types
// ---------------------------------------------------------------------------

/// A SAML client (Service Provider) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SAMLClient {
    pub client_id: String,
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub issuer: String,
    #[serde(default)]
    pub acs_url: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Result of creating or updating a SAML client.
#[derive(Debug, Clone)]
pub enum CreateOrUpdateClientResult {
    Ok { client: SAMLClient },
    UnknownClientId,
}

/// Result of listing SAML clients.
#[derive(Debug, Clone)]
pub struct ListClientsOkResult {
    pub clients: Vec<SAMLClient>,
}

/// Result of removing a SAML client.
#[derive(Debug, Clone)]
pub enum RemoveClientResult {
    Ok { did_client_exist: bool },
}

/// Result of creating a SAML login request.
#[derive(Debug, Clone)]
pub enum CreateLoginRequestResult {
    Ok {
        redirect_url: String,
        saml_request: String,
    },
    UnknownClientId,
}

/// Result of verifying a SAML response.
#[derive(Debug, Clone)]
pub enum VerifySAMLResponseResult {
    Ok {
        user_id: String,
        email: String,
        attributes: serde_json::Value,
    },
    InvalidResponse {
        reason: String,
    },
    UnknownClientId,
}
