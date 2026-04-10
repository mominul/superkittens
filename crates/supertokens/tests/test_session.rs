mod common;

use serial_test::serial;
use serde_json::json;

use supertokens::recipe::session::interfaces::RecipeInterface;
use supertokens::recipe::session::recipe_implementation::RecipeImplementationImpl;
use supertokens::recipe::session::types::SessionConfig;
use supertokens::recipe::session::utils::validate_and_normalise_user_input;
use supertokens::types::user::RecipeUserId;

fn make_session_recipe_impl() -> RecipeImplementationImpl {
    let app_info = supertokens::AppInfo::from_input(&common::test_app_info()).unwrap();
    let querier =
        supertokens::querier::Querier::get_instance(Some("session".to_string())).unwrap();
    let config = validate_and_normalise_user_input(&app_info, SessionConfig::default()).unwrap();
    RecipeImplementationImpl {
        querier,
        config,
        app_info,
    }
}

// ---------------------------------------------------------------------------
// Session Create / Revoke Lifecycle
// (ported from test_session.py)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_new_session() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("test-user-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await;

    assert!(
        session.is_ok(),
        "create_new_session should succeed: {:?}",
        session.err()
    );

    let session = session.unwrap();
    assert!(!session.get_handle().is_empty());
    assert_eq!(session.get_user_id(), user_id);
    assert_eq!(session.get_tenant_id(), "public");
    assert_eq!(session.get_recipe_user_id().get_as_string(), user_id);

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_session_with_custom_payload() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("test-user-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let access_token_payload = json!({"role": "admin", "premium": true});

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            Some(access_token_payload),
            Some(json!({"db_data": "hello"})),
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    // Get session info to verify stored data
    let info = recipe_impl
        .get_session_information(session.get_handle(), &mut ctx)
        .await
        .unwrap();

    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.user_id, user_id);
    assert_eq!(info.session_data_in_database["db_data"], "hello");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_revoke_session() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("test-user-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let handle = session.get_handle().to_string();

    // Revoke the session
    let revoked = recipe_impl
        .revoke_session(&handle, &mut ctx)
        .await
        .unwrap();
    assert!(revoked, "Session should be revoked");

    // Verify session no longer exists
    let info = recipe_impl
        .get_session_information(&handle, &mut ctx)
        .await
        .unwrap();
    assert!(info.is_none(), "Session info should be None after revoke");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_revoke_nonexistent_session() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();

    let revoked = recipe_impl
        .revoke_session("nonexistent-handle", &mut ctx)
        .await
        .unwrap();
    assert!(!revoked, "Revoking nonexistent session should return false");

    common::reset();
}

// ---------------------------------------------------------------------------
// Session Handles
// (ported from test_session.py::test_creating_many_sessions_for_one_user_and_looping)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_multiple_sessions_for_user() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("multi-session-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    // Create multiple sessions
    let mut handles = Vec::new();
    for _ in 0..3 {
        let session = recipe_impl
            .create_new_session(
                &user_id,
                &recipe_user_id,
                None,
                None,
                None,
                "public",
                &mut ctx,
            )
            .await
            .unwrap();
        handles.push(session.get_handle().to_string());
    }

    // Get all session handles for user
    let all_handles = recipe_impl
        .get_all_session_handles_for_user(&user_id, false, "public", false, &mut ctx)
        .await
        .unwrap();

    assert!(
        all_handles.len() >= 3,
        "Should have at least 3 session handles, got {}",
        all_handles.len()
    );

    // Revoke all sessions
    let revoked = recipe_impl
        .revoke_all_sessions_for_user(&user_id, false, "public", false, &mut ctx)
        .await
        .unwrap();

    assert!(
        revoked.len() >= 3,
        "Should have revoked at least 3 sessions"
    );

    // Verify all gone
    let remaining = recipe_impl
        .get_all_session_handles_for_user(&user_id, false, "public", false, &mut ctx)
        .await
        .unwrap();
    assert!(remaining.is_empty(), "Should have no sessions left");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_revoke_multiple_sessions() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("revoke-multi-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let mut handles = Vec::new();
    for _ in 0..3 {
        let session = recipe_impl
            .create_new_session(
                &user_id,
                &recipe_user_id,
                None,
                None,
                None,
                "public",
                &mut ctx,
            )
            .await
            .unwrap();
        handles.push(session.get_handle().to_string());
    }

    // Revoke first two
    let to_revoke = handles[..2].to_vec();
    let revoked = recipe_impl
        .revoke_multiple_sessions(&to_revoke, &mut ctx)
        .await
        .unwrap();
    assert_eq!(revoked.len(), 2);

    // Third session should still exist
    let info = recipe_impl
        .get_session_information(&handles[2], &mut ctx)
        .await
        .unwrap();
    assert!(info.is_some(), "Third session should still exist");

    common::reset();
}

