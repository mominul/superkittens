use axum::body::Body;
use http::header::{
    HeaderValue, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
    ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS,
    ACCESS_CONTROL_MAX_AGE, ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN,
    VARY,
};
use http::{Method, Request, Response, StatusCode};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use supertokens::Supertokens;

/// Tower layer that adds CORS headers required by SuperTokens.
///
/// This layer:
/// - Handles preflight OPTIONS requests automatically
/// - Adds `Access-Control-Allow-Headers` for all recipe CORS headers
/// - Adds `Access-Control-Expose-Headers` for response headers recipes need to expose
/// - Sets `Access-Control-Allow-Credentials: true`
///
/// You can provide additional allowed origins/methods/headers via the builder.
#[derive(Clone)]
pub struct SuperTokensCorsLayer {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<Method>,
    max_age: Option<u64>,
}

impl SuperTokensCorsLayer {
    /// Create a new CORS layer.
    ///
    /// `allowed_origins` are the origins you want to allow (e.g., your frontend URL).
    pub fn new(allowed_origins: Vec<String>) -> Self {
        Self {
            allowed_origins,
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ],
            max_age: Some(86400),
        }
    }

    /// Set allowed HTTP methods.
    pub fn allowed_methods(mut self, methods: Vec<Method>) -> Self {
        self.allowed_methods = methods;
        self
    }

    /// Set the max-age for preflight caching (seconds).
    pub fn max_age(mut self, max_age: u64) -> Self {
        self.max_age = Some(max_age);
        self
    }
}

impl<S> Layer<S> for SuperTokensCorsLayer {
    type Service = SuperTokensCorsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SuperTokensCorsMiddleware {
            inner,
            config: self.clone(),
        }
    }
}

/// The CORS middleware service.
#[derive(Clone)]
pub struct SuperTokensCorsMiddleware<S> {
    inner: S,
    config: SuperTokensCorsLayer,
}

impl<S> Service<Request<Body>> for SuperTokensCorsMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        std::mem::swap(&mut self.inner, &mut inner);
        let config = self.config.clone();

        Box::pin(async move {
            let origin = req
                .headers()
                .get(ORIGIN)
                .and_then(|v| v.to_str().ok())
                .map(String::from);

            let is_preflight = req.method() == Method::OPTIONS
                && req.headers().contains_key(ACCESS_CONTROL_REQUEST_METHOD);

            // Check if origin is allowed
            let allowed_origin = origin.as_ref().and_then(|o| {
                if config.allowed_origins.iter().any(|ao| ao == o || ao == "*") {
                    Some(o.clone())
                } else {
                    None
                }
            });

            if is_preflight {
                // Handle preflight
                let mut response = Response::new(Body::empty());
                *response.status_mut() = StatusCode::NO_CONTENT;

                if let Some(ref origin_val) = allowed_origin {
                    set_cors_headers(&mut response, origin_val, &config);

                    // For preflight, also add the requested headers
                    if let Some(requested_headers) =
                        req.headers().get(ACCESS_CONTROL_REQUEST_HEADERS)
                    {
                        // Merge requested headers with SuperTokens headers
                        let st_headers = get_supertokens_cors_headers();
                        let mut all_headers: Vec<String> = st_headers;

                        if let Ok(requested) = requested_headers.to_str() {
                            for h in requested.split(',') {
                                let h = h.trim().to_string();
                                if !all_headers.iter().any(|ah| ah.eq_ignore_ascii_case(&h)) {
                                    all_headers.push(h);
                                }
                            }
                        }

                        if let Ok(val) = HeaderValue::from_str(&all_headers.join(", ")) {
                            response
                                .headers_mut()
                                .insert(ACCESS_CONTROL_ALLOW_HEADERS, val);
                        }
                    }

                    if let Some(max_age) = config.max_age {
                        if let Ok(val) = HeaderValue::from_str(&max_age.to_string()) {
                            response.headers_mut().insert(ACCESS_CONTROL_MAX_AGE, val);
                        }
                    }
                }

                return Ok(response);
            }

            // Normal request — call inner and add CORS headers to response
            let mut response = inner.call(req).await?;

            if let Some(ref origin_val) = allowed_origin {
                set_cors_headers(&mut response, origin_val, &config);
            }

            Ok(response)
        })
    }
}

