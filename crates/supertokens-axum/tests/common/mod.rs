#![allow(dead_code)]

use std::sync::Arc;

use supertokens::querier::Querier;
use supertokens::recipe_module::RecipeModule;
use supertokens::user_context::UserContext;
use supertokens::{InputAppInfo, SupertokensConfig, SupertokensInit, Supertokens};

/// Returns the SuperTokens Core URL from env vars, defaulting to http://localhost:3567.
pub fn core_url() -> String {
    let host =
        std::env::var("SUPERTOKENS_CORE_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port =
        std::env::var("SUPERTOKENS_CORE_PORT").unwrap_or_else(|_| "3567".to_string());
    format!("http://{}:{}", host, port)
}

/// Reset all singletons between tests.
pub fn reset() {
    Supertokens::reset();
    Querier::reset();
    supertokens::recipe_module::reset_get_tenant_id();
}

/// Standard InputAppInfo for tests.
pub fn test_app_info() -> InputAppInfo {
    InputAppInfo {
        app_name: "SuperTokens".to_string(),
        api_domain: "http://api.supertokens.io".to_string(),
        website_domain: Some("http://supertokens.io".to_string()),
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: None,
    }
}

/// Build a SupertokensInit config with the given recipe list, using the test Core.
pub fn st_init_args(recipe_list: Vec<Arc<dyn RecipeModule>>) -> SupertokensInit {
    SupertokensInit {
        app_info: test_app_info(),
        supertokens_config: SupertokensConfig {
            connection_uri: core_url(),
            api_key: None,
            network_interceptor: None,
            disable_core_call_cache: false,
        },
        recipe_list,
        telemetry: Some(false),
        debug: false,
    }
}

/// Parse connection URI into Hosts (mirrors internal SDK logic).
fn parse_connection_uri(
    uri: &str,
) -> std::result::Result<Vec<supertokens::types::config::Host>, supertokens::SuperTokensError> {
    let mut hosts = Vec::new();
    for part in uri.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let domain = supertokens::normalised_url_domain::NormalisedURLDomain::new(part)?;
        let base_path = supertokens::normalised_url_path::NormalisedURLPath::new(part)?;
        hosts.push(supertokens::types::config::Host { domain, base_path });
    }
    Ok(hosts)
}

/// Initialize the SDK with a session recipe using default config.
pub fn init_with_session() -> std::result::Result<(), supertokens::SuperTokensError> {
    init_with_session_config(supertokens::recipe::session::types::SessionConfig::default())
}

/// Initialize the SDK with a session recipe and custom session config.
pub fn init_with_session_config(
    session_config: supertokens::recipe::session::types::SessionConfig,
) -> std::result::Result<(), supertokens::SuperTokensError> {
    let app_info_input = test_app_info();
    let connection_uri = core_url();

    // Parse hosts and init Querier first, since SessionRecipe::new needs it
    let hosts = parse_connection_uri(&connection_uri)?;
    Querier::init(hosts, None, None, false);

    let app_info_normalised = supertokens::AppInfo::from_input(&app_info_input)?;
    let session_recipe = Arc::new(
        supertokens::recipe::session::recipe::SessionRecipe::new(
            app_info_normalised,
            session_config,
        )?,
    );

    Supertokens::init(SupertokensInit {
        app_info: app_info_input,
        supertokens_config: SupertokensConfig {
            connection_uri,
            api_key: None,
            network_interceptor: None,
            disable_core_call_cache: false,
        },
        recipe_list: vec![session_recipe],
        telemetry: Some(false),
        debug: false,
    })
}

/// Helper to create a fresh UserContext.
pub fn new_user_context() -> UserContext {
    UserContext::new()
}
