mod common;

use serial_test::serial;
use supertokens::querier::Querier;
use supertokens::recipe::passwordless::interfaces::RecipeInterface;
use supertokens::recipe::passwordless::recipe_implementation::RecipeImplementationImpl;
use supertokens::recipe::passwordless::types::*;

fn make_passwordless_impl() -> RecipeImplementationImpl {
    let querier = Querier::get_instance(Some("passwordless".to_string())).unwrap();
    RecipeImplementationImpl { querier }
}

// ---------------------------------------------------------------------------
// Create Code Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_code_with_email() {
    common::reset();
    common::init_with_session().unwrap();

    let imp = make_passwordless_impl();
    let mut ctx = common::new_user_context();
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    let result = imp
        .create_code(
            Some(&email),
            None,
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await;

    assert!(result.is_ok(), "create_code should not error: {:?}", result.err());

    let code = result.unwrap();
    assert!(!code.pre_auth_session_id.is_empty(), "pre_auth_session_id should not be empty");
    assert!(!code.code_id.is_empty(), "code_id should not be empty");
    assert!(!code.device_id.is_empty(), "device_id should not be empty");
    assert!(code.code_life_time > 0, "code_life_time should be positive");
    assert!(code.time_created > 0, "time_created should be positive");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_code_with_phone() {
    common::reset();
    common::init_with_session().unwrap();

    let imp = make_passwordless_impl();
    let mut ctx = common::new_user_context();

    let result = imp
        .create_code(
            None,
            Some("+1234567890"),
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await;

    assert!(result.is_ok(), "create_code with phone should not error: {:?}", result.err());

    let code = result.unwrap();
    assert!(!code.pre_auth_session_id.is_empty(), "pre_auth_session_id should not be empty");
    assert!(!code.code_id.is_empty(), "code_id should not be empty");
    assert!(!code.device_id.is_empty(), "device_id should not be empty");

    common::reset();
}

// ---------------------------------------------------------------------------
// Consume Code Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_consume_code_with_link_code() {
    common::reset();
    common::init_with_session().unwrap();

    let imp = make_passwordless_impl();
    let mut ctx = common::new_user_context();
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    // Create a code
    let code = imp
        .create_code(Some(&email), None, None, None, None, "public", &mut ctx)
        .await
        .expect("create_code should succeed");

    let link_code = code.link_code.as_deref().expect("link_code should be present");

    // Consume using link_code
    let result = imp
        .consume_code(
            &code.pre_auth_session_id,
            None,
            None,
            Some(link_code),
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .expect("consume_code should succeed");

    match result {
        ConsumeCodeResult::Ok {
            created_new_recipe_user,
            user,
            recipe_user_id,
        } => {
            assert!(created_new_recipe_user, "Should be a new recipe user");
            assert!(!user.id.is_empty(), "User id should not be empty");
            assert!(
                !recipe_user_id.get_as_string().is_empty(),
                "recipe_user_id should not be empty"
            );
        }
        other => panic!("Expected ConsumeCodeResult::Ok, got {:?}", other),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_consume_code_with_user_input_code() {
    common::reset();
    common::init_with_session().unwrap();

    let imp = make_passwordless_impl();
    let mut ctx = common::new_user_context();
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    // Create a code
    let code = imp
        .create_code(Some(&email), None, None, None, None, "public", &mut ctx)
        .await
        .expect("create_code should succeed");

    let user_input_code = code.user_input_code.as_deref().expect("user_input_code should be present");

    // Consume using user_input_code + device_id
    let result = imp
        .consume_code(
            &code.pre_auth_session_id,
            Some(user_input_code),
            Some(&code.device_id),
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .expect("consume_code should succeed");

    match result {
        ConsumeCodeResult::Ok {
            created_new_recipe_user,
            user,
            recipe_user_id,
        } => {
            assert!(created_new_recipe_user, "Should be a new recipe user");
            assert!(!user.id.is_empty(), "User id should not be empty");
            assert!(
                !recipe_user_id.get_as_string().is_empty(),
                "recipe_user_id should not be empty"
            );
        }
        other => panic!("Expected ConsumeCodeResult::Ok, got {:?}", other),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_consume_incorrect_code() {
    common::reset();
    common::init_with_session().unwrap();

    let imp = make_passwordless_impl();
    let mut ctx = common::new_user_context();
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    // Create a code
    let code = imp
        .create_code(Some(&email), None, None, None, None, "public", &mut ctx)
        .await
        .expect("create_code should succeed");

    // Consume with a wrong user_input_code
    let result = imp
        .consume_code(
            &code.pre_auth_session_id,
            Some("WRONG-CODE"),
            Some(&code.device_id),
            None,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .expect("consume_code should not return an error");

    assert!(
        matches!(result, ConsumeCodeResult::IncorrectUserInputCode { .. }),
        "Expected IncorrectUserInputCode, got {:?}",
        result
    );

    common::reset();
}

// ---------------------------------------------------------------------------
// List Codes Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_list_codes_by_email() {
    common::reset();
    common::init_with_session().unwrap();

    let imp = make_passwordless_impl();
    let mut ctx = common::new_user_context();
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    // Create a code for the email
    imp.create_code(Some(&email), None, None, None, None, "public", &mut ctx)
        .await
        .expect("create_code should succeed");

    // List codes by email
    let devices = imp
        .list_codes_by_email(&email, "public", &mut ctx)
        .await
        .expect("list_codes_by_email should succeed");

    assert!(!devices.is_empty(), "Should have at least one device");
    assert_eq!(
        devices[0].email.as_deref(),
        Some(email.as_str()),
        "Device email should match"
    );
    assert!(!devices[0].codes.is_empty(), "Device should have at least one code");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_list_codes_by_device_id() {
    common::reset();
    common::init_with_session().unwrap();

    let imp = make_passwordless_impl();
    let mut ctx = common::new_user_context();
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    // Create a code
    let code = imp
        .create_code(Some(&email), None, None, None, None, "public", &mut ctx)
        .await
        .expect("create_code should succeed");

    // List codes by device_id
    let device = imp
        .list_codes_by_device_id(&code.device_id, "public", &mut ctx)
        .await
        .expect("list_codes_by_device_id should succeed");

    assert!(device.is_some(), "Should find the device");
    let device = device.unwrap();
    assert_eq!(
        device.pre_auth_session_id, code.pre_auth_session_id,
        "pre_auth_session_id should match"
    );
    assert!(!device.codes.is_empty(), "Device should have at least one code");

    common::reset();
}

// ---------------------------------------------------------------------------
// Revoke Codes Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_revoke_all_codes() {
    common::reset();
    common::init_with_session().unwrap();

    let imp = make_passwordless_impl();
    let mut ctx = common::new_user_context();
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    // Create two codes for the same email
    imp.create_code(Some(&email), None, None, None, None, "public", &mut ctx)
        .await
        .expect("create_code 1 should succeed");
    imp.create_code(Some(&email), None, None, None, None, "public", &mut ctx)
        .await
        .expect("create_code 2 should succeed");

    // Verify codes exist
    let devices = imp
        .list_codes_by_email(&email, "public", &mut ctx)
        .await
        .expect("list_codes_by_email should succeed");
    assert!(!devices.is_empty(), "Should have devices before revoke");

    // Revoke all codes
    imp.revoke_all_codes(Some(&email), None, "public", &mut ctx)
        .await
        .expect("revoke_all_codes should succeed");

    // Verify codes are gone
    let devices_after = imp
        .list_codes_by_email(&email, "public", &mut ctx)
        .await
        .expect("list_codes_by_email should succeed after revoke");
    assert!(
        devices_after.is_empty(),
        "Should have no devices after revoke_all_codes"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_revoke_single_code() {
    common::reset();
    common::init_with_session().unwrap();

    let imp = make_passwordless_impl();
    let mut ctx = common::new_user_context();
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    // Create a code
    let code = imp
        .create_code(Some(&email), None, None, None, None, "public", &mut ctx)
        .await
        .expect("create_code should succeed");

    // Revoke the single code by code_id
    imp.revoke_code(&code.code_id, "public", &mut ctx)
        .await
        .expect("revoke_code should succeed");

    // Verify the device/code is gone
    let devices = imp
        .list_codes_by_email(&email, "public", &mut ctx)
        .await
        .expect("list_codes_by_email should succeed");
    assert!(
        devices.is_empty(),
        "Should have no devices after revoking the only code"
    );

    common::reset();
}

// ---------------------------------------------------------------------------
// Create New Code For Device Test
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_new_code_for_device() {
    common::reset();
    common::init_with_session().unwrap();

    let imp = make_passwordless_impl();
    let mut ctx = common::new_user_context();
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());

    // Create initial code
    let initial_code = imp
        .create_code(Some(&email), None, None, None, None, "public", &mut ctx)
        .await
        .expect("create_code should succeed");

    // Create a new code for the same device
    let result = imp
        .create_new_code_for_device(&initial_code.device_id, None, "public", &mut ctx)
        .await
        .expect("create_new_code_for_device should succeed");

    match result {
        CreateNewCodeForDeviceResult::Ok {
            pre_auth_session_id,
            code_id,
            device_id,
            code_life_time,
            time_created,
            ..
        } => {
            assert!(
                !pre_auth_session_id.is_empty(),
                "pre_auth_session_id should not be empty"
            );
            assert!(!code_id.is_empty(), "code_id should not be empty");
            assert_eq!(
                device_id, initial_code.device_id,
                "device_id should match the original"
            );
            assert!(
                code_id != initial_code.code_id,
                "New code_id should differ from the initial one"
            );
            assert!(code_life_time > 0, "code_life_time should be positive");
            assert!(time_created > 0, "time_created should be positive");
        }
        other => panic!(
            "Expected CreateNewCodeForDeviceResult::Ok, got {:?}",
            other
        ),
    }

    common::reset();
}
