use crate::error::SuperTokensError;
use serde::{Deserialize, Serialize};

/// A form field with a validation error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorFormField {
    pub id: String,
    pub error: String,
}

/// Error raised when form field validation fails.
#[derive(Debug, Clone)]
pub struct FieldError {
    pub message: String,
    pub form_fields: Vec<ErrorFormField>,
}

impl FieldError {
    pub fn new(message: &str, form_fields: Vec<ErrorFormField>) -> Self {
        Self {
            message: message.to_string(),
            form_fields,
        }
    }
}

impl std::fmt::Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FieldError: {}", self.message)
    }
}

impl std::error::Error for FieldError {}

/// Raise a form field error.
pub fn raise_form_field_exception(
    message: &str,
    form_fields: Vec<ErrorFormField>,
) -> SuperTokensError {
    SuperTokensError::EmailPassword(crate::error::EmailPasswordError::FieldError {
        message: message.to_string(),
        form_fields: form_fields.into_iter().map(|f| (f.id, f.error)).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_form_field_creation() {
        let field = ErrorFormField {
            id: "email".to_string(),
            error: "Invalid email".to_string(),
        };
        assert_eq!(field.id, "email");
        assert_eq!(field.error, "Invalid email");
    }

    #[test]
    fn test_field_error_new() {
        let fields = vec![ErrorFormField {
            id: "password".to_string(),
            error: "Too short".to_string(),
        }];
        let err = FieldError::new("Validation failed", fields.clone());
        assert_eq!(err.message, "Validation failed");
        assert_eq!(err.form_fields.len(), 1);
        assert_eq!(err.form_fields[0].id, "password");
    }

    #[test]
    fn test_field_error_display() {
        let err = FieldError::new("Bad input", vec![]);
        assert_eq!(format!("{}", err), "FieldError: Bad input");
    }

    #[test]
    fn test_raise_form_field_exception_single_field() {
        let fields = vec![ErrorFormField {
            id: "email".to_string(),
            error: "Required".to_string(),
        }];
        let err = raise_form_field_exception("Form error", fields);
        match err {
            SuperTokensError::EmailPassword(crate::error::EmailPasswordError::FieldError {
                message,
                form_fields,
            }) => {
                assert_eq!(message, "Form error");
                assert_eq!(form_fields.len(), 1);
                assert_eq!(
                    form_fields[0],
                    ("email".to_string(), "Required".to_string())
                );
            }
            _ => panic!("Expected EmailPassword FieldError variant"),
        }
    }

    #[test]
    fn test_raise_form_field_exception_multiple_fields() {
        let fields = vec![
            ErrorFormField {
                id: "email".to_string(),
                error: "Invalid".to_string(),
            },
            ErrorFormField {
                id: "password".to_string(),
                error: "Too weak".to_string(),
            },
        ];
        let err = raise_form_field_exception("Multiple errors", fields);
        match err {
            SuperTokensError::EmailPassword(crate::error::EmailPasswordError::FieldError {
                form_fields,
                ..
            }) => {
                assert_eq!(form_fields.len(), 2);
            }
            _ => panic!("Expected EmailPassword FieldError variant"),
        }
    }
}
