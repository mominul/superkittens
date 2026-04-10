use async_trait::async_trait;
use http::Request;
use std::collections::HashMap;
use std::sync::Arc;
use supertokens::framework::request::BaseRequest;
use tokio::sync::Mutex;

/// Axum adapter implementing `BaseRequest`.
pub struct AxumRequest {
    uri: http::Uri,
    method: http::Method,
    headers: http::HeaderMap,
    body: Arc<Mutex<Option<bytes::Bytes>>>,
}

impl AxumRequest {
    pub fn new(
        uri: http::Uri,
        method: http::Method,
        headers: http::HeaderMap,
        body: bytes::Bytes,
    ) -> Self {
        Self {
            uri,
            method,
            headers,
            body: Arc::new(Mutex::new(Some(body))),
        }
    }

    /// Construct from an Axum request (consuming the body).
    pub async fn from_request<B>(req: Request<B>) -> Self
    where
        B: http_body_util::BodyExt + Send,
        B::Error: std::fmt::Debug,
    {
        let (parts, body) = req.into_parts();
        let bytes = http_body_util::BodyExt::collect(body)
            .await
            .map(|c| c.to_bytes())
            .unwrap_or_default();
        Self::new(parts.uri, parts.method, parts.headers, bytes)
    }
}

#[async_trait]
impl BaseRequest for AxumRequest {
    fn get_original_url(&self) -> String {
        self.uri.to_string()
    }

    fn get_path(&self) -> String {
        self.uri.path().to_string()
    }

    fn method(&self) -> String {
        self.method.as_str().to_lowercase()
    }

