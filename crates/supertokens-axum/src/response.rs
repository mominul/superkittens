use http::header::{HeaderName, HeaderValue, SET_COOKIE};
use std::collections::HashMap;
use supertokens::framework::response::{BaseResponse, SameSite};

/// Axum adapter implementing `BaseResponse`.
pub struct AxumResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub cookies: Vec<String>,
    pub body: ResponseBody,
}

pub enum ResponseBody {
    Json(serde_json::Value),
    Html(String),
    Redirect(String),
    Empty,
}

impl AxumResponse {
    pub fn new() -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            cookies: Vec::new(),
            body: ResponseBody::Empty,
        }
    }

    /// Convert into an axum `Response`.
    pub fn into_axum_response(self) -> axum::response::Response {
        use axum::response::IntoResponse;
        use http::StatusCode;

        let status = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::OK);

        let mut response = match self.body {
            ResponseBody::Json(json) => axum::Json(json).into_response(),
            ResponseBody::Html(html) => axum::response::Html(html).into_response(),
            ResponseBody::Redirect(url) => axum::response::Redirect::to(&url).into_response(),
            ResponseBody::Empty => StatusCode::OK.into_response(),
        };

        *response.status_mut() = status;

        for (key, value) in &self.headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                response.headers_mut().insert(name, val);
            }
        }

        for cookie in &self.cookies {
            if let Ok(val) = HeaderValue::from_str(cookie) {
                response.headers_mut().append(SET_COOKIE, val);
            }
        }

        response
    }
}

impl Default for AxumResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseResponse for AxumResponse {
    fn set_cookie(
        &mut self,
        key: &str,
        value: &str,
        expires: u64,
        path: &str,
        domain: Option<&str>,
        secure: bool,
        httponly: bool,
        samesite: SameSite,
    ) {
        let mut cookie = format!(
            "{}={}; Max-Age={}; Path={}",
            key,
            value,
            expires / 1000,
            path
        );
        if let Some(d) = domain {
            cookie.push_str(&format!("; Domain={}", d));
        }
        if secure {
            cookie.push_str("; Secure");
        }
        if httponly {
            cookie.push_str("; HttpOnly");
        }
        cookie.push_str(&format!("; SameSite={}", samesite));
        self.cookies.push(cookie);
    }

    fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    fn get_header(&self, key: &str) -> Option<String> {
        self.headers.get(key).cloned()
    }

    fn remove_header(&mut self, key: &str) {
        self.headers.remove(key);
    }

    fn set_status_code(&mut self, status_code: u16) {
        self.status_code = status_code;
    }

    fn set_json_content(&mut self, content: serde_json::Value) {
        self.body = ResponseBody::Json(content);
    }

    fn set_html_content(&mut self, content: &str) {
        self.body = ResponseBody::Html(content.to_string());
    }

