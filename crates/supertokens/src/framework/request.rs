use async_trait::async_trait;
use std::collections::HashMap;

/// Framework-agnostic request abstraction.
///
/// Each framework integration (Axum, Actix, etc.) implements this trait
/// to adapt incoming HTTP requests for the SDK.
#[async_trait]
pub trait BaseRequest: Send + Sync {
    /// The original URL of the request.
    fn get_original_url(&self) -> String;

    /// The request path (without query string).
    fn get_path(&self) -> String;

    /// The HTTP method (lowercase).
    fn method(&self) -> String;

    /// Get a single query parameter by key.
    fn get_query_param(&self, key: &str) -> Option<String>;

    /// Get all query parameters.
    fn get_query_params(&self) -> HashMap<String, String>;

    /// Parse the request body as JSON.
    async fn json(&self) -> Option<serde_json::Value>;

    /// Parse the request body as form data.
    async fn form_data(&self) -> HashMap<String, String>;

    /// Get a cookie value by key.
    fn get_cookie(&self, key: &str) -> Option<String>;

    /// Get a header value by key (case-insensitive).
    fn get_header(&self, key: &str) -> Option<String>;

    /// Auto-detect content type and parse body as JSON or form data.
    async fn get_json_or_form_data(&self) -> Option<serde_json::Value> {
        let content_type = self.get_header("content-type").unwrap_or_default();
        if content_type.contains("application/json") {
            self.json().await
        } else {
            let form = self.form_data().await;
            if form.is_empty() {
                // Try JSON as fallback
                self.json().await
            } else {
                Some(serde_json::to_value(form).unwrap_or_default())
            }
        }
    }
}
