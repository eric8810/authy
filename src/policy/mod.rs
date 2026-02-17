use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

use crate::error::{AuthyError, Result};

/// A policy defines which secrets a scope can access.
/// Deny patterns override allow patterns. Default is deny.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub name: String,
    pub description: Option<String>,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

impl Policy {
    pub fn new(name: String, allow: Vec<String>, deny: Vec<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            name,
            description: None,
            allow,
            deny,
            created_at: now,
            modified_at: now,
        }
    }

    /// Check if a secret name is allowed by this policy.
    /// Deny overrides allow. Default deny.
    pub fn can_read(&self, secret_name: &str) -> Result<bool> {
        let deny_set = build_globset(&self.deny)?;
        if deny_set.is_match(secret_name) {
            return Ok(false);
        }

        let allow_set = build_globset(&self.allow)?;
        Ok(allow_set.is_match(secret_name))
    }

    /// Return all secret names from a list that this policy allows.
    pub fn filter_secrets<'a>(&self, names: &[&'a str]) -> Result<Vec<&'a str>> {
        let mut allowed = Vec::new();
        for name in names {
            if self.can_read(name)? {
                allowed.push(*name);
            }
        }
        Ok(allowed)
    }
}

fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern)
            .map_err(|e| AuthyError::Other(format!("Invalid glob pattern '{}': {}", pattern, e)))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|e| AuthyError::Other(format!("Failed to build glob set: {}", e)))
}
