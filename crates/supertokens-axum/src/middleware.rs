use axum::body::Body;
use axum::response::IntoResponse;
use http::Request;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::request::AxumRequest;
use crate::response::AxumResponse;
use supertokens::user_context::UserContext;
use supertokens::Supertokens;

/// Tower layer that intercepts SuperTokens API routes.
#[derive(Clone)]
pub struct SuperTokensLayer;

impl SuperTokensLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SuperTokensLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for SuperTokensLayer {
    type Service = SuperTokensMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SuperTokensMiddleware { inner }
    }
}

/// Tower middleware service for SuperTokens.
#[derive(Clone)]
pub struct SuperTokensMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for SuperTokensMiddleware<S>
where
    S: Service<Request<Body>, Response = axum::response::Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    type Response = axum::response::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        // swap so inner is ready
        std::mem::swap(&mut self.inner, &mut inner);

        Box::pin(async move {
            let st = match Supertokens::get_instance() {
                Ok(st) => st,
                Err(_) => return inner.call(req).await,
            };

            // Build AxumRequest from the incoming request
            let (parts, body) = req.into_parts();
            let bytes = http_body_util::BodyExt::collect(body)
                .await
                .map(|c| c.to_bytes())
                .unwrap_or_default();

            let axum_req = AxumRequest::new(
                parts.uri.clone(),
                parts.method.clone(),
                parts.headers.clone(),
                bytes.clone(),
            );
            let mut axum_resp = AxumResponse::new();
            let mut user_context = UserContext::new();

            match st
                .middleware(&axum_req, &mut axum_resp, &mut user_context)
                .await
            {
                Ok(true) => {
                    // SuperTokens handled the request
                    Ok(axum_resp.into_axum_response())
                }
                Ok(false) => {
                    // Not a SuperTokens route — pass through to inner service
                    let reconstructed = Request::from_parts(parts, Body::from(bytes));
                    inner.call(reconstructed).await
                }
                Err(err) => {
                    // Try to handle the error
                    let mut error_resp = AxumResponse::new();
                    match st
                        .handle_supertokens_error(
                            &axum_req,
                            err,
                            &mut error_resp,
                            &mut user_context,
                        )
                        .await
                    {
                        Ok(()) => Ok(error_resp.into_axum_response()),
                        Err(_) => {
                            // Unhandled error — return 500
                            Ok(http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
                        }
                    }
                }
            }
        })
    }
}