    fn redirect(&mut self, url: &str) {
        self.body = ResponseBody::Redirect(url.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use supertokens::framework::response::{BaseResponse, SameSite};

    #[test]
    fn test_new_defaults() {
        let resp = AxumResponse::new();
        assert_eq!(resp.status_code, 200);
        assert!(resp.headers.is_empty());
        assert!(resp.cookies.is_empty());
        assert!(matches!(resp.body, ResponseBody::Empty));
    }

    #[test]
    fn test_default_matches_new() {
        let resp = AxumResponse::default();
        assert_eq!(resp.status_code, 200);
    }

    #[test]
    fn test_set_header_and_get_header() {
        let mut resp = AxumResponse::new();
        resp.set_header("x-custom", "value1");
        assert_eq!(resp.get_header("x-custom"), Some("value1".to_string()));
    }

    #[test]
    fn test_get_header_missing() {
        let resp = AxumResponse::new();
        assert_eq!(resp.get_header("x-missing"), None);
    }

    #[test]
    fn test_set_header_overwrites() {
        let mut resp = AxumResponse::new();
        resp.set_header("key", "first");
        resp.set_header("key", "second");
        assert_eq!(resp.get_header("key"), Some("second".to_string()));
    }

    #[test]
    fn test_remove_header() {
        let mut resp = AxumResponse::new();
        resp.set_header("x-remove-me", "val");
        assert!(resp.get_header("x-remove-me").is_some());
        resp.remove_header("x-remove-me");
        assert_eq!(resp.get_header("x-remove-me"), None);
    }

    #[test]
    fn test_remove_header_nonexistent() {
        let mut resp = AxumResponse::new();
        resp.remove_header("does-not-exist"); // should not panic
    }

    #[test]
    fn test_set_status_code() {
        let mut resp = AxumResponse::new();
        resp.set_status_code(404);
        assert_eq!(resp.status_code, 404);
    }

    #[test]
    fn test_set_json_content() {
        let mut resp = AxumResponse::new();
        let json = serde_json::json!({"status": "OK"});
        resp.set_json_content(json.clone());
        match &resp.body {
            ResponseBody::Json(v) => assert_eq!(v, &json),
            _ => panic!("expected Json body"),
        }
    }

    #[test]
    fn test_set_html_content() {
        let mut resp = AxumResponse::new();
        resp.set_html_content("<h1>Hello</h1>");
        match &resp.body {
            ResponseBody::Html(s) => assert_eq!(s, "<h1>Hello</h1>"),
            _ => panic!("expected Html body"),
        }
    }

    #[test]
    fn test_redirect() {
        let mut resp = AxumResponse::new();
        resp.redirect("https://example.com/callback");
        match &resp.body {
            ResponseBody::Redirect(url) => assert_eq!(url, "https://example.com/callback"),
            _ => panic!("expected Redirect body"),
        }
    }

    #[test]
    fn test_set_cookie_basic() {
        let mut resp = AxumResponse::new();
        resp.set_cookie(
            "sAccessToken",
            "abc123",
            3600_000, // 3600 seconds in ms
            "/",
            None,
            true,
            true,
            SameSite::Lax,
        );
        assert_eq!(resp.cookies.len(), 1);
        let cookie = &resp.cookies[0];
        assert!(cookie.contains("sAccessToken=abc123"));
        assert!(cookie.contains("Max-Age=3600"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=lax"));
        // Should not contain Domain when None
        assert!(!cookie.contains("Domain="));
    }

    #[test]
    fn test_set_cookie_with_domain() {
        let mut resp = AxumResponse::new();
        resp.set_cookie(
            "session",
            "val",
            60_000,
            "/auth",
            Some(".example.com"),
            false,
            false,
            SameSite::None,
        );
        let cookie = &resp.cookies[0];
        assert!(cookie.contains("Domain=.example.com"));
        assert!(!cookie.contains("Secure"));
        assert!(!cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=none"));
        assert!(cookie.contains("Path=/auth"));
    }

    #[test]
    fn test_set_cookie_strict() {
        let mut resp = AxumResponse::new();
        resp.set_cookie("k", "v", 1000, "/", None, false, false, SameSite::Strict);
        let cookie = &resp.cookies[0];
        assert!(cookie.contains("SameSite=strict"));
    }

    #[test]
    fn test_set_cookie_multiple() {
        let mut resp = AxumResponse::new();
        resp.set_cookie("a", "1", 1000, "/", None, false, false, SameSite::Lax);
        resp.set_cookie("b", "2", 2000, "/", None, false, false, SameSite::Lax);
        assert_eq!(resp.cookies.len(), 2);
    }

    #[test]
    fn test_into_axum_response_status_and_json() {
        let mut resp = AxumResponse::new();
        resp.set_status_code(201);
        resp.set_json_content(serde_json::json!({"created": true}));
        resp.set_header("x-request-id", "abc");

        let axum_resp = resp.into_axum_response();
        assert_eq!(axum_resp.status().as_u16(), 201);
        assert_eq!(
            axum_resp
                .headers()
                .get("x-request-id")
                .unwrap()
                .to_str()
                .unwrap(),
            "abc"
        );
        // content-type should be application/json from axum::Json
        assert!(axum_resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("application/json"));
    }

    #[test]
    fn test_into_axum_response_cookies() {
        let mut resp = AxumResponse::new();
        resp.set_cookie("tok", "xyz", 5000, "/", None, true, true, SameSite::Lax);
        resp.set_cookie(
            "ref",
            "abc",
            10000,
            "/",
            None,
            true,
            false,
            SameSite::Strict,
        );

        let axum_resp = resp.into_axum_response();
        let set_cookie_values: Vec<&str> = axum_resp
            .headers()
            .get_all(http::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect();
        assert_eq!(set_cookie_values.len(), 2);
        assert!(set_cookie_values.iter().any(|c| c.contains("tok=xyz")));
        assert!(set_cookie_values.iter().any(|c| c.contains("ref=abc")));
    }

    #[test]
    fn test_into_axum_response_empty_body() {
        let resp = AxumResponse::new();
        let axum_resp = resp.into_axum_response();
        assert_eq!(axum_resp.status().as_u16(), 200);
    }

    #[test]
    fn test_into_axum_response_404() {
        let mut resp = AxumResponse::new();
        resp.set_status_code(404);
        let axum_resp = resp.into_axum_response();
        assert_eq!(axum_resp.status().as_u16(), 404);
    }
}
