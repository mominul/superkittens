mod common;

use serial_test::serial;

use supertokens::recipe::emailpassword::interfaces::RecipeInterface;
use supertokens::recipe::emailpassword::recipe_implementation::RecipeImplementationImpl;
use supertokens::recipe::emailpassword::types::{
    ConsumePasswordResetTokenResult, CreateResetPasswordTokenResult, EmailPasswordConfig,
    SignInResult, SignUpResult, UpdateEmailOrPasswordResult,
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
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
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
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();
    assert!(matches!(result, SignUpResult::Ok { .. }));

    // Second signup with same email should return EmailAlreadyExists
    let result = recipe_impl
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
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
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();
    assert!(matches!(signup_result, SignUpResult::Ok { .. }));

    // Then sign in
    let result = recipe_impl
        .sign_in(&email, "validPassword1!", "public", None, None, &mut ctx)
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
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    let result = recipe_impl
        .sign_in(&email, "wrongPassword1!", "public", None, None, &mut ctx)
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
        .sign_in(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, SignInResult::WrongCredentials),
        "Expected WrongCredentials for nonexistent user"
    );

    common::reset();
}

// ---------------------------------------------------------------------------
// Verify Credentials Tests
// (ported from test_signin.py — verify_credentials is sign_in without session)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_verify_credentials_success() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("verify+{}@example.com", uuid::Uuid::new_v4());

    recipe_impl
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    let result = recipe_impl
        .verify_credentials(&email, "validPassword1!", "public", &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, SignInResult::Ok { .. }),
        "Expected Ok for valid credentials"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_verify_credentials_wrong_password() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("verifywrong+{}@example.com", uuid::Uuid::new_v4());

    recipe_impl
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    let result = recipe_impl
        .verify_credentials(&email, "wrongPassword1!", "public", &mut ctx)
        .await
        .unwrap();

    assert!(matches!(result, SignInResult::WrongCredentials));

    common::reset();
}

// ---------------------------------------------------------------------------
// Create New Recipe User (without account linking)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_new_recipe_user() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("newuser+{}@example.com", uuid::Uuid::new_v4());

    let result = recipe_impl
        .create_new_recipe_user(&email, "validPassword1!", "public", &mut ctx)
        .await
        .unwrap();

    match result {
        SignUpResult::Ok {
            user,
            recipe_user_id,
        } => {
            assert!(!user.id.is_empty());
            assert!(!recipe_user_id.get_as_string().is_empty());
            // Verify email is normalised
            let stored_email = user
                .login_methods
                .iter()
                .find_map(|lm| lm.email.as_ref())
                .expect("should have email");
            assert_eq!(stored_email, &email.to_lowercase());
        }
        SignUpResult::EmailAlreadyExists => {
            panic!("Expected Ok, got EmailAlreadyExists");
        }
    }

    common::reset();
}

// ---------------------------------------------------------------------------
// Password Reset Token Tests
// (ported from test_passwordreset.py)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_reset_password_token_success() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("resetpw+{}@example.com", uuid::Uuid::new_v4());

    // Sign up first
    let signup_result = recipe_impl
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    let user_id = match signup_result {
        SignUpResult::Ok { user, .. } => user.id.clone(),
        _ => panic!("Signup should succeed"),
    };

    // Create reset token
    let result = recipe_impl
        .create_reset_password_token(&user_id, &email, "public", &mut ctx)
        .await
        .unwrap();

    match result {
        CreateResetPasswordTokenResult::Ok { token } => {
            assert!(!token.is_empty(), "Token should not be empty");
        }
        CreateResetPasswordTokenResult::UnknownUserId => {
            panic!("Expected Ok, got UnknownUserId");
        }
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_reset_password_token_unknown_user() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();

    let result = recipe_impl
        .create_reset_password_token(
            "nonexistent-user-id",
            "nobody@example.com",
            "public",
            &mut ctx,
        )
        .await
        .unwrap();

    assert!(
        matches!(result, CreateResetPasswordTokenResult::UnknownUserId),
        "Expected UnknownUserId for nonexistent user"
    );

    common::reset();
}

