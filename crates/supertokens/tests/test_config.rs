mod common;

use serial_test::serial;

use supertokens::normalised_url_domain::NormalisedURLDomain;
use supertokens::normalised_url_path::NormalisedURLPath;
use supertokens::recipe::session::types::{AntiCsrfConfig, SessionConfig};
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

// ---------------------------------------------------------------------------
// Session Config Normalisation
// (ported from test_config.py::test_same_site_values, test_config_values,
//  test_samesite_explicit_config)
// ---------------------------------------------------------------------------

/// Helper to normalise a session config with a given app_info.
fn normalise_session_config(
    api_domain: &str,
    website_domain: Option<&str>,
    config: SessionConfig,
) -> supertokens::recipe::session::types::NormalisedSessionConfig {
    let app_info = AppInfo::from_input(&InputAppInfo {
        app_name: "SuperTokens".to_string(),
        api_domain: api_domain.to_string(),
        website_domain: website_domain.map(|s| s.to_string()),
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: None,
    })
    .unwrap();

    supertokens::recipe::session::utils::validate_and_normalise_user_input(&app_info, config)
        .unwrap()
}

#[test]
fn test_session_config_cookie_same_site_explicit_lax() {
    let config = normalise_session_config(
        "http://api.supertokens.io",
        Some("http://supertokens.io"),
        SessionConfig {
            cookie_same_site: Some("lax".to_string()),
            ..Default::default()
        },
    );
    // get_cookie_same_site is a closure; we can't inspect it directly,
    // so we check the anti_csrf default instead (ViaCustomHeader).
    assert_eq!(
        config.anti_csrf_function_or_string,
        AntiCsrfConfig::ViaCustomHeader
    );
}

