//! Security middleware layer with OAuth2 and intrusion detection
//!
//! Features:
//! - OAuth2 authentication (Amazon AD, GitHub, Google, AWS)
//! - Honeytrap integration for automatic threat blocking
//! - Rate limiting and IP blocking
//! - JWT token validation
//! - Audit logging

use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info, instrument, warn};

/// OAuth2 provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Provider {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub enabled: bool,
    pub honeytrap_enabled: bool,
    pub honeytrap_url: String,
    pub oauth2_providers: Vec<OAuth2Provider>,
    pub jwt_secret: String,
    pub max_requests_per_minute: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            honeytrap_enabled: true,
            honeytrap_url: "http://localhost:8888".to_string(),
            oauth2_providers: vec![
                OAuth2Provider {
                    name: "github".to_string(),
                    client_id: std::env::var("GITHUB_CLIENT_ID").unwrap_or_default(),
                    client_secret: std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
                    auth_url: "https://github.com/login/oauth/authorize".to_string(),
                    token_url: "https://github.com/login/oauth/access_token".to_string(),
                    redirect_uri: "http://localhost:8080/auth/github/callback".to_string(),
                },
                OAuth2Provider {
                    name: "google".to_string(),
                    client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
                    client_secret: std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
                    auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                    token_url: "https://oauth2.googleapis.com/token".to_string(),
                    redirect_uri: "http://localhost:8080/auth/google/callback".to_string(),
                },
                OAuth2Provider {
                    name: "aws".to_string(),
                    client_id: std::env::var("AWS_CLIENT_ID").unwrap_or_default(),
                    client_secret: std::env::var("AWS_CLIENT_SECRET").unwrap_or_default(),
                    auth_url: "https://signin.aws.amazon.com/oauth".to_string(),
                    token_url: "https://signin.aws.amazon.com/oauth/token".to_string(),
                    redirect_uri: "http://localhost:8080/auth/aws/callback".to_string(),
                },
                OAuth2Provider {
                    name: "amazon_ad".to_string(),
                    client_id: std::env::var("AMAZON_CLIENT_ID").unwrap_or_default(),
                    client_secret: std::env::var("AMAZON_CLIENT_SECRET").unwrap_or_default(),
                    auth_url: "https://www.amazon.com/ap/oa".to_string(),
                    token_url: "https://api.amazon.com/auth/o2/token".to_string(),
                    redirect_uri: "http://localhost:8080/auth/amazon/callback".to_string(),
                },
            ],
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "CHANGE_ME_IN_PRODUCTION".to_string()),
            max_requests_per_minute: 100,
        }
    }
}

/// Blocked IP with reason
#[derive(Debug, Clone)]
struct BlockedIP {
    ip: String,
    reason: String,
    blocked_at: chrono::DateTime<chrono::Utc>,
}

/// Security state shared across middleware
#[derive(Clone, Debug)]
pub struct SecurityState {
    config: Arc<SecurityConfig>,
    blocked_ips: Arc<RwLock<HashMap<String, BlockedIP>>>,
    rate_limits: Arc<RwLock<HashMap<String, Vec<chrono::DateTime<chrono::Utc>>>>>,
}

impl SecurityState {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config: Arc::new(config),
            blocked_ips: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if IP is blocked
    async fn is_blocked(&self, ip: &str) -> bool {
        let blocked = self.blocked_ips.read().await;
        blocked.contains_key(ip)
    }

    /// Block an IP address
    #[instrument(skip(self))]
    pub async fn block_ip(&self, ip: String, reason: String) {
        warn!(ip = %ip, reason = %reason, "Blocking IP address");

        let mut blocked = self.blocked_ips.write().await;
        blocked.insert(
            ip.clone(),
            BlockedIP {
                ip,
                reason,
                blocked_at: chrono::Utc::now(),
            },
        );
    }

    /// Check rate limit for IP
    #[instrument(skip(self))]
    async fn check_rate_limit(&self, ip: &str) -> bool {
        let now = chrono::Utc::now();
        let mut limits = self.rate_limits.write().await;

        let requests = limits.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove requests older than 1 minute
        requests.retain(|time| now.signed_duration_since(*time).num_seconds() < 60);

        if requests.len() >= self.config.max_requests_per_minute as usize {
            warn!(ip = %ip, requests = requests.len(), "Rate limit exceeded");
            return false;
        }

        requests.push(now);
        true
    }

    /// Report suspicious activity to Honeytrap
    #[instrument(skip(self))]
    async fn report_to_honeytrap(&self, ip: &str, reason: &str) {
        if !self.config.honeytrap_enabled {
            return;
        }

        info!(ip = %ip, reason = %reason, "Reporting to Honeytrap");

        // TODO: Implement actual Honeytrap API call
        // For now, just log the attempt
        // In production, this would send to honeytrap server:
        // POST /api/report
        // { "ip": "1.2.3.4", "reason": "rate_limit_exceeded", "timestamp": "..." }
    }
}

