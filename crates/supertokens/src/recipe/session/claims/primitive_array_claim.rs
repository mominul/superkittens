use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashSet;

use crate::recipe::session::interfaces::{SessionClaim, SessionClaimValidator};
use crate::recipe::session::types::SingleClaimValidationResult;
use crate::user_context::UserContext;
use crate::utils::get_timestamp_ms;

/// A claim that stores an array of primitive values with a timestamp.
///
/// Stored as: `{ "key": { "v": [values...], "t": timestamp_ms } }`
pub struct PrimitiveArrayClaim {
    pub key: String,
    pub default_max_age_in_sec: Option<u64>,
}

impl PrimitiveArrayClaim {
    pub fn new(key: impl Into<String>, default_max_age_in_sec: Option<u64>) -> Self {
        Self {
            key: key.into(),
            default_max_age_in_sec,
        }
    }

    pub fn includes(
        &self,
        val: Value,
        max_age: Option<u64>,
        id: Option<String>,
    ) -> ArrayClaimValidator {
        ArrayClaimValidator {
            id: id.unwrap_or_else(|| self.key.clone()),
            key: self.key.clone(),
            expected: vec![val],
            max_age_in_sec: max_age.or(self.default_max_age_in_sec),
            mode: ArrayValidatorMode::Includes,
        }
    }

    pub fn excludes(
        &self,
        val: Value,
        max_age: Option<u64>,
        id: Option<String>,
    ) -> ArrayClaimValidator {
        ArrayClaimValidator {
            id: id.unwrap_or_else(|| self.key.clone()),
            key: self.key.clone(),
            expected: vec![val],
            max_age_in_sec: max_age.or(self.default_max_age_in_sec),
            mode: ArrayValidatorMode::Excludes,
        }
    }

    pub fn includes_all(
        &self,
        vals: Vec<Value>,
        max_age: Option<u64>,
        id: Option<String>,
    ) -> ArrayClaimValidator {
        ArrayClaimValidator {
            id: id.unwrap_or_else(|| self.key.clone()),
            key: self.key.clone(),
            expected: vals,
            max_age_in_sec: max_age.or(self.default_max_age_in_sec),
            mode: ArrayValidatorMode::IncludesAll,
        }
    }

    pub fn includes_any(
        &self,
        vals: Vec<Value>,
        max_age: Option<u64>,
        id: Option<String>,
    ) -> ArrayClaimValidator {
        ArrayClaimValidator {
            id: id.unwrap_or_else(|| self.key.clone()),
            key: self.key.clone(),
            expected: vals,
            max_age_in_sec: max_age.or(self.default_max_age_in_sec),
            mode: ArrayValidatorMode::IncludesAny,
        }
    }

    pub fn excludes_all(
        &self,
        vals: Vec<Value>,
        max_age: Option<u64>,
        id: Option<String>,
    ) -> ArrayClaimValidator {
        ArrayClaimValidator {
            id: id.unwrap_or_else(|| self.key.clone()),
            key: self.key.clone(),
            expected: vals,
            max_age_in_sec: max_age.or(self.default_max_age_in_sec),
            mode: ArrayValidatorMode::ExcludesAll,
        }
    }
}

impl SessionClaim for PrimitiveArrayClaim {
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

#[derive(Debug, Clone)]
pub enum ArrayValidatorMode {
    Includes,
    Excludes,
    IncludesAll,
    IncludesAny,
    ExcludesAll,
}

pub struct ArrayClaimValidator {
    pub id: String,
    pub key: String,
    pub expected: Vec<Value>,
    pub max_age_in_sec: Option<u64>,
    pub mode: ArrayValidatorMode,
}

#[async_trait]
impl SessionClaimValidator for ArrayClaimValidator {
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

        let claim_val = match claim_data
            .and_then(|d| d.get("v"))
            .and_then(|v| v.as_array())
        {
            None => {
                return SingleClaimValidationResult {
                    is_valid: false,
                    reason: Some(serde_json::json!({
                        "message": "value does not exist",
                        "expectedToInclude": self.expected,
                        "actualValue": Value::Null,
                    })),
                };
            }
            Some(arr) => arr,
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

        // Build a set for O(1) lookups (using JSON string representation)
        let claim_set: HashSet<String> = claim_val.iter().map(|v| v.to_string()).collect();

        match self.mode {
            ArrayValidatorMode::Includes | ArrayValidatorMode::IncludesAll => {
                for expected in &self.expected {
                    if !claim_set.contains(&expected.to_string()) {
                        return SingleClaimValidationResult {
                            is_valid: false,
                            reason: Some(serde_json::json!({
                                "message": "value does not include expected",
                                "expectedToInclude": expected,
                                "actualValue": claim_val,
                            })),
                        };
                    }
                }
            }
            ArrayValidatorMode::IncludesAny => {
                let found = self
                    .expected
                    .iter()
                    .any(|e| claim_set.contains(&e.to_string()));
                if !found {
                    return SingleClaimValidationResult {
                        is_valid: false,
                        reason: Some(serde_json::json!({
                            "message": "value does not include any of expected",
                            "expectedToIncludeAnyOf": self.expected,
                            "actualValue": claim_val,
                        })),
                    };
                }
            }
            ArrayValidatorMode::Excludes | ArrayValidatorMode::ExcludesAll => {
                for expected in &self.expected {
                    if claim_set.contains(&expected.to_string()) {
                        return SingleClaimValidationResult {
                            is_valid: false,
                            reason: Some(serde_json::json!({
                                "message": "value includes excluded item",
                                "expectedToExclude": expected,
                                "actualValue": claim_val,
                            })),
                        };
                    }
                }
            }
        }

        SingleClaimValidationResult {
            is_valid: true,
            reason: None,
        }
    }
}
