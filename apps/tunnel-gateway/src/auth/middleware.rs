use crate::auth::supabase_client::SupabaseClient;
use std::sync::Arc;
use tonic::{Request, Status};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String,
    pub email: String,
    pub role: String,
}

/// gRPC interceptor for Supabase JWT authentication
#[derive(Clone)]
pub struct AuthInterceptor {
    supabase: Arc<SupabaseClient>,
}

impl AuthInterceptor {
    pub fn new(supabase: Arc<SupabaseClient>) -> Self {
        Self { supabase }
    }
    
    /// Intercept and validate Supabase JWT token from metadata
    pub async fn intercept<T>(&self, mut req: Request<T>) -> Result<Request<T>, Status> {
        // Get authorization header
        let token = match req.metadata().get("authorization") {
            Some(t) => t.to_str().map_err(|_| {
                Status::unauthenticated("Invalid authorization header")
            })?,
            None => {
                return Err(Status::unauthenticated("Missing authorization token"));
            }
        };
        
        // Extract token from "Bearer <token>" format
        let token = extract_bearer_token(token)
            .ok_or_else(|| Status::unauthenticated("Invalid token format. Use: Bearer <token>"))?;
        
        // Validate token with Supabase
        let verify_response = self.supabase.verify_token(&token)
            .await
            .map_err(|e| {
                tracing::warn!("Token validation failed: {}", e);
                Status::unauthenticated("Invalid or expired token")
            })?;
        
        // Add user info to request extensions for downstream services
        let claims = AuthClaims {
            sub: verify_response.sub,
            email: verify_response.email,
            role: verify_response.role,
        };
        req.extensions_mut().insert(claims);
        
        Ok(req)
    }
}

/// Extract token from "Bearer <token>" format
fn extract_bearer_token(auth_header: &str) -> Option<String> {
    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        None
    }
}

/// Helper function to extract user ID from request extensions
pub fn get_user_id_from_request<T>(req: &Request<T>) -> Result<String, Status> {
    req.extensions()
        .get::<AuthClaims>()
        .map(|claims| claims.sub.clone())
        .ok_or_else(|| Status::unauthenticated("User not authenticated"))
}

/// Helper function to extract email from request extensions
pub fn get_email_from_request<T>(req: &Request<T>) -> Result<String, Status> {
    req.extensions()
        .get::<AuthClaims>()
        .map(|claims| claims.email.clone())
        .ok_or_else(|| Status::unauthenticated("User not authenticated"))
}
