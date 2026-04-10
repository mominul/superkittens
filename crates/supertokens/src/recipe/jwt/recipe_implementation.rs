use async_trait::async_trait;

use super::interfaces::RecipeInterface;
use super::types::*;
use crate::error::SuperTokensError;
use crate::normalised_url_path::NormalisedURLPath;
use crate::querier::Querier;
use crate::user_context::UserContext;

pub struct RecipeImplementationImpl {
    pub querier: Querier,
    pub jwks_domain: String,
}

#[async_trait]
impl RecipeInterface for RecipeImplementationImpl {
    async fn create_jwt(
        &self,
        payload: &serde_json::Map<String, serde_json::Value>,
        validity_seconds: Option<u64>,
        use_static_signing_key: Option<bool>,
        user_context: &mut UserContext,
    ) -> Result<CreateJwtResult, SuperTokensError> {
        // Default validity: 100 years (matches Python SDK default)
        let validity = validity_seconds.unwrap_or(3_153_600_000);
        let mut body = serde_json::json!({
            "payload": payload,
            "algorithm": "RS256",
            "jwksDomain": self.jwks_domain,
            "validity": validity,
        });
        if let Some(s) = use_static_signing_key {
            body["useStaticSigningKey"] = serde_json::json!(s);
        }

        let path = NormalisedURLPath::new("/recipe/jwt")?;
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
                let jwt = response
                    .get("jwt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(CreateJwtResult::Ok { jwt })
            }
            "UNSUPPORTED_ALGORITHM_ERROR" => Ok(CreateJwtResult::UnsupportedAlgorithm),
            _ => Err(crate::error::raise_general_exception(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn get_jwks(
        &self,
        user_context: &mut UserContext,
    ) -> Result<GetJWKSResult, SuperTokensError> {
        let path = NormalisedURLPath::new("/.well-known/jwks.json")?;
        let response = self
            .querier
            .send_get_request(&path, None, user_context)
            .await?;

        let keys: Vec<JsonWebKey> = response
            .get("keys")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|k| serde_json::from_value(k.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        let validity_in_secs = response.get("validityInSecs").and_then(|v| v.as_u64());

        Ok(GetJWKSResult {
            keys,
            validity_in_secs,
        })
    }
}
