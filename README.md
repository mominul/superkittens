# superkittens

A Rust SDK for [SuperTokens](https://supertokens.com/) — open-source authentication for your apps. Ported from the [Python SDK](https://github.com/supertokens/supertokens-python) (v0.31.2).

## Workspace

| Crate | Description |
|-------|-------------|
| [`supertokens`](crates/supertokens) | Core SDK — framework-agnostic recipes, querier, types |
| [`supertokens-axum`](crates/supertokens-axum) | [Axum](https://github.com/tokio-rs/axum) integration — middleware, extractors, CORS |

## Supported Recipes

| Recipe | Description |
|--------|-------------|
| `session` | Session management with JWT, cookie/header tokens, anti-CSRF |
| `emailpassword` | Email + password sign-up/sign-in with password reset |
| `passwordless` | Magic link and OTP-based authentication |
| `thirdparty` | OAuth/social login (Google, GitHub, Apple, etc.) |
| `emailverification` | Email verification flow |
| `multitenancy` | Multi-tenant support with per-tenant config |
| `accountlinking` | Link multiple auth methods to a single user |
| `userroles` | Role-based access control |
| `usermetadata` | Arbitrary user metadata storage |
| `totp` | Time-based one-time password (2FA) |
| `multifactorauth` | Multi-factor authentication orchestration |
| `oauth2provider` | OAuth 2.0 provider functionality |
| `webauthn` | Passkey / WebAuthn authentication |
| `jwt` | JWT creation and JWKS |
| `openid` | OpenID Connect discovery |
| `dashboard` | SuperTokens dashboard integration |
| `saml` | SAML authentication |

## Quick Start

### Prerequisites

- Rust 1.80+
- A running [SuperTokens Core](https://supertokens.com/docs/community/self-hosting) instance

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
supertokens = { path = "crates/supertokens" }
supertokens-axum = { path = "crates/supertokens-axum" }
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

### Example

```rust
use std::sync::Arc;
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use supertokens::{
    InputAppInfo, Supertokens, SupertokensConfig, SupertokensInit, AppInfo,
};
use supertokens::recipe::session::recipe::SessionRecipe;
use supertokens::recipe::session::types::SessionConfig;
use supertokens_axum::{Session, SuperTokensRouter};

async fn protected(session: Session) -> Json<Value> {
    Json(json!({ "user_id": session.get_user_id() }))
}

#[tokio::main]
async fn main() {
    let app_info = InputAppInfo {
        app_name: "My App".to_string(),
        api_domain: "http://localhost:3001".to_string(),
        website_domain: Some("http://localhost:3000".to_string()),
        api_base_path: None,
        api_gateway_path: None,
        website_base_path: None,
        origin: None,
    };

    let normalised = AppInfo::from_input(&app_info).unwrap();

    let session = Arc::new(
        SessionRecipe::new(normalised, SessionConfig::default()).unwrap()
    );

    Supertokens::init(SupertokensInit {
        app_info,
        supertokens_config: SupertokensConfig {
            connection_uri: "http://localhost:3567".to_string(),
            api_key: None,
            network_interceptor: None,
            disable_core_call_cache: false,
        },
        recipe_list: vec![session],
        telemetry: Some(false),
        debug: false,
    }).unwrap();

    let app = Router::new()
        .route("/api/me", get(protected))
        .with_supertokens(vec!["http://localhost:3000".to_string()]);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

See [`examples/axum_emailpassword.rs`](crates/supertokens-axum/examples/axum_emailpassword.rs) for a more complete example with EmailPassword + Session.

## Axum Integration

### Middleware

`SuperTokensLayer` intercepts SuperTokens API routes (`/auth/*`) and dispatches them to the appropriate recipe handler:

```rust
use supertokens_axum::SuperTokensLayer;

let app = Router::new()
    .route("/api/hello", get(|| async { "hi" }))
    .layer(SuperTokensLayer::new());
```

### CORS

`SuperTokensCorsLayer` handles preflight requests and adds the CORS headers required by the SuperTokens frontend SDK:

```rust
use supertokens_axum::SuperTokensCorsLayer;

let app = Router::new()
    .layer(SuperTokensCorsLayer::new(vec!["http://localhost:3000".to_string()]));
```

Or use the convenience extension trait that adds both:

```rust
use supertokens_axum::SuperTokensRouter;

let app = Router::new()
    .with_supertokens(vec!["http://localhost:3000".to_string()]);
```

### Session Extractors

```rust
use supertokens_axum::{Session, OptionalSession};

// Requires a valid session (returns 401 otherwise)
async fn protected(session: Session) -> String {
    format!("Hello {}", session.get_user_id())
}

// Session is optional
async fn maybe_auth(session: OptionalSession) -> String {
    match session.0 {
        Some(s) => format!("Hello {}", s.get_user_id()),
        None => "Hello guest".to_string(),
    }
}
```

### Route-Level Session Verification

Use `VerifySessionLayer` to protect specific routes or groups:

```rust
use supertokens_axum::VerifySessionLayer;

let app = Router::new()
    .route("/api/admin", get(admin_handler))
    .layer(VerifySessionLayer::new())  // all routes above require a session
    .route("/api/public", get(public_handler));  // no session required
```

## Architecture

```
supertokens (core crate)
  src/
    supertokens.rs      # Singleton, middleware dispatch, recipe registration
    querier.rs          # HTTP client to SuperTokens Core (round-robin, retry)
    recipe_module.rs    # RecipeModule trait
    user_context.rs     # Request-scoped typed context map
    recipe/             # 18 recipe modules, each with:
      {name}/
        types.rs              # Input/output types, result enums
        interfaces.rs         # RecipeInterface + ApiInterface traits
        recipe_implementation.rs  # Default impl (talks to Core)

supertokens-axum (framework crate)
  src/
    middleware.rs       # SuperTokensLayer — intercepts /auth/* routes
    cors.rs             # SuperTokensCorsLayer — CORS with recipe headers
    verify_session.rs   # VerifySessionLayer — per-route session verification
    extractors.rs       # Session / OptionalSession axum extractors
    router.rs           # SuperTokensRouter extension trait
    request.rs          # AxumRequest (impl BaseRequest)
    response.rs         # AxumResponse (impl BaseResponse)
```

### Key Design Decisions

- **Override system**: Wrap the default `Arc<dyn RecipeInterface>` with your own struct to customize behavior
- **Result enums over exceptions**: Python's discriminated unions become Rust enums (e.g., `SignUpResult::Ok { .. } | SignUpResult::EmailAlreadyExists`)
- **Async throughout**: All I/O uses `#[async_trait]` with Tokio
- **Thread-safe**: `OnceLock<Arc<Supertokens>>` singleton, `Arc<dyn RecipeModule>` for recipes

## Development

```bash
# Check compilation
cargo check --workspace

# Run tests (129 tests)
cargo test --workspace

# Lint
cargo clippy --all-targets

# Format
cargo fmt --all

# Build docs
cargo doc --workspace --no-deps --open

# Run example (requires SuperTokens Core at localhost:3567)
cargo run --example axum_emailpassword -p supertokens-axum
```

## License

Apache-2.0
