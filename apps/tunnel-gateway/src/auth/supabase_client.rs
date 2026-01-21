use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone)]
pub struct SupabaseClient {
    client: Client,
    url: String,
    anon_key: String,
    service_role_key: String,
}

#[derive(Debug, Serialize)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<UserMetadata>,
}

#[derive(Debug, Serialize)]
pub struct UserMetadata {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub user: SupabaseUser,
}

#[derive(Debug, Deserialize)]
pub struct SupabaseUser {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub user_metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SupabaseError {
    pub error: String,
    pub error_description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyRequest {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyResponse {
    pub aud: String,
    pub exp: u64,
    pub sub: String,
    pub email: String,
    pub role: String,
}

impl SupabaseClient {
    pub fn new() -> Result<Self, String> {
        let url = env::var("SUPABASE_URL")
            .map_err(|_| "SUPABASE_URL not set in environment")?;
        let anon_key = env::var("SUPABASE_ANON_KEY")
            .map_err(|_| "SUPABASE_ANON_KEY not set in environment")?;
        let service_role_key = env::var("SUPABASE_SERVICE_ROLE_KEY")
            .map_err(|_| "SUPABASE_SERVICE_ROLE_KEY not set in environment")?;

        Ok(Self {
            client: Client::new(),
            url,
            anon_key,
            service_role_key,
        })
    }

    pub async fn sign_up(
        &self,
        email: &str,
        password: &str,
        username: &str,
    ) -> Result<AuthResponse, String> {
        let signup_url = format!("{}/auth/v1/signup", self.url);
        
        let payload = SignUpRequest {
            email: email.to_string(),
            password: password.to_string(),
            data: Some(UserMetadata {
                username: username.to_string(),
            }),
        };

        let response = self.client
            .post(&signup_url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<SupabaseError>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error_description.unwrap_or(error.error))
        }
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthResponse, String> {
        let signin_url = format!("{}/auth/v1/token?grant_type=password", self.url);
        
        let payload = SignInRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self.client
            .post(&signin_url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<SupabaseError>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error_description.unwrap_or(error.error))
        }
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, String> {
        let refresh_url = format!("{}/auth/v1/token?grant_type=refresh_token", self.url);
        
        let payload = RefreshRequest {
            refresh_token: refresh_token.to_string(),
        };

        let response = self.client
            .post(&refresh_url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<SupabaseError>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error_description.unwrap_or(error.error))
        }
    }

    pub async fn verify_token(&self, token: &str) -> Result<VerifyResponse, String> {
        let user_url = format!("{}/auth/v1/user", self.url);

        let response = self.client
            .get(&user_url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            let user = response
                .json::<SupabaseUser>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            
            Ok(VerifyResponse {
                aud: "authenticated".to_string(),
                exp: 0, // Supabase handles expiration
                sub: user.id,
                email: user.email,
                role: "authenticated".to_string(),
            })
        } else {
            Err("Invalid or expired token".to_string())
        }
    }

    pub async fn sign_out(&self, access_token: &str) -> Result<(), String> {
        let signout_url = format!("{}/auth/v1/logout", self.url);

        let response = self.client
            .post(&signout_url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err("Failed to sign out".to_string())
        }
    }
}

impl Default for SupabaseClient {
    fn default() -> Self {
        Self::new().expect("Failed to initialize Supabase client")
    }
}
