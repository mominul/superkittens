mod common;

use serial_test::serial;
use std::collections::HashMap;

use supertokens::querier::Querier;
use supertokens::recipe::thirdparty::interfaces::RecipeInterface;
use supertokens::recipe::thirdparty::recipe_implementation::RecipeImplementation;
use supertokens::recipe::thirdparty::types::*;

fn make_tp_impl() -> RecipeImplementation {
    let querier = Querier::get_instance(Some("thirdparty".to_string())).unwrap();
    RecipeImplementation { querier }
}

// ---------------------------------------------------------------------------
// manually_create_or_update_user Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_manually_create_new_user() {
    common::reset();
    common::init_with_session().unwrap();

    let tp_impl = make_tp_impl();
    let mut ctx = common::new_user_context();

    let third_party_id = "google";
    let third_party_user_id = format!("tp-user-{}", uuid::Uuid::new_v4());
    let email = format!("tp+{}@example.com", uuid::Uuid::new_v4());

    let result = tp_impl
        .manually_create_or_update_user(
            third_party_id,
            &third_party_user_id,
            &email,
            false,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await;

    assert!(
        result.is_ok(),
        "manually_create_or_update_user should not error: {:?}",
        result.err()
    );

    match result.unwrap() {
        ManuallyCreateOrUpdateUserResult::Ok {
            user,
            recipe_user_id,
            created_new_recipe_user,
        } => {
            assert!(
                created_new_recipe_user,
                "Should have created a new recipe user"
            );
            assert!(!user.id.is_empty());
            assert!(!recipe_user_id.get_as_string().is_empty());
        }
        other => panic!("Expected Ok, got {:?}", other),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_manually_create_existing_user() {
    common::reset();
    common::init_with_session().unwrap();

    let tp_impl = make_tp_impl();
    let mut ctx = common::new_user_context();

    let third_party_id = "google";
    let third_party_user_id = format!("tp-user-{}", uuid::Uuid::new_v4());
    let email = format!("tp+{}@example.com", uuid::Uuid::new_v4());

    // First call — creates the user
    let first_result = tp_impl
        .manually_create_or_update_user(
            third_party_id,
            &third_party_user_id,
            &email,
            false,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    assert!(
        matches!(
            first_result,
            ManuallyCreateOrUpdateUserResult::Ok {
                created_new_recipe_user: true,
                ..
            }
        ),
        "First call should create a new user"
    );

    // Second call — same user, should not create new
    let second_result = tp_impl
        .manually_create_or_update_user(
            third_party_id,
            &third_party_user_id,
            &email,
            false,
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    match second_result {
        ManuallyCreateOrUpdateUserResult::Ok {
            created_new_recipe_user,
            ..
        } => {
            assert!(
                !created_new_recipe_user,
                "Second call should not create a new recipe user"
            );
        }
        other => panic!("Expected Ok, got {:?}", other),
    }

    common::reset();
}

// ---------------------------------------------------------------------------
// sign_in_up Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_sign_in_up_new_user() {
    common::reset();
    common::init_with_session().unwrap();

    let tp_impl = make_tp_impl();
    let mut ctx = common::new_user_context();

    let third_party_id = "github";
    let third_party_user_id = format!("tp-user-{}", uuid::Uuid::new_v4());
    let email = format!("tp+{}@example.com", uuid::Uuid::new_v4());

    let result = tp_impl
        .sign_in_up(
            third_party_id,
            &third_party_user_id,
            &email,
            false,
            HashMap::new(),
            RawUserInfoFromProvider {
                from_id_token_payload: None,
                from_user_info_api: None,
            },
            None,
            None,
            "public",
            &mut ctx,
        )
        .await;

    assert!(
        result.is_ok(),
        "sign_in_up should not error: {:?}",
        result.err()
    );

    match result.unwrap() {
        SignInUpResult::Ok(ok_result) => {
            assert!(
                ok_result.created_new_recipe_user,
                "Should have created a new recipe user"
            );
            assert!(!ok_result.user.id.is_empty());
            assert!(!ok_result.recipe_user_id.get_as_string().is_empty());
        }
        other => panic!("Expected Ok, got {:?}", other),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_sign_in_up_existing_user() {
    common::reset();
    common::init_with_session().unwrap();

    let tp_impl = make_tp_impl();
    let mut ctx = common::new_user_context();

    let third_party_id = "github";
    let third_party_user_id = format!("tp-user-{}", uuid::Uuid::new_v4());
    let email = format!("tp+{}@example.com", uuid::Uuid::new_v4());

    // First sign_in_up — creates user
    let first_result = tp_impl
        .sign_in_up(
            third_party_id,
            &third_party_user_id,
            &email,
            false,
            HashMap::new(),
            RawUserInfoFromProvider {
                from_id_token_payload: None,
                from_user_info_api: None,
            },
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    assert!(
        matches!(
            first_result,
            SignInUpResult::Ok(SignInUpOkResult {
                created_new_recipe_user: true,
                ..
            })
        ),
        "First sign_in_up should create a new user"
    );

    // Second sign_in_up — same user, should not create new
    let second_result = tp_impl
        .sign_in_up(
            third_party_id,
            &third_party_user_id,
            &email,
            false,
            HashMap::new(),
            RawUserInfoFromProvider {
                from_id_token_payload: None,
                from_user_info_api: None,
            },
            None,
            None,
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    match second_result {
        SignInUpResult::Ok(ok_result) => {
            assert!(
                !ok_result.created_new_recipe_user,
                "Second sign_in_up should not create a new recipe user"
            );
        }
        other => panic!("Expected Ok, got {:?}", other),
    }

    common::reset();
}

// ---------------------------------------------------------------------------
// get_provider Test
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_provider_returns_none() {
    common::reset();
    common::init_with_session().unwrap();

    let tp_impl = make_tp_impl();
    let mut ctx = common::new_user_context();

    let result = tp_impl
        .get_provider("google", None, "public", &mut ctx)
        .await;

    assert!(
        result.is_ok(),
        "get_provider should not error: {:?}",
        result.err()
    );
    assert!(
        result.unwrap().is_none(),
        "get_provider stub should return None"
    );

    common::reset();
}
