use super::primitive_claim::{HasValueValidator, PrimitiveClaim};
use crate::recipe::session::interfaces::SessionClaim;
use serde_json::Value;

/// A boolean-valued session claim.
///
/// Convenience wrapper around `PrimitiveClaim` with `is_true`/`is_false` validators.
pub struct BooleanClaim {
    inner: PrimitiveClaim,
}

impl BooleanClaim {
    pub fn new(key: impl Into<String>, default_max_age_in_sec: Option<u64>) -> Self {
        Self {
            inner: PrimitiveClaim::new(key, default_max_age_in_sec),
        }
    }

    pub fn is_true(&self, max_age: Option<u64>, id: Option<String>) -> HasValueValidator {
        self.inner
            .has_value_validator(Value::Bool(true), max_age, id)
    }

    pub fn is_false(&self, max_age: Option<u64>, id: Option<String>) -> HasValueValidator {
        self.inner
            .has_value_validator(Value::Bool(false), max_age, id)
    }
}

impl SessionClaim for BooleanClaim {
    fn get_key(&self) -> &str {
        self.inner.get_key()
    }

    fn add_to_payload(&self, payload: &mut Value, value: Value) {
        self.inner.add_to_payload(payload, value);
    }

    fn remove_from_payload_by_merge(&self, payload: &mut Value) {
        self.inner.remove_from_payload_by_merge(payload);
    }

    fn remove_from_payload(&self, payload: &mut Value) {
        self.inner.remove_from_payload(payload);
    }

    fn get_value_from_payload(&self, payload: &Value) -> Option<Value> {
        self.inner.get_value_from_payload(payload)
    }

    fn get_last_refetch_time(&self, payload: &Value) -> Option<u64> {
        self.inner.get_last_refetch_time(payload)
    }
}
