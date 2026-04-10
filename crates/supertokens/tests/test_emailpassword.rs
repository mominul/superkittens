mod common;

use serial_test::serial;

use supertokens::recipe::emailpassword::interfaces::RecipeInterface;
use supertokens::recipe::emailpassword::recipe_implementation::RecipeImplementationImpl;
use supertokens::recipe::emailpassword::types::{
    EmailPasswordConfig, SignInResult, SignUpResult,
};
use supertokens::recipe::emailpassword::utils::validate_and_normalise_user_input;

fn make_recipe_impl() -> RecipeImplementationImpl {
    let app_info = supertokens::AppInfo::from_input(&common::test_app_info()).unwrap();
    let querier =
        supertokens::querier::Querier::get_instance(Some("emailpassword".to_string())).unwrap();
    let config = validate_and_normalise_user_input(
        &app_info,
        EmailPasswordConfig {
            sign_up_feature: None,
            override_: None,
        },
    )
    .unwrap();

    RecipeImplementationImpl {
        querier,
        config,
        app_info,
    }
}

// ---------------------------------------------------------------------------
// Sign Up Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_signup_with_valid_email_and_password() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("test+{}@example.com", uuid::Uuid::new_v4());

    let result = recipe_impl
        .sign_up(
            &email,
            "validPassword1!",
            "public",
            None,
            None,
            &mut ctx,
        )
        .await;

    assert!(
        result.is_ok(),
        "sign_up should not error: {:?}",
        result.err()
    );

    match result.unwrap() {
        SignUpResult::Ok {
            user,
            recipe_user_id,
        } => {
            assert!(!user.id.is_empty());
            assert!(!recipe_user_id.get_as_string().is_empty());
        }
        SignUpResult::EmailAlreadyExists => {
            panic!("Expected Ok, got EmailAlreadyExists");
        }
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_signup_duplicate_email() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("duplicate+{}@example.com", uuid::Uuid::new_v4());

    // First signup should succeed
    let result = recipe_impl
        .sign_up(
            &email,
            "validPassword1!",
            "public",
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();
    assert!(matches!(result, SignUpResult::Ok { .. }));

    // Second signup with same email should return EmailAlreadyExists
    let result = recipe_impl
        .sign_up(
            &email,
            "validPassword1!",
            "public",
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();
    assert!(
        matches!(result, SignUpResult::EmailAlreadyExists),
        "Expected EmailAlreadyExists for duplicate email"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_signup_normalises_email() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();

    // Use a unique email to avoid conflicts with other tests sharing the same Core
    let unique = uuid::Uuid::new_v4();
    let mixed_case_email = format!("Test+{}@Example.COM", unique);

    let result = recipe_impl
        .sign_up(
            &mixed_case_email,
            "validPassword1!",
            "public",
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();

    match result {
        SignUpResult::Ok { user, .. } => {
            let stored_email = user
                .login_methods
                .iter()
                .find_map(|lm| lm.email.as_ref())
                .expect("should have email in login methods");
            assert_eq!(stored_email, &mixed_case_email.to_lowercase());
        }
        other => panic!("Expected Ok result, got {:?}", other),
    }

    common::reset();
}

// ---------------------------------------------------------------------------
// Sign In Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_signin_success() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("signin+{}@example.com", uuid::Uuid::new_v4());

    // First sign up
    let signup_result = recipe_impl
        .sign_up(
            &email,
            "validPassword1!",
            "public",
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();
    assert!(matches!(signup_result, SignUpResult::Ok { .. }));

    // Then sign in
    let result = recipe_impl
        .sign_in(
            &email,
            "validPassword1!",
            "public",
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();

    match result {
        SignInResult::Ok {
            user,
            recipe_user_id,
        } => {
            assert!(!user.id.is_empty());
            assert!(!recipe_user_id.get_as_string().is_empty());
        }
        _ => panic!("Expected Ok, got {:?}", result),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_signin_wrong_password() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("wrongpw+{}@example.com", uuid::Uuid::new_v4());

    recipe_impl
        .sign_up(
            &email,
            "validPassword1!",
            "public",
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();

    let result = recipe_impl
        .sign_in(
            &email,
            "wrongPassword1!",
            "public",
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();

    assert!(
        matches!(result, SignInResult::WrongCredentials),
        "Expected WrongCredentials for wrong password"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_signin_nonexistent_user() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("nonexist+{}@example.com", uuid::Uuid::new_v4());

    let result = recipe_impl
        .sign_in(
            &email,
            "validPassword1!",
            "public",
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();

    assert!(
        matches!(result, SignInResult::WrongCredentials),
        "Expected WrongCredentials for nonexistent user"
    );

    common::reset();
}
