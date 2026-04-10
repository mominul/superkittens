mod common;

use serde_json::{json, Value};

use supertokens::recipe::session::claims::{BooleanClaim, PrimitiveArrayClaim, PrimitiveClaim};
use supertokens::recipe::session::interfaces::{SessionClaim, SessionClaimValidator};
use supertokens::utils::get_timestamp_ms;
use supertokens::UserContext;

// ---------------------------------------------------------------------------
// PrimitiveClaim — build / payload tests
// (ported from test_primitive_claim.py)
// ---------------------------------------------------------------------------

#[test]
fn test_primitive_claim_add_to_payload() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let mut payload = json!({});
    claim.add_to_payload(&mut payload, json!("hello"));

    let data = &payload["test-claim"];
    assert_eq!(data["v"], "hello");
    assert!(data["t"].is_u64());
}

#[test]
fn test_primitive_claim_add_to_payload_with_existing() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let mut payload = json!({"existing-key": "existing-value"});
    claim.add_to_payload(&mut payload, json!(42));

    assert_eq!(payload["existing-key"], "existing-value");
    assert_eq!(payload["test-claim"]["v"], 42);
}

#[test]
fn test_primitive_claim_get_value_from_empty_payload() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let payload = json!({});
    assert_eq!(claim.get_value_from_payload(&payload), None);
}

#[test]
fn test_primitive_claim_get_value_set_by_add_to_payload() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let mut payload = json!({});
    claim.add_to_payload(&mut payload, json!("hello"));
    assert_eq!(
        claim.get_value_from_payload(&payload),
        Some(json!("hello"))
    );
}

#[test]
fn test_primitive_claim_get_last_refetch_time_empty() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let payload = json!({});
    assert_eq!(claim.get_last_refetch_time(&payload), None);
}

#[test]
fn test_primitive_claim_get_last_refetch_time_after_add() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let before = get_timestamp_ms();
    let mut payload = json!({});
    claim.add_to_payload(&mut payload, json!("val"));
    let after = get_timestamp_ms();

    let t = claim.get_last_refetch_time(&payload).unwrap();
    assert!(t >= before && t <= after);
}

#[test]
fn test_primitive_claim_remove_from_payload() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let mut payload = json!({"test-claim": {"v": 1, "t": 1000}});
    claim.remove_from_payload(&mut payload);
    assert!(payload.get("test-claim").is_none());
}

#[test]
fn test_primitive_claim_remove_from_payload_by_merge() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let mut payload = json!({"test-claim": {"v": 1, "t": 1000}});
    claim.remove_from_payload_by_merge(&mut payload);
    assert_eq!(payload["test-claim"], Value::Null);
}

// ---------------------------------------------------------------------------
// PrimitiveClaim — has_value validator
// (ported from test_primitive_claim.py)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_has_value_validator_empty_payload() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), None, None);
    let ctx = UserContext::new();
    let result = validator.validate(&json!({}), &ctx).await;
    assert!(!result.is_valid);
    assert!(result.reason.is_some());
}

