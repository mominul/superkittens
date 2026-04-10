use thiserror::Error;

#[derive(Debug, Error)]
pub enum SuperTokensError {
    #[error("General error: {message}")]
    General {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Bad input error: {message}")]
    BadInput { message: String },

    #[error("Plugin error: {message}")]
    Plugin { message: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Querier error: {message} (status {status_code})")]
    Querier {
        message: String,
        status_code: u16,
        response_text: Option<String>,
    },

    #[error("No compatible CDI version found. SDK supports: {sdk_versions}, Core supports: {core_versions}")]
    IncompatibleCdiVersion {
        sdk_versions: String,
        core_versions: String,
    },

    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    #[error("EmailPassword error: {0}")]
    EmailPassword(#[from] EmailPasswordError),
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },

    #[error("Try refresh token: {message}")]
    TryRefreshToken { message: String },

    #[error("Token theft detected: user_id={user_id}, session_handle={session_handle}")]
    TokenTheftDetected {
        user_id: String,
        session_handle: String,
    },

    #[error("Invalid claims: {0:?}")]
    InvalidClaims(Vec<ClaimValidationError>),

    #[error("Clear duplicate session cookies")]
    ClearDuplicateSessionCookies,
}

#[derive(Debug, Clone)]
pub struct ClaimValidationError {
    pub id: String,
    pub reason: Option<serde_json::Value>,
}

#[derive(Debug, Error)]
pub enum EmailPasswordError {
    #[error("Field error: {message}")]
    FieldError {
        message: String,
        form_fields: Vec<(String, String)>,
    },
}

pub type Result<T> = std::result::Result<T, SuperTokensError>;

pub fn raise_general_exception(msg: impl Into<String>) -> SuperTokensError {
    SuperTokensError::General {
        message: msg.into(),
        source: None,
    }
}

pub fn raise_bad_input_exception(msg: impl Into<String>) -> SuperTokensError {
    SuperTokensError::BadInput {
        message: msg.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raise_general_exception() {
        let err = raise_general_exception("something went wrong");
        match &err {
            SuperTokensError::General { message, source } => {
                assert_eq!(message, "something went wrong");
                assert!(source.is_none());
            }
            _ => panic!("Expected General variant"),
        }
        assert!(format!("{}", err).contains("something went wrong"));
    }

    #[test]
    fn test_raise_bad_input_exception() {
        let err = raise_bad_input_exception("bad input");
        match &err {
            SuperTokensError::BadInput { message } => {
                assert_eq!(message, "bad input");
            }
            _ => panic!("Expected BadInput variant"),
        }
        assert!(format!("{}", err).contains("bad input"));
    }

    #[test]
    fn test_general_error_display() {
        let err = SuperTokensError::General {
            message: "test msg".to_string(),
            source: None,
        };
        assert_eq!(format!("{}", err), "General error: test msg");
    }

    #[test]
    fn test_bad_input_display() {
        let err = SuperTokensError::BadInput {
            message: "invalid".to_string(),
        };
        assert_eq!(format!("{}", err), "Bad input error: invalid");
    }

    #[test]
    fn test_session_error_display() {
        let err = SessionError::Unauthorized {
            message: "expired".to_string(),
        };
        assert!(format!("{}", err).contains("expired"));

        let err2 = SessionError::TryRefreshToken {
            message: "refresh needed".to_string(),
        };
        assert!(format!("{}", err2).contains("refresh needed"));
    }

    #[test]
    fn test_querier_error_display() {
        let err = SuperTokensError::Querier {
            message: "not found".to_string(),
            status_code: 404,
            response_text: Some("{}".to_string()),
        };
        let display = format!("{}", err);
        assert!(display.contains("not found"));
        assert!(display.contains("404"));
    }
}
