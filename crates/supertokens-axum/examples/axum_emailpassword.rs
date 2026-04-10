//! Example: Axum server with SuperTokens EmailPassword + Session recipes.
//!
//! This demonstrates how to:
//! - Initialize the SuperTokens SDK with connection URI and app info
//! - Register Session and EmailPassword recipes
//! - Set up an Axum router with SuperTokens middleware and CORS
//! - Create a protected route that requires a valid session
//! - Create a public route accessible without authentication
//!
//! Run with:
//!   cargo run --example axum_emailpassword -p supertokens-axum
//!
//! Note: A running SuperTokens Core is required at the configured `connection_uri`.

use std::sync::Arc;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;

// SuperTokens core SDK types
use supertokens::error::SuperTokensError;
use supertokens::ingredients::email_delivery::EmailDeliveryInterface;
use supertokens::recipe::emailpassword::recipe::EmailPasswordRecipe;
use supertokens::recipe::emailpassword::types::{EmailPasswordConfig, EmailTemplateVars};
use supertokens::recipe::session::recipe::SessionRecipe;
use supertokens::recipe::session::types::SessionConfig;
use supertokens::user_context::UserContext;
use supertokens::{InputAppInfo, Supertokens, SupertokensConfig, SupertokensInit};

// Axum integration re-exports
use supertokens_axum::{Session, SuperTokensRouter};

// ---------------------------------------------------------------------------
// Email delivery stub (logs instead of sending real emails)
// ---------------------------------------------------------------------------

/// A no-op email delivery implementation for demonstration purposes.
/// In production, replace this with an SMTP or third-party email provider.
struct ConsoleEmailDelivery;

#[async_trait::async_trait]
impl EmailDeliveryInterface<EmailTemplateVars> for ConsoleEmailDelivery {
    async fn send_email(
        &self,
        input: EmailTemplateVars,
        _user_context: &UserContext,
    ) -> Result<(), SuperTokensError> {
        println!(
            "[EmailDelivery] Password reset link for {}: {}",
            input.user.email, input.password_reset_link
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// A protected route — the `Session` extractor rejects unauthenticated requests
/// with a 401 response automatically.
async fn protected_handler(session: Session) -> Json<Value> {
    let user_id = session.get_user_id();
    Json(json!({
        "message": format!("Hello, authenticated user {}!", user_id),
        "session_handle": session.get_handle(),
        "tenant_id": session.get_tenant_id(),
    }))
}

/// A public route — no session required.
async fn public_handler() -> Json<Value> {
    Json(json!({
        "message": "This is a public endpoint. No authentication required.",
    }))
}

/// Health-check endpoint.
async fn health() -> &'static str {
    "OK"
}

// ---------------------------------------------------------------------------
// Application entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    // 1. Initialize the SuperTokens SDK.
    //
    //    - `connection_uri` points to your running SuperTokens Core instance.
    //    - `app_info` describes your application (name, domains, base paths).
    //    - `recipe_list` registers the authentication recipes you want to use.
    //
    //    This must be called exactly once, before the server starts.
    let app_info = InputAppInfo {
        app_name: "My Rust App".to_string(),
        api_domain: "http://localhost:3001".to_string(),
        website_domain: Some("http://localhost:3000".to_string()),
        api_base_path: None, // defaults to "/auth"
        api_gateway_path: None,
        website_base_path: None, // defaults to "/auth"
        origin: None,
    };

    let supertokens_config = SupertokensConfig {
        connection_uri: "http://localhost:3567".to_string(),
        api_key: None,
        network_interceptor: None,
        disable_core_call_cache: false,
    };

    // We need AppInfo to construct recipes. Build it via the same normalisation
    // the SDK uses internally.
    let app_info_normalised =
        supertokens::AppInfo::from_input(&app_info).expect("Invalid app info");

    // 2. Set up recipes.
    //
    //    SessionRecipe handles token creation, refresh, and sign-out.
    //    EmailPasswordRecipe handles sign-up, sign-in, and password reset.
    let session_recipe = Arc::new(
        SessionRecipe::new(app_info_normalised.clone(), SessionConfig::default())
            .expect("Failed to create session recipe"),
    );

    let emailpassword_recipe = Arc::new(
        EmailPasswordRecipe::new(
            app_info_normalised,
            EmailPasswordConfig {
                sign_up_feature: None,
                override_: None,
            },
            Arc::new(ConsoleEmailDelivery),
        )
        .expect("Failed to create emailpassword recipe"),
    );

    // 3. Initialize the SDK singleton.
    Supertokens::init(SupertokensInit {
        app_info,
        supertokens_config,
        recipe_list: vec![session_recipe, emailpassword_recipe],
        telemetry: Some(false),
        debug: false,
    })
    .expect("Failed to initialize SuperTokens");

    // 4. Build the Axum router.
    //
    //    `with_supertokens` adds two layers:
    //      - SuperTokensLayer: intercepts /auth/* API routes (sign-up, sign-in, refresh, etc.)
    //      - SuperTokensCorsLayer: adds CORS headers required by the SuperTokens frontend SDK
    //
    //    Your own routes sit alongside the SuperTokens routes in the same router.
    let app = Router::new()
        .route("/api/protected", get(protected_handler))
        .route("/api/public", get(public_handler))
        .route("/health", get(health))
        .with_supertokens(vec!["http://localhost:3000".to_string()]);

    // 5. Bind and serve.
    let listener = TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    println!("Server listening on http://localhost:3001");
    println!("  Public endpoint:    GET /api/public");
    println!("  Protected endpoint: GET /api/protected");
    println!("  Health check:       GET /health");
    println!("  SuperTokens APIs:   POST /auth/signup, /auth/signin, etc.");

    axum::serve(listener, app).await.expect("Server error");
}
