use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// A type-erased context map that flows through all SDK operations.
///
/// Replaces Python's `Dict[str, Any]` user_context parameter.
/// Uses `Arc<dyn Any + Send + Sync>` for thread-safe, cloneable values.
#[derive(Debug, Clone, Default)]
pub struct UserContext {
    map: HashMap<String, Arc<dyn Any + Send + Sync>>,
}

impl UserContext {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert a typed value into the context.
    pub fn insert<T: Any + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.map.insert(key.into(), Arc::new(value));
    }

    /// Insert a pre-wrapped Arc value.
    pub fn insert_arc(&mut self, key: impl Into<String>, value: Arc<dyn Any + Send + Sync>) {
        self.map.insert(key.into(), value);
    }

    /// Get a reference to a typed value.
    pub fn get<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.map.get(key).and_then(|v| v.downcast_ref::<T>())
    }

    /// Get an Arc to a typed value.
    pub fn get_arc<T: Any + Send + Sync>(&self, key: &str) -> Option<Arc<T>> {
        self.map
            .get(key)
            .and_then(|v| Arc::clone(v).downcast::<T>().ok())
    }

    /// Check if a key exists.
    pub fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// Remove a value by key.
    pub fn remove(&mut self, key: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.map.remove(key)
    }

    /// Get the raw map for inspection.
    pub fn raw(&self) -> &HashMap<String, Arc<dyn Any + Send + Sync>> {
        &self.map
    }
}

/// Internal keys used by the SDK (the `_default` namespace from Python).
pub mod internal_keys {
    pub const REQUEST: &str = "_default.request";
    pub const KEEP_CACHE_ALIVE: &str = "_default.keep_cache_alive";
    pub const CORE_CALL_CACHE: &str = "_default.core_call_cache";
    pub const GLOBAL_CACHE_TAG: &str = "_default.global_cache_tag";
}

/// Core call cache stored in user context.
pub type CoreCallCache = HashMap<String, serde_json::Value>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut ctx = UserContext::new();
        ctx.insert("name", "Alice".to_string());
        assert_eq!(ctx.get::<String>("name"), Some(&"Alice".to_string()));
    }

    #[test]
    fn test_get_wrong_type() {
        let mut ctx = UserContext::new();
        ctx.insert("count", 42u32);
        assert_eq!(ctx.get::<String>("count"), None);
        assert_eq!(ctx.get::<u32>("count"), Some(&42u32));
    }

    #[test]
    fn test_contains_key() {
        let mut ctx = UserContext::new();
        ctx.insert("x", 1u32);
        assert!(ctx.contains_key("x"));
        assert!(!ctx.contains_key("y"));
    }

    #[test]
    fn test_remove() {
        let mut ctx = UserContext::new();
        ctx.insert("x", 1u32);
        ctx.remove("x");
        assert!(!ctx.contains_key("x"));
    }
}
