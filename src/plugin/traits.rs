//! Plugin trait definitions

use crate::error::Result;
use crate::reql::Datum;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Plugin capability flags
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginCapability {
    /// Adds custom ReQL operations
    QueryOperations,
    /// Provides storage backend
    StorageBackend,
    /// Authentication provider
    Authentication,
    /// Data transformation/aggregation
    Transformation,
    /// Custom network protocol
    Protocol,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub capabilities: Vec<PluginCapability>,
}

/// Main plugin trait
///
/// All plugins must implement this trait to be loadable by RethinkDB
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize the plugin
    /// Called once after plugin is loaded
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Shutdown the plugin
    /// Called before plugin is unloaded
    fn shutdown(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }

    /// Execute a plugin function
    ///
    /// # Arguments
    /// * `function` - Name of the function to execute
    /// * `args` - Function arguments as Datum values
    ///
    /// # Returns
    /// Result Datum value
    fn execute(
        &self,
        function: &str,
        args: Vec<Datum>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Datum>> + Send + '_>>;

    /// List available functions provided by this plugin
    fn list_functions(&self) -> Vec<String> {
        vec![]
    }
}

/// Example plugin implementation
#[derive(Debug)]
pub struct ExamplePlugin {
    metadata: PluginMetadata,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "example".to_string(),
                version: "1.0.0".to_string(),
                author: "RethinkDB Team".to_string(),
                description: "Example plugin".to_string(),
                capabilities: vec![PluginCapability::QueryOperations],
            },
        }
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn execute(
        &self,
        function: &str,
        args: Vec<Datum>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Datum>> + Send + '_>> {
        let function = function.to_string();
        Box::pin(async move {
            match function.as_str() {
                "hello" => {
                    let name = args.first().and_then(|d| d.as_string()).unwrap_or("World");
                    Ok(Datum::String(format!("Hello, {}!", name)))
                }
                _ => Err(crate::error::Error::Plugin(format!(
                    "Unknown function: {}",
                    function
                ))),
            }
        })
    }

    fn list_functions(&self) -> Vec<String> {
        vec!["hello".to_string()]
    }
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_plugin() {
        let plugin = ExamplePlugin::new();
        let result = plugin
            .execute("hello", vec![Datum::String("Rust".to_string())])
            .await
            .unwrap();

        assert_eq!(result, Datum::String("Hello, Rust!".to_string()));
    }
}
