use tonic::{Request, Response, Status};
use identra_proto::auth::auth_service_server::AuthService;
use identra_proto::auth::{
    LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, 
    RegisterRequest, RegisterResponse, VerifyTokenRequest, VerifyTokenResponse,
};
use crate::auth::supabase_client::SupabaseClient;
use std::sync::Arc;

pub struct AuthServiceImpl {
    supabase: Arc<SupabaseClient>,
}

impl AuthServiceImpl {
    pub fn new(supabase: Arc<SupabaseClient>) -> Self {
        Self { supabase }
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        
        // Validation
        if req.username.trim().is_empty() {
            return Ok(Response::new(RegisterResponse {
                success: false,
                message: "Username cannot be empty".to_string(),
                user_id: String::new(),
            }));
        }
        
        if req.password.len() < 8 {
            return Ok(Response::new(RegisterResponse {
                success: false,
                message: "Password must be at least 8 characters".to_string(),
                user_id: String::new(),
            }));
        }
        
        // Use Supabase Auth for registration
        match self.supabase.sign_up(&req.email, &req.password, &req.username).await {
            Ok(auth_response) => {
                tracing::info!("User registered: {} ({})", req.username, auth_response.user.id);
                
                Ok(Response::new(RegisterResponse {
                    success: true,
                    message: "User registered successfully".to_string(),
                    user_id: auth_response.user.id,
                }))
            }
            Err(e) => {
                tracing::error!("Registration failed: {}", e);
                Ok(Response::new(RegisterResponse {
                    success: false,
                    message: e,
                    user_id: String::new(),
                }))
            }
        }
    }
    
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        
        // Supabase uses email for login
        // We treat username field as email
        match self.supabase.sign_in(&req.username, &req.password).await {
            Ok(auth_response) => {
                tracing::info!("User logged in: {}", auth_response.user.id);
                
                Ok(Response::new(LoginResponse {
                    success: true,
                    message: "Login successful".to_string(),
                    access_token: auth_response.access_token,
                    refresh_token: auth_response.refresh_token,
                    expires_in: auth_response.expires_in as i64,
                }))
            }
            Err(e) => {
                tracing::warn!("Login failed for user: {}", req.username);
                Ok(Response::new(LoginResponse {
                    success: false,
                    message: "Invalid credentials".to_string(),
                    access_token: String::new(),
                    refresh_token: String::new(),
                    expires_in: 0,
                }))
            }
        }
    }
    
    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();
        
        match self.supabase.refresh_token(&req.refresh_token).await {
            Ok(auth_response) => {
                Ok(Response::new(RefreshTokenResponse {
                    success: true,
                    access_token: auth_response.access_token,
                    expires_in: auth_response.expires_in as i64,
                }))
            }
            Err(e) => {
                tracing::error!("Token refresh failed: {}", e);
                Ok(Response::new(RefreshTokenResponse {
                    success: false,
                    access_token: String::new(),
                    expires_in: 0,
                }))
            }
        }
    }
    
    async fn verify_token(
        &self,
        request: Request<VerifyTokenRequest>,
    ) -> Result<Response<VerifyTokenResponse>, Status> {
        let req = request.into_inner();
        
        match self.supabase.verify_token(&req.token).await {
            Ok(verify_response) => {
                Ok(Response::new(VerifyTokenResponse {
                    valid: true,
                    user_id: verify_response.sub,
                    username: verify_response.email,
                    expires_at: verify_response.exp as i64,
                }))
            }
            Err(e) => {
                tracing::warn!("Token verification failed: {}", e);
                Ok(Response::new(VerifyTokenResponse {
                    valid: false,
                    user_id: String::new(),
                    username: String::new(),
                    expires_at: 0,
                }))
            }
        }
    }
}
