use std::sync::Arc;

use super::constants::*;
use super::types::*;
use crate::error::SuperTokensError;
use crate::types::config::AppInfo;

/// Default password validator: 8-100 chars, at least one letter and one number.
pub fn default_password_validator() -> FormFieldValidator {
    Arc::new(|value: String, _tenant_id: String| {
        Box::pin(async move {
            if value.len() < 8 {
                return Ok(Some(
                    "Password must contain at least 8 characters, including a number".to_string(),
                ));
            }
            if value.len() > 100 {
                return Ok(Some(
                    "Password's length must be lesser than 256 characters".to_string(),
                ));
            }
            let has_letter = value.chars().any(|c| c.is_alphabetic());
            let has_number = value.chars().any(|c| c.is_ascii_digit());
            if !has_letter || !has_number {
                return Ok(Some(
                    "Password must contain at least one alphabet and one number".to_string(),
                ));
            }
            Ok(None)
        })
    })
}

/// Default email validator: basic RFC-style check.
pub fn default_email_validator() -> FormFieldValidator {
    Arc::new(|value: String, _tenant_id: String| {
        Box::pin(async move {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Ok(Some("Email cannot be empty".to_string()));
            }
            // Basic email validation
            let parts: Vec<&str> = trimmed.split('@').collect();
            if parts.len() != 2 {
                return Ok(Some("Email is not valid".to_string()));
            }
            let (local, domain) = (parts[0], parts[1]);
            if local.is_empty() || domain.is_empty() {
                return Ok(Some("Email is not valid".to_string()));
            }
            if !domain.contains('.') {
                return Ok(Some("Email is not valid".to_string()));
            }
            let domain_parts: Vec<&str> = domain.split('.').collect();
            if domain_parts.iter().any(|p| p.is_empty()) {
                return Ok(Some("Email is not valid".to_string()));
            }
            Ok(None)
        })
    })
}

/// A no-op validator that always passes.
fn default_validator() -> FormFieldValidator {
    Arc::new(|_value: String, _tenant_id: String| Box::pin(async move { Ok(None) }))
}

/// Normalise sign-up form fields, ensuring email and password fields exist
/// with appropriate validators.
pub fn normalise_sign_up_form_fields(
    form_fields: Option<Vec<InputFormField>>,
) -> Vec<NormalisedFormField> {
    let input_fields = form_fields.unwrap_or_default();
    let mut result: Vec<NormalisedFormField> = Vec::new();

    let mut has_email = false;
    let mut has_password = false;

    for field in &input_fields {
        if field.id == FORM_FIELD_EMAIL_ID {
            has_email = true;
            result.push(NormalisedFormField {
                id: FORM_FIELD_EMAIL_ID.to_string(),
                validate: field
                    .validate
                    .clone()
                    .unwrap_or_else(default_email_validator),
                optional: false,
            });
        } else if field.id == FORM_FIELD_PASSWORD_ID {
            has_password = true;
            result.push(NormalisedFormField {
                id: FORM_FIELD_PASSWORD_ID.to_string(),
                validate: field
                    .validate
                    .clone()
                    .unwrap_or_else(default_password_validator),
                optional: false,
            });
        } else {
            result.push(NormalisedFormField {
                id: field.id.clone(),
                validate: field.validate.clone().unwrap_or_else(default_validator),
                optional: field.optional.unwrap_or(false),
            });
        }
    }

    if !has_email {
        result.push(NormalisedFormField {
            id: FORM_FIELD_EMAIL_ID.to_string(),
            validate: default_email_validator(),
            optional: false,
        });
    }
    if !has_password {
        result.push(NormalisedFormField {
            id: FORM_FIELD_PASSWORD_ID.to_string(),
            validate: default_password_validator(),
            optional: false,
        });
    }

    result
}

/// Normalise sign-in form fields (email + password only, with simple password validator).
fn normalise_sign_in_form_fields(
    sign_up_fields: &[NormalisedFormField],
) -> Vec<NormalisedFormField> {
    let mut result = Vec::new();

    // Email field with the same validator as signup
    if let Some(email_field) = sign_up_fields.iter().find(|f| f.id == FORM_FIELD_EMAIL_ID) {
        result.push(NormalisedFormField {
            id: FORM_FIELD_EMAIL_ID.to_string(),
            validate: email_field.validate.clone(),
            optional: false,
        });
    }

    // Password field with a simple non-empty validator for sign-in
    result.push(NormalisedFormField {
        id: FORM_FIELD_PASSWORD_ID.to_string(),
        validate: default_validator(),
        optional: false,
    });

    result
}

