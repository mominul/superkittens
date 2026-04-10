mod common;

use axum::body::Body;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use http::header::{
    ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS,
    ORIGIN,
};
use http::{Request, StatusCode};
use serial_test::serial;
use supertokens_axum::{OptionalSession, Session, SuperTokensRouter, VerifySessionLayer};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// 1. Non-SuperTokens routes pass through the middleware unchanged
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_non_supertokens_route_passes_through() {
    common::reset();
    common::init_with_session().expect("init failed");

    let app = Router::new()
        .route("/hello", get(|| async { "Hello" }))
        .with_supertokens_middleware();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/hello")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"Hello");
}

// ---------------------------------------------------------------------------
// 2. SuperTokens route without RID header
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_supertokens_route_without_rid_header() {
    common::reset();
    common::init_with_session().expect("init failed");

    let app = Router::new()
        .route("/auth/signup", get(|| async { "fallback" }))
        .with_supertokens_middleware();

    // Without the `rid` header the middleware may not recognise this as a
    // SuperTokens request and will let it fall through to the inner router.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/signup")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // The request should still succeed (either handled by ST or passed through).
    // We just verify we get a response without a panic / 500.
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "unexpected status: {}",
        response.status()
    );
}

// ---------------------------------------------------------------------------
// 3. VerifySessionLayer with no access token returns 401
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_verify_session_layer_no_token_returns_401() {
    common::reset();
    common::init_with_session().expect("init failed");

    let app = Router::new()
        .route("/protected", get(|| async { "secret" }))
        .layer(VerifySessionLayer::new())
        .with_supertokens_middleware();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// 4. VerifySessionLayer::optional() with no token allows request through
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_verify_session_layer_optional_no_token() {
    common::reset();
    common::init_with_session().expect("init failed");

    let app = Router::new()
        .route("/maybe-protected", get(|| async { "public-ok" }))
        .layer(VerifySessionLayer::optional())
        .with_supertokens_middleware();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/maybe-protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"public-ok");
}

// ---------------------------------------------------------------------------
// 5. Session extractor without token returns 401
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_session_extractor_no_token_returns_401() {
    common::reset();
    common::init_with_session().expect("init failed");

    async fn handler(session: Session) -> impl IntoResponse {
        format!("user: {}", session.get_user_id())
    }

    let app = Router::new()
        .route("/protected", get(handler))
        .with_supertokens_middleware();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// 6. OptionalSession extractor without token returns None (handler succeeds)
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_optional_session_extractor_no_token() {
    common::reset();
    common::init_with_session().expect("init failed");

    async fn handler(opt: OptionalSession) -> impl IntoResponse {
        if opt.0.is_some() {
            "has-session".to_string()
        } else {
            "no-session".to_string()
        }
    }

    let app = Router::new()
        .route("/optional", get(handler))
        .with_supertokens_middleware();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/optional")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"no-session");
}

// ---------------------------------------------------------------------------
// 7. SuperTokensCorsLayer adds correct CORS headers
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
async fn test_with_supertokens_cors_adds_headers() {
    // CORS layer does not require Core — it only reads from the SDK singleton
    // (and falls back to defaults if not initialized).
    common::reset();

    let app = Router::new()
        .route("/ping", get(|| async { "pong" }))
        .with_supertokens_cors(vec!["http://localhost:3000".to_string()]);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ping")
                .header(ORIGIN, "http://localhost:3000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert_eq!(
        headers
            .get(ACCESS_CONTROL_ALLOW_ORIGIN)
            .expect("missing Allow-Origin header")
            .to_str()
            .unwrap(),
        "http://localhost:3000"
    );
    assert_eq!(
        headers
            .get(ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .expect("missing Allow-Credentials header")
            .to_str()
            .unwrap(),
        "true"
    );

    let exposed = headers
        .get(ACCESS_CONTROL_EXPOSE_HEADERS)
        .expect("missing Expose-Headers header")
        .to_str()
        .unwrap();
    // SuperTokens default CORS headers must be present
    assert!(
        exposed.contains("rid"),
        "exposed headers should contain rid"
    );
    assert!(
        exposed.contains("anti-csrf"),
        "exposed headers should contain anti-csrf"
    );
    assert!(
        exposed.contains("front-token"),
        "exposed headers should contain front-token"
    );
}

// ---------------------------------------------------------------------------
// 8. .with_supertokens(origins) sets up both middleware and CORS
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_router_with_supertokens_combined() {
    common::reset();
    common::init_with_session().expect("init failed");

    let origins = vec!["http://localhost:3000".to_string()];

    let app = Router::new()
        .route("/hello", get(|| async { "world" }))
        .with_supertokens(origins);

    // Normal (non-auth) request with an allowed origin should pass through
    // AND have CORS headers attached.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/hello")
                .header(ORIGIN, "http://localhost:3000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify CORS headers are present (proves CORS layer is active)
    assert_eq!(
        response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_ORIGIN)
            .expect("CORS layer should add Allow-Origin")
            .to_str()
            .unwrap(),
        "http://localhost:3000"
    );
    assert_eq!(
        response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .expect("CORS layer should add Allow-Credentials")
            .to_str()
            .unwrap(),
        "true"
    );

    // Verify the body is correct (proves middleware passed through)
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"world");
}
