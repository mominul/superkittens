use serde::{Deserialize, Serialize};

/// A JSON Web Key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonWebKey {
    pub kty: String,
    pub kid: String,
    pub n: String,
    pub e: String,
    pub alg: String,
    #[serde(rename = "use")]
    pub use_: String,
}

/// Result of create_jwt.
#[derive(Debug, Clone)]
pub enum CreateJwtResult {
    Ok { jwt: String },
    UnsupportedAlgorithm,
}

/// Result of get_jwks.
#[derive(Debug, Clone)]
pub struct GetJWKSResult {
    pub keys: Vec<JsonWebKey>,
    pub validity_in_secs: Option<u64>,
}