    fn get_query_param(&self, key: &str) -> Option<String> {
        self.uri.query().and_then(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.into_owned())
        })
    }

    fn get_query_params(&self) -> HashMap<String, String> {
        self.uri
            .query()
            .map(|q| {
                url::form_urlencoded::parse(q.as_bytes())
                    .map(|(k, v)| (k.into_owned(), v.into_owned()))
                    .collect()
            })
            .unwrap_or_default()
    }

    async fn json(&self) -> Option<serde_json::Value> {
        let mut body = self.body.lock().await;
        let bytes = body.take()?;
        let result = serde_json::from_slice(&bytes).ok();
        // Put the bytes back for potential re-reads
        *body = Some(bytes);
        result
    }

    async fn form_data(&self) -> HashMap<String, String> {
        let body = self.body.lock().await;
        if let Some(ref bytes) = *body {
            url::form_urlencoded::parse(bytes)
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect()
        } else {
            HashMap::new()
        }
    }

    fn get_cookie(&self, key: &str) -> Option<String> {
        self.headers
            .get_all(http::header::COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|s| s.split(';'))
            .find_map(|cookie| {
                let mut parts = cookie.trim().splitn(2, '=');
                let name = parts.next()?.trim();
                let value = parts.next()?.trim();
                if name == key {
                    Some(value.to_string())
                } else {
                    None
                }
            })
    }

    fn get_header(&self, key: &str) -> Option<String> {
        self.headers
            .get(key)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use supertokens::framework::request::BaseRequest;

    fn make_request(
        uri: &str,
        method: http::Method,
        headers: Vec<(&str, &str)>,
        body: &[u8],
    ) -> AxumRequest {
        let mut header_map = http::HeaderMap::new();
        for (k, v) in headers {
            header_map.append(
                http::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                http::header::HeaderValue::from_str(v).unwrap(),
            );
        }
        AxumRequest::new(
            uri.parse().unwrap(),
            method,
            header_map,
            bytes::Bytes::from(body.to_vec()),
        )
    }

    #[test]
    fn test_get_path() {
        let req = make_request("/auth/signin?foo=bar", http::Method::GET, vec![], b"");
        assert_eq!(req.get_path(), "/auth/signin");
    }

    #[test]
    fn test_get_path_root() {
        let req = make_request("/", http::Method::GET, vec![], b"");
        assert_eq!(req.get_path(), "/");
    }

    #[test]
    fn test_method_lowercase() {
        let req = make_request("/", http::Method::POST, vec![], b"");
        assert_eq!(req.method(), "post");

        let req2 = make_request("/", http::Method::DELETE, vec![], b"");
        assert_eq!(req2.method(), "delete");
    }

    #[test]
    fn test_get_query_param_found() {
        let req = make_request("/path?key=value&other=123", http::Method::GET, vec![], b"");
        assert_eq!(req.get_query_param("key"), Some("value".to_string()));
        assert_eq!(req.get_query_param("other"), Some("123".to_string()));
    }

    #[test]
    fn test_get_query_param_missing() {
        let req = make_request("/path?key=value", http::Method::GET, vec![], b"");
        assert_eq!(req.get_query_param("nope"), None);
    }

    #[test]
    fn test_get_query_param_no_query_string() {
        let req = make_request("/path", http::Method::GET, vec![], b"");
        assert_eq!(req.get_query_param("key"), None);
    }

    #[test]
    fn test_get_query_params() {
        let req = make_request("/p?a=1&b=2&c=hello", http::Method::GET, vec![], b"");
        let params = req.get_query_params();
        assert_eq!(params.get("a"), Some(&"1".to_string()));
        assert_eq!(params.get("b"), Some(&"2".to_string()));
        assert_eq!(params.get("c"), Some(&"hello".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_get_query_params_empty() {
        let req = make_request("/p", http::Method::GET, vec![], b"");
        let params = req.get_query_params();
        assert!(params.is_empty());
    }

    #[test]
    fn test_get_cookie_single() {
        let req = make_request(
            "/",
            http::Method::GET,
            vec![("cookie", "session=abc123")],
            b"",
        );
        assert_eq!(req.get_cookie("session"), Some("abc123".to_string()));
    }

    #[test]
    fn test_get_cookie_multiple_in_one_header() {
        let req = make_request(
            "/",
            http::Method::GET,
            vec![("cookie", "a=1; b=2; c=3")],
            b"",
        );
        assert_eq!(req.get_cookie("a"), Some("1".to_string()));
        assert_eq!(req.get_cookie("b"), Some("2".to_string()));
        assert_eq!(req.get_cookie("c"), Some("3".to_string()));
    }

    #[test]
    fn test_get_cookie_missing() {
        let req = make_request("/", http::Method::GET, vec![("cookie", "a=1")], b"");
        assert_eq!(req.get_cookie("b"), None);
    }

    #[test]
    fn test_get_cookie_no_cookie_header() {
        let req = make_request("/", http::Method::GET, vec![], b"");
        assert_eq!(req.get_cookie("a"), None);
    }

    #[test]
    fn test_get_header() {
        let req = make_request(
            "/",
            http::Method::GET,
            vec![
                ("x-custom", "myvalue"),
                ("content-type", "application/json"),
            ],
            b"",
        );
        assert_eq!(req.get_header("x-custom"), Some("myvalue".to_string()));
        assert_eq!(
            req.get_header("content-type"),
            Some("application/json".to_string())
        );
    }

    #[test]
    fn test_get_header_missing() {
        let req = make_request("/", http::Method::GET, vec![], b"");
        assert_eq!(req.get_header("x-missing"), None);
    }

    #[tokio::test]
    async fn test_json_valid() {
        let body = br#"{"email":"test@example.com","password":"secret"}"#;
        let req = make_request("/", http::Method::POST, vec![], body);
        let json = req.json().await;
        assert!(json.is_some());
        let val = json.unwrap();
        assert_eq!(val["email"], "test@example.com");
        assert_eq!(val["password"], "secret");
    }

    #[tokio::test]
    async fn test_json_invalid() {
        let req = make_request("/", http::Method::POST, vec![], b"not json");
        let json = req.json().await;
        assert!(json.is_none());
    }

    #[tokio::test]
    async fn test_json_empty_body() {
        let req = make_request("/", http::Method::POST, vec![], b"");
        let json = req.json().await;
        assert!(json.is_none());
    }

    #[tokio::test]
    async fn test_json_can_be_called_twice() {
        let body = br#"{"key":"value"}"#;
        let req = make_request("/", http::Method::POST, vec![], body);
        let first = req.json().await;
        let second = req.json().await;
        assert!(first.is_some());
        assert!(second.is_some());
        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn test_form_data() {
        let body = b"username=alice&password=secret";
        let req = make_request("/", http::Method::POST, vec![], body);
        let form = req.form_data().await;
        assert_eq!(form.get("username"), Some(&"alice".to_string()));
        assert_eq!(form.get("password"), Some(&"secret".to_string()));
    }

    #[test]
    fn test_get_original_url() {
        let req = make_request(
            "/auth/signin?rid=emailpassword",
            http::Method::GET,
            vec![],
            b"",
        );
        assert_eq!(req.get_original_url(), "/auth/signin?rid=emailpassword");
    }

    #[tokio::test]
    async fn test_from_request() {
        let body = axum::body::Body::from(r#"{"ok":true}"#);
        let http_req = http::Request::builder()
            .uri("/test?q=1")
            .method(http::Method::PUT)
            .header("x-test", "hello")
            .body(body)
            .unwrap();
        let req = AxumRequest::from_request(http_req).await;
        assert_eq!(req.get_path(), "/test");
        assert_eq!(req.method(), "put");
        assert_eq!(req.get_query_param("q"), Some("1".to_string()));
        assert_eq!(req.get_header("x-test"), Some("hello".to_string()));
        let json = req.json().await.unwrap();
        assert_eq!(json["ok"], true);
    }
}