// ---------------------------------------------------------------------------
// Consume Password Reset Token + Full Reset Flow
// (ported from test_passwordreset.py::test_valid_token_input_and_passoword_has_changed)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_full_password_reset_flow() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("fullreset+{}@example.com", uuid::Uuid::new_v4());

    // 1. Sign up
    let signup_result = recipe_impl
        .sign_up(&email, "oldPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    let user_id = match signup_result {
        SignUpResult::Ok { user, .. } => user.id.clone(),
        _ => panic!("Signup should succeed"),
    };

    // 2. Create reset token
    let token = match recipe_impl
        .create_reset_password_token(&user_id, &email, "public", &mut ctx)
        .await
        .unwrap()
    {
        CreateResetPasswordTokenResult::Ok { token } => token,
        _ => panic!("Token creation should succeed"),
    };

    // 3. Consume token
    let consume_result = recipe_impl
        .consume_password_reset_token(&token, "public", &mut ctx)
        .await
        .unwrap();

    match consume_result {
        ConsumePasswordResetTokenResult::Ok {
            email: returned_email,
            user_id: returned_user_id,
        } => {
            assert_eq!(returned_email, email.to_lowercase());
            assert_eq!(returned_user_id, user_id);
        }
        ConsumePasswordResetTokenResult::PasswordResetTokenInvalid => {
            panic!("Expected Ok, got PasswordResetTokenInvalid");
        }
    }

    // 4. Update password using update_email_or_password
    let update_result = recipe_impl
        .update_email_or_password(&user_id, None, Some("newPassword1!"), None, None, &mut ctx)
        .await
        .unwrap();
    assert!(
        matches!(update_result, UpdateEmailOrPasswordResult::Ok),
        "Password update should succeed"
    );

    // 5. Old password should no longer work
    let signin_old = recipe_impl
        .sign_in(&email, "oldPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();
    assert!(
        matches!(signin_old, SignInResult::WrongCredentials),
        "Old password should not work"
    );

    // 6. New password should work
    let signin_new = recipe_impl
        .sign_in(&email, "newPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();
    assert!(
        matches!(signin_new, SignInResult::Ok { .. }),
        "New password should work"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_consume_invalid_reset_token() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();

    let result = recipe_impl
        .consume_password_reset_token("invalid-token", "public", &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(
            result,
            ConsumePasswordResetTokenResult::PasswordResetTokenInvalid
        ),
        "Expected PasswordResetTokenInvalid for invalid token"
    );

    common::reset();
}

// ---------------------------------------------------------------------------
// Update Email or Password
// (ported from test_updateemailorpassword.py)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_update_email() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("updateemail+{}@example.com", uuid::Uuid::new_v4());
    let new_email = format!("newemail+{}@example.com", uuid::Uuid::new_v4());

    // Sign up
    let signup_result = recipe_impl
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    let (_user_id, recipe_user_id) = match signup_result {
        SignUpResult::Ok {
            user,
            recipe_user_id,
        } => (user.id.clone(), recipe_user_id.get_as_string().to_string()),
        _ => panic!("Signup should succeed"),
    };

    // Update email
    let result = recipe_impl
        .update_email_or_password(
            &recipe_user_id,
            Some(&new_email),
            None,
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();

    assert!(matches!(result, UpdateEmailOrPasswordResult::Ok));

    // Old email should not work for sign in
    let signin_old = recipe_impl
        .sign_in(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();
    assert!(matches!(signin_old, SignInResult::WrongCredentials));

    // New email should work
    let signin_new = recipe_impl
        .sign_in(
            &new_email,
            "validPassword1!",
            "public",
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();
    assert!(matches!(signin_new, SignInResult::Ok { .. }));

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_update_password_with_policy_violation() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("policytest+{}@example.com", uuid::Uuid::new_v4());

    let signup_result = recipe_impl
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    let recipe_user_id = match signup_result {
        SignUpResult::Ok { recipe_user_id, .. } => recipe_user_id.get_as_string().to_string(),
        _ => panic!("Signup should succeed"),
    };

    // Update with a weak password and password policy enforcement
    let result = recipe_impl
        .update_email_or_password(
            &recipe_user_id,
            None,
            Some("test"), // too short, no number
            Some(true),   // apply_password_policy
            Some("public"),
            &mut ctx,
        )
        .await
        .unwrap();

    assert!(
        matches!(
            result,
            UpdateEmailOrPasswordResult::PasswordPolicyViolation { .. }
        ),
        "Expected PasswordPolicyViolation, got {:?}",
        result
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_update_unknown_user() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();

    let result = recipe_impl
        .update_email_or_password(
            "nonexistent-user-id",
            None,
            Some("newPassword1!"),
            None,
            None,
            &mut ctx,
        )
        .await
        .unwrap();

    assert!(
        matches!(result, UpdateEmailOrPasswordResult::UnknownUserId),
        "Expected UnknownUserId, got {:?}",
        result
    );

    common::reset();
}

// ---------------------------------------------------------------------------
// Password Reset Link Generation
// (ported from test_passwordreset.py::test_that_generated_password_link_is_correct)
// ---------------------------------------------------------------------------

#[test]
fn test_password_reset_link_format() {
    use supertokens::recipe::emailpassword::utils::get_password_reset_link;

    let app_info = supertokens::AppInfo::from_input(&supertokens::InputAppInfo {
        app_name: "SuperTokens".to_string(),
        api_domain: "http://api.supertokens.io".to_string(),
        website_domain: Some("http://supertokens.io".to_string()),
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: None,
    })
    .unwrap();

    let link = get_password_reset_link(&app_info, "test-token-123", "public");

    assert!(link.contains("supertokens.io"), "Link: {}", link);
    assert!(link.contains("/auth/reset-password"), "Link: {}", link);
    assert!(link.contains("token=test-token-123"), "Link: {}", link);
    assert!(link.contains("tenantId=public"), "Link: {}", link);
}

// ---------------------------------------------------------------------------
// Validator Tests (unit tests, no Core needed)
// (ported from test_updateemailorpassword.py, test_passwordreset.py)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_default_password_validator_valid() {
    let validator = supertokens::recipe::emailpassword::utils::default_password_validator();
    let result = (validator)("validPass1!".to_string(), "public".to_string())
        .await
        .unwrap();
    assert!(result.is_none(), "Valid password should pass");
}

#[tokio::test]
async fn test_default_password_validator_too_short() {
    let validator = supertokens::recipe::emailpassword::utils::default_password_validator();
    let result = (validator)("abc".to_string(), "public".to_string())
        .await
        .unwrap();
    assert!(result.is_some(), "Short password should fail");
    assert!(result.unwrap().contains("8 characters"));
}

#[tokio::test]
async fn test_default_password_validator_no_number() {
    let validator = supertokens::recipe::emailpassword::utils::default_password_validator();
    let result = (validator)("abcdefghi".to_string(), "public".to_string())
        .await
        .unwrap();
    assert!(result.is_some(), "Password without number should fail");
    assert!(result.unwrap().contains("alphabet and one number"));
}

#[tokio::test]
async fn test_default_password_validator_no_letter() {
    let validator = supertokens::recipe::emailpassword::utils::default_password_validator();
    let result = (validator)("123456789".to_string(), "public".to_string())
        .await
        .unwrap();
    assert!(result.is_some(), "Password without letter should fail");
}

#[tokio::test]
async fn test_default_email_validator_valid() {
    let validator = supertokens::recipe::emailpassword::utils::default_email_validator();
    let result = (validator)("test@example.com".to_string(), "public".to_string())
        .await
        .unwrap();
    assert!(result.is_none(), "Valid email should pass");
}

#[tokio::test]
async fn test_default_email_validator_empty() {
    let validator = supertokens::recipe::emailpassword::utils::default_email_validator();
    let result = (validator)("".to_string(), "public".to_string())
        .await
        .unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().contains("empty"));
}

#[tokio::test]
async fn test_default_email_validator_no_at() {
    let validator = supertokens::recipe::emailpassword::utils::default_email_validator();
    let result = (validator)("testexample.com".to_string(), "public".to_string())
        .await
        .unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().contains("not valid"));
}

#[tokio::test]
async fn test_default_email_validator_no_domain_dot() {
    let validator = supertokens::recipe::emailpassword::utils::default_email_validator();
    let result = (validator)("test@example".to_string(), "public".to_string())
        .await
        .unwrap();
    assert!(result.is_some());
}

// ---------------------------------------------------------------------------
// Form Field Normalisation
// (ported from test_signup.py, test_signin.py validation tests)
// ---------------------------------------------------------------------------

#[test]
fn test_normalise_sign_up_form_fields_defaults() {
    let fields = supertokens::recipe::emailpassword::utils::normalise_sign_up_form_fields(None);

    // Should have at least email and password
    assert!(fields.iter().any(|f| f.id == "email"));
    assert!(fields.iter().any(|f| f.id == "password"));

    // Email and password should not be optional
    let email_field = fields.iter().find(|f| f.id == "email").unwrap();
    assert!(!email_field.optional);

    let password_field = fields.iter().find(|f| f.id == "password").unwrap();
    assert!(!password_field.optional);
}

#[test]
fn test_normalise_sign_up_form_fields_with_custom() {
    use supertokens::recipe::emailpassword::types::InputFormField;

    let custom_fields = vec![InputFormField {
        id: "name".to_string(),
        validate: None,
        optional: Some(true),
    }];

    let fields = supertokens::recipe::emailpassword::utils::normalise_sign_up_form_fields(Some(
        custom_fields,
    ));

    // Should have email, password, and name
    assert!(fields.iter().any(|f| f.id == "email"));
    assert!(fields.iter().any(|f| f.id == "password"));
    assert!(fields.iter().any(|f| f.id == "name"));

    let name_field = fields.iter().find(|f| f.id == "name").unwrap();
    assert!(name_field.optional);
}

// ---------------------------------------------------------------------------
// Sign In with Case-Insensitive Email
// (ported from test_emailexists.py::test_sending_an_unnormalised_email)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_signin_case_insensitive_email() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let uuid = uuid::Uuid::new_v4();
    let email = format!("casetest+{}@example.com", uuid);

    // Sign up with lowercase
    recipe_impl
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    // Sign in with mixed case should work (email is normalised)
    let mixed = format!("CaseTest+{}@Example.COM", uuid);
    let result = recipe_impl
        .sign_in(&mixed, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, SignInResult::Ok { .. }),
        "Case-insensitive email should match"
    );

    common::reset();
}

