use crate::error::{raise_general_exception, SuperTokensError};
use url::Url;

/// A validated and normalised URL path component.
///
/// The inner value is never exposed directly — use `get_as_string_dangerous()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalisedURLPath {
    value: String,
}

impl NormalisedURLPath {
    pub fn new(input: &str) -> Result<Self, SuperTokensError> {
        let value = normalise_url_path_or_throw(input)?;
        Ok(Self { value })
    }

    pub fn get_as_string_dangerous(&self) -> &str {
        &self.value
    }

    pub fn startswith(&self, other: &NormalisedURLPath) -> bool {
        self.value.starts_with(&other.value)
    }

    pub fn append(&self, other: &NormalisedURLPath) -> Self {
        Self {
            value: format!("{}{}", self.value, other.value),
        }
    }

    pub fn equals(&self, other: &NormalisedURLPath) -> bool {
        self.value == other.value
    }

    pub fn is_a_recipe_path(&self) -> bool {
        let parts: Vec<&str> = self.value.split('/').collect();
        // parts[0] is "" (leading slash), so check parts[1] or parts[2]
        parts.len() > 1 && parts.get(1) == Some(&"recipe")
            || parts.len() > 2 && parts.get(2) == Some(&"recipe")
    }
}

impl std::fmt::Display for NormalisedURLPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

fn domain_given(input: &str) -> bool {
    if input.starts_with("https://")
        || input.starts_with("http://")
        || input.starts_with("supertokens://")
    {
        return true;
    }

    // Check if it looks like a domain (e.g., "example.com/path")
    // A domain will have a dot before any slash
    if let Some(slash_pos) = input.find('/') {
        let before_slash = &input[..slash_pos];
        before_slash.contains('.')
    } else {
        // No slash — could be just a domain like "example.com"
        // But if it starts with "/" or has no dot, it's a path
        input.contains('.') && !input.starts_with('/')
    }
}

fn normalise_url_path_or_throw(input: &str) -> Result<String, SuperTokensError> {
    let trimmed = input.trim();

    if trimmed.is_empty() || trimmed == "/" {
        return Ok(String::new());
    }

    // If it has a domain, parse it as a full URL
    if domain_given(trimmed) {
        let url_str = if let Some(rest) = trimmed.strip_prefix("supertokens://") {
            format!("http://{}", rest)
        } else if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            format!("https://{}", trimmed)
        } else {
            trimmed.to_string()
        };

        let parsed = Url::parse(&url_str)
            .map_err(|e| raise_general_exception(format!("Invalid URL path: {} — {}", input, e)))?;

        let path = parsed.path().trim_end_matches('/');
        return Ok(path.to_string());
    }

    // Pure path — ensure it starts with / and parse via dummy URL
    let path_str = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{}", trimmed)
    };

    let dummy_url = format!("http://example.com{}", path_str);
    let parsed = Url::parse(&dummy_url)
        .map_err(|e| raise_general_exception(format!("Invalid URL path: {} — {}", input, e)))?;

    let path = parsed.path().trim_end_matches('/');
    Ok(path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_path() {
        let p = NormalisedURLPath::new("").unwrap();
        assert_eq!(p.get_as_string_dangerous(), "");
    }

    #[test]
    fn test_root_path() {
        let p = NormalisedURLPath::new("/").unwrap();
        assert_eq!(p.get_as_string_dangerous(), "");
    }

    #[test]
    fn test_simple_path() {
        let p = NormalisedURLPath::new("/auth").unwrap();
        assert_eq!(p.get_as_string_dangerous(), "/auth");
    }

    #[test]
    fn test_strips_trailing_slash() {
        let p = NormalisedURLPath::new("/auth/").unwrap();
        assert_eq!(p.get_as_string_dangerous(), "/auth");
    }

    #[test]
    fn test_from_full_url() {
        let p = NormalisedURLPath::new("https://example.com/auth/signin").unwrap();
        assert_eq!(p.get_as_string_dangerous(), "/auth/signin");
    }

    #[test]
    fn test_append() {
        let a = NormalisedURLPath::new("/auth").unwrap();
        let b = NormalisedURLPath::new("/signin").unwrap();
        assert_eq!(a.append(&b).get_as_string_dangerous(), "/auth/signin");
    }

    #[test]
    fn test_startswith() {
        let a = NormalisedURLPath::new("/auth/signin").unwrap();
        let b = NormalisedURLPath::new("/auth").unwrap();
        assert!(a.startswith(&b));
    }

    #[test]
    fn test_is_recipe_path() {
        let p = NormalisedURLPath::new("/recipe/session").unwrap();
        assert!(p.is_a_recipe_path());
    }
}
