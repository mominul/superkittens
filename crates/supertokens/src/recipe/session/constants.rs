pub const SESSION_REFRESH: &str = "/session/refresh";
pub const SIGNOUT: &str = "/signout";

pub const ACCESS_TOKEN_COOKIE_KEY: &str = "sAccessToken";
pub const REFRESH_TOKEN_COOKIE_KEY: &str = "sRefreshToken";

pub const FRONT_TOKEN_HEADER_SET_KEY: &str = "front-token";
pub const ANTI_CSRF_HEADER_KEY: &str = "anti-csrf";
pub const RID_HEADER_KEY: &str = "rid";
pub const AUTH_MODE_HEADER_KEY: &str = "st-auth-mode";
pub const AUTHORIZATION_HEADER_KEY: &str = "authorization";
pub const ACCESS_TOKEN_HEADER_KEY: &str = "st-access-token";
pub const REFRESH_TOKEN_HEADER_KEY: &str = "st-refresh-token";
pub const ACCESS_CONTROL_EXPOSE_HEADERS: &str = "Access-Control-Expose-Headers";

pub const PROTECTED_PROPS: &[&str] = &[
    "sub",
    "iat",
    "exp",
    "sessionHandle",
    "parentRefreshTokenHash1",
    "refreshTokenHash1",
    "antiCsrfToken",
    "rsub",
    "tId",
];

pub const DEFAULT_TENANT_ID: &str = "public";
