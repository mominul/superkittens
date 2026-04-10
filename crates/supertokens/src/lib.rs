pub mod auth_utils;
pub mod constants;
pub mod error;
pub mod framework;
pub mod ingredients;
pub mod logger;
pub mod normalised_url_domain;
pub mod normalised_url_path;
pub mod querier;
pub mod recipe;
pub mod recipe_module;
pub mod supertokens;
pub mod types;
pub mod user_context;
pub mod utils;

// Re-export key types at the crate root for convenience.
pub use error::{Result, SuperTokensError};
pub use supertokens::{Supertokens, SupertokensInit};
pub use types::config::{AppInfo, InputAppInfo, SupertokensConfig};
pub use types::user::{LoginMethod, RecipeUserId, User};
pub use user_context::UserContext;