// ---------------------------------------------------------------------------
// Sign Up with User Info Validation
// (ported from test_signup.py)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_signup_returns_correct_user_info() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("userinfo+{}@example.com", uuid::Uuid::new_v4());

    let result = recipe_impl
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    match result {
        SignUpResult::Ok {
            user,
            recipe_user_id,
        } => {
            assert!(!user.id.is_empty());
            assert_eq!(recipe_user_id.get_as_string(), user.id);
            assert!(!user.login_methods.is_empty());

            let login_method = &user.login_methods[0];
            assert_eq!(
                login_method.email.as_deref(),
                Some(email.to_lowercase().as_str())
            );
            assert_eq!(
                login_method.recipe_id,
                supertokens::types::user::RecipeId::EmailPassword
            );
            assert!(login_method.time_joined > 0);
        }
        _ => panic!("Expected Ok"),
    }

    common::reset();
}

// ---------------------------------------------------------------------------
// Sign In Returns Same User ID
// (ported from test_signin.py::test_singinAPI_works_when_input_is_fine)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_signin_returns_same_user_id() {
    common::reset();
    common::init_with_emailpassword().unwrap();

    let recipe_impl = make_recipe_impl();
    let mut ctx = common::new_user_context();
    let email = format!("sameid+{}@example.com", uuid::Uuid::new_v4());

    let signup_result = recipe_impl
        .sign_up(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    let signup_user_id = match signup_result {
        SignUpResult::Ok { user, .. } => user.id.clone(),
        _ => panic!("Signup should succeed"),
    };

    let signin_result = recipe_impl
        .sign_in(&email, "validPassword1!", "public", None, None, &mut ctx)
        .await
        .unwrap();

    match signin_result {
        SignInResult::Ok { user, .. } => {
            assert_eq!(user.id, signup_user_id, "User IDs should match");
        }
        _ => panic!("Signin should succeed"),
    }

    common::reset();
}
