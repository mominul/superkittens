mod common;

use serial_test::serial;
use supertokens::querier::Querier;
use supertokens::recipe::usermetadata::interfaces::RecipeInterface;
use supertokens::recipe::usermetadata::recipe_implementation::RecipeImplementationImpl;

fn make_metadata_impl() -> RecipeImplementationImpl {
    let querier = Querier::get_instance(Some("usermetadata".to_string())).unwrap();
    RecipeImplementationImpl { querier }
}

// ===========================================================================
// Get Empty Metadata
// (ported from test_metadata.py::test_get_empty_metadata)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_empty_metadata() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_metadata_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe.get_user_metadata(&user_id, &mut ctx).await.unwrap();

    assert!(
        result.metadata.is_empty(),
        "Metadata for non-existent user should be empty"
    );

    common::reset();
}

// ===========================================================================
// Update and Get Metadata
// (ported from test_metadata.py::test_update_and_get_metadata)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_update_and_get_metadata() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_metadata_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();

    let mut metadata_update = serde_json::Map::new();
    metadata_update.insert(
        "name".to_string(),
        serde_json::Value::String("test".to_string()),
    );
    metadata_update.insert("age".to_string(), serde_json::json!(25));

    let update_result = recipe
        .update_user_metadata(&user_id, &metadata_update, &mut ctx)
        .await
        .unwrap();

    assert_eq!(update_result.metadata.get("name").unwrap(), "test");
    assert_eq!(update_result.metadata.get("age").unwrap(), 25);

    // Get and verify
    let get_result = recipe.get_user_metadata(&user_id, &mut ctx).await.unwrap();

    assert_eq!(get_result.metadata.get("name").unwrap(), "test");
    assert_eq!(get_result.metadata.get("age").unwrap(), 25);

    common::reset();
}

// ===========================================================================
// Metadata Shallow Merge
// (ported from test_metadata.py::test_metadata_shallow_merge)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_metadata_shallow_merge() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_metadata_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();

    // First update: set "a" = "1"
    let mut update1 = serde_json::Map::new();
    update1.insert("a".to_string(), serde_json::Value::String("1".to_string()));
    recipe
        .update_user_metadata(&user_id, &update1, &mut ctx)
        .await
        .unwrap();

    // Second update: set "b" = "2"
    let mut update2 = serde_json::Map::new();
    update2.insert("b".to_string(), serde_json::Value::String("2".to_string()));
    recipe
        .update_user_metadata(&user_id, &update2, &mut ctx)
        .await
        .unwrap();

    // Verify both "a" and "b" exist
    let result = recipe.get_user_metadata(&user_id, &mut ctx).await.unwrap();

    assert_eq!(result.metadata.get("a").unwrap(), "1");
    assert_eq!(result.metadata.get("b").unwrap(), "2");

    // Third update: change "a" to "3"
    let mut update3 = serde_json::Map::new();
    update3.insert("a".to_string(), serde_json::Value::String("3".to_string()));
    recipe
        .update_user_metadata(&user_id, &update3, &mut ctx)
        .await
        .unwrap();

    // Verify "a" changed but "b" remains
    let result2 = recipe.get_user_metadata(&user_id, &mut ctx).await.unwrap();

    assert_eq!(
        result2.metadata.get("a").unwrap(),
        "3",
        "\"a\" should be updated to \"3\""
    );
    assert_eq!(
        result2.metadata.get("b").unwrap(),
        "2",
        "\"b\" should remain unchanged"
    );

    common::reset();
}

// ===========================================================================
// Clear Metadata
// (ported from test_metadata.py::test_clear_metadata)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_clear_metadata() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_metadata_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();

    // Set some metadata
    let mut metadata_update = serde_json::Map::new();
    metadata_update.insert(
        "name".to_string(),
        serde_json::Value::String("test".to_string()),
    );
    metadata_update.insert("age".to_string(), serde_json::json!(25));

    recipe
        .update_user_metadata(&user_id, &metadata_update, &mut ctx)
        .await
        .unwrap();

    // Verify metadata was set
    let before_clear = recipe.get_user_metadata(&user_id, &mut ctx).await.unwrap();
    assert!(
        !before_clear.metadata.is_empty(),
        "Metadata should not be empty before clear"
    );

    // Clear metadata
    recipe
        .clear_user_metadata(&user_id, &mut ctx)
        .await
        .unwrap();

    // Verify metadata is now empty
    let after_clear = recipe.get_user_metadata(&user_id, &mut ctx).await.unwrap();

    assert!(
        after_clear.metadata.is_empty(),
        "Metadata should be empty after clearing"
    );

    common::reset();
}
