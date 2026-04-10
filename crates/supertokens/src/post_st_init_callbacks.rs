//! Post-initialization callbacks for the SuperTokens SDK.
//!
//! Recipes can register callbacks during construction that run after
//! `Supertokens::init()` has stored the singleton. This allows recipes
//! to discover each other and register cross-recipe hooks.

use std::sync::Mutex;

type BoxedCallback = Box<dyn FnOnce() + Send>;

static CALLBACKS: Mutex<Vec<BoxedCallback>> = Mutex::new(Vec::new());

/// Register a callback to run after `Supertokens::init()` completes.
pub fn add_post_init_callback(callback: impl FnOnce() + Send + 'static) {
    if let Ok(mut cbs) = CALLBACKS.lock() {
        cbs.push(Box::new(callback));
    }
}

/// Run and clear all registered post-init callbacks.
/// Called by `Supertokens::init()` after the instance is stored.
pub(crate) fn run_post_init_callbacks() {
    let callbacks = {
        let mut cbs = CALLBACKS.lock().expect("PostSTInitCallbacks lock poisoned");
        std::mem::take(&mut *cbs)
    };
    for cb in callbacks {
        cb();
    }
}

/// Clear all registered callbacks without running them (testing only).
#[cfg(feature = "testing")]
pub fn reset() {
    if let Ok(mut cbs) = CALLBACKS.lock() {
        cbs.clear();
    }
}
