mod common;

use serial_test::serial;
use supertokens::querier::Querier;
use supertokens::recipe::emailpassword::interfaces::RecipeInterface as EmailPasswordRecipeInterface;
use supertokens::recipe::emailpassword::recipe_implementation::RecipeImplementationImpl as EmailPasswordRecipeImpl;
use supertokens::recipe::emailpassword::types::{EmailPasswordConfig, SignUpResult};
use supertokens::recipe::emailpassword::utils::validate_and_normalise_user_input;
use supertokens::recipe::emailverification::interfaces::RecipeInterface;
use supertokens::recipe::emailverification::recipe_implementation::RecipeImplementationImpl;
use supertokens::recipe::emailverification::types::*;
use supertokens::types::user::RecipeUserId;

fn make_ev_impl() -> RecipeImplementationImpl {
    let querier = Querier::get_instance(Some("emailverification".to_string())).unwrap();
    RecipeImplementationImpl { querier }
}

fn make_ep_impl() -> EmailPasswordRecipeImpl {
    let app_info = supertokens::AppInfo::from_input(&common::test_app_info()).unwrap();
    let querier = Querier::get_instance(Some("emailpassword".to_string())).unwrap();
    let config = validate_and_normalise_user_input(
        &app_info,
        EmailPasswordConfig {
            sign_up_feature: None,
            override_: None,
        },
    )
    .unwrap();

    EmailPasswordRecipeImpl {
        querier,
        config,
        app_info,
    }
}

/// Helper: sign up a new emailpassword user and return (recipe_user_id, email).
async fn create_ep_user(
    ep_impl: &EmailPasswordRecipeImpl,
    ctx: &mut supertokens::UserContext,
) -> (RecipeUserId, String) {
    let email = format!("ev-test+{}@example.com", uuid::Uuid::new_v4());
    let result = ep_impl
        .sign_up(&email, "validPassword1!", "public", None, None, ctx)
        .await
        .unwrap();

    match result {
        SignUpResult::Ok {
            recipe_user_id, ..
        } => (recipe_user_id, email),
        SignUpResult::EmailAlreadyExists => panic!("Expected Ok, got EmailAlreadyExists"),
    }
}

// ---------------------------------------------------------------------------
// Test: create_email_verification_token
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_email_verification_token() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let ep_impl = make_ep_impl();
    let ev_impl = make_ev_impl();
    let mut ctx = common::new_user_context();

    let (recipe_user_id, email) = create_ep_user(&ep_impl, &mut ctx).await;

    let result = ev_impl
        .create_email_verification_token(&recipe_user_id, &email, "public", &mut ctx)
        .await
        .unwrap();

    match result {
        CreateEmailVerificationTokenResult::Ok { token } => {
            assert!(!token.is_empty(), "Token should not be empty");
        }
        CreateEmailVerificationTokenResult::EmailAlreadyVerified => {
            panic!("Expected Ok with token, got EmailAlreadyVerified");
        }
    }

    common::reset();
}

// ---------------------------------------------------------------------------
// Test: verify_email_using_token
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_verify_email_using_token() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let ep_impl = make_ep_impl();
    let ev_impl = make_ev_impl();
    let mut ctx = common::new_user_context();

    let (recipe_user_id, email) = create_ep_user(&ep_impl, &mut ctx).await;

    // Create token
    let token = match ev_impl
        .create_email_verification_token(&recipe_user_id, &email, "public", &mut ctx)
        .await
        .unwrap()
    {
        CreateEmailVerificationTokenResult::Ok { token } => token,
        _ => panic!("Token creation should succeed"),
    };

    // Verify using token
    let result = ev_impl
        .verify_email_using_token(&token, "public", false, &mut ctx)
        .await
        .unwrap();

    match result {
        VerifyEmailUsingTokenResult::Ok { user } => {
            assert_eq!(
                user.recipe_user_id.get_as_string(),
                recipe_user_id.get_as_string(),
                "recipe_user_id should match"
            );
            assert_eq!(user.email, email, "email should match");
        }
        VerifyEmailUsingTokenResult::InvalidToken => {
            panic!("Expected Ok, got InvalidToken");
        }
    }

    common::reset();
}

