mod common;

use supertokens::UserContext;

#[test]
fn test_user_context_insert_and_get() {
    let mut ctx = UserContext::new();
    ctx.insert("key", "value".to_string());
    assert_eq!(ctx.get::<String>("key"), Some(&"value".to_string()));
}

#[test]
fn test_user_context_type_mismatch_returns_none() {
    let mut ctx = UserContext::new();
    ctx.insert("count", 42u32);
    assert_eq!(ctx.get::<String>("count"), None);
}

#[test]
fn test_user_context_overwrite() {
    let mut ctx = UserContext::new();
    ctx.insert("key", "first".to_string());
    ctx.insert("key", "second".to_string());
    assert_eq!(ctx.get::<String>("key"), Some(&"second".to_string()));
}

#[test]
fn test_user_context_remove() {
    let mut ctx = UserContext::new();
    ctx.insert("key", 1u32);
    ctx.remove("key");
    assert_eq!(ctx.get::<u32>("key"), None);
}

#[test]
fn test_user_context_contains_key() {
    let mut ctx = UserContext::new();
    assert!(!ctx.contains_key("key"));
    ctx.insert("key", true);
    assert!(ctx.contains_key("key"));
}

#[test]
fn test_user_context_multiple_types() {
    let mut ctx = UserContext::new();
    ctx.insert("string", "hello".to_string());
    ctx.insert("number", 42u64);
    ctx.insert("flag", true);

    assert_eq!(ctx.get::<String>("string"), Some(&"hello".to_string()));
    assert_eq!(ctx.get::<u64>("number"), Some(&42u64));
    assert_eq!(ctx.get::<bool>("flag"), Some(&true));
}
