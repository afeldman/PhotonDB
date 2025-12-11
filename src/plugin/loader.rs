//! Plugin loader - handles dynamic loading of plugins

use super::traits::Plugin;
use crate::error::{Error, Result};
use std::path::PathBuf;
use std::sync::Arc;

/// Plugin loader
pub struct PluginLoader {
    // Future: Store loaded libraries for unloading
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {}
    }

    /// Load a plugin from a dynamic library
    ///
    /// # Safety
    /// This loads external code. Only load trusted plugins!
    pub async fn load(&self, _path: PathBuf) -> Result<Arc<dyn Plugin>> {
        // TODO: Implement actual dynamic loading with libloading
        // For now, return error as placeholder
        Err(Error::Plugin(
            "Dynamic plugin loading not yet implemented. Use built-in plugins.".to_string(),
        ))
    }

    /// Load a built-in plugin by name
    pub fn load_builtin(&self, name: &str) -> Result<Arc<dyn Plugin>> {
        match name {
            "example" => Ok(Arc::new(super::traits::ExamplePlugin::new())),
            _ => Err(Error::Plugin(format!("Unknown built-in plugin: {}", name))),
        }
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
