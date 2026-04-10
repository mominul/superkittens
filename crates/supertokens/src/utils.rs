use crate::framework::request::BaseRequest;
use crate::framework::response::BaseResponse;
use regex::Regex;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------- IP address detection ----------

static IP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(25[0-5]|2[0-4]\d|[01]?\d\d?)\.(25[0-5]|2[0-4]\d|[01]?\d\d?)\.(25[0-5]|2[0-4]\d|[01]?\d\d?)\.(25[0-5]|2[0-4]\d|[01]?\d\d?)$"
    ).unwrap()
});

pub fn is_an_ip_address(s: &str) -> bool {
    // Strip port if present
    let host = if let Some(idx) = s.rfind(':') {
        let after_colon = &s[idx + 1..];
        if after_colon.parse::<u16>().is_ok() {
            &s[..idx]
        } else {
            s
        }
    } else {
        s
    };
    IP_REGEX.is_match(host)
}

// ---------- Version comparison ----------

/// Compare two semver-style version strings segment by segment.
/// Returns the greater of the two versions.
pub fn get_max_version(v1: &str, v2: &str) -> String {
    let p1: Vec<u32> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
    let p2: Vec<u32> = v2.split('.').filter_map(|s| s.parse().ok()).collect();
    let len = p1.len().max(p2.len());
    for i in 0..len {
        let a = p1.get(i).copied().unwrap_or(0);
        let b = p2.get(i).copied().unwrap_or(0);
        if a > b {
            return v1.to_string();
        }
        if b > a {
            return v2.to_string();
        }
    }
    v1.to_string() // equal
}

/// Find the maximum compatible version between two version lists.
pub fn find_max_version(versions_1: &[&str], versions_2: &[&str]) -> Option<String> {
    let mut result: Option<String> = None;
    for &v1 in versions_1 {
        for &v2 in versions_2 {
            if v1 == v2 {
                result = Some(match &result {
                    Some(current) => get_max_version(current, v1),
                    None => v1.to_string(),
                });
            }
        }
    }
    result
}

/// Check if `version >= minimum_version`.
pub fn is_version_gte(version: &str, minimum_version: &str) -> bool {
    get_max_version(version, minimum_version) == version
}

// ---------- HTTP helpers ----------

pub fn normalise_http_method(method: &str) -> String {
    method.to_lowercase()
}

pub fn is_4xx_error(status_code: u16) -> bool {
    status_code / 100 == 4
}

pub fn is_5xx_error(status_code: u16) -> bool {
    status_code / 100 == 5
}

// ---------- Response helpers ----------

pub fn send_200_response(data: serde_json::Value, response: &mut dyn BaseResponse) {
    response.set_status_code(200);
    response.set_json_content(data);
}

pub fn send_non_200_response(
    body: serde_json::Value,
    status_code: u16,
    response: &mut dyn BaseResponse,
) {
    response.set_status_code(status_code);
    response.set_json_content(body);
}

pub fn send_non_200_response_with_message(
    message: &str,
    status_code: u16,
    response: &mut dyn BaseResponse,
) {
    send_non_200_response(
        serde_json::json!({ "message": message }),
        status_code,
        response,
    );
}

pub fn send_unauthorised_access_response(response: &mut dyn BaseResponse) {
    send_non_200_response_with_message("unauthorised", 401, response);
}

// ---------- Base64 ----------

use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;

pub fn utf_base64encode(s: &str, urlsafe: bool) -> String {
    if urlsafe {
        URL_SAFE_NO_PAD.encode(s.as_bytes())
    } else {
        STANDARD.encode(s.as_bytes())
    }
}