/// Validate and normalise user input into a NormalisedEmailPasswordConfig.
pub fn validate_and_normalise_user_input(
    _app_info: &AppInfo,
    config: EmailPasswordConfig,
) -> Result<NormalisedEmailPasswordConfig, SuperTokensError> {
    let sign_up_form_fields =
        normalise_sign_up_form_fields(config.sign_up_feature.and_then(|f| f.form_fields));

    let sign_in_form_fields = normalise_sign_in_form_fields(&sign_up_form_fields);

    // Password reset form needs the password validator
    let password_field = sign_up_form_fields
        .iter()
        .find(|f| f.id == FORM_FIELD_PASSWORD_ID)
        .cloned()
        .unwrap_or(NormalisedFormField {
            id: FORM_FIELD_PASSWORD_ID.to_string(),
            validate: default_password_validator(),
            optional: false,
        });

    let email_field = sign_up_form_fields
        .iter()
        .find(|f| f.id == FORM_FIELD_EMAIL_ID)
        .cloned()
        .unwrap_or(NormalisedFormField {
            id: FORM_FIELD_EMAIL_ID.to_string(),
            validate: default_email_validator(),
            optional: false,
        });

    let override_ = config.override_.unwrap_or(OverrideConfig {
        functions: None,
        apis: None,
    });

    Ok(NormalisedEmailPasswordConfig {
        sign_up_feature: SignUpFeature {
            form_fields: sign_up_form_fields,
        },
        sign_in_feature: SignInFeature {
            form_fields: sign_in_form_fields,
        },
        reset_password_using_token_feature: ResetPasswordUsingTokenFeature {
            form_fields_for_password_reset_form: vec![password_field],
            form_fields_for_generate_token_form: vec![email_field],
        },
        override_,
    })
}

/// Get the password reset link URL.
pub fn get_password_reset_link(app_info: &AppInfo, token: &str, tenant_id: &str) -> String {
    let origin = app_info
        .website_domain
        .as_ref()
        .map(|d| d.to_string())
        .or_else(|| app_info.origin.as_ref().map(|d| d.to_string()))
        .unwrap_or_default();
    let base_path = &app_info.website_base_path;
    format!(
        "{}{}{}?token={}&tenantId={}",
        origin,
        base_path,
        super::constants::RESET_PASSWORD,
        urlencoding::encode(token),
        urlencoding::encode(tenant_id),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_password_validator() {
        let validator = default_password_validator();

        // Too short
        let result = (validator)("abc".to_string(), "public".to_string())
            .await
            .unwrap();
        assert!(result.is_some());

        // No number
        let result = (validator)("abcdefgh".to_string(), "public".to_string())
            .await
            .unwrap();
        assert!(result.is_some());

        // No letter
        let result = (validator)("12345678".to_string(), "public".to_string())
            .await
            .unwrap();
        assert!(result.is_some());

        // Valid
        let result = (validator)("abcdef12".to_string(), "public".to_string())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_default_email_validator() {
        let validator = default_email_validator();

        // Empty
        let result = (validator)("".to_string(), "public".to_string())
            .await
            .unwrap();
        assert!(result.is_some());

        // No @
        let result = (validator)("test".to_string(), "public".to_string())
            .await
            .unwrap();
        assert!(result.is_some());

        // No domain dot
        let result = (validator)("test@com".to_string(), "public".to_string())
            .await
            .unwrap();
        assert!(result.is_some());

        // Valid
        let result = (validator)("test@example.com".to_string(), "public".to_string())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_normalise_sign_up_form_fields_defaults() {
        let fields = normalise_sign_up_form_fields(None);
        assert_eq!(fields.len(), 2);
        assert!(fields.iter().any(|f| f.id == FORM_FIELD_EMAIL_ID));
        assert!(fields.iter().any(|f| f.id == FORM_FIELD_PASSWORD_ID));
    }

    #[test]
    fn test_normalise_sign_up_form_fields_custom() {
        let fields = normalise_sign_up_form_fields(Some(vec![InputFormField {
            id: "name".to_string(),
            validate: None,
            optional: Some(true),
        }]));
        // Should have email, password, and name
        assert_eq!(fields.len(), 3);
        assert!(fields.iter().any(|f| f.id == "name" && f.optional));
    }
}
