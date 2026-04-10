mod common;

use serial_test::serial;

use supertokens::querier::Querier;

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
