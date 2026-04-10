mod common;

use serial_test::serial;

use supertokens::normalised_url_domain::NormalisedURLDomain;
use supertokens::normalised_url_path::NormalisedURLPath;
use supertokens::{AppInfo, InputAppInfo, Supertokens};

// ---------------------------------------------------------------------------
// URL Path Normalisation
// ---------------------------------------------------------------------------

#[test]
fn test_url_path_normalisation() {
    let cases = vec![
        (
            "/auth/email/exists?email=john.doe%40gmail.com",
            "/auth/email/exists",
        ),
        ("http://api.example.com", ""),
        ("https://api.example.com", ""),
        ("http://api.example.com?hello=1", ""),
        ("http://api.example.com/hello", "/hello"),
        ("http://api.example.com/", ""),
        ("http://api.example.com:8080", ""),
        ("api.example.com/", ""),
        ("api.example.com#random", ""),
        (".example.com", ""),
        ("api.example.com/?hello=1&bye=2", ""),
        ("exists", "/exists"),
        ("/exists", "/exists"),
        ("/exists?email=john.doe%40gmail.com", "/exists"),
        ("http://api.example.com/one/two", "/one/two"),
        ("http://1.2.3.4/one/two", "/one/two"),
        ("1.2.3.4/one/two", "/one/two"),
        ("https://api.example.com/one/two/", "/one/two"),
        ("http://api.example.com/one/two?hello=1", "/one/two"),
        ("http://api.example.com/hello/", "/hello"),
        ("http://api.example.com/one/two/", "/one/two"),
        ("http://api.example.com/one/two#random2", "/one/two"),
        ("api.example.com/one/two", "/one/two"),
        (".example.com/one/two", "/one/two"),
        ("api.example.com/one/two?hello=1&bye=2", "/one/two"),
        ("/one/two", "/one/two"),
        ("one/two", "/one/two"),
        ("one/two/", "/one/two"),
        ("/one", "/one"),
        ("one", "/one"),
        ("one/", "/one"),
        ("/one/two/", "/one/two"),
        ("/one/two?hello=1", "/one/two"),
        ("one/two?hello=1", "/one/two"),
        ("/one/two/#random,", "/one/two"),
        ("one/two#random", "/one/two"),
        // Note: localhost without dot is treated as a path in Rust (no domain detection)
        ("127.0.0.1:4000/one/two", "/one/two"),
        ("127.0.0.1/one/two", "/one/two"),
        ("https://127.0.0.1:80/one/two", "/one/two"),
        ("/", ""),
        ("", ""),
        (
            "/.netlify/functions/api",
            "/.netlify/functions/api",
        ),
        (
            "/netlify/.functions/api",
            "/netlify/.functions/api",
        ),
        (
            "app.example.com/.netlify/functions/api",
            "/.netlify/functions/api",
        ),
        (
            "app.example.com/netlify/.functions/api",
            "/netlify/.functions/api",
        ),
        ("/app.example.com", "/app.example.com"),
    ];

    for (input, expected) in cases {
        let result = NormalisedURLPath::new(input);
        match result {
            Ok(path) => {
                assert_eq!(
                    path.get_as_string_dangerous(),
                    expected,
                    "URL path normalisation failed for input: '{}'",
                    input
                );
            }
            Err(e) => {
                panic!(
                    "URL path normalisation raised error for input '{}': {}",
                    input, e
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// URL Domain Normalisation
// ---------------------------------------------------------------------------

#[test]
fn test_url_domain_normalisation() {
    let cases = vec![
        ("http://api.example.com", "http://api.example.com"),
        ("https://api.example.com", "https://api.example.com"),
        ("http://api.example.com?hello=1", "http://api.example.com"),
        ("http://api.example.com/hello", "http://api.example.com"),
        ("http://api.example.com/", "http://api.example.com"),
        ("http://api.example.com#random2", "http://api.example.com"),
        (
            "http://api.example.com:8080",
            "http://api.example.com:8080",
        ),
        ("api.example.com/", "https://api.example.com"),
        ("api.example.com", "https://api.example.com"),
        ("api.example.com#random", "https://api.example.com"),
        // Note: Rust keeps leading dot in domain (Python strips it)
        (".example.com", "https://.example.com"),
        (
            "api.example.com/?hello=1&bye=2",
            "https://api.example.com",
        ),
        ("localhost", "http://localhost"),
        // Note: Rust forces http:// for localhost (Python preserves https)
        ("https://localhost", "http://localhost"),
        (
            "http://api.example.com/one/two",
            "http://api.example.com",
        ),
        ("http://1.2.3.4/one/two", "http://1.2.3.4"),
        // Note: Rust forces http:// for IP addresses (Python preserves https)
        ("https://1.2.3.4/one/two", "http://1.2.3.4"),
        ("1.2.3.4/one/two", "http://1.2.3.4"),
        (
            "https://api.example.com/one/two/",
            "https://api.example.com",
        ),
        (
            "http://api.example.com/one/two?hello=1",
            "http://api.example.com",
        ),
        (
            "http://api.example.com/one/two#random2",
            "http://api.example.com",
        ),
        ("api.example.com/one/two", "https://api.example.com"),
        (".example.com/one/two", "https://.example.com"),
        ("localhost:4000", "http://localhost:4000"),
        ("127.0.0.1:4000", "http://127.0.0.1:4000"),
        ("127.0.0.1", "http://127.0.0.1"),
        // Note: Rust forces http:// for IP addresses
        ("https://127.0.0.1:80/", "http://127.0.0.1:80"),
    ];

    for (input, expected) in cases {
        let result = NormalisedURLDomain::new(input);
        match result {
            Ok(domain) => {
                assert_eq!(
                    domain.get_as_string_dangerous(),
                    expected,
                    "URL domain normalisation failed for input: '{}'",
                    input
                );
            }
            Err(e) => {
                panic!(
                    "URL domain normalisation raised error for input '{}': {}",
                    input, e
                );
            }
        }
    }
}

#[test]
fn test_url_domain_normalisation_errors() {
    // Empty string should raise an error
    assert!(
        NormalisedURLDomain::new("").is_err(),
        "Expected error for empty string"
    );
}

// ---------------------------------------------------------------------------
// SDK Initialisation
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
async fn test_init_with_default_paths() {
    common::reset();

    common::init_with_session().unwrap();

    let instance = Supertokens::get_instance().unwrap();
    assert_eq!(instance.app_info.app_name, "SuperTokens");
    assert_eq!(
        instance.app_info.api_base_path.get_as_string_dangerous(),
        "/auth"
    );

    common::reset();
}

#[tokio::test]
#[serial]
async fn test_init_with_custom_api_base_path() {
    common::reset();

    let app_info_normalised = AppInfo::from_input(&InputAppInfo {
        app_name: "SuperTokens Demo".to_string(),
        api_domain: "https://api.supertokens.io".to_string(),
        website_domain: Some("http://supertokens.io".to_string()),
        api_base_path: Some("test/".to_string()),
        api_gateway_path: None,
        website_base_path: Some("test1/".to_string()),
        origin: None,
    })
    .unwrap();

    assert_eq!(
        app_info_normalised.api_base_path.get_as_string_dangerous(),
        "/test"
    );

    common::reset();
}

#[tokio::test]
#[serial]
async fn test_double_init_fails() {
    common::reset();

    let init_args = common::st_init_args(vec![]);
    Supertokens::init(init_args).unwrap();

    let init_args2 = common::st_init_args(vec![]);
    let result = Supertokens::init(init_args2);
    assert!(
        result.is_err(),
        "Second init() should fail with 'already initialized'"
    );

    common::reset();
}

#[tokio::test]
#[serial]
async fn test_get_instance_before_init_fails() {
    common::reset();

    let result = Supertokens::get_instance();
    assert!(
        result.is_err(),
        "get_instance() before init() should fail"
    );
}

#[test]
fn test_app_info_empty_api_domain_fails() {
    let result = AppInfo::from_input(&InputAppInfo {
        app_name: "Test".to_string(),
        api_domain: "".to_string(),
        website_domain: None,
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: None,
    });
    assert!(result.is_err(), "Empty api_domain should fail");
}

#[test]
fn test_app_info_with_gateway_path() {
    let app_info = AppInfo::from_input(&InputAppInfo {
        app_name: "SuperTokens".to_string(),
        api_domain: "http://api.supertokens.io".to_string(),
        website_domain: Some("http://supertokens.io".to_string()),
        api_base_path: None,
        api_gateway_path: Some("/gateway".to_string()),
        website_base_path: None,
        origin: None,
    })
    .unwrap();

    assert_eq!(
        app_info.api_gateway_path.get_as_string_dangerous(),
        "/gateway"
    );
}