#[tokio::test]
async fn test_has_value_validator_mismatching_payload() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), None, None);
    let ctx = UserContext::new();
    let payload = json!({"test-claim": {"v": "other", "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
    let reason = result.reason.unwrap();
    assert_eq!(reason["message"], "wrong value");
}

#[tokio::test]
async fn test_has_value_validator_matching_payload() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), None, None);
    let ctx = UserContext::new();
    let payload = json!({"test-claim": {"v": "expected", "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_has_value_validator_old_values_without_max_age() {
    // Without max_age, old values are still valid
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), None, None);
    let ctx = UserContext::new();
    // Timestamp from the past
    let payload = json!({"test-claim": {"v": "expected", "t": 1000}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_has_value_validator_expired_with_max_age() {
    // With max_age, old values are rejected
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), Some(5), None);
    let ctx = UserContext::new();
    // Old timestamp
    let payload = json!({"test-claim": {"v": "expected", "t": 1000}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
    let reason = result.reason.unwrap();
    assert_eq!(reason["message"], "claim value expired");
}

#[tokio::test]
async fn test_has_value_validator_fresh_with_max_age() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), Some(300), None);
    let ctx = UserContext::new();
    let payload = json!({"test-claim": {"v": "expected", "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_has_value_validator_with_default_max_age() {
    // Default max_age is used when no explicit max_age is provided
    let claim = PrimitiveClaim::new("test-claim", Some(5)); // 5s default max_age
    let validator = claim.has_value_validator(json!("expected"), None, None);
    let ctx = UserContext::new();
    // Old timestamp → should fail with default max_age
    let payload = json!({"test-claim": {"v": "expected", "t": 1000}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
}

// ---------------------------------------------------------------------------
// PrimitiveClaim — should_refetch
// ---------------------------------------------------------------------------

#[test]
fn test_should_refetch_when_value_not_set() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), None, None);
    let ctx = UserContext::new();
    assert!(validator.should_refetch(&json!({}), &ctx));
}

#[test]
fn test_should_not_refetch_when_value_is_set_no_max_age() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), None, None);
    let ctx = UserContext::new();
    let payload = json!({"test-claim": {"v": "anything", "t": get_timestamp_ms()}});
    assert!(!validator.should_refetch(&payload, &ctx));
}

#[test]
fn test_should_refetch_when_value_is_old_with_max_age() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), Some(5), None);
    let ctx = UserContext::new();
    // Old timestamp
    let payload = json!({"test-claim": {"v": "anything", "t": 1000}});
    assert!(validator.should_refetch(&payload, &ctx));
}

#[test]
fn test_should_not_refetch_when_value_is_fresh_with_max_age() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), Some(300), None);
    let ctx = UserContext::new();
    let payload = json!({"test-claim": {"v": "anything", "t": get_timestamp_ms()}});
    assert!(!validator.should_refetch(&payload, &ctx));
}

