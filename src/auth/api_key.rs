// API Key Authentication
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub name: Option<String>,
    pub rate_limit: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyValidator {
    valid_keys: HashSet<String>,
    keys_with_metadata: Vec<ApiKey>,
}

impl ApiKeyValidator {
    /// Create a new validator by reading API_KEY_* environment variables
    pub fn from_env() -> Self {
        let mut valid_keys = HashSet::new();
        let mut keys_with_metadata = Vec::new();

        // Read all environment variables
        for (key, value) in env::vars() {
            // Check if the key starts with API_KEY_
            if let Some(key_suffix) = key.strip_prefix("API_KEY_") {
                // Parse the value - could be just a key or JSON with metadata
                if value.starts_with('{') {
                    // Try to parse as JSON with metadata
                    match serde_json::from_str::<ApiKey>(&value) {
                        Ok(api_key) => {
                            info!("Loaded API key {} with metadata", key_suffix);
                            valid_keys.insert(api_key.key.clone());
                            keys_with_metadata.push(api_key);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to parse API_KEY_{} as JSON: {}, treating as plain key",
                                key_suffix, e
                            );
                            valid_keys.insert(value.clone());
                            keys_with_metadata.push(ApiKey {
                                key: value.clone(),
                                name: Some(format!("Key {}", key_suffix)),
                                rate_limit: None,
                            });
                        }
                    }
                } else {
                    // Plain API key string
                    info!("Loaded API key {}", key_suffix);
                    valid_keys.insert(value.clone());
                    keys_with_metadata.push(ApiKey {
                        key: value.clone(),
                        name: Some(format!("Key {}", key_suffix)),
                        rate_limit: None,
                    });
                }
            }
        }

        // Also support legacy API_KEYS environment variable (comma-separated)
        if let Ok(api_keys_csv) = env::var("API_KEYS") {
            info!("Loading keys from API_KEYS environment variable");
            for key in api_keys_csv.split(',').map(|s| s.trim()) {
                if !key.is_empty() {
                    valid_keys.insert(key.to_string());
                    keys_with_metadata.push(ApiKey {
                        key: key.to_string(),
                        name: Some("Legacy key".to_string()),
                        rate_limit: None,
                    });
                }
            }
        }

        info!("Loaded {} API keys total", valid_keys.len());
        debug!("Valid keys count: {:?}", valid_keys.len());

        Self {
            valid_keys,
            keys_with_metadata,
        }
    }

    /// Create a validator from a static list (for testing)
    pub fn from_keys(keys: Vec<String>) -> Self {
        let mut valid_keys = HashSet::new();
        let mut keys_with_metadata = Vec::new();

        for (i, key) in keys.iter().enumerate() {
            valid_keys.insert(key.clone());
            keys_with_metadata.push(ApiKey {
                key: key.clone(),
                name: Some(format!("Static key {}", i + 1)),
                rate_limit: None,
            });
        }

        Self {
            valid_keys,
            keys_with_metadata,
        }
    }

    /// Validate an API key
    pub fn validate(&self, key: &str) -> bool {
        self.valid_keys.contains(key)
    }

    /// Get metadata for a key
    pub fn get_key_metadata(&self, key: &str) -> Option<&ApiKey> {
        self.keys_with_metadata.iter().find(|k| k.key == key)
    }

    /// Get the number of loaded keys
    pub fn key_count(&self) -> usize {
        self.valid_keys.len()
    }

    /// Check if any keys are loaded
    pub fn has_keys(&self) -> bool {
        !self.valid_keys.is_empty()
    }

    /// Reload keys from environment (for key rotation)
    pub fn reload(&mut self) {
        *self = Self::from_env();
        info!("Reloaded API keys, now have {} keys", self.key_count());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_keys() {
        let validator =
            ApiKeyValidator::from_keys(vec!["test-key-1".to_string(), "test-key-2".to_string()]);

        assert!(validator.validate("test-key-1"));
        assert!(validator.validate("test-key-2"));
        assert!(!validator.validate("invalid-key"));
        assert_eq!(validator.key_count(), 2);
    }

    #[test]
    fn test_get_metadata() {
        let validator = ApiKeyValidator::from_keys(vec!["test-key".to_string()]);

        let metadata = validator.get_key_metadata("test-key");
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().key, "test-key");
    }
}
