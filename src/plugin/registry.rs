//! Plugin registry - tracks loaded plugins

use super::traits::{PluginCapability, PluginMetadata};
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::sync::RwLock;

/// Plugin registry
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, PluginMetadata>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// Register a plugin
    pub fn register(&self, metadata: &PluginMetadata) -> Result<()> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| Error::Internal(format!("Lock error: {}", e)))?;

        if plugins.contains_key(&metadata.name) {
            return Err(Error::Plugin(format!(
                "Plugin '{}' already registered",
                metadata.name
            )));
        }

        plugins.insert(metadata.name.clone(), metadata.clone());
        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister(&self, name: &str) -> Result<()> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| Error::Internal(format!("Lock error: {}", e)))?;

        plugins
            .remove(name)
            .ok_or_else(|| Error::Plugin(format!("Plugin '{}' not found", name)))?;

        Ok(())
    }

    /// Get plugin metadata
    pub fn get(&self, name: &str) -> Option<PluginMetadata> {
        let plugins = self.plugins.read().ok()?;
        plugins.get(name).cloned()
    }

    /// List all registered plugins
    pub fn list(&self) -> Vec<PluginMetadata> {
        self.plugins
            .read()
            .map(|p| p.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Find plugins by capability
    pub fn find_by_capability(&self, capability: &PluginCapability) -> Vec<String> {
        self.plugins
            .read()
            .map(|plugins| {
                plugins
                    .iter()
                    .filter(|(_, meta)| meta.capabilities.contains(capability))
                    .map(|(name, _)| name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
