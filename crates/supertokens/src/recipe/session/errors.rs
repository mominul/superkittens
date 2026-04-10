use super::types::ClaimValidationError;
use crate::types::user::RecipeUserId;

/// Session-specific errors.
#[derive(Debug)]
pub enum SessionError {
    Unauthorised {
        message: String,
        clear_tokens: bool,
    },

    TryRefreshToken {
        message: String,
    },

    TokenTheftDetected {
        user_id: String,
        recipe_user_id: RecipeUserId,
        session_handle: String,
    },

    InvalidClaims {
        message: String,
        payload: Vec<ClaimValidationError>,
    },

    ClearDuplicateSessionCookies {
        message: String,
    },
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorised { message, .. } => write!(f, "Unauthorised: {}", message),
            Self::TryRefreshToken { message } => write!(f, "TryRefreshToken: {}", message),
            Self::TokenTheftDetected {
                user_id,
                session_handle,
                ..
            } => write!(
                f,
                "TokenTheftDetected: user_id={}, session_handle={}",
                user_id, session_handle
            ),
            Self::InvalidClaims { message, .. } => write!(f, "InvalidClaims: {}", message),
            Self::ClearDuplicateSessionCookies { message } => {
                write!(f, "ClearDuplicateSessionCookies: {}", message)
            }
        }
    }
}

impl std::error::Error for SessionError {}

impl SessionError {
    pub fn unauthorised(msg: impl Into<String>) -> Self {
        Self::Unauthorised {
            message: msg.into(),
            clear_tokens: true,
        }
    }

    pub fn unauthorised_no_clear(msg: impl Into<String>) -> Self {
        Self::Unauthorised {
            message: msg.into(),
            clear_tokens: false,
        }
    }

    pub fn try_refresh_token(msg: impl Into<String>) -> Self {
        Self::TryRefreshToken {
            message: msg.into(),
        }
    }

    pub fn token_theft_detected(
        user_id: impl Into<String>,
        recipe_user_id: RecipeUserId,
        session_handle: impl Into<String>,
    ) -> Self {
        Self::TokenTheftDetected {
            user_id: user_id.into(),
            recipe_user_id,
            session_handle: session_handle.into(),
        }
    }

    pub fn invalid_claims(msg: impl Into<String>, payload: Vec<ClaimValidationError>) -> Self {
        Self::InvalidClaims {
            message: msg.into(),
            payload,
        }
    }
}

/// Convert SessionError into the top-level SuperTokensError.
impl From<SessionError> for crate::error::SuperTokensError {
    fn from(e: SessionError) -> Self {
        match e {
            SessionError::Unauthorised { message, .. } => {
                crate::error::SuperTokensError::Session(crate::error::SessionError::Unauthorized {
                    message,
                })
            }
            SessionError::TryRefreshToken { message } => crate::error::SuperTokensError::Session(
                crate::error::SessionError::TryRefreshToken { message },
            ),
            SessionError::TokenTheftDetected {
                user_id,
                session_handle,
                ..
            } => crate::error::SuperTokensError::Session(
                crate::error::SessionError::TokenTheftDetected {
                    user_id,
                    session_handle,
                },
            ),
            SessionError::InvalidClaims { payload, .. } => {
                crate::error::SuperTokensError::Session(crate::error::SessionError::InvalidClaims(
                    payload
                        .into_iter()
                        .map(|e| crate::error::ClaimValidationError {
                            id: e.id,
                            reason: e.reason,
                        })
                        .collect(),
                ))
            }
            SessionError::ClearDuplicateSessionCookies { .. } => {
                crate::error::SuperTokensError::Session(
                    crate::error::SessionError::ClearDuplicateSessionCookies,
                )
            }
        }
    }
}