pub fn utf_base64decode(s: &str, urlsafe: bool) -> Result<String, base64::DecodeError> {
    // Add padding tolerance like Python SDK
    let padded = format!("{}==", s.trim_end_matches('='));
    let bytes = if urlsafe {
        URL_SAFE_NO_PAD.decode(padded.trim_end_matches('='))?
    } else {
        STANDARD.decode(&padded)?
    };
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub fn encode_base64(value: &str) -> String {
    STANDARD.encode(value.as_bytes())
}

// ---------- Email ----------

pub fn normalise_email(email: &str) -> String {
    email.trim().to_lowercase()
}

// ---------- Time ----------

pub fn get_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn humanize_time(ms: u64) -> String {
    if ms < 1000 {
        return format!("{} ms", ms);
    }
    let seconds = ms / 1000;
    if seconds < 60 {
        return format!("{} second{}", seconds, if seconds == 1 { "" } else { "s" });
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" });
    }
    let hours = minutes / 60;
    format!("{} hour{}", hours, if hours == 1 { "" } else { "s" })
}

// ---------- Header helpers ----------

pub fn get_rid_from_header(request: &dyn BaseRequest) -> Option<String> {
    request.get_header(crate::constants::RID_KEY_HEADER)
}

pub fn get_header(request: &dyn BaseRequest, key: &str) -> Option<String> {
    request.get_header(key)
}

pub fn frontend_has_interceptor(request: &dyn BaseRequest) -> bool {
    get_rid_from_header(request).is_some()
}

// ---------- FDI version ----------

pub fn get_latest_fdi_version_from_fdi_list(fdi_header_value: &str) -> String {
    let versions: Vec<&str> = fdi_header_value.split(',').map(|s| s.trim()).collect();
    let mut max: Option<String> = None;
    for v in versions {
        if v.is_empty() {
            continue;
        }
        max = Some(match max {
            Some(current) => get_max_version(&current, v),
            None => v.to_string(),
        });
    }
    max.unwrap_or_default()
}

pub fn has_greater_than_equal_to_fdi(request: &dyn BaseRequest, version: &str) -> bool {
    let fdi_header = request.get_header(crate::constants::FDI_KEY_HEADER);
    match fdi_header {
        None => true, // No header means latest FDI
        Some(val) => {
            let latest = get_latest_fdi_version_from_fdi_list(&val);
            is_version_gte(&latest, version)
        }
    }
}

// ---------- Domain helpers ----------

pub fn get_top_level_domain_for_same_site_resolution(url: &str) -> String {
    let url_lower = url.to_lowercase();
    // Remove protocol
    let without_protocol = if let Some(idx) = url_lower.find("://") {
        &url_lower[idx + 3..]
    } else {
        &url_lower
    };
    // Remove port and path
    let host = without_protocol
        .split(':')
        .next()
        .unwrap_or(without_protocol);
    let host = host.split('/').next().unwrap_or(host);

    if host == "localhost" || is_an_ip_address(host) {
        return host.to_string();
    }

    // Simple TLD extraction: take last two parts
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 2 {
        // Handle amazonaws.com and similar multi-part TLDs
        if host.ends_with(".amazonaws.com") {
            return host.to_string();
        }
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        host.to_string()
    }
}

// ---------- List helpers ----------

pub fn get_filtered_list<T>(predicate: impl Fn(&T) -> bool, list: &[T]) -> Vec<&T> {
    list.iter().filter(|item| predicate(item)).collect()
}

pub fn find_first_occurrence_in_list<T>(predicate: impl Fn(&T) -> bool, list: &[T]) -> Option<&T> {
    list.iter().find(|item| predicate(item))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_an_ip_address() {
        assert!(is_an_ip_address("192.168.1.1"));
        assert!(is_an_ip_address("192.168.1.1:8080"));
        assert!(is_an_ip_address("0.0.0.0"));
        assert!(!is_an_ip_address("example.com"));
        assert!(!is_an_ip_address("localhost"));
    }

    #[test]
    fn test_get_max_version() {
        assert_eq!(get_max_version("1.0", "2.0"), "2.0");
        assert_eq!(get_max_version("2.1", "2.0"), "2.1");
        assert_eq!(get_max_version("1.0.0", "1.0.0"), "1.0.0");
        assert_eq!(get_max_version("5.4", "5.3"), "5.4");
    }

    #[test]
    fn test_find_max_version() {
        assert_eq!(
            find_max_version(&["1.0", "2.0", "3.0"], &["2.0", "3.0", "4.0"]),
            Some("3.0".to_string())
        );
        assert_eq!(find_max_version(&["1.0"], &["2.0"]), None);
    }

    #[test]
    fn test_is_version_gte() {
        assert!(is_version_gte("2.0", "1.0"));
        assert!(is_version_gte("1.0", "1.0"));
        assert!(!is_version_gte("1.0", "2.0"));
    }

    #[test]
    fn test_humanize_time() {
        assert_eq!(humanize_time(500), "500 ms");
        assert_eq!(humanize_time(5000), "5 seconds");
        assert_eq!(humanize_time(60000), "1 minute");
        assert_eq!(humanize_time(7200000), "2 hours");
    }

    #[test]
    fn test_normalise_email() {
        assert_eq!(
            normalise_email("  Alice@Example.COM  "),
            "alice@example.com"
        );
    }

    #[test]
    fn test_top_level_domain() {
        assert_eq!(
            get_top_level_domain_for_same_site_resolution("https://api.example.com"),
            "example.com"
        );
        assert_eq!(
            get_top_level_domain_for_same_site_resolution("http://localhost:3000"),
            "localhost"
        );
        assert_eq!(
            get_top_level_domain_for_same_site_resolution("http://192.168.1.1"),
            "192.168.1.1"
        );
    }
}