// ---------------------------------------------------------------------------
// Test: is_email_verified
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_is_email_verified() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let ep_impl = make_ep_impl();
    let ev_impl = make_ev_impl();
    let mut ctx = common::new_user_context();

    let (recipe_user_id, email) = create_ep_user(&ep_impl, &mut ctx).await;

    // Initially not verified
    let verified = ev_impl
        .is_email_verified(&recipe_user_id, &email, &mut ctx)
        .await
        .unwrap();
    assert!(!verified, "Email should not be verified initially");

    // Create and use token to verify
    let token = match ev_impl
        .create_email_verification_token(&recipe_user_id, &email, "public", &mut ctx)
        .await
        .unwrap()
    {
        CreateEmailVerificationTokenResult::Ok { token } => token,
        _ => panic!("Token creation should succeed"),
    };

    ev_impl
        .verify_email_using_token(&token, "public", false, &mut ctx)
        .await
        .unwrap();

    // Now should be verified
    let verified = ev_impl
        .is_email_verified(&recipe_user_id, &email, &mut ctx)
        .await
        .unwrap();
    assert!(verified, "Email should be verified after using token");

    common::reset();
}

// ---------------------------------------------------------------------------
// Test: verify with invalid token
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_verify_with_invalid_token() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let ev_impl = make_ev_impl();
    let mut ctx = common::new_user_context();

    let result = ev_impl
        .verify_email_using_token("bogus-token-that-does-not-exist", "public", false, &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, VerifyEmailUsingTokenResult::InvalidToken),
        "Expected InvalidToken for bogus token"
    );

    common::reset();
}

// ---------------------------------------------------------------------------
// Test: revoke_email_verification_tokens
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_revoke_email_verification_tokens() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let ep_impl = make_ep_impl();
    let ev_impl = make_ev_impl();
    let mut ctx = common::new_user_context();

    let (recipe_user_id, email) = create_ep_user(&ep_impl, &mut ctx).await;

    // Create a token
    let token = match ev_impl
        .create_email_verification_token(&recipe_user_id, &email, "public", &mut ctx)
        .await
        .unwrap()
    {
        CreateEmailVerificationTokenResult::Ok { token } => token,
        _ => panic!("Token creation should succeed"),
    };

    // Revoke all tokens
    ev_impl
        .revoke_email_verification_tokens(&recipe_user_id, &email, "public", &mut ctx)
        .await
        .unwrap();

    // The old token should now be invalid
    let result = ev_impl
        .verify_email_using_token(&token, "public", false, &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, VerifyEmailUsingTokenResult::InvalidToken),
        "Revoked token should be invalid"
    );

    common::reset();
}

// ---------------------------------------------------------------------------
// Test: unverify_email
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_unverify_email() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let ep_impl = make_ep_impl();
    let ev_impl = make_ev_impl();
    let mut ctx = common::new_user_context();

    let (recipe_user_id, email) = create_ep_user(&ep_impl, &mut ctx).await;

    // Verify the email first
    let token = match ev_impl
        .create_email_verification_token(&recipe_user_id, &email, "public", &mut ctx)
        .await
        .unwrap()
    {
        CreateEmailVerificationTokenResult::Ok { token } => token,
        _ => panic!("Token creation should succeed"),
    };

    ev_impl
        .verify_email_using_token(&token, "public", false, &mut ctx)
        .await
        .unwrap();

    // Confirm verified
    let verified = ev_impl
        .is_email_verified(&recipe_user_id, &email, &mut ctx)
        .await
        .unwrap();
    assert!(verified, "Email should be verified");

    // Unverify
    ev_impl
        .unverify_email(&recipe_user_id, &email, &mut ctx)
        .await
        .unwrap();

    // Confirm no longer verified
    let verified = ev_impl
        .is_email_verified(&recipe_user_id, &email, &mut ctx)
        .await
        .unwrap();
    assert!(!verified, "Email should not be verified after unverify");

    common::reset();
}

// ---------------------------------------------------------------------------
// Test: create token for already verified email
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_token_for_already_verified_email() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let ep_impl = make_ep_impl();
    let ev_impl = make_ev_impl();
    let mut ctx = common::new_user_context();

    let (recipe_user_id, email) = create_ep_user(&ep_impl, &mut ctx).await;

    // Verify the email first
    let token = match ev_impl
        .create_email_verification_token(&recipe_user_id, &email, "public", &mut ctx)
        .await
        .unwrap()
    {
        CreateEmailVerificationTokenResult::Ok { token } => token,
        _ => panic!("Token creation should succeed"),
    };

    ev_impl
        .verify_email_using_token(&token, "public", false, &mut ctx)
        .await
        .unwrap();

    // Try to create another token - should get EmailAlreadyVerified
    let result = ev_impl
        .create_email_verification_token(&recipe_user_id, &email, "public", &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, CreateEmailVerificationTokenResult::EmailAlreadyVerified),
        "Expected EmailAlreadyVerified for already verified email, got {:?}",
        result
    );

    common::reset();
}
