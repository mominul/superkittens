use crate::error::SuperTokensError;
use crate::normalised_url_domain::NormalisedURLDomain;
use crate::normalised_url_path::NormalisedURLPath;

/// User-provided application info.
#[derive(Debug, Clone)]
pub struct InputAppInfo {
    pub app_name: String,
    pub api_domain: String,
    pub api_gateway_path: Option<String>,
    pub api_base_path: Option<String>,
    pub website_base_path: Option<String>,
    pub website_domain: Option<String>,
    pub origin: Option<String>,
}

/// Processed and normalised application info.
#[derive(Debug, Clone)]
pub struct AppInfo {
    pub app_name: String,
    pub api_domain: NormalisedURLDomain,
    pub api_gateway_path: NormalisedURLPath,
    pub api_base_path: NormalisedURLPath,
    pub website_base_path: NormalisedURLPath,
    pub website_domain: Option<NormalisedURLDomain>,
    pub origin: Option<NormalisedURLDomain>,
    pub top_level_api_domain: String,
}

impl AppInfo {
    pub fn from_input(input: &InputAppInfo) -> Result<Self, SuperTokensError> {
        let api_domain = NormalisedURLDomain::new(&input.api_domain)?;

        let api_gateway_path = match &input.api_gateway_path {
            Some(p) => NormalisedURLPath::new(p)?,
            None => NormalisedURLPath::new("")?,
        };

        let api_base_path_raw = match &input.api_base_path {
            Some(p) => NormalisedURLPath::new(p)?,
            None => NormalisedURLPath::new("/auth")?,
        };
        let api_base_path = api_gateway_path.append(&api_base_path_raw);

        let website_base_path = match &input.website_base_path {
            Some(p) => NormalisedURLPath::new(p)?,
            None => NormalisedURLPath::new("/auth")?,
        };

        let website_domain = input
            .website_domain
            .as_ref()
            .map(|d| NormalisedURLDomain::new(d))
            .transpose()?;

        let origin = input
            .origin
            .as_ref()
            .map(|o| NormalisedURLDomain::new(o))
            .transpose()?;

        // Extract top-level domain for same-site resolution
        let api_domain_str = api_domain.get_as_string_dangerous();
        let top_level_api_domain =
            crate::utils::get_top_level_domain_for_same_site_resolution(api_domain_str);

        Ok(Self {
            app_name: input.app_name.clone(),
            api_domain,
            api_gateway_path,
            api_base_path,
            website_base_path,
            website_domain,
            origin,
            top_level_api_domain,
        })
    }
}

/// Connection + authentication config for the SuperTokens Core.
#[derive(Clone)]
pub struct SupertokensConfig {
    pub connection_uri: String,
    pub api_key: Option<String>,
    pub network_interceptor: Option<NetworkInterceptor>,
    pub disable_core_call_cache: bool,
}

impl std::fmt::Debug for SupertokensConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SupertokensConfig")
            .field("connection_uri", &self.connection_uri)
            .field("api_key", &self.api_key.as_ref().map(|_| "***"))
            .field(
                "network_interceptor",
                &self.network_interceptor.as_ref().map(|_| "<fn>"),
            )
            .field("disable_core_call_cache", &self.disable_core_call_cache)
            .finish()
    }
}

/// A parsed Core host.
#[derive(Debug, Clone)]
pub struct Host {
    pub domain: NormalisedURLDomain,
    pub base_path: NormalisedURLPath,
}

/// Network interceptor callback type.
///
/// Receives (url, method, headers, params, body) and returns potentially modified versions.
pub type NetworkInterceptor = std::sync::Arc<
    dyn Fn(
            NetworkInterceptorRequest,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = NetworkInterceptorRequest> + Send>>
        + Send
        + Sync,
>;

/// Data passed through the network interceptor.
#[derive(Debug, Clone)]
pub struct NetworkInterceptorRequest {
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub params: Option<std::collections::HashMap<String, String>>,
    pub body: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(app_name: &str, api_domain: &str) -> InputAppInfo {
        InputAppInfo {
            app_name: app_name.to_string(),
            api_domain: api_domain.to_string(),
            api_gateway_path: None,
            api_base_path: None,
            website_base_path: None,
            website_domain: None,
            origin: None,
        }
    }

    #[test]
    fn test_app_info_basic() {
        let input = make_input("MyApp", "https://api.example.com");
        let info = AppInfo::from_input(&input).unwrap();
        assert_eq!(info.app_name, "MyApp");
        assert_eq!(
            info.api_domain.get_as_string_dangerous(),
            "https://api.example.com"
        );
    }

    #[test]
    fn test_app_info_default_base_paths() {
        let input = make_input("TestApp", "https://api.example.com");
        let info = AppInfo::from_input(&input).unwrap();
        assert_eq!(info.api_base_path.get_as_string_dangerous(), "/auth");
        assert_eq!(info.website_base_path.get_as_string_dangerous(), "/auth");
    }

    #[test]
    fn test_app_info_custom_api_base_path() {
        let input = InputAppInfo {
            app_name: "App".to_string(),
            api_domain: "https://api.example.com".to_string(),
            api_gateway_path: None,
            api_base_path: Some("/custom".to_string()),
            website_base_path: None,
            website_domain: None,
            origin: None,
        };
        let info = AppInfo::from_input(&input).unwrap();
        assert_eq!(info.api_base_path.get_as_string_dangerous(), "/custom");
    }

    #[test]
    fn test_app_info_with_gateway_path() {
        let input = InputAppInfo {
            app_name: "App".to_string(),
            api_domain: "https://api.example.com".to_string(),
            api_gateway_path: Some("/gateway".to_string()),
            api_base_path: None,
            website_base_path: None,
            website_domain: None,
            origin: None,
        };
        let info = AppInfo::from_input(&input).unwrap();
        // gateway + default base path = /gateway/auth
        assert_eq!(
            info.api_base_path.get_as_string_dangerous(),
            "/gateway/auth"
        );
    }

    #[test]
    fn test_app_info_with_website_domain() {
        let input = InputAppInfo {
            app_name: "App".to_string(),
            api_domain: "https://api.example.com".to_string(),
            api_gateway_path: None,
            api_base_path: None,
            website_base_path: None,
            website_domain: Some("https://example.com".to_string()),
            origin: None,
        };
        let info = AppInfo::from_input(&input).unwrap();
        assert!(info.website_domain.is_some());
        assert_eq!(
            info.website_domain.unwrap().get_as_string_dangerous(),
            "https://example.com"
        );
    }

    #[test]
    fn test_app_info_empty_domain_errors() {
        let input = make_input("App", "");
        assert!(AppInfo::from_input(&input).is_err());
    }
}
