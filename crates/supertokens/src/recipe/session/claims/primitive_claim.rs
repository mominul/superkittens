use async_trait::async_trait;
use serde_json::Value;

use crate::recipe::session::interfaces::{SessionClaim, SessionClaimValidator};
use crate::recipe::session::types::SingleClaimValidationResult;
use crate::user_context::UserContext;
use crate::utils::get_timestamp_ms;

/// A claim that stores a single primitive value with a timestamp.
///
/// Stored in the payload as: `{ "key": { "v": value, "t": timestamp_ms } }`
pub struct PrimitiveClaim {
    pub key: String,
    pub default_max_age_in_sec: Option<u64>,
}

impl PrimitiveClaim {
    pub fn new(key: impl Into<String>, default_max_age_in_sec: Option<u64>) -> Self {
        Self {
            key: key.into(),
            default_max_age_in_sec,
        }
    }

    /// Create a `has_value` validator for this claim.
    pub fn has_value_validator(
        &self,
        val: Value,
        max_age_in_sec: Option<u64>,
        id: Option<String>,
    ) -> HasValueValidator {
        HasValueValidator {
            id: id.unwrap_or_else(|| self.key.clone()),
            key: self.key.clone(),
            expected_value: val,
            max_age_in_sec: max_age_in_sec.or(self.default_max_age_in_sec),
        }
    }
}

impl SessionClaim for PrimitiveClaim {
    fn get_key(&self) -> &str {
        &self.key
    }

    fn add_to_payload(&self, payload: &mut Value, value: Value) {
        let obj = serde_json::json!({
            "v": value,
            "t": get_timestamp_ms(),
        });
        if let Value::Object(ref mut map) = payload {
            map.insert(self.key.clone(), obj);
        }
    }

    fn remove_from_payload_by_merge(&self, payload: &mut Value) {
        if let Value::Object(ref mut map) = payload {
            map.insert(self.key.clone(), Value::Null);
        }
    }

    fn remove_from_payload(&self, payload: &mut Value) {
        if let Value::Object(ref mut map) = payload {
            map.remove(&self.key);
        }
    }

    fn get_value_from_payload(&self, payload: &Value) -> Option<Value> {
        payload.get(&self.key).and_then(|v| v.get("v")).cloned()
    }

    fn get_last_refetch_time(&self, payload: &Value) -> Option<u64> {
        payload
            .get(&self.key)
            .and_then(|v| v.get("t"))
            .and_then(|v| v.as_u64())
    }
}

/// Validator that checks a claim has a specific value.
pub struct HasValueValidator {
    pub id: String,
    pub key: String,
    pub expected_value: Value,
    pub max_age_in_sec: Option<u64>,
}

#[async_trait]
impl SessionClaimValidator for HasValueValidator {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_claim(&self) -> Option<&dyn SessionClaim> {
        None
    }

    fn should_refetch(&self, payload: &Value, _user_context: &UserContext) -> bool {
        let claim_data = payload.get(&self.key);
        match claim_data {
            None => true,
            Some(data) => {
                if data.get("v").is_none() {
                    return true;
                }
                if let Some(max_age) = self.max_age_in_sec {
                    if let Some(t) = data.get("t").and_then(|v| v.as_u64()) {
                        return t < get_timestamp_ms() - (max_age * 1000);
                    }
                    return true;
                }
                false
            }
        }
    }

    async fn validate(
        &self,
        payload: &Value,
        _user_context: &UserContext,
    ) -> SingleClaimValidationResult {
        let claim_data = payload.get(&self.key);

        let value = match claim_data.and_then(|d| d.get("v")) {
            None => {
                return SingleClaimValidationResult {
                    is_valid: false,
                    reason: Some(serde_json::json!({
                        "message": "value does not exist",
                        "expectedValue": self.expected_value,
                        "actualValue": Value::Null,
                    })),
                };
            }
            Some(v) => v,
        };

        // Check max age
        if let Some(max_age) = self.max_age_in_sec {
            if let Some(t) = claim_data.and_then(|d| d.get("t")).and_then(|v| v.as_u64()) {
                let now = get_timestamp_ms();
                if t < now - (max_age * 1000) {
                    return SingleClaimValidationResult {
                        is_valid: false,
                        reason: Some(serde_json::json!({
                            "message": "claim value expired",
                            "ageInSeconds": (now - t) / 1000,
                            "maxAgeInSeconds": max_age,
                        })),
                    };
                }
            }
        }

        // Compare value
        if value != &self.expected_value {
            return SingleClaimValidationResult {
                is_valid: false,
                reason: Some(serde_json::json!({
                    "message": "wrong value",
                    "expectedValue": self.expected_value,
                    "actualValue": value,
                })),
            };
        }

        SingleClaimValidationResult {
            is_valid: true,
            reason: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_to_payload() {
        let claim = PrimitiveClaim::new("test-claim", None);
        let mut payload = serde_json::json!({});
        claim.add_to_payload(&mut payload, serde_json::json!("hello"));

        assert!(payload.get("test-claim").is_some());
        let data = &payload["test-claim"];
        assert_eq!(data["v"], "hello");
        assert!(data["t"].is_u64());
    }

    #[test]
    fn test_get_value_from_payload() {
        let claim = PrimitiveClaim::new("test-claim", None);
        let payload = serde_json::json!({
            "test-claim": { "v": 42, "t": 1000 }
        });

        assert_eq!(
            claim.get_value_from_payload(&payload),
            Some(serde_json::json!(42))
        );
    }

    #[test]
    fn test_remove_from_payload() {
        let claim = PrimitiveClaim::new("test-claim", None);
        let mut payload = serde_json::json!({"test-claim": {"v": 1, "t": 1000}});
        claim.remove_from_payload(&mut payload);
        assert!(payload.get("test-claim").is_none());
    }

    #[tokio::test]
    async fn test_has_value_validator_valid() {
        let claim = PrimitiveClaim::new("role", None);
        let validator = claim.has_value_validator(serde_json::json!("admin"), None, None);

        let payload = serde_json::json!({"role": {"v": "admin", "t": get_timestamp_ms()}});
        let ctx = UserContext::new();
        let result = validator.validate(&payload, &ctx).await;
        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_has_value_validator_wrong_value() {
        let claim = PrimitiveClaim::new("role", None);
        let validator = claim.has_value_validator(serde_json::json!("admin"), None, None);

        let payload = serde_json::json!({"role": {"v": "user", "t": get_timestamp_ms()}});
        let ctx = UserContext::new();
        let result = validator.validate(&payload, &ctx).await;
        assert!(!result.is_valid);
    }
}
