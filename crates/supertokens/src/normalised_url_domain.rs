use crate::error::{raise_general_exception, SuperTokensError};
use crate::utils::is_an_ip_address;
use url::Url;

/// A validated and normalised URL domain (scheme + host + optional port).
///
/// The inner value is never exposed directly — use `get_as_string_dangerous()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalisedURLDomain {
    value: String,
}

impl NormalisedURLDomain {
    pub fn new(input: &str) -> Result<Self, SuperTokensError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(raise_general_exception(
                "URL domain cannot be an empty string",
            ));
        }
        let value = normalise_url_domain_or_throw(trimmed)?;
        Ok(Self { value })
    }

    pub fn get_as_string_dangerous(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for NormalisedURLDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

fn normalise_url_domain_or_throw(input: &str) -> Result<String, SuperTokensError> {
    let mut s = input.trim().to_lowercase();

    // Remove trailing slash
    s = s.trim_end_matches('/').to_string();

    // If no protocol, try to figure out a sensible one
    if !s.starts_with("http://") && !s.starts_with("https://") && !s.starts_with("supertokens://") {
        // If it's localhost or an IP, use http
        if s.starts_with("localhost") || is_an_ip_address(&s) || s.starts_with("[") {
            s = format!("http://{}", s);
        } else {
            s = format!("https://{}", s);
        }
    }

    // Handle supertokens:// protocol — convert to http:// for parsing, then restore
    if let Some(rest) = s.strip_prefix("supertokens://") {
        let for_parsing = format!("http://{}", rest);
        let parsed = Url::parse(&for_parsing)
            .map_err(|e| raise_general_exception(format!("Invalid URL: {} — {}", input, e)))?;
        let host = parsed
            .host_str()
            .ok_or_else(|| raise_general_exception(format!("Invalid URL domain: {}", input)))?;
        return if let Some(port) = parsed.port() {
            Ok(format!("supertokens://{}:{}", host, port))
        } else {
            Ok(format!("supertokens://{}", host))
        };
    }

    let parsed = Url::parse(&s)
        .map_err(|e| raise_general_exception(format!("Invalid URL: {} — {}", input, e)))?;

    let scheme = parsed.scheme();
    let host = parsed
        .host_str()
        .ok_or_else(|| raise_general_exception(format!("Invalid URL domain: {}", input)))?;

    // Force http for localhost and IPs
    let final_scheme = if host == "localhost" || is_an_ip_address(host) || host.starts_with('[') {
        "http"
    } else {
        scheme
    };

    if let Some(port) = parsed.port() {
        Ok(format!("{}://{}:{}", final_scheme, host, port))
    } else {
        Ok(format!("{}://{}", final_scheme, host))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_domain() {
        let d = NormalisedURLDomain::new("https://example.com").unwrap();
        assert_eq!(d.get_as_string_dangerous(), "https://example.com");
    }

    #[test]
    fn test_strips_trailing_slash() {
        let d = NormalisedURLDomain::new("https://example.com/").unwrap();
        assert_eq!(d.get_as_string_dangerous(), "https://example.com");
    }

    #[test]
    fn test_localhost_forces_http() {
        let d = NormalisedURLDomain::new("localhost:3000").unwrap();
        assert_eq!(d.get_as_string_dangerous(), "http://localhost:3000");

        let d2 = NormalisedURLDomain::new("https://localhost").unwrap();
        assert_eq!(d2.get_as_string_dangerous(), "http://localhost");
    }

    #[test]
    fn test_ip_forces_http() {
        let d = NormalisedURLDomain::new("192.168.1.1:8080").unwrap();
        assert_eq!(d.get_as_string_dangerous(), "http://192.168.1.1:8080");
    }

    #[test]
    fn test_adds_https_for_domains() {
        let d = NormalisedURLDomain::new("example.com").unwrap();
        assert_eq!(d.get_as_string_dangerous(), "https://example.com");
    }

    #[test]
    fn test_lowercases() {
        let d = NormalisedURLDomain::new("HTTPS://Example.COM").unwrap();
        assert_eq!(d.get_as_string_dangerous(), "https://example.com");
    }

    #[test]
    fn test_empty_string_errors() {
        assert!(NormalisedURLDomain::new("").is_err());
    }
}
