use super::super::constants::{FORM_FIELD_EMAIL_ID, FORM_FIELD_PASSWORD_ID};
use super::super::errors::{raise_form_field_exception, ErrorFormField};
use super::super::types::{FormField, NormalisedFormField};
use crate::error::SuperTokensError;

/// Validate form fields from a request against the configured form fields.
///
/// Returns the validated form fields on success, or raises a FieldError.
pub async fn validate_form_fields_or_throw_error(
    config_form_fields: &[NormalisedFormField],
    form_fields_raw: &[FormField],
    tenant_id: &str,
) -> Result<Vec<FormField>, SuperTokensError> {
    let mut validated_fields = Vec::new();
    let mut errors: Vec<ErrorFormField> = Vec::new();

    for config_field in config_form_fields {
        // Find the matching input field
        let input_field = form_fields_raw.iter().find(|f| f.id == config_field.id);

        match input_field {
            Some(field) => {
                let value = field.value.trim().to_string();

                // Email and password must be strings (they are, since FormField.value is String)
                // but check they're not empty for required fields
                if !config_field.optional && value.is_empty() {
                    errors.push(ErrorFormField {
                        id: config_field.id.clone(),
                        error: "Field is not optional".to_string(),
                    });
                    continue;
                }

                // Run the validator
                if !value.is_empty()
                    || config_field.id == FORM_FIELD_EMAIL_ID
                    || config_field.id == FORM_FIELD_PASSWORD_ID
                {
                    let validation_error =
                        (config_field.validate)(value.clone(), tenant_id.to_string()).await?;

                    if let Some(error_msg) = validation_error {
                        errors.push(ErrorFormField {
                            id: config_field.id.clone(),
                            error: error_msg,
                        });
                        continue;
                    }
                }

                validated_fields.push(FormField {
                    id: config_field.id.clone(),
                    value,
                });
            }
            None => {
                if !config_field.optional {
                    errors.push(ErrorFormField {
                        id: config_field.id.clone(),
                        error: "Field is not optional".to_string(),
                    });
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(raise_form_field_exception(
            "Error in input formFields",
            errors,
        ));
    }

    Ok(validated_fields)
}
