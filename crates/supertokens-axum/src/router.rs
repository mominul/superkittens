use axum::Router;

use crate::cors::SuperTokensCorsLayer;
use crate::middleware::SuperTokensLayer;

/// Extension trait for Axum `Router` to easily add SuperTokens middleware and CORS.
///
/// # Example
/// ```compile_fail
/// use axum::Router;
/// use supertokens_axum::SuperTokensRouter;
///
/// let app = Router::new()
///     .route("/hello", get(|| async { "Hello, World!" }))
///     .with_supertokens(vec!["http://localhost:3000".to_string()]);
/// ```
pub trait SuperTokensRouter {
    /// Add SuperTokens middleware layer to the router.
    ///
    /// This intercepts SuperTokens API routes (e.g., `/auth/session/refresh`)
    /// and dispatches them to the appropriate recipe handler.
    fn with_supertokens_middleware(self) -> Self;

    /// Add both SuperTokens middleware and CORS layers.
    ///
    /// This is the recommended setup for most applications:
    /// 1. Adds the SuperTokens API middleware
    /// 2. Adds CORS headers with the correct allowed headers for SuperTokens
    fn with_supertokens(self, allowed_origins: Vec<String>) -> Self;

    /// Add only the SuperTokens CORS layer (without the API middleware).
    ///
    /// Use this if you want to handle CORS separately but still need
    /// SuperTokens-compatible CORS headers.
    fn with_supertokens_cors(self, allowed_origins: Vec<String>) -> Self;
}

impl SuperTokensRouter for Router {
    fn with_supertokens_middleware(self) -> Self {
        self.layer(SuperTokensLayer::new())
    }

    fn with_supertokens(self, allowed_origins: Vec<String>) -> Self {
        // Order matters: layers are applied bottom-up, so CORS wraps the outermost,
        // and SuperTokens middleware runs first (inner).
        self.layer(SuperTokensCorsLayer::new(allowed_origins))
            .layer(SuperTokensLayer::new())
    }

    fn with_supertokens_cors(self, allowed_origins: Vec<String>) -> Self {
        self.layer(SuperTokensCorsLayer::new(allowed_origins))
    }
}
