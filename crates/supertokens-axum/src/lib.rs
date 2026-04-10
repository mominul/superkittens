//! # SuperTokens Axum Integration
//!
//! This crate provides Axum integration for the SuperTokens authentication SDK.
//!
//! ## Quick Start
//!
//! ```no_run
//! use axum::{Router, routing::get};
//! use supertokens_axum::{SuperTokensRouter, extractors::Session};
//!
//! let app = Router::new()
//!     .route("/protected", get(protected_handler))
//!     .with_supertokens(vec!["http://localhost:3000".to_string()]);
//!
//! async fn protected_handler(session: Session) -> String {
//!     format!("Hello, {}!", session.get_user_id())
//! }
//! ```
//!
//! ## Components
//!
//! - **[`middleware::SuperTokensLayer`]** — Tower middleware that intercepts SuperTokens API routes
//! - **[`cors::SuperTokensCorsLayer`]** — CORS layer with SuperTokens headers
//! - **[`verify_session::VerifySessionLayer`]** — Session verification middleware for protected routes
//! - **[`extractors::Session`]** / **[`extractors::OptionalSession`]** — Axum extractors
//! - **[`router::SuperTokensRouter`]** — Extension trait for ergonomic Router setup
//! - **[`request::AxumRequest`]** / **[`response::AxumResponse`]** — Framework adapters

pub mod cors;
pub mod extractors;
pub mod middleware;
pub mod request;
pub mod response;
pub mod router;
pub mod verify_session;

// Re-export key types for convenience
pub use cors::SuperTokensCorsLayer;
pub use extractors::{OptionalSession, Session};
pub use middleware::SuperTokensLayer;
pub use router::SuperTokensRouter;
pub use verify_session::{SessionExtension, VerifySessionConfig, VerifySessionLayer};
