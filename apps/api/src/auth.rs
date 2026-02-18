use serde::Deserialize;
use worker::{Env, Request};

use crate::error::ApiError;

pub struct AuthenticatedUser {
    pub user_id: String,
}

#[derive(Deserialize)]
struct SupabaseUser {
    id: String,
}

pub async fn require_user(req: &Request, env: &Env) -> Result<AuthenticatedUser, ApiError> {
    let token = extract_bearer_token(req).ok_or_else(|| {
        ApiError::Unauthorized("Missing or invalid Authorization header".to_string())
    })?;

    let supabase_url = get_env_var(env, "SUPABASE_URL").map_err(ApiError::Internal)?;

    verify_token_via_supabase(&token, &supabase_url).await
}

fn extract_bearer_token(req: &Request) -> Option<String> {
    let header = req.headers().get("Authorization").ok()??;
    let token = header.strip_prefix("Bearer ")?;
    Some(token.to_string())
}

fn get_env_var(env: &Env, name: &str) -> Result<String, String> {
    env.secret(name)
        .or_else(|_| env.var(name))
        .map(|v| v.to_string())
        .map_err(|_| format!("Missing env var: {}", name))
}

/// Verify a Supabase access token by calling Supabase's /auth/v1/user endpoint.
/// Supabase handles all JWT validation internally and returns the user if valid.
async fn verify_token_via_supabase(
    token: &str,
    supabase_url: &str,
) -> Result<AuthenticatedUser, ApiError> {
    let url = format!("{}/auth/v1/user", supabase_url);

    let headers = worker::Headers::new();
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
    headers.set("apikey", token).ok();

    let req = worker::Request::new_with_init(
        &url,
        worker::RequestInit::new()
            .with_method(worker::Method::Get)
            .with_headers(headers),
    )
    .map_err(|e| ApiError::Internal(format!("Request build failed: {}", e)))?;

    let mut resp = worker::Fetch::Request(req)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Supabase auth check failed: {}", e)))?;

    if resp.status_code() == 401 {
        return Err(ApiError::Unauthorized(
            "Invalid or expired token".to_string(),
        ));
    }

    if resp.status_code() >= 400 {
        let txt = resp.text().await.unwrap_or_default();
        return Err(ApiError::Internal(format!(
            "Supabase auth error: {} {}",
            resp.status_code(),
            txt
        )));
    }

    let user: SupabaseUser = resp
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse user: {}", e)))?;

    Ok(AuthenticatedUser { user_id: user.id })
}
