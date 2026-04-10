/// API route for creating a new passwordless code.
pub const CREATE_CODE_API: &str = "/signinup/code";

/// API route for resending a passwordless code.
pub const RESEND_CODE_API: &str = "/signinup/code/resend";

/// API route for consuming a passwordless code.
pub const CONSUME_CODE_API: &str = "/signinup/code/consume";

/// API route for checking if an email exists for passwordless.
pub const DOES_EMAIL_EXIST_API: &str = "/passwordless/email/exists";

/// API route for checking if a phone number exists for passwordless.
pub const DOES_PHONE_NUMBER_EXIST_API: &str = "/passwordless/phonenumber/exists";
