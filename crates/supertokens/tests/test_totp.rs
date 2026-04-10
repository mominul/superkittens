mod common;

use serial_test::serial;

use supertokens::querier::Querier;
use supertokens::recipe::totp::interfaces::RecipeInterface;
use supertokens::recipe::totp::recipe_implementation::RecipeImplementationImpl;
use supertokens::recipe::totp::types::*;

fn make_totp_impl() -> RecipeImplementationImpl {
    let querier = Querier::get_instance(Some("totp".to_string())).unwrap();
    RecipeImplementationImpl { querier }
}

// ===========================================================================
// Create Device
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_device() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_totp_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .create_device(&user_id, None, Some("my-device"), None, None, &mut ctx)
        .await
        .unwrap();

    match result {
        CreateDeviceResult::Ok(ok) => {
            assert!(!ok.secret.is_empty(), "Secret should be non-empty");
            assert!(
                !ok.qr_code_string.is_empty(),
                "QR code string should be non-empty"
            );
            assert_eq!(ok.device_name, "my-device");
        }
        other => panic!("Expected Ok, got {:?}", other),
    }

    common::reset();
}

// ===========================================================================
// List Devices
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_list_devices() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_totp_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    recipe
        .create_device(&user_id, None, Some("test-device"), None, None, &mut ctx)
        .await
        .unwrap();

    let result = recipe.list_devices(&user_id, &mut ctx).await.unwrap();

    assert!(!result.devices.is_empty(), "Devices list should not be empty");
    assert!(
        result.devices.iter().any(|d| d.name == "test-device"),
        "Should find the created device"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_list_devices_empty() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_totp_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe.list_devices(&user_id, &mut ctx).await.unwrap();

    assert!(
        result.devices.is_empty(),
        "Devices list should be empty for unknown user"
    );

    common::reset();
}

// ===========================================================================
// Remove Device
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_remove_device() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_totp_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    recipe
        .create_device(&user_id, None, Some("to-remove"), None, None, &mut ctx)
        .await
        .unwrap();

    let result = recipe
        .remove_device(&user_id, "to-remove", &mut ctx)
        .await
        .unwrap();

    assert!(
        result.did_device_exist,
        "Device should have existed before removal"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_remove_nonexistent_device() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_totp_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .remove_device(&user_id, "does-not-exist", &mut ctx)
        .await
        .unwrap();

    assert!(
        !result.did_device_exist,
        "Device should not have existed"
    );

    common::reset();
}

// ===========================================================================
// Update Device
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_update_device_name() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_totp_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    recipe
        .create_device(&user_id, None, Some("old-name"), None, None, &mut ctx)
        .await
        .unwrap();

    let result = recipe
        .update_device(&user_id, "old-name", "new-name", &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, UpdateDeviceResult::Ok),
        "Expected Ok, got {:?}",
        result
    );

    // Verify the new name appears in the device list
    let devices = recipe.list_devices(&user_id, &mut ctx).await.unwrap();
    assert!(
        devices.devices.iter().any(|d| d.name == "new-name"),
        "Should find device with new name"
    );
    assert!(
        !devices.devices.iter().any(|d| d.name == "old-name"),
        "Should not find device with old name"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_update_unknown_device() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_totp_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .update_device(&user_id, "nonexistent", "new-name", &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, UpdateDeviceResult::UnknownDevice),
        "Expected UnknownDevice, got {:?}",
        result
    );

    common::reset();
}