fn set_cors_headers(response: &mut Response<Body>, origin: &str, config: &SuperTokensCorsLayer) {
    if let Ok(val) = HeaderValue::from_str(origin) {
        response
            .headers_mut()
            .insert(ACCESS_CONTROL_ALLOW_ORIGIN, val);
    }

    response.headers_mut().insert(
        ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );

    let methods: Vec<&str> = config.allowed_methods.iter().map(|m| m.as_str()).collect();
    if let Ok(val) = HeaderValue::from_str(&methods.join(", ")) {
        response
            .headers_mut()
            .insert(ACCESS_CONTROL_ALLOW_METHODS, val);
    }

    // Expose headers needed by SuperTokens frontend
    let expose_headers = get_supertokens_cors_headers();
    if let Ok(val) = HeaderValue::from_str(&expose_headers.join(", ")) {
        response
            .headers_mut()
            .insert(ACCESS_CONTROL_EXPOSE_HEADERS, val);
    }

    response
        .headers_mut()
        .insert(VARY, HeaderValue::from_static("Origin"));
}

/// Get CORS headers from SuperTokens instance, or fallback defaults.
fn get_supertokens_cors_headers() -> Vec<String> {
    if let Ok(st) = Supertokens::get_instance() {
        st.get_all_cors_headers()
    } else {
        // Sensible defaults if SuperTokens is not yet initialized
        vec![
            "rid".to_string(),
            "fdi-version".to_string(),
            "anti-csrf".to_string(),
            "authorization".to_string(),
            "st-auth-mode".to_string(),
            "front-token".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    /// A trivial service that returns 200 OK for any request.
    async fn ok_service(req: Request<Body>) -> Result<Response<Body>, std::convert::Infallible> {
        let _ = req;
        Ok(Response::new(Body::empty()))
    }

    fn build_service() -> SuperTokensCorsMiddleware<
        tower::util::ServiceFn<
            impl FnMut(
                    Request<Body>,
                ) -> std::pin::Pin<
                    Box<
                        dyn std::future::Future<
                                Output = Result<Response<Body>, std::convert::Infallible>,
                            > + Send,
                    >,
                > + Clone,
        >,
    > {
        let layer = SuperTokensCorsLayer::new(vec!["http://localhost:3000".to_string()]);
        let svc = tower::service_fn(|req: Request<Body>| {
            Box::pin(ok_service(req))
                as std::pin::Pin<
                    Box<
                        dyn std::future::Future<
                                Output = Result<Response<Body>, std::convert::Infallible>,
                            > + Send,
                    >,
                >
        });
        layer.layer(svc)
    }

    #[test]
    fn test_layer_builder_defaults() {
        let layer = SuperTokensCorsLayer::new(vec!["http://localhost:3000".to_string()]);
        assert_eq!(layer.allowed_origins, vec!["http://localhost:3000"]);
        assert_eq!(layer.max_age, Some(86400));
        assert_eq!(layer.allowed_methods.len(), 5);
    }

    #[test]
    fn test_layer_builder_custom_methods() {
        let layer = SuperTokensCorsLayer::new(vec!["*".to_string()])
            .allowed_methods(vec![Method::GET, Method::POST]);
        assert_eq!(layer.allowed_methods.len(), 2);
    }

    #[test]
    fn test_layer_builder_custom_max_age() {
        let layer = SuperTokensCorsLayer::new(vec![]).max_age(600);
        assert_eq!(layer.max_age, Some(600));
    }

    #[test]
    fn test_get_supertokens_cors_headers_defaults() {
        // Without SuperTokens initialized, should return sensible defaults
        let headers = get_supertokens_cors_headers();
        assert!(headers.contains(&"rid".to_string()));
        assert!(headers.contains(&"fdi-version".to_string()));
        assert!(headers.contains(&"anti-csrf".to_string()));
        assert!(headers.contains(&"authorization".to_string()));
        assert!(headers.contains(&"st-auth-mode".to_string()));
        assert!(headers.contains(&"front-token".to_string()));
        assert_eq!(headers.len(), 6);
    }

    #[test]
    fn test_set_cors_headers_fn() {
        let config = SuperTokensCorsLayer::new(vec!["http://localhost:3000".to_string()]);
        let mut response = Response::new(Body::empty());
        set_cors_headers(&mut response, "http://localhost:3000", &config);

        let headers = response.headers();
        assert_eq!(
            headers
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap()
                .to_str()
                .unwrap(),
            "http://localhost:3000"
        );
        assert_eq!(
            headers
                .get(ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap()
                .to_str()
                .unwrap(),
            "true"
        );
        assert!(headers.get(ACCESS_CONTROL_ALLOW_METHODS).is_some());
        assert!(headers.get(ACCESS_CONTROL_EXPOSE_HEADERS).is_some());
        assert_eq!(headers.get(VARY).unwrap().to_str().unwrap(), "Origin");
    }

    #[tokio::test]
    async fn test_preflight_request() {
        let svc = build_service();

        let req = Request::builder()
            .method(Method::OPTIONS)
            .uri("/auth/session")
            .header(ORIGIN, "http://localhost:3000")
            .header(ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .header(ACCESS_CONTROL_REQUEST_HEADERS, "content-type, x-custom")
            .body(Body::empty())
            .unwrap();

        let resp = svc.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            resp.headers()
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap()
                .to_str()
                .unwrap(),
            "http://localhost:3000"
        );
        assert_eq!(
            resp.headers()
                .get(ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap()
                .to_str()
                .unwrap(),
            "true"
        );
        assert!(resp.headers().get(ACCESS_CONTROL_ALLOW_HEADERS).is_some());
        assert!(resp.headers().get(ACCESS_CONTROL_MAX_AGE).is_some());

        // Verify the allow-headers includes both supertokens defaults and the requested headers
        let allow_headers = resp
            .headers()
            .get(ACCESS_CONTROL_ALLOW_HEADERS)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(allow_headers.contains("rid"));
        assert!(allow_headers.contains("x-custom"));
    }

    #[tokio::test]
    async fn test_normal_request_with_allowed_origin() {
        let svc = build_service();

        let req = Request::builder()
            .method(Method::GET)
            .uri("/auth/session")
            .header(ORIGIN, "http://localhost:3000")
            .body(Body::empty())
            .unwrap();

        let resp = svc.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap()
                .to_str()
                .unwrap(),
            "http://localhost:3000"
        );
        assert_eq!(
            resp.headers()
                .get(ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap()
                .to_str()
                .unwrap(),
            "true"
        );
    }

    #[tokio::test]
    async fn test_request_with_disallowed_origin() {
        let svc = build_service();

        let req = Request::builder()
            .method(Method::GET)
            .uri("/auth/session")
            .header(ORIGIN, "http://evil.com")
            .body(Body::empty())
            .unwrap();

        let resp = svc.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        // No CORS headers should be set for disallowed origin
        assert!(resp.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
    }

    #[tokio::test]
    async fn test_request_without_origin() {
        let svc = build_service();

        let req = Request::builder()
            .method(Method::GET)
            .uri("/auth/session")
            .body(Body::empty())
            .unwrap();

        let resp = svc.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
    }

    #[tokio::test]
    async fn test_wildcard_origin() {
        let layer = SuperTokensCorsLayer::new(vec!["*".to_string()]);
        let svc = tower::service_fn(|req: Request<Body>| {
            Box::pin(ok_service(req))
                as std::pin::Pin<
                    Box<
                        dyn std::future::Future<
                                Output = Result<Response<Body>, std::convert::Infallible>,
                            > + Send,
                    >,
                >
        });
        let svc = layer.layer(svc);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(ORIGIN, "http://any-origin.com")
            .body(Body::empty())
            .unwrap();

        let resp = svc.oneshot(req).await.unwrap();
        assert_eq!(
            resp.headers()
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap()
                .to_str()
                .unwrap(),
            "http://any-origin.com"
        );
    }
}