// ---------------------------------------------------------------------------
// Session Data Operations
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_update_session_data_in_database() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("update-data-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            Some(json!({"initial": "data"})),
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let handle = session.get_handle().to_string();

    // Update session data
    let updated = recipe_impl
        .update_session_data_in_database(
            &handle,
            json!({"updated": "new_data", "count": 42}),
            &mut ctx,
        )
        .await
        .unwrap();
    assert!(updated);

    // Verify updated data
    let info = recipe_impl
        .get_session_information(&handle, &mut ctx)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(info.session_data_in_database["updated"], "new_data");
    assert_eq!(info.session_data_in_database["count"], 42);

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_update_session_data_nonexistent_handle() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();

    let updated = recipe_impl
        .update_session_data_in_database(
            "nonexistent-handle",
            json!({"data": "value"}),
            &mut ctx,
        )
        .await
        .unwrap();
    assert!(!updated, "Should return false for nonexistent handle");

    common::reset();
}

// ---------------------------------------------------------------------------
// Merge Into Access Token Payload
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_merge_into_access_token_payload() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("merge-payload-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            Some(json!({"initial": "value"})),
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let handle = session.get_handle().to_string();

    // Merge additional data
    let merged = recipe_impl
        .merge_into_access_token_payload(
            &handle,
            json!({"extra": "data"}),
            &mut ctx,
        )
        .await
        .unwrap();
    assert!(merged);

    // Verify merged data
    let info = recipe_impl
        .get_session_information(&handle, &mut ctx)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        info.custom_claims_in_access_token_payload["extra"],
        "data"
    );

    common::reset();
}

// ---------------------------------------------------------------------------
// Session Refresh
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_refresh_session() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("refresh-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            Some(true), // disable anti-csrf for testing
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let tokens = session.get_all_session_tokens_dangerously();
    let refresh_token = tokens.refresh_token.expect("Should have refresh token");

    // Refresh the session
    let refreshed = recipe_impl
        .refresh_session(&refresh_token, None, true, &mut ctx)
        .await;

    assert!(
        refreshed.is_ok(),
        "Refresh should succeed: {:?}",
        refreshed.err()
    );

    let refreshed = refreshed.unwrap();
    assert_eq!(refreshed.get_user_id(), user_id);
    assert!(!refreshed.get_handle().is_empty());

    // New tokens should be issued
    let new_tokens = refreshed.get_all_session_tokens_dangerously();
    assert!(new_tokens.access_and_front_token_updated);

    common::reset();
}