// ---------------------------------------------------------------------------
// BooleanClaim
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_boolean_claim_is_true_valid() {
    let claim = BooleanClaim::new("is-admin", None);
    let validator = claim.is_true(None, None);
    let ctx = UserContext::new();
    let payload = json!({"is-admin": {"v": true, "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_boolean_claim_is_true_invalid() {
    let claim = BooleanClaim::new("is-admin", None);
    let validator = claim.is_true(None, None);
    let ctx = UserContext::new();
    let payload = json!({"is-admin": {"v": false, "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
}

#[tokio::test]
async fn test_boolean_claim_is_false_valid() {
    let claim = BooleanClaim::new("is-banned", None);
    let validator = claim.is_false(None, None);
    let ctx = UserContext::new();
    let payload = json!({"is-banned": {"v": false, "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_boolean_claim_is_false_invalid() {
    let claim = BooleanClaim::new("is-banned", None);
    let validator = claim.is_false(None, None);
    let ctx = UserContext::new();
    let payload = json!({"is-banned": {"v": true, "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
}

#[test]
fn test_boolean_claim_add_to_and_get_from_payload() {
    let claim = BooleanClaim::new("flag", None);
    let mut payload = json!({});
    claim.add_to_payload(&mut payload, json!(true));
    assert_eq!(claim.get_value_from_payload(&payload), Some(json!(true)));
}

// ---------------------------------------------------------------------------
// PrimitiveArrayClaim — build / payload tests
// (ported from test_primitive_array_claim.py)
// ---------------------------------------------------------------------------

#[test]
fn test_array_claim_add_to_payload() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let mut payload = json!({});
    claim.add_to_payload(&mut payload, json!(["admin", "user"]));
    assert_eq!(payload["roles"]["v"], json!(["admin", "user"]));
    assert!(payload["roles"]["t"].is_u64());
}

#[test]
fn test_array_claim_get_value_from_empty_payload() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    assert_eq!(claim.get_value_from_payload(&json!({})), None);
}

#[test]
fn test_array_claim_get_value_set_by_add_to_payload() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let mut payload = json!({});
    claim.add_to_payload(&mut payload, json!(["admin", "user"]));
    assert_eq!(
        claim.get_value_from_payload(&payload),
        Some(json!(["admin", "user"]))
    );
}

#[test]
fn test_array_claim_get_last_refetch_time_empty() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    assert_eq!(claim.get_last_refetch_time(&json!({})), None);
}

// ---------------------------------------------------------------------------
// PrimitiveArrayClaim — includes validator
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_array_includes_validator_empty_payload() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes(json!("admin"), None, None);
    let ctx = UserContext::new();
    let result = validator.validate(&json!({}), &ctx).await;
    assert!(!result.is_valid);
    let reason = result.reason.unwrap();
    assert_eq!(reason["message"], "value does not exist");
}

#[tokio::test]
async fn test_array_includes_validator_missing_value() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes(json!("admin"), None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user", "editor"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
    let reason = result.reason.unwrap();
    assert_eq!(reason["message"], "value does not include expected");
}

#[tokio::test]
async fn test_array_includes_validator_matching() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes(json!("admin"), None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["admin", "user"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_array_includes_validator_expired_value() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes(json!("admin"), Some(5), None);
    let ctx = UserContext::new();
    // Old timestamp
    let payload = json!({"roles": {"v": ["admin"], "t": 1000}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
    let reason = result.reason.unwrap();
    assert_eq!(reason["message"], "claim value expired");
}

#[tokio::test]
async fn test_array_includes_validator_old_values_without_max_age() {
    // Without max_age, old values are still valid
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes(json!("admin"), None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["admin"], "t": 1000}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

// ---------------------------------------------------------------------------
// PrimitiveArrayClaim — excludes validator
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_array_excludes_validator_empty_payload() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.excludes(json!("banned"), None, None);
    let ctx = UserContext::new();
    let result = validator.validate(&json!({}), &ctx).await;
    assert!(!result.is_valid);
}

#[tokio::test]
async fn test_array_excludes_validator_value_present() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.excludes(json!("banned"), None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user", "banned"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
    let reason = result.reason.unwrap();
    assert_eq!(reason["message"], "value includes excluded item");
}

#[tokio::test]
async fn test_array_excludes_validator_value_absent() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.excludes(json!("banned"), None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user", "admin"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

// ---------------------------------------------------------------------------
// PrimitiveArrayClaim — includes_all validator
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_array_includes_all_validator_all_present() {
    let claim = PrimitiveArrayClaim::new("permissions", None);
    let validator = claim.includes_all(vec![json!("read"), json!("write")], None, None);
    let ctx = UserContext::new();
    let payload = json!({"permissions": {"v": ["read", "write", "delete"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_array_includes_all_validator_some_missing() {
    let claim = PrimitiveArrayClaim::new("permissions", None);
    let validator = claim.includes_all(vec![json!("read"), json!("write")], None, None);
    let ctx = UserContext::new();
    let payload = json!({"permissions": {"v": ["read"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
}

// ---------------------------------------------------------------------------
// PrimitiveArrayClaim — includes_any validator
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_array_includes_any_validator_one_present() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes_any(vec![json!("admin"), json!("superadmin")], None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user", "admin"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_array_includes_any_validator_none_present() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes_any(vec![json!("admin"), json!("superadmin")], None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user", "editor"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
    let reason = result.reason.unwrap();
    assert_eq!(reason["message"], "value does not include any of expected");
}

// ---------------------------------------------------------------------------
// PrimitiveArrayClaim — excludes_all validator
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_array_excludes_all_validator_all_absent() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.excludes_all(vec![json!("banned"), json!("suspended")], None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user", "admin"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_array_excludes_all_validator_one_present() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.excludes_all(vec![json!("banned"), json!("suspended")], None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user", "banned"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
}

// ---------------------------------------------------------------------------
// PrimitiveArrayClaim — should_refetch
// ---------------------------------------------------------------------------

#[test]
fn test_array_should_refetch_when_value_not_set() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes(json!("admin"), None, None);
    let ctx = UserContext::new();
    assert!(validator.should_refetch(&json!({}), &ctx));
}

#[test]
fn test_array_should_not_refetch_when_value_set_no_max_age() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes(json!("admin"), None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user"], "t": get_timestamp_ms()}});
    assert!(!validator.should_refetch(&payload, &ctx));
}

#[test]
fn test_array_should_refetch_when_value_old_with_max_age() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes(json!("admin"), Some(5), None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user"], "t": 1000}});
    assert!(validator.should_refetch(&payload, &ctx));
}

#[test]
fn test_array_should_not_refetch_when_value_fresh_with_max_age() {
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes(json!("admin"), Some(300), None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user"], "t": get_timestamp_ms()}});
    assert!(!validator.should_refetch(&payload, &ctx));
}

#[test]
fn test_array_should_not_refetch_no_max_age_with_default_inf() {
    // No max_age and no default → should NOT refetch even with old timestamp
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.includes(json!("admin"), None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["user"], "t": 1000}});
    assert!(!validator.should_refetch(&payload, &ctx));
}

// ---------------------------------------------------------------------------
// PrimitiveArrayClaim — default max_age
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_array_default_max_age_expires_old_values() {
    let claim = PrimitiveArrayClaim::new("roles", Some(5)); // 5 second default
    let validator = claim.includes(json!("admin"), None, None); // uses default
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["admin"], "t": 1000}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(!result.is_valid);
    let reason = result.reason.unwrap();
    assert_eq!(reason["message"], "claim value expired");
}

#[tokio::test]
async fn test_array_explicit_max_age_overrides_default() {
    let claim = PrimitiveArrayClaim::new("roles", Some(5)); // 5 second default
    // Explicit max_age of None won't help here since None means "use default"
    // But explicit Some(999999) should accept old values
    let validator = claim.includes(json!("admin"), Some(999999999), None);
    let ctx = UserContext::new();
    // Fresh enough for 999999999 seconds
    let payload = json!({"roles": {"v": ["admin"], "t": get_timestamp_ms() - 10000}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(result.is_valid);
}

// ---------------------------------------------------------------------------
// PrimitiveClaim — validator ID
// ---------------------------------------------------------------------------

#[test]
fn test_primitive_claim_has_value_validator_with_id() {
    let claim = PrimitiveClaim::new("test-claim", None);
    let validator = claim.has_value_validator(json!("expected"), None, None);
    // The validator ID should default to the claim key when no explicit id is given
    assert_eq!(validator.get_id(), "test-claim");

    // With an explicit id, it should use that instead
    let validator_custom = claim.has_value_validator(
        json!("expected"),
        None,
        Some("custom-id".to_string()),
    );
    assert_eq!(validator_custom.get_id(), "custom-id");
}

// ---------------------------------------------------------------------------
// PrimitiveArrayClaim — includes_all / excludes_all edge cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_array_includes_all_empty_required() {
    // includes_all with an empty required list should always be valid
    let claim = PrimitiveArrayClaim::new("permissions", None);
    let validator = claim.includes_all(vec![], None, None);
    let ctx = UserContext::new();
    let payload = json!({"permissions": {"v": ["read", "write"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(
        result.is_valid,
        "includes_all with empty required list should be valid"
    );
}

#[tokio::test]
async fn test_array_excludes_all_empty_excluded() {
    // excludes_all with an empty excluded list should always be valid
    let claim = PrimitiveArrayClaim::new("roles", None);
    let validator = claim.excludes_all(vec![], None, None);
    let ctx = UserContext::new();
    let payload = json!({"roles": {"v": ["admin", "user"], "t": get_timestamp_ms()}});
    let result = validator.validate(&payload, &ctx).await;
    assert!(
        result.is_valid,
        "excludes_all with empty excluded list should be valid"
    );
}