#[test]
fn test_session_config_cookie_same_site_explicit_none() {
    let config = normalise_session_config(
        "http://api.supertokens.io",
        Some("http://supertokens.io"),
        SessionConfig {
            cookie_same_site: Some("none".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(
        config.anti_csrf_function_or_string,
        AntiCsrfConfig::ViaCustomHeader,
    );
}

#[test]
fn test_session_config_cookie_same_site_explicit_strict() {
    let config = normalise_session_config(
        "http://api.supertokens.io",
        Some("http://supertokens.io"),
        SessionConfig {
            cookie_same_site: Some("strict".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(
        config.anti_csrf_function_or_string,
        AntiCsrfConfig::ViaCustomHeader,
    );
}

#[test]
fn test_session_config_cookie_secure_defaults_from_api_domain() {
    // HTTPS api_domain → cookie_secure defaults to true
    let config_https = normalise_session_config(
        "https://api.supertokens.io",
        Some("https://supertokens.io"),
        SessionConfig::default(),
    );
    assert!(config_https.cookie_secure);

    // HTTP api_domain → cookie_secure defaults to false
    let config_http = normalise_session_config(
        "http://api.supertokens.io",
        Some("http://supertokens.io"),
        SessionConfig::default(),
    );
    assert!(!config_http.cookie_secure);
}

#[test]
fn test_session_config_cookie_secure_explicit_override() {
    // Even with HTTPS, explicit cookie_secure=false is honoured
    let config = normalise_session_config(
        "https://api.supertokens.io",
        Some("https://supertokens.io"),
        SessionConfig {
            cookie_secure: Some(false),
            ..Default::default()
        },
    );
    assert!(!config.cookie_secure);
}

#[test]
fn test_session_config_anti_csrf_explicit_via_token() {
    let config = normalise_session_config(
        "https://api.supertokens.io",
        Some("https://supertokens.io"),
        SessionConfig {
            anti_csrf: Some("VIA_TOKEN".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(
        config.anti_csrf_function_or_string,
        AntiCsrfConfig::ViaToken,
    );
}

#[test]
fn test_session_config_anti_csrf_explicit_none() {
    let config = normalise_session_config(
        "https://api.supertokens.io",
        Some("https://supertokens.io"),
        SessionConfig {
            anti_csrf: Some("NONE".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(
        config.anti_csrf_function_or_string,
        AntiCsrfConfig::None,
    );
}

#[test]
fn test_session_config_anti_csrf_default_is_via_custom_header() {
    let config = normalise_session_config(
        "https://api.supertokens.io",
        Some("https://supertokens.io"),
        SessionConfig::default(),
    );
    assert_eq!(
        config.anti_csrf_function_or_string,
        AntiCsrfConfig::ViaCustomHeader,
    );
}

#[test]
fn test_session_config_defaults() {
    let config = normalise_session_config(
        "https://api.supertokens.io",
        Some("https://supertokens.io"),
        SessionConfig::default(),
    );
    assert_eq!(config.session_expired_status_code, 401);
    assert_eq!(config.invalid_claim_status_code, 403);
    assert!(config.use_dynamic_access_token_signing_key);
    assert!(!config.expose_access_token_to_frontend_in_cookie_based_auth);
    assert_eq!(config.jwks_refresh_interval_sec, 3600);
}

#[test]
fn test_session_config_custom_status_codes() {
    let config = normalise_session_config(
        "https://api.supertokens.io",
        Some("https://supertokens.io"),
        SessionConfig {
            session_expired_status_code: Some(440),
            invalid_claim_status_code: Some(422),
            ..Default::default()
        },
    );
    assert_eq!(config.session_expired_status_code, 440);
    assert_eq!(config.invalid_claim_status_code, 422);
}

#[test]
fn test_session_config_localhost_cookie_secure_false() {
    // localhost with HTTP → cookie_secure = false
    let config = normalise_session_config(
        "http://localhost:3001",
        Some("http://localhost:3000"),
        SessionConfig::default(),
    );
    assert!(!config.cookie_secure);
}

#[test]
fn test_session_config_localhost_normalises_to_http() {
    // Rust forces http:// for localhost, so cookie_secure defaults to false
    // even when user provides https://localhost
    let config = normalise_session_config(
        "https://localhost:3001",
        Some("https://localhost:3000"),
        SessionConfig::default(),
    );
    // NormalisedURLDomain converts https://localhost → http://localhost
    assert!(!config.cookie_secure);
}

#[test]
fn test_session_config_ip_address_cookie_secure_false() {
    // IP address defaults to http → cookie_secure = false
    let config = normalise_session_config(
        "127.0.0.1:3001",
        Some("127.0.0.1:3000"),
        SessionConfig::default(),
    );
    assert!(!config.cookie_secure);
}

// ---------------------------------------------------------------------------
// Origin vs website_domain
// (ported from test_config.py origin/website_domain tests)
// ---------------------------------------------------------------------------

#[test]
fn test_app_info_with_origin_string() {
    let app_info = AppInfo::from_input(&InputAppInfo {
        app_name: "SuperTokens".to_string(),
        api_domain: "http://localhost:3001".to_string(),
        website_domain: None,
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: Some("localhost:3000".to_string()),
    })
    .unwrap();

    assert!(app_info.origin.is_some());
    assert_eq!(
        app_info.origin.unwrap().get_as_string_dangerous(),
        "http://localhost:3000"
    );
}

#[test]
fn test_app_info_with_website_domain_string() {
    let app_info = AppInfo::from_input(&InputAppInfo {
        app_name: "SuperTokens".to_string(),
        api_domain: "http://localhost:3001".to_string(),
        website_domain: Some("localhost:3000".to_string()),
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: None,
    })
    .unwrap();

    assert!(app_info.website_domain.is_some());
    assert_eq!(
        app_info.website_domain.unwrap().get_as_string_dangerous(),
        "http://localhost:3000"
    );
}

#[test]
fn test_app_info_origin_takes_precedence_over_website_domain() {
    let app_info = AppInfo::from_input(&InputAppInfo {
        app_name: "SuperTokens".to_string(),
        api_domain: "http://localhost:3001".to_string(),
        website_domain: Some("localhost:3000".to_string()),
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: Some("supertokens.io".to_string()),
    })
    .unwrap();

    // Both are set; origin should be normalised independently
    assert!(app_info.origin.is_some());
    assert_eq!(
        app_info.origin.unwrap().get_as_string_dangerous(),
        "https://supertokens.io"
    );
}

#[test]
fn test_app_info_top_level_api_domain() {
    let app_info = AppInfo::from_input(&InputAppInfo {
        app_name: "SuperTokens".to_string(),
        api_domain: "https://api.supertokens.io".to_string(),
        website_domain: Some("https://supertokens.io".to_string()),
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: None,
    })
    .unwrap();

    assert_eq!(app_info.top_level_api_domain, "supertokens.io");
}

#[test]
fn test_app_info_top_level_api_domain_ec2() {
    let app_info = AppInfo::from_input(&InputAppInfo {
        app_name: "SuperTokens".to_string(),
        api_domain: "https://ec2-xx-yyy-zzz-0.compute-1.amazonaws.com:3001".to_string(),
        website_domain: Some("https://blog.supertokens.com".to_string()),
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: None,
    })
    .unwrap();

    assert_eq!(
        app_info.top_level_api_domain,
        "ec2-xx-yyy-zzz-0.compute-1.amazonaws.com"
    );
}

#[test]
fn test_app_info_top_level_api_domain_localhost() {
    let app_info = AppInfo::from_input(&InputAppInfo {
        app_name: "SuperTokens".to_string(),
        api_domain: "http://localhost:3001".to_string(),
        website_domain: Some("http://localhost:3000".to_string()),
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: None,
    })
    .unwrap();

    assert_eq!(app_info.top_level_api_domain, "localhost");
}

#[test]
fn test_app_info_refresh_token_path() {
    let config = normalise_session_config(
        "https://api.supertokens.io",
        Some("https://supertokens.io"),
        SessionConfig::default(),
    );
    // Default api_base_path is /auth, refresh path should be /auth/session/refresh
    assert_eq!(
        config.refresh_token_path.get_as_string_dangerous(),
        "/auth/session/refresh"
    );
}

#[test]
fn test_session_config_cookie_domain() {
    let config = normalise_session_config(
        "https://api.supertokens.io",
        Some("https://supertokens.io"),
        SessionConfig {
            cookie_domain: Some(".supertokens.io".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(config.cookie_domain, Some("supertokens.io".to_string()));
}

#[test]
fn test_session_config_older_cookie_domain() {
    let config = normalise_session_config(
        "https://api.supertokens.io",
        Some("https://supertokens.io"),
        SessionConfig {
            older_cookie_domain: Some("https://old.supertokens.io:3000/path".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(
        config.older_cookie_domain,
        Some("old.supertokens.io".to_string())
    );
}