// ---------------------------------------------------------------------------
// Session Verify (via get_session)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_session_with_valid_access_token() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("verify-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            Some(true),
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let tokens = session.get_all_session_tokens_dangerously();
    let access_token = &tokens.access_token;

    // Verify the session
    let verified = recipe_impl
        .get_session(
            Some(access_token),
            None,
            Some(false), // disable anti-csrf check
            Some(true),
            None,
            &mut ctx,
        )
        .await;

    assert!(
        verified.is_ok(),
        "get_session should succeed: {:?}",
        verified.err()
    );

    let verified = verified.unwrap();
    assert!(verified.is_some(), "Session should be found");
    let verified = verified.unwrap();
    assert_eq!(verified.get_user_id(), user_id);

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_session_with_check_database() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("verify-db-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            Some(true),
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let tokens = session.get_all_session_tokens_dangerously();
    let access_token = &tokens.access_token;

    // Verify with database check
    let verified = recipe_impl
        .get_session(
            Some(access_token),
            None,
            Some(false),
            Some(true),
            Some(true), // check_database = true
            &mut ctx,
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(verified.get_user_id(), user_id);

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_session_no_token_session_not_required() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();

    // No access token, session_required=false → should return None
    let result = recipe_impl
        .get_session(None, None, Some(false), Some(false), None, &mut ctx)
        .await
        .unwrap();

    assert!(result.is_none(), "Should return None when no token and session not required");

    common::reset();
}

// ---------------------------------------------------------------------------
// Session Information
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_session_information() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("info-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let handle = session.get_handle().to_string();

    let info = recipe_impl
        .get_session_information(&handle, &mut ctx)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(info.user_id, user_id);
    assert_eq!(info.session_handle, handle);
    assert_eq!(info.tenant_id, "public");
    assert!(info.time_created > 0);
    assert!(info.expiry > info.time_created);

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_session_information_nonexistent() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();

    let info = recipe_impl
        .get_session_information("nonexistent", &mut ctx)
        .await
        .unwrap();

    assert!(info.is_none());

    common::reset();
}

// ---------------------------------------------------------------------------
// Session Container Methods
// (ported from test_session.py::test_creating_many_sessions_for_one_user_and_looping)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_session_container_update_and_get_data() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("container-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            Some(json!({"initial": true})),
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    // Get session data from database
    let data = session.get_session_data_from_database(&mut ctx).await.unwrap();
    assert_eq!(data["initial"], true);

    // Update session data
    session
        .update_session_data_in_database(json!({"updated": "value"}), &mut ctx)
        .await
        .unwrap();

    // Verify updated
    let data = session.get_session_data_from_database(&mut ctx).await.unwrap();
    assert_eq!(data["updated"], "value");
    assert!(data.get("initial").is_none(), "Old data should be replaced");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_session_container_merge_payload() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("container-merge-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    // Merge into access token payload
    session
        .merge_into_access_token_payload(json!({"custom_key": "custom_value"}), &mut ctx)
        .await
        .unwrap();

    // Verify via session info
    let info = recipe_impl
        .get_session_information(session.get_handle(), &mut ctx)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        info.custom_claims_in_access_token_payload["custom_key"],
        "custom_value"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_session_container_time_created_and_expiry() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("container-time-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let time_created = session.get_time_created(&mut ctx).await.unwrap();
    let expiry = session.get_expiry(&mut ctx).await.unwrap();

    assert!(time_created > 0);
    assert!(expiry > time_created, "Expiry should be after creation time");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_session_container_revoke() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("container-revoke-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let handle = session.get_handle().to_string();

    // Revoke via container
    session.revoke_session(&mut ctx).await.unwrap();

    // Verify revoked
    let info = recipe_impl
        .get_session_information(&handle, &mut ctx)
        .await
        .unwrap();
    assert!(info.is_none());

    common::reset();
}

// ---------------------------------------------------------------------------
// Session Tokens
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_session_tokens_dangerously() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("tokens-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let tokens = session.get_all_session_tokens_dangerously();
    assert!(!tokens.access_token.is_empty(), "Access token should be present");
    assert!(!tokens.front_token.is_empty(), "Front token should be present");
    assert!(
        tokens.refresh_token.is_some(),
        "Refresh token should be present for new session"
    );
    assert!(tokens.access_and_front_token_updated);

    common::reset();
}

// ---------------------------------------------------------------------------
// JWT Parsing
// (ported from test_access_token_version.py)
// ---------------------------------------------------------------------------

#[test]
fn test_parse_jwt_v3_with_kid() {
    use supertokens::recipe::session::jwt::parse_jwt_without_signature_verification;

    // Create a minimal JWT with a kid header (v3+ format)
    let header = base64_url_encode(r#"{"kid":"test-key-id","alg":"RS256","typ":"JWT","version":"3"}"#);
    let payload = base64_url_encode(r#"{"sub":"user1","exp":9999999999}"#);
    let token = format!("{}.{}.signature", header, payload);

    let result = parse_jwt_without_signature_verification(&token);
    assert!(result.is_ok(), "Should parse JWT: {:?}", result.err());

    let info = result.unwrap();
    assert_eq!(info.version, 3);
    assert_eq!(info.kid, Some("test-key-id".to_string()));
}

#[test]
fn test_parse_jwt_v2_standard_header() {
    use supertokens::recipe::session::jwt::parse_jwt_without_signature_verification;

    // V2 is detected by matching the exact standard v2 header base64 string
    let v2_header = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsInZlcnNpb24iOiIyIn0";
    let payload = base64_url_encode(r#"{"sub":"user1"}"#);
    let token = format!("{}.{}.signature", v2_header, payload);

    let result = parse_jwt_without_signature_verification(&token);
    assert!(result.is_ok(), "Should parse v2 JWT: {:?}", result.err());

    let info = result.unwrap();
    assert_eq!(info.version, 2);
    assert_eq!(info.kid, None);
}

#[test]
fn test_parse_jwt_no_version_defaults_to_latest() {
    use supertokens::recipe::session::jwt::parse_jwt_without_signature_verification;

    // Header without version field but with kid → defaults to v5
    let header = base64_url_encode(r#"{"alg":"RS256","typ":"JWT","kid":"k1"}"#);
    let payload = base64_url_encode(r#"{"sub":"user1"}"#);
    let token = format!("{}.{}.signature", header, payload);

    let result = parse_jwt_without_signature_verification(&token);
    assert!(result.is_ok(), "Should parse JWT: {:?}", result.err());

    let info = result.unwrap();
    assert_eq!(info.version, 5); // defaults to LATEST_TOKEN_VERSION
    assert_eq!(info.kid, Some("k1".to_string()));
}

#[test]
fn test_parse_jwt_no_version_no_kid_errors() {
    use supertokens::recipe::session::jwt::parse_jwt_without_signature_verification;

    // No version (defaults to v5) and no kid → should error
    let header = base64_url_encode(r#"{"alg":"RS256","typ":"JWT"}"#);
    let payload = base64_url_encode(r#"{"sub":"user1"}"#);
    let token = format!("{}.{}.signature", header, payload);

    let result = parse_jwt_without_signature_verification(&token);
    assert!(result.is_err(), "v5 without kid should fail");
    assert!(result.unwrap_err().contains("kid"));
}

#[test]
fn test_parse_jwt_invalid_parts() {
    use supertokens::recipe::session::jwt::parse_jwt_without_signature_verification;

    // Too few parts
    let result = parse_jwt_without_signature_verification("invalid");
    assert!(result.is_err());

    // Too many parts
    let result = parse_jwt_without_signature_verification("a.b.c.d");
    assert!(result.is_err());
}

fn base64_url_encode(s: &str) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    URL_SAFE_NO_PAD.encode(s.as_bytes())
}

// ---------------------------------------------------------------------------
// Regenerate Access Token
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_regenerate_access_token() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("regen-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let tokens = session.get_all_session_tokens_dangerously();

    let result = recipe_impl
        .regenerate_access_token(&tokens.access_token, None, &mut ctx)
        .await;

    assert!(
        result.is_ok(),
        "regenerate_access_token should succeed: {:?}",
        result.err()
    );

    let result = result.unwrap();
    assert!(result.is_some(), "Should return a result");

    let result = result.unwrap();
    assert_eq!(result.session.user_id, user_id);

    common::reset();
}

// ---------------------------------------------------------------------------
// JWKS / Session Verify Extended
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_session_verify_after_refresh_still_works() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("jwks-refresh-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    // Create a session
    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            Some(true),
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let tokens = session.get_all_session_tokens_dangerously();
    let refresh_token = tokens.refresh_token.expect("Should have refresh token");

    // Refresh the session to get new tokens (potentially with rotated keys)
    let refreshed = recipe_impl
        .refresh_session(&refresh_token, None, true, &mut ctx)
        .await
        .unwrap();

    let new_tokens = refreshed.get_all_session_tokens_dangerously();
    let new_access_token = &new_tokens.access_token;

    // Verify the new access token works with get_session (exercises JWKS path)
    let verified = recipe_impl
        .get_session(
            Some(new_access_token),
            None,
            Some(false),
            Some(true),
            None,
            &mut ctx,
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(verified.get_user_id(), user_id);

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_session_verify_with_revoked_session_and_check_database() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("jwks-revoke-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    // Create a session
    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            None,
            None,
            Some(true),
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let tokens = session.get_all_session_tokens_dangerously();
    let access_token = &tokens.access_token;
    let handle = session.get_handle().to_string();

    // Revoke the session
    let revoked = recipe_impl
        .revoke_session(&handle, &mut ctx)
        .await
        .unwrap();
    assert!(revoked);

    // Without check_database, the token's signature is still valid so
    // get_session may still succeed. With check_database=true it should
    // detect the revocation and return an error.
    let result = recipe_impl
        .get_session(
            Some(access_token),
            None,
            Some(false),
            Some(true),
            Some(true), // check_database = true
            &mut ctx,
        )
        .await;

    // After revocation with check_database=true, the result should either
    // be an error (UNAUTHORISED) or Ok(None).
    let is_failed = result.is_err() || result.as_ref().unwrap().is_none();
    assert!(
        is_failed,
        "Revoked session with check_database=true should fail or return None"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_session_access_token_payload_after_merge() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("jwks-merge-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    // Create a session with initial payload
    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            Some(json!({"role": "user"})),
            None,
            Some(true),
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let handle = session.get_handle().to_string();

    // Merge additional data into access token payload
    let merged = recipe_impl
        .merge_into_access_token_payload(
            &handle,
            json!({"premium": true}),
            &mut ctx,
        )
        .await
        .unwrap();
    assert!(merged);

    // Fetch session info and verify both initial and merged claims are present
    let info = recipe_impl
        .get_session_information(&handle, &mut ctx)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        info.custom_claims_in_access_token_payload["role"], "user",
        "Original claim should be preserved"
    );
    assert_eq!(
        info.custom_claims_in_access_token_payload["premium"], true,
        "Merged claim should be present"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_session_change_session_data_does_not_affect_access_token() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe_impl = make_session_recipe_impl();
    let mut ctx = common::new_user_context();
    let user_id = format!("jwks-dbdata-{}", uuid::Uuid::new_v4());
    let recipe_user_id = RecipeUserId::new(user_id.clone());

    // Create a session with an access token payload claim
    let session = recipe_impl
        .create_new_session(
            &user_id,
            &recipe_user_id,
            Some(json!({"token_key": "token_value"})),
            Some(json!({"db_key": "db_value"})),
            Some(true),
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    let handle = session.get_handle().to_string();
    let tokens = session.get_all_session_tokens_dangerously();
    let access_token = &tokens.access_token;

    // Update session data in the database
    let updated = recipe_impl
        .update_session_data_in_database(
            &handle,
            json!({"db_key": "new_db_value", "extra_db": 123}),
            &mut ctx,
        )
        .await
        .unwrap();
    assert!(updated);

    // Verify the access token via get_session - the access token payload
    // should still contain the original claim, unaffected by the database
    // data change.
    let verified = recipe_impl
        .get_session(
            Some(access_token),
            None,
            Some(false),
            Some(true),
            None,
            &mut ctx,
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(verified.get_user_id(), user_id);

    // Confirm via session info that the database data changed but the
    // access token payload did not.
    let info = recipe_impl
        .get_session_information(&handle, &mut ctx)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        info.session_data_in_database["db_key"], "new_db_value",
        "Database data should be updated"
    );
    assert_eq!(
        info.session_data_in_database["extra_db"], 123,
        "New database field should be present"
    );
    assert_eq!(
        info.custom_claims_in_access_token_payload["token_key"], "token_value",
        "Access token payload should be unchanged"
    );
    assert!(
        info.custom_claims_in_access_token_payload.get("db_key").is_none(),
        "Database-only key should not appear in access token payload"
    );

    common::reset();
}
