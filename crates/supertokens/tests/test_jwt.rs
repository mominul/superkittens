mod common;

use serial_test::serial;
use supertokens::querier::Querier;
use supertokens::recipe::jwt::interfaces::RecipeInterface;
use supertokens::recipe::jwt::recipe_implementation::RecipeImplementationImpl;
use supertokens::recipe::jwt::types::*;

fn make_jwt_impl() -> RecipeImplementationImpl {
    let querier = Querier::get_instance(Some("jwt".to_string())).unwrap();
    RecipeImplementationImpl {
        querier,
        jwks_domain: "http://api.supertokens.io".to_string(),
    }
}

#[tokio::test]
#[serial]
#[ignore = "requires fix: /recipe/jwt endpoint changed in CDI 5.4"]
async fn test_create_jwt_with_default_validity() {
    common::reset();
    common::init_with_session().unwrap();

    let jwt_impl = make_jwt_impl();
    let payload = serde_json::Map::new();
    let mut ctx = common::new_user_context();

    let result = jwt_impl
        .create_jwt(&payload, None, None, &mut ctx)
        .await
        .unwrap();

    match result {
        CreateJwtResult::Ok { jwt } => {
            assert!(!jwt.is_empty(), "JWT should be non-empty");
        }
        CreateJwtResult::UnsupportedAlgorithm => {
            panic!("Expected Ok result, got UnsupportedAlgorithm");
        }
    }
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_jwt_with_custom_validity() {
    common::reset();
    common::init_with_session().unwrap();

    let jwt_impl = make_jwt_impl();
    let payload = serde_json::Map::new();
    let mut ctx = common::new_user_context();

    let result = jwt_impl
        .create_jwt(&payload, Some(3600), None, &mut ctx)
        .await
        .unwrap();

    match result {
        CreateJwtResult::Ok { jwt } => {
            assert!(!jwt.is_empty(), "JWT should be non-empty");
        }
        CreateJwtResult::UnsupportedAlgorithm => {
            panic!("Expected Ok result, got UnsupportedAlgorithm");
        }
    }
}

#[tokio::test]
#[serial]
#[ignore = "requires fix: /recipe/jwt endpoint changed in CDI 5.4"]
async fn test_create_jwt_with_payload() {
    common::reset();
    common::init_with_session().unwrap();

    let jwt_impl = make_jwt_impl();
    let mut payload = serde_json::Map::new();
    payload.insert("key".to_string(), serde_json::Value::String("value".to_string()));
    payload.insert("num".to_string(), serde_json::json!(42));
    let mut ctx = common::new_user_context();

    let result = jwt_impl
        .create_jwt(&payload, None, None, &mut ctx)
        .await
        .unwrap();

    match result {
        CreateJwtResult::Ok { jwt } => {
            assert!(!jwt.is_empty(), "JWT should be non-empty");
            // Verify the JWT has three dot-separated parts (header.payload.signature)
            let parts: Vec<&str> = jwt.split('.').collect();
            assert_eq!(parts.len(), 3, "JWT should have three parts");
        }
        CreateJwtResult::UnsupportedAlgorithm => {
            panic!("Expected Ok result, got UnsupportedAlgorithm");
        }
    }
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_jwks() {
    common::reset();
    common::init_with_session().unwrap();

    let jwt_impl = make_jwt_impl();
    let mut ctx = common::new_user_context();

    let result = jwt_impl.get_jwks(&mut ctx).await.unwrap();

    assert!(!result.keys.is_empty(), "JWKS keys should be non-empty");

    for key in &result.keys {
        assert!(!key.kty.is_empty(), "Key kty should be non-empty");
        assert!(!key.kid.is_empty(), "Key kid should be non-empty");
        assert!(!key.n.is_empty(), "Key n should be non-empty");
        assert!(!key.e.is_empty(), "Key e should be non-empty");
        assert!(!key.alg.is_empty(), "Key alg should be non-empty");
    }
}
