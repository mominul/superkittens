use serde::{Deserialize, Serialize};
use std::fmt;

/// A unique identifier for a user within a specific recipe.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeUserId(String);

impl RecipeUserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn get_as_string(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RecipeUserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The recipe that created a login method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecipeId {
    EmailPassword,
    ThirdParty,
    Passwordless,
    Webauthn,
}

impl fmt::Display for RecipeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmailPassword => write!(f, "emailpassword"),
            Self::ThirdParty => write!(f, "thirdparty"),
            Self::Passwordless => write!(f, "passwordless"),
            Self::Webauthn => write!(f, "webauthn"),
        }
    }
}

/// Third-party provider info attached to a login method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThirdPartyInfo {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
}

/// WebAuthn credential info.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WebauthnInfo {
    #[serde(rename = "credentialIds", default)]
    pub credential_ids: Vec<String>,
}

/// Input form of WebauthnInfo for comparison purposes.
#[derive(Debug, Clone)]
pub struct WebauthnInfoInput {
    pub credential_id: String,
}

/// Account information shared across login methods.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "phoneNumber")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "thirdParty")]
    pub third_party: Option<ThirdPartyInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webauthn: Option<WebauthnInfo>,
}

/// A single login method associated with a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginMethod {
    #[serde(rename = "recipeId")]
    pub recipe_id: RecipeId,
    #[serde(rename = "recipeUserId")]
    pub recipe_user_id: RecipeUserId,
    #[serde(rename = "tenantIds")]
    pub tenant_ids: Vec<String>,
    #[serde(rename = "timeJoined")]
    pub time_joined: u64,
    pub verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "phoneNumber")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "thirdParty")]
    pub third_party: Option<ThirdPartyInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webauthn: Option<WebauthnInfo>,
}

impl LoginMethod {
    /// Case-insensitive email comparison after trimming.
    pub fn has_same_email_as(&self, email: Option<&str>) -> bool {
        match (&self.email, email) {
            (Some(a), Some(b)) => a.trim().eq_ignore_ascii_case(b.trim()),
            (None, None) => true,
            _ => false,
        }
    }

    /// Phone number comparison (simple string match after trimming).
    pub fn has_same_phone_number_as(&self, phone_number: Option<&str>) -> bool {
        match (&self.phone_number, phone_number) {
            (Some(a), Some(b)) => a.trim() == b.trim(),
            (None, None) => true,
            _ => false,
        }
    }

    /// Third-party info comparison: case-insensitive id + user_id after trimming.
    pub fn has_same_third_party_info_as(&self, third_party: Option<&ThirdPartyInfo>) -> bool {
        match (&self.third_party, third_party) {
            (Some(a), Some(b)) => {
                a.id.trim().eq_ignore_ascii_case(b.id.trim())
                    && a.user_id.trim().eq_ignore_ascii_case(b.user_id.trim())
            }
            (None, None) => true,
            _ => false,
        }
    }

    /// Check if a given credential_id exists in this login method's webauthn credential_ids.
    pub fn has_same_webauthn_info_as(&self, input: Option<&WebauthnInfoInput>) -> bool {
        match (&self.webauthn, input) {
            (Some(w), Some(inp)) => w.credential_ids.contains(&inp.credential_id),
            (None, None) => true,
            _ => false,
        }
    }

    /// Build from a JSON value returned by the SuperTokens core.
    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json.clone())
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

/// A SuperTokens user, potentially with multiple login methods (account linking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    #[serde(rename = "isPrimaryUser")]
    pub is_primary_user: bool,
    #[serde(rename = "tenantIds")]
    pub tenant_ids: Vec<String>,
    pub emails: Vec<String>,
    #[serde(rename = "phoneNumbers")]
    pub phone_numbers: Vec<String>,
    #[serde(rename = "thirdParty")]
    pub third_party: Vec<ThirdPartyInfo>,
    #[serde(default)]
    pub webauthn: WebauthnInfo,
    #[serde(rename = "loginMethods")]
    pub login_methods: Vec<LoginMethod>,
    #[serde(rename = "timeJoined")]
    pub time_joined: u64,
}

impl User {
    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json.clone())
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_user_id() {
        let id = RecipeUserId::new("abc-123");
        assert_eq!(id.get_as_string(), "abc-123");
        assert_eq!(format!("{}", id), "abc-123");
    }

    #[test]
    fn test_login_method_email_comparison() {
        let lm = LoginMethod {
            recipe_id: RecipeId::EmailPassword,
            recipe_user_id: RecipeUserId::new("u1"),
            tenant_ids: vec!["public".into()],
            time_joined: 0,
            verified: false,
            email: Some("Alice@Example.COM".into()),
            phone_number: None,
            third_party: None,
            webauthn: None,
        };
        assert!(lm.has_same_email_as(Some("alice@example.com")));
        assert!(!lm.has_same_email_as(Some("bob@example.com")));
        assert!(!lm.has_same_email_as(None));
    }

    #[test]
    fn test_user_from_json() {
        let json = serde_json::json!({
            "id": "user-1",
            "isPrimaryUser": false,
            "tenantIds": ["public"],
            "emails": ["test@example.com"],
            "phoneNumbers": [],
            "thirdParty": [],
            "loginMethods": [{
                "recipeId": "emailpassword",
                "recipeUserId": "u1",
                "tenantIds": ["public"],
                "timeJoined": 1000,
                "verified": true,
                "email": "test@example.com"
            }],
            "timeJoined": 1000
        });
        let user = User::from_json(&json).unwrap();
        assert_eq!(user.id, "user-1");
        assert_eq!(user.login_methods.len(), 1);
    }
}
