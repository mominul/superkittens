use serde::Serialize;

/// OpenID discovery configuration.
#[derive(Debug, Clone, Serialize)]
pub struct GetOpenIdDiscoveryConfigurationResult {
    pub issuer: String,
    pub jwks_uri: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub revocation_endpoint: String,
    pub token_introspection_endpoint: String,
    pub end_session_endpoint: String,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
}
