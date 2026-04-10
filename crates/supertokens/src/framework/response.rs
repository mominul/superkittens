/// Framework-agnostic response abstraction.
///
/// Each framework integration implements this trait to adapt outgoing
/// HTTP responses from the SDK.
pub trait BaseResponse: Send + Sync {
    /// Set a cookie on the response.
    #[allow(clippy::too_many_arguments)]
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
    );

    /// Set a response header.
    fn set_header(&mut self, key: &str, value: &str);

    /// Get a response header value.
    fn get_header(&self, key: &str) -> Option<String>;

    /// Remove a response header.
    fn remove_header(&mut self, key: &str);

    /// Set the HTTP status code.
    fn set_status_code(&mut self, status_code: u16);

    /// Set the response body as JSON.
    fn set_json_content(&mut self, content: serde_json::Value);

    /// Set the response body as HTML.
    fn set_html_content(&mut self, content: &str);

    /// Return a redirect response.
    fn redirect(&mut self, url: &str);
}

/// SameSite cookie attribute values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Lax,
    Strict,
    None,
}

impl std::fmt::Display for SameSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SameSite::Lax => write!(f, "lax"),
            SameSite::Strict => write!(f, "strict"),
            SameSite::None => write!(f, "none"),
        }
    }
}
