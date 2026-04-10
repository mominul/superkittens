mod common;

use serial_test::serial;
use std::sync::Arc;

use supertokens::querier::Querier;
use supertokens::recipe::multifactorauth::interfaces::RecipeInterface;
use supertokens::recipe::multifactorauth::recipe_implementation::{
    add_func_to_get_factors_setup_for_user, RecipeImplementationImpl,
};

fn make_mfa_impl() -> RecipeImplementationImpl {
    let querier = Querier::get_instance(Some("multifactorauth".to_string())).unwrap();
    RecipeImplementationImpl { querier }
}

// ===========================================================================
// Get Factors Setup for User
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_factors_setup_for_user_empty() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_mfa_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .get_factors_setup_for_user(&user_id, &mut ctx)
        .await
        .unwrap();

    assert!(
        result.factor_ids.is_empty(),
        "Factor IDs should be empty for a random user"
    );

    common::reset();
}

// ===========================================================================
// Add / Get Required Secondary Factors
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_add_and_get_required_secondary_factors() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_mfa_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();

    recipe
        .add_to_required_secondary_factors_for_user(&user_id, "totp", &mut ctx)
        .await
        .unwrap();

    let result = recipe
        .get_required_secondary_factors_for_user(&user_id, &mut ctx)
        .await
        .unwrap();

    assert!(
        result.factor_ids.contains(&"totp".to_string()),
        "Should contain 'totp' as a required secondary factor"
    );

    common::reset();
}

// ===========================================================================
// Remove Required Secondary Factor
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_remove_required_secondary_factor() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_mfa_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();

    recipe
        .add_to_required_secondary_factors_for_user(&user_id, "totp", &mut ctx)
        .await
        .unwrap();

    recipe
        .remove_from_required_secondary_factors_for_user(&user_id, "totp", &mut ctx)
        .await
        .unwrap();

    let result = recipe
        .get_required_secondary_factors_for_user(&user_id, &mut ctx)
        .await
        .unwrap();

    assert!(
        !result.factor_ids.contains(&"totp".to_string()),
        "Should not contain 'totp' after removal"
    );

    common::reset();
}

// ===========================================================================
// Add Duplicate Factor
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_add_duplicate_factor() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_mfa_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();

    recipe
        .add_to_required_secondary_factors_for_user(&user_id, "totp", &mut ctx)
        .await
        .unwrap();

    // Add the same factor again
    recipe
        .add_to_required_secondary_factors_for_user(&user_id, "totp", &mut ctx)
        .await
        .unwrap();

    let result = recipe
        .get_required_secondary_factors_for_user(&user_id, &mut ctx)
        .await
        .unwrap();

    let totp_count = result.factor_ids.iter().filter(|f| *f == "totp").count();

    assert_eq!(
        totp_count, 1,
        "Should not duplicate the factor; found {} occurrences",
        totp_count
    );

    common::reset();
}

// ===========================================================================
// Factor Setup Callback Registration
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_factors_setup_with_registered_callback() {
    common::reset();
    common::init_with_session().unwrap();

    // Register a custom callback that always reports "custom-factor"
    let func: supertokens::recipe::multifactorauth::recipe_implementation::GetFactorsSetupForUserFn =
        Arc::new(|_user_id| {
            Box::pin(async { Ok(vec!["custom-factor".to_string()]) })
        });
    add_func_to_get_factors_setup_for_user(func);

    let recipe = make_mfa_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .get_factors_setup_for_user(&user_id, &mut ctx)
        .await
        .unwrap();

    assert!(
        result.factor_ids.contains(&"custom-factor".to_string()),
        "Should contain 'custom-factor' from registered callback"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_factors_setup_deduplicates() {
    common::reset();
    common::init_with_session().unwrap();

    // Register two callbacks that both return "totp"
    for _ in 0..2 {
        let func: supertokens::recipe::multifactorauth::recipe_implementation::GetFactorsSetupForUserFn =
            Arc::new(|_user_id| {
                Box::pin(async { Ok(vec!["totp".to_string()]) })
            });
        add_func_to_get_factors_setup_for_user(func);
    }

    let recipe = make_mfa_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .get_factors_setup_for_user(&user_id, &mut ctx)
        .await
        .unwrap();

    let totp_count = result.factor_ids.iter().filter(|f| *f == "totp").count();
    assert_eq!(totp_count, 1, "Should deduplicate factor IDs");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_totp_registers_via_post_init_callback() {
    common::reset();

    // Register TOTP's MFA callback
    supertokens::recipe::totp::register_mfa_factor_setup_callback();

    common::init_with_session().unwrap();

    let recipe = make_mfa_impl();
    let mut ctx = common::new_user_context();

    // A random user with no TOTP devices should return empty
    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .get_factors_setup_for_user(&user_id, &mut ctx)
        .await
        .unwrap();

    assert!(
        result.factor_ids.is_empty(),
        "User with no TOTP devices should have no factors set up"
    );

    common::reset();
}
