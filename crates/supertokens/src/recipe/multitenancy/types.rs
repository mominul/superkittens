use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Tenant configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(rename = "thirdPartyId")]
    pub third_party_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, flatten)]
    pub extra: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub tenant_id: String,
    #[serde(default)]
    pub core_config: serde_json::Map<String, Value>,
    #[serde(default)]
    pub first_factors: Option<Vec<String>>,
    #[serde(default)]
    pub required_secondary_factors: Option<Vec<String>>,
    #[serde(default)]
    pub third_party_providers: Vec<ProviderConfig>,
}

impl TenantConfig {
    pub fn from_json(json: &Value) -> Option<Self> {
        let tenant_id = json.get("tenantId")?.as_str()?.to_string();
        let core_config = json
            .get("coreConfig")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let first_factors = json.get("firstFactors").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect()
            })
        });
        let required_secondary_factors = json.get("requiredSecondaryFactors").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect()
            })
        });
        let third_party_providers = json
            .get("thirdParty")
            .and_then(|tp| tp.get("providers"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| serde_json::from_value(p.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Some(TenantConfig {
            tenant_id,
            core_config,
            first_factors,
            required_secondary_factors,
            third_party_providers,
        })
    }
}

/// Input for creating/updating tenants.
#[derive(Debug, Clone, Default)]
pub struct TenantConfigCreateOrUpdate {
    pub core_config: Option<serde_json::Map<String, Value>>,
    pub first_factors: Option<Option<Vec<String>>>,
    pub required_secondary_factors: Option<Option<Vec<String>>>,
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CreateOrUpdateTenantOkResult {
    pub created_new: bool,
}

#[derive(Debug, Clone)]
pub struct DeleteTenantOkResult {
    pub did_exist: bool,
}

#[derive(Debug, Clone)]
pub struct ListAllTenantsOkResult {
    pub tenants: Vec<TenantConfig>,
}

#[derive(Debug, Clone)]
pub struct CreateOrUpdateThirdPartyConfigOkResult {
    pub created_new: bool,
}

#[derive(Debug, Clone)]
pub struct DeleteThirdPartyConfigOkResult {
    pub did_config_exist: bool,
}

#[derive(Debug, Clone)]
pub enum AssociateUserToTenantResult {
    Ok { was_already_associated: bool },
    UnknownUserId,
    EmailAlreadyExists,
    PhoneNumberAlreadyExists,
    ThirdPartyUserAlreadyExists,
    NotAllowed { reason: String },
}

#[derive(Debug, Clone)]
pub struct DisassociateUserFromTenantOkResult {
    pub was_associated: bool,
}

// ---------------------------------------------------------------------------
// Login methods
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct LoginMethodEmailPassword {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginMethodPasswordless {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginMethodThirdParty {
    pub enabled: bool,
    pub providers: Vec<ThirdPartyProvider>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginMethodWebauthn {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThirdPartyProvider {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginMethodsGetOkResult {
    pub email_password: LoginMethodEmailPassword,
    pub passwordless: LoginMethodPasswordless,
    pub third_party: LoginMethodThirdParty,
    pub webauthn: LoginMethodWebauthn,
    pub first_factors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tenant_config_from_json_minimal() {
        let json = json!({"tenantId": "public"});
        let config = TenantConfig::from_json(&json).unwrap();
        assert_eq!(config.tenant_id, "public");
        assert!(config.core_config.is_empty());
        assert!(config.first_factors.is_none());
        assert!(config.required_secondary_factors.is_none());
        assert!(config.third_party_providers.is_empty());
    }

    #[test]
    fn test_tenant_config_from_json_with_factors() {
        let json = json!({
            "tenantId": "t1",
            "firstFactors": ["emailpassword", "thirdparty"],
            "requiredSecondaryFactors": ["totp"]
        });
        let config = TenantConfig::from_json(&json).unwrap();
        assert_eq!(config.first_factors.as_ref().unwrap().len(), 2);
        assert_eq!(
            config.required_secondary_factors.as_ref().unwrap(),
            &vec!["totp".to_string()]
        );
    }

    #[test]
    fn test_tenant_config_from_json_with_providers() {
        let json = json!({
            "tenantId": "t1",
            "thirdParty": {
                "providers": [
                    {"thirdPartyId": "google", "name": "Google"},
                    {"thirdPartyId": "github"}
                ]
            }
        });
        let config = TenantConfig::from_json(&json).unwrap();
        assert_eq!(config.third_party_providers.len(), 2);
        assert_eq!(config.third_party_providers[0].third_party_id, "google");
        assert_eq!(
            config.third_party_providers[0].name,
            Some("Google".to_string())
        );
        assert_eq!(config.third_party_providers[1].third_party_id, "github");
        assert_eq!(config.third_party_providers[1].name, None);
    }

    #[test]
    fn test_tenant_config_from_json_with_core_config() {
        let json = json!({
            "tenantId": "t1",
            "coreConfig": {"key": "value"}
        });
        let config = TenantConfig::from_json(&json).unwrap();
        assert_eq!(config.core_config.get("key").unwrap(), "value");
    }

    #[test]
    fn test_tenant_config_from_json_missing_tenant_id_returns_none() {
        let json = json!({"coreConfig": {}});
        assert!(TenantConfig::from_json(&json).is_none());
    }

    #[test]
    fn test_tenant_config_from_json_null_tenant_id_returns_none() {
        let json = json!({"tenantId": null});
        assert!(TenantConfig::from_json(&json).is_none());
    }
}
