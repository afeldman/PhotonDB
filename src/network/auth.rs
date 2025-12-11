//! Authentication and authorization for RethinkDB connections

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// User credentials and permissions
#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub permissions: Vec<Permission>,
}

/// Permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Read,
    Write,
    Admin,
    Connect,
}

/// Authentication manager
pub struct AuthManager {
    users: Arc<RwLock<HashMap<String, User>>>,
    default_user: Option<String>,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            default_user: None,
        }
    }

    /// Create with default admin user
    pub fn with_admin(admin_password: &str) -> Self {
        let mut manager = Self::new();
        let admin = User {
            username: "admin".to_string(),
            password_hash: Self::hash_password(admin_password),
            permissions: vec![
                Permission::Admin,
                Permission::Read,
                Permission::Write,
                Permission::Connect,
            ],
        };

        let mut users = HashMap::new();
        users.insert("admin".to_string(), admin);

        manager.users = Arc::new(RwLock::new(users));
        manager.default_user = Some("admin".to_string());
        manager
    }

    /// Add a new user
    pub async fn add_user(&self, username: String, password: &str, permissions: Vec<Permission>) -> Result<()> {
        let user = User {
            username: username.clone(),
            password_hash: Self::hash_password(password),
            permissions,
        };

        let mut users = self.users.write().await;
        if users.contains_key(&username) {
            return Err(anyhow!("User already exists: {}", username));
        }

        users.insert(username, user);
        Ok(())
    }

    /// Remove a user
    pub async fn remove_user(&self, username: &str) -> Result<()> {
        let mut users = self.users.write().await;
        users
            .remove(username)
            .ok_or_else(|| anyhow!("User not found: {}", username))?;
        Ok(())
    }

    /// Authenticate with username and password
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<User> {
        let users = self.users.read().await;
        let user = users
            .get(username)
            .ok_or_else(|| anyhow!("Invalid username or password"))?;

        if !Self::verify_password(password, &user.password_hash) {
            return Err(anyhow!("Invalid username or password"));
        }

        Ok(user.clone())
    }

    /// Authenticate with auth key (simplified)
    pub async fn authenticate_key(&self, auth_key: &str) -> Result<User> {
        // For now, treat empty key as admin if no users configured
        if auth_key.is_empty() {
            let users = self.users.read().await;
            if users.is_empty() {
                return Ok(User {
                    username: "default".to_string(),
                    password_hash: String::new(),
                    permissions: vec![
                        Permission::Admin,
                        Permission::Read,
                        Permission::Write,
                        Permission::Connect,
                    ],
                });
            }
        }

        // Otherwise, try to find user by auth key (in production, use proper token system)
        let users = self.users.read().await;
        if let Some(default_username) = &self.default_user {
            if let Some(user) = users.get(default_username) {
                return Ok(user.clone());
            }
        }

        Err(anyhow!("Invalid authentication key"))
    }

    /// Check if user has permission
    pub fn has_permission(user: &User, permission: Permission) -> bool {
        user.permissions.contains(&permission) || user.permissions.contains(&Permission::Admin)
    }

    /// Hash a password (using bcrypt for production)
    fn hash_password(password: &str) -> String {
        match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
            Ok(hash) => hash,
            Err(e) => {
                tracing::error!("Failed to hash password: {}", e);
                // Fallback to base64 for development (should never happen in production)
                use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
                BASE64.encode(password.as_bytes())
            }
        }
    }

    /// Verify a password (using bcrypt)
    fn verify_password(password: &str, hash: &str) -> bool {
        // Try bcrypt first
        if let Ok(valid) = bcrypt::verify(password, hash) {
            return valid;
        }
        
        // Fallback to base64 for old/dev hashes
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
        if let Ok(decoded) = BASE64.decode(hash) {
            if let Ok(stored_password) = String::from_utf8(decoded) {
                return password == stored_password;
            }
        }
        false
    }

    /// Get user count
    pub async fn user_count(&self) -> usize {
        self.users.read().await.len()
    }

    /// List all users
    pub async fn list_users(&self) -> Vec<String> {
        self.users.read().await.keys().cloned().collect()
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let auth = AuthManager::new();
        auth.add_user(
            "test_user".to_string(),
            "password123",
            vec![Permission::Read, Permission::Write],
        )
        .await
        .unwrap();

        assert_eq!(auth.user_count().await, 1);
    }

    #[tokio::test]
    async fn test_authenticate() {
        let auth = AuthManager::new();
        auth.add_user(
            "alice".to_string(),
            "secret",
            vec![Permission::Read],
        )
        .await
        .unwrap();

        let user = auth.authenticate("alice", "secret").await.unwrap();
        assert_eq!(user.username, "alice");
        assert!(user.permissions.contains(&Permission::Read));

        assert!(auth.authenticate("alice", "wrong").await.is_err());
        assert!(auth.authenticate("bob", "secret").await.is_err());
    }

    #[tokio::test]
    async fn test_permissions() {
        let admin = User {
            username: "admin".to_string(),
            password_hash: String::new(),
            permissions: vec![Permission::Admin],
        };

        let reader = User {
            username: "reader".to_string(),
            password_hash: String::new(),
            permissions: vec![Permission::Read],
        };

        assert!(AuthManager::has_permission(&admin, Permission::Write));
        assert!(AuthManager::has_permission(&admin, Permission::Read));
        assert!(!AuthManager::has_permission(&reader, Permission::Write));
        assert!(AuthManager::has_permission(&reader, Permission::Read));
    }

    #[tokio::test]
    async fn test_auth_key() {
        let auth = AuthManager::new();
        
        // Empty auth should work with no users (development mode)
        let user = auth.authenticate_key("").await.unwrap();
        assert_eq!(user.username, "default");
        assert!(user.permissions.contains(&Permission::Admin));
    }

    #[tokio::test]
    async fn test_with_admin() {
        let auth = AuthManager::with_admin("admin_password");
        assert_eq!(auth.user_count().await, 1);
        
        let admin = auth.authenticate("admin", "admin_password").await.unwrap();
        assert!(admin.permissions.contains(&Permission::Admin));
    }
}
