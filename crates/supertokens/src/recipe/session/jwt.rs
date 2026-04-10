use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::Value;

/// Parsed JWT information (without signature verification).
#[derive(Debug, Clone)]
pub struct ParsedJwtInfo {
    pub version: u32,
    pub raw_token_string: String,
    pub raw_payload: String,
    pub header: String,
    pub payload: Value,
    pub signature: String,
    pub kid: Option<String>,
    pub parsed_header: Option<Value>,
}

/// Standard v2 header (base64-encoded) for quick comparison.
const STANDARD_V2_HEADER: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsInZlcnNpb24iOiIyIn0";
const LATEST_TOKEN_VERSION: u32 = 5;

/// Parse a JWT without verifying its signature.
///
/// Extracts the header, payload, and signature parts. Determines the token
/// version and key ID (kid) from the header.
pub fn parse_jwt_without_signature_verification(jwt: &str) -> Result<ParsedJwtInfo, String> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT: expected 3 parts separated by '.'".to_string());
    }

    let header_b64 = parts[0];
    let payload_b64 = parts[1];
    let signature = parts[2];

    // Decode payload
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|e| format!("Failed to decode JWT payload: {}", e))?;
    let payload: Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| format!("Failed to parse JWT payload JSON: {}", e))?;

    // Check if header matches standard v2
    if header_b64 == STANDARD_V2_HEADER {
        return Ok(ParsedJwtInfo {
            version: 2,
            raw_token_string: jwt.to_string(),
            raw_payload: payload_b64.to_string(),
            header: header_b64.to_string(),
            payload,
            signature: signature.to_string(),
            kid: None,
            parsed_header: None,
        });
    }

    // Decode and parse header
    let header_bytes = URL_SAFE_NO_PAD
        .decode(header_b64)
        .map_err(|e| format!("Failed to decode JWT header: {}", e))?;
    let parsed_header: Value = serde_json::from_slice(&header_bytes)
        .map_err(|e| format!("Failed to parse JWT header JSON: {}", e))?;

    // Validate header type
    let typ = parsed_header.get("typ").and_then(|v| v.as_str());
    if typ != Some("JWT") {
        return Err("Invalid JWT header: typ must be 'JWT'".to_string());
    }

    // Extract version
    let version = parsed_header
        .get("version")
        .map(|v| match v {
            Value::Number(n) => n.as_u64().unwrap_or(LATEST_TOKEN_VERSION as u64) as u32,
            Value::String(s) => s.parse::<u32>().unwrap_or(LATEST_TOKEN_VERSION),
            _ => LATEST_TOKEN_VERSION,
        })
        .unwrap_or(LATEST_TOKEN_VERSION);

    // Extract kid
    let kid = parsed_header
        .get("kid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // v3+ must have kid
    if version >= 3 && kid.is_none() {
        return Err(format!(
            "Invalid JWT header: v{} token must have 'kid' field",
            version
        ));
    }

    Ok(ParsedJwtInfo {
        version,
        raw_token_string: jwt.to_string(),
        raw_payload: payload_b64.to_string(),
        header: header_b64.to_string(),
        payload,
        signature: signature.to_string(),
        kid,
        parsed_header: Some(parsed_header),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    fn make_jwt(header: &Value, payload: &Value) -> String {
        let h = URL_SAFE_NO_PAD.encode(serde_json::to_vec(header).unwrap());
        let p = URL_SAFE_NO_PAD.encode(serde_json::to_vec(payload).unwrap());
        format!("{}.{}.test_sig", h, p)
    }

    #[test]
    fn test_parse_v2_standard_header() {
        let payload = serde_json::json!({"sub": "user1", "iat": 1000});
        let p = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());
        let jwt = format!("{}.{}.sig", STANDARD_V2_HEADER, p);

        let info = parse_jwt_without_signature_verification(&jwt).unwrap();
        assert_eq!(info.version, 2);
        assert!(info.kid.is_none());
        assert!(info.parsed_header.is_none());
    }

    #[test]
    fn test_parse_v3_with_kid() {
        let header =
            serde_json::json!({"alg": "RS256", "typ": "JWT", "version": 3, "kid": "key-1"});
        let payload = serde_json::json!({"sub": "user1"});
        let jwt = make_jwt(&header, &payload);

        let info = parse_jwt_without_signature_verification(&jwt).unwrap();
        assert_eq!(info.version, 3);
        assert_eq!(info.kid.as_deref(), Some("key-1"));
    }

    #[test]
    fn test_parse_v3_without_kid_fails() {
        let header = serde_json::json!({"alg": "RS256", "typ": "JWT", "version": 3});
        let payload = serde_json::json!({"sub": "user1"});
        let jwt = make_jwt(&header, &payload);

        let err = parse_jwt_without_signature_verification(&jwt).unwrap_err();
        assert!(err.contains("kid"));
    }

    #[test]
    fn test_invalid_jwt_parts() {
        assert!(parse_jwt_without_signature_verification("not.a.valid.jwt").is_err());
        assert!(parse_jwt_without_signature_verification("onlyone").is_err());
    }
}