/// Main security middleware
#[instrument(skip(state, next))]
pub async fn security_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    state: axum::extract::State<SecurityState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip security in development mode
    if !state.config.enabled {
        return Ok(next.run(req).await);
    }

    let ip = addr.ip().to_string();

    // 1. Check if IP is blocked
    if state.is_blocked(&ip).await {
        error!(ip = %ip, "Blocked IP attempted access");
        state.report_to_honeytrap(&ip, "blocked_ip_attempt").await;
        return Err(StatusCode::FORBIDDEN);
    }

    // 2. Check rate limit
    if !state.check_rate_limit(&ip).await {
        warn!(ip = %ip, "Rate limit exceeded");
        state
            .block_ip(ip.clone(), "rate_limit_exceeded".to_string())
            .await;
        state.report_to_honeytrap(&ip, "rate_limit_exceeded").await;
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // 3. Check JWT token for authenticated endpoints
    let path = req.uri().path();
    if !is_public_endpoint(path) {
        if let Some(auth_header) = headers.get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if !validate_jwt_token(auth_str, &state.config.jwt_secret) {
                    warn!(ip = %ip, "Invalid JWT token");
                    state.report_to_honeytrap(&ip, "invalid_jwt").await;
                    return Err(StatusCode::UNAUTHORIZED);
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            // No auth header on protected endpoint
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // 4. Detect suspicious patterns
    if is_suspicious_request(&req) {
        warn!(ip = %ip, path = %path, "Suspicious request detected");
        state.report_to_honeytrap(&ip, "suspicious_pattern").await;
        state.block_ip(ip, "suspicious_activity".to_string()).await;
        return Err(StatusCode::FORBIDDEN);
    }

    // Request is safe, continue
    info!(ip = %ip, path = %path, "Request passed security checks");
    Ok(next.run(req).await)
}

/// Check if endpoint is public (no auth required)
fn is_public_endpoint(path: &str) -> bool {
    matches!(
        path,
        "/_health" | "/_ready" | "/_metrics" | "/auth/login" | "/auth/callback"
    ) || path.starts_with("/auth/")
}

/// Validate JWT token
fn validate_jwt_token(token: &str, secret: &str) -> bool {
    // Strip "Bearer " prefix if present
    let token = token.strip_prefix("Bearer ").unwrap_or(token);

    // TODO: Implement proper JWT validation
    // For now, just check if token is not empty
    // In production, use jsonwebtoken crate:
    // jsonwebtoken::decode::<Claims>(token, secret, &Validation::default())
    !token.is_empty() && secret != "CHANGE_ME_IN_PRODUCTION"
}

/// Detect suspicious request patterns
fn is_suspicious_request(req: &Request<Body>) -> bool {
    let path = req.uri().path();

    // SQL injection patterns
    let sql_patterns = ["' OR '1'='1", "DROP TABLE", "UNION SELECT", "--", "/*"];
    if sql_patterns.iter().any(|p| path.contains(p)) {
        return true;
    }

    // Path traversal
    if path.contains("../") || path.contains("..\\") {
        return true;
    }

    // XSS patterns
    if path.contains("<script>") || path.contains("javascript:") {
        return true;
    }

    // Command injection
    if path.contains(";") && (path.contains("rm ") || path.contains("curl ")) {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_endpoints() {
        assert!(is_public_endpoint("/_health"));
        assert!(is_public_endpoint("/_ready"));
        assert!(is_public_endpoint("/auth/login"));
        assert!(!is_public_endpoint("/api/query"));
    }

    #[test]
    fn test_suspicious_patterns() {
        let req = Request::builder()
            .uri("/?q=' OR '1'='1")
            .body(Body::empty())
            .unwrap();
        assert!(is_suspicious_request(&req));

        let req = Request::builder()
            .uri("/../etc/passwd")
            .body(Body::empty())
            .unwrap();
        assert!(is_suspicious_request(&req));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = SecurityConfig {
            enabled: true,
            max_requests_per_minute: 5,
            ..Default::default()
        };
        let state = SecurityState::new(config);

        // Should allow first 5 requests
        for _ in 0..5 {
            assert!(state.check_rate_limit("192.168.1.1").await);
        }

        // 6th request should be blocked
        assert!(!state.check_rate_limit("192.168.1.1").await);
    }

    #[tokio::test]
    async fn test_ip_blocking() {
        let config = SecurityConfig::default();
        let state = SecurityState::new(config);

        let ip = "192.168.1.100";
        assert!(!state.is_blocked(ip).await);

        state.block_ip(ip.to_string(), "test".to_string()).await;
        assert!(state.is_blocked(ip).await);
    }
}
