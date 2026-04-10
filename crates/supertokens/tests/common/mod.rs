#![allow(dead_code)]

use std::sync::Arc;

use supertokens::querier::Querier;
use supertokens::recipe_module::RecipeModule;
use supertokens::UserContext;
use supertokens::{InputAppInfo, Supertokens, SupertokensConfig, SupertokensInit};

/// Returns the SuperTokens Core URL from env vars, defaulting to http://localhost:3567.
pub fn core_url() -> String {
    let host = std::env::var("SUPERTOKENS_CORE_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("SUPERTOKENS_CORE_PORT").unwrap_or_else(|_| "3567".to_string());
    format!("http://{}:{}", host, port)
}

/// Creates an isolated app on the Core for test isolation.
/// Returns the connection URI with the app path (e.g. http://localhost:3567/appid-<uuid>).
pub async fn get_new_core_app_url() -> String {
    let base = core_url();
    let app_id = format!("appid-{}", uuid::Uuid::new_v4());

    let client = reqwest::Client::new();
    let url = format!("{}/recipe/multitenancy/app/v2", base);
    let resp = client
        .put(&url)
        .json(&serde_json::json!({ "appId": app_id }))
        .send()
        .await
        .expect("Failed to create test app on Core");
    assert!(
        resp.status().is_success(),
        "Failed to create test app: {}",
        resp.text().await.unwrap_or_default()
    );

    format!("{}/{}", base, app_id)
}

/// Reset all singletons between tests.
pub fn reset() {
    Supertokens::reset();
    Querier::reset();
    supertokens::recipe_module::reset_get_tenant_id();
    supertokens::post_st_init_callbacks::reset();
    supertokens::recipe::multifactorauth::recipe_implementation::reset_factor_setup_funcs();
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

/// Build a SupertokensInit config using an isolated app URL.
pub fn st_init_args_with_connection_uri(
    connection_uri: String,
    recipe_list: Vec<Arc<dyn RecipeModule>>,
) -> SupertokensInit {
    SupertokensInit {
        app_info: test_app_info(),
        supertokens_config: SupertokensConfig {
            connection_uri,
            api_key: None,
            network_interceptor: None,
            disable_core_call_cache: false,
        },
        recipe_list,
        telemetry: Some(false),
        debug: false,
    }
}

/// Initialize the SDK with a session recipe using default config.
/// Handles the Querier init ordering issue (recipes need Querier, Supertokens::init creates Querier).
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
    let session_recipe = Arc::new(supertokens::recipe::session::recipe::SessionRecipe::new(
        app_info_normalised,
        session_config,
    )?);

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

/// No-op email delivery for tests.
pub struct NoopEmailDelivery;

#[async_trait::async_trait]
impl
    supertokens::ingredients::email_delivery::EmailDeliveryInterface<
        supertokens::recipe::emailpassword::types::EmailTemplateVars,
    > for NoopEmailDelivery
{
    async fn send_email(
        &self,
        _input: supertokens::recipe::emailpassword::types::EmailTemplateVars,
        _user_context: &UserContext,
    ) -> std::result::Result<(), supertokens::SuperTokensError> {
        Ok(())
    }
}

/// Initialize the SDK with session + emailpassword recipes.
pub fn init_with_emailpassword() -> std::result::Result<(), supertokens::SuperTokensError> {
    let app_info_input = test_app_info();
    let connection_uri = core_url();

    let hosts = parse_connection_uri(&connection_uri)?;
    Querier::init(hosts, None, None, false);

    let app_info_normalised = supertokens::AppInfo::from_input(&app_info_input)?;

    let session_recipe = Arc::new(supertokens::recipe::session::recipe::SessionRecipe::new(
        app_info_normalised.clone(),
        supertokens::recipe::session::types::SessionConfig::default(),
    )?);

    let emailpassword_recipe = Arc::new(
        supertokens::recipe::emailpassword::recipe::EmailPasswordRecipe::new(
            app_info_normalised,
            supertokens::recipe::emailpassword::types::EmailPasswordConfig {
                sign_up_feature: None,
                override_: None,
            },
            Arc::new(NoopEmailDelivery),
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
        recipe_list: vec![session_recipe, emailpassword_recipe],
        telemetry: Some(false),
        debug: false,
    })
}

/// Helper to create a fresh UserContext.
pub fn new_user_context() -> UserContext {
    UserContext::new()
}
