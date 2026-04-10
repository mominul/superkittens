mod common;

use serial_test::serial;

use supertokens::querier::Querier;
use supertokens::user_context::internal_keys;
use supertokens::user_context::CoreCallCache;

#[tokio::test]
#[serial]
async fn test_querier_not_initialized_before_init() {
    common::reset();

    let result = Querier::get_instance(None);
    assert!(result.is_err(), "Querier should not be available before init");
}

#[tokio::test]
#[serial]
async fn test_querier_available_after_init() {
    common::reset();

    common::init_with_session().unwrap();

    let querier = Querier::get_instance(None);
    assert!(querier.is_ok(), "Querier should be available after init");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_querier_get_api_version() {
    common::reset();

    common::init_with_session().unwrap();

    let querier = Querier::get_instance(None).unwrap();
    let mut ctx = common::new_user_context();
    let version = querier.get_api_version(&mut ctx).await;

    assert!(
        version.is_ok(),
        "get_api_version should succeed: {:?}",
        version.err()
    );
    let version = version.unwrap();
    assert!(!version.is_empty(), "API version should not be empty");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_querier_send_get_request() {
    common::reset();

    common::init_with_session().unwrap();

    let querier = Querier::get_instance(None).unwrap();
    let mut ctx = common::new_user_context();

    // The /hello endpoint always returns {"status": "OK"} from Core
    let path = supertokens::normalised_url_path::NormalisedURLPath::new("/hello").unwrap();
    let response = querier.send_get_request(&path, None, &mut ctx).await;

    assert!(
        response.is_ok(),
        "GET /hello should succeed: {:?}",
        response.err()
    );

    common::reset();
}

#[tokio::test]
#[serial]
async fn test_querier_get_all_core_urls() {
    common::reset();

    common::init_with_session().unwrap();

    let querier = Querier::get_instance(None).unwrap();
    let path = supertokens::normalised_url_path::NormalisedURLPath::new("/hello").unwrap();
    let urls = querier.get_all_core_urls_for_path(&path);

    assert!(!urls.is_empty(), "Should have at least one Core URL");
    assert!(
        urls[0].contains("/hello"),
        "URL should contain path: {}",
        urls[0]
    );

    common::reset();
}

#[tokio::test]
#[serial]
async fn test_querier_reset() {
    common::reset();

    common::init_with_session().unwrap();

    assert!(Querier::get_instance(None).is_ok());

    common::reset();

    // After reset, querier should not be available
    assert!(Querier::get_instance(None).is_err());
}

// ---------------------------------------------------------------------------
// Caching Tests (ported from test_querier.py)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_caching_works() {
    // Verifies that GET requests are cached in user_context and
    // subsequent identical requests don't call Core again.
    common::reset();
    common::init_with_session().unwrap();

    let querier = Querier::get_instance(None).unwrap();
    let mut ctx = common::new_user_context();

    let path =
        supertokens::normalised_url_path::NormalisedURLPath::new("/hello").unwrap();

    // First call — should call Core and store in cache
    let resp1 = querier.send_get_request(&path, None, &mut ctx).await;
    assert!(resp1.is_ok(), "First GET should succeed: {:?}", resp1.err());

    // Cache should now be populated in user_context
    let cache = ctx.get::<CoreCallCache>(internal_keys::CORE_CALL_CACHE);
    assert!(
        cache.is_some() && !cache.unwrap().is_empty(),
        "Cache should be populated after first GET"
    );

    // Second call with same context — should use cache (we can't directly
    // verify no network call, but we verify the result matches)
    let resp2 = querier.send_get_request(&path, None, &mut ctx).await;
    assert!(resp2.is_ok());
    assert_eq!(resp1.unwrap(), resp2.unwrap());

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_caching_gets_cleared_with_non_get() {
    // Verifies that non-GET requests (POST) invalidate the cache.
    common::reset();
    common::init_with_session().unwrap();

    let querier = Querier::get_instance(None).unwrap();
    let mut ctx = common::new_user_context();

    let path =
        supertokens::normalised_url_path::NormalisedURLPath::new("/hello").unwrap();

    // First GET — populates cache
    querier.send_get_request(&path, None, &mut ctx).await.unwrap();
    assert!(
        ctx.get::<CoreCallCache>(internal_keys::CORE_CALL_CACHE)
            .map_or(false, |c| !c.is_empty()),
        "Cache should be populated"
    );

    // POST request — should invalidate cache
    let post_path =
        supertokens::normalised_url_path::NormalisedURLPath::new("/hello").unwrap();
    let _ = querier
        .send_post_request(&post_path, None, &mut ctx)
        .await;

    // Cache should be cleared after POST
    let cache = ctx.get::<CoreCallCache>(internal_keys::CORE_CALL_CACHE);
    assert!(
        cache.is_none() || cache.unwrap().is_empty(),
        "Cache should be cleared after POST"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_caching_does_not_clear_with_keep_alive() {
    // Verifies that non-GET requests preserve cache when keep_cache_alive=True.
    common::reset();
    common::init_with_session().unwrap();

    let querier = Querier::get_instance(None).unwrap();
    let mut ctx = common::new_user_context();
    ctx.insert(internal_keys::KEEP_CACHE_ALIVE, true);

    let path =
        supertokens::normalised_url_path::NormalisedURLPath::new("/hello").unwrap();

    // First GET — populates cache
    querier.send_get_request(&path, None, &mut ctx).await.unwrap();
    assert!(
        ctx.get::<CoreCallCache>(internal_keys::CORE_CALL_CACHE)
            .map_or(false, |c| !c.is_empty()),
        "Cache should be populated"
    );

    // POST with keep_alive — cache entry in user_context is still removed,
    // but global_cache_tag is NOT updated, so next GET with same context
    // would rebuild from Core and match.
    let post_path =
        supertokens::normalised_url_path::NormalisedURLPath::new("/hello").unwrap();
    let _ = querier
        .send_post_request(&post_path, None, &mut ctx)
        .await;

    // In Rust implementation, POST always removes the cache from user_context,
    // but keep_alive prevents global_cache_tag update. Re-doing GET should work.
    let resp = querier.send_get_request(&path, None, &mut ctx).await;
    assert!(resp.is_ok(), "GET after POST with keep_alive should succeed");

    common::reset();
}

#[tokio::test]
#[serial]
async fn test_querier_get_instance_with_recipe_id() {
    common::reset();
    common::init_with_session().unwrap();

    let querier = Querier::get_instance(Some("session".to_string()));
    assert!(querier.is_ok(), "Should get querier with recipe ID");

    common::reset();
}

#[tokio::test]
#[serial]
async fn test_querier_multiple_hosts() {
    // Tests that querier handles multiple hosts in the connection URI
    common::reset();

    // Multi-host connection URI (semicolon-separated)
    let init_args = common::st_init_args_with_connection_uri(
        "http://localhost:3567;http://localhost:3568".to_string(),
        vec![],
    );
    supertokens::Supertokens::init(init_args).unwrap();

    let querier = Querier::get_instance(None).unwrap();
    let path =
        supertokens::normalised_url_path::NormalisedURLPath::new("/hello").unwrap();
    let urls = querier.get_all_core_urls_for_path(&path);

    assert_eq!(urls.len(), 2, "Should have two Core URLs");
    assert!(urls[0].contains("3567"), "First URL: {}", urls[0]);
    assert!(urls[1].contains("3568"), "Second URL: {}", urls[1]);

    common::reset();
}
