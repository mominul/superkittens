use tracing_subscriber::EnvFilter;

static INIT: std::sync::Once = std::sync::Once::new();

/// Enable debug logging via the `tracing` crate.
///
/// Sets up a subscriber that filters based on `SUPERTOKENS_DEBUG` env var
/// or enables all `supertokens` crate logs at debug level.
pub fn enable_debug_logging() {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_env("SUPERTOKENS_DEBUG")
            .unwrap_or_else(|_| EnvFilter::new("supertokens=debug"));

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .init();
    });
}

/// Log a debug message with the `supertokens` target.
#[macro_export]
macro_rules! st_log_debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "supertokens", $($arg)*)
    };
}

/// Log an info message with the `supertokens` target.
#[macro_export]
macro_rules! st_log_info {
    ($($arg:tt)*) => {
        tracing::info!(target: "supertokens", $($arg)*)
    };
}

/// Log a warning with the `supertokens` target.
#[macro_export]
macro_rules! st_log_warn {
    ($($arg:tt)*) => {
        tracing::warn!(target: "supertokens", $($arg)*)
    };
}

/// Log an error with the `supertokens` target.
#[macro_export]
macro_rules! st_log_error {
    ($($arg:tt)*) => {
        tracing::error!(target: "supertokens", $($arg)*)
    };
}
