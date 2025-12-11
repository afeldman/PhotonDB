//! Plugin system for RethinkDB
//!
//! Allows dynamic loading of plugins to extend database functionality.
//! Plugins can add:
//! - Custom query operations
//! - Storage backends
//! - Authentication providers
//! - Data transformations

use crate::error::{Error, Result};
use crate::reql::Datum;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub mod loader;
pub mod registry;
pub mod traits;

pub use loader::PluginLoader;
pub use registry::PluginRegistry;
pub use traits::{Plugin, PluginCapability, PluginMetadata};

/// Plugin manager - central component for plugin lifecycle
pub struct PluginManager {
    registry: Arc<PluginRegistry>,
    loader: PluginLoader,
    plugins: HashMap<String, Arc<dyn Plugin>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
            loader: PluginLoader::new(),
            plugins: HashMap::new(),
        }
    }

    /// Load a plugin from a dynamic library
    pub async fn load_plugin(&mut self, path: PathBuf) -> Result<()> {
        let plugin = self.loader.load(path).await?;
        let metadata = plugin.metadata();

        tracing::info!(
            name = %metadata.name,
            version = %metadata.version,
            "Loading plugin"
        );

        self.registry.register(&metadata)?;
        self.plugins.insert(metadata.name.clone(), plugin);

        Ok(())
    }

    /// Unload a plugin by name
    pub async fn unload_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.remove(name) {
            plugin.shutdown().await?;
            self.registry.unregister(name)?;
            tracing::info!(name = %name, "Plugin unloaded");
            Ok(())
        } else {
            Err(Error::Plugin(format!("Plugin '{}' not found", name)))
        }
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(name).cloned()
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Execute a plugin function
    pub async fn execute(
        &self,
        plugin_name: &str,
        function_name: &str,
        args: Vec<Datum>,
    ) -> Result<Datum> {
        let plugin = self
            .get_plugin(plugin_name)
            .ok_or_else(|| Error::Plugin(format!("Plugin '{}' not found", plugin_name)))?;

        plugin.execute(function_name, args).await
    }

    /// Shutdown all plugins
    pub async fn shutdown(&mut self) -> Result<()> {
        for (name, plugin) in self.plugins.drain() {
            tracing::info!(name = %name, "Shutting down plugin");
            if let Err(e) = plugin.shutdown().await {
                tracing::error!(name = %name, error = %e, "Plugin shutdown error");
            }
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.list_plugins().len(), 0);
    }
}
