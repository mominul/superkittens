/// Standard JSON response helpers matching Python SDK patterns.
use serde_json::Value;

/// Create a standard "status: OK" JSON response with additional fields.
pub fn ok_response(mut data: serde_json::Map<String, Value>) -> Value {
    data.insert("status".into(), Value::String("OK".into()));
    Value::Object(data)
}

/// Create a standard error response with a message.
pub fn error_response(message: &str) -> Value {
    serde_json::json!({ "message": message })
}
