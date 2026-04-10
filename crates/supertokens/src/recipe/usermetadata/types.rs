use serde_json::Value;

/// Result of get/update user metadata.
#[derive(Debug, Clone)]
pub struct MetadataResult {
    pub metadata: serde_json::Map<String, Value>,
}

/// Result of clearing user metadata.
#[derive(Debug, Clone)]
pub struct ClearUserMetadataResult;
