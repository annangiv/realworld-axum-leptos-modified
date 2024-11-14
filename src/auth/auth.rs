use axum::http::header::{HeaderName, HeaderValue};
use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const AUTH_TOKEN: &str = "auth_token";
const TOKEN_DURATION: u32 = 3600; // 1 hour in seconds

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: i64,
}

#[server]
pub async fn validate_auth_token(token: String) -> Result<bool, ServerFnError> {
    match crate::auth::server::decode_token(&token) {
        Ok(_) => Ok(true),
        Err(e) => {
            tracing::error!("Token validation failed: {:?}", e);
            Ok(false)
        }
    }
}

pub fn set_auth_cookie(response_options: &leptos_axum::ResponseOptions, token: &str) {
    response_options.insert_header(
        HeaderName::from_static("set-cookie"),
        HeaderValue::from_str(&format!(
            "{}={}; Path=/; HttpOnly; SameSite=Strict; Secure; Max-Age={}",
            AUTH_TOKEN, token, TOKEN_DURATION
        ))
        .expect("Failed to create header value"),
    );
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    let response_options = expect_context::<leptos_axum::ResponseOptions>();
    
    response_options.insert_header(
        HeaderName::from_static("set-cookie"),
        HeaderValue::from_str(&format!(
            "{}=; Path=/; HttpOnly; SameSite=Strict; Secure; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
            AUTH_TOKEN
        )).expect("Failed to create header value")
    );

    Ok(())
}

#[server]
pub async fn authenticate(
    email: String,
    password: String,
) -> Result<super::LoginMessages, ServerFnError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use crate::models::User;

    let user = match User::get_by_email(&email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::info!("No user found with email: {}", email);
            return Ok(super::LoginMessages::Unsuccessful("Invalid credentials".into()));
        }
        Err(e) => {
            tracing::error!("Database error: {:?}", e);
            return Err(ServerFnError::ServerError("Server error".into()));
        }
    };

    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Failed to parse hash: {:?}", e);
            return Err(ServerFnError::ServerError("Server error".into()));
        }
    };

    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        let response_options = expect_context::<leptos_axum::ResponseOptions>();
        let token = crate::auth::server::generate_token(user.id);
        set_auth_cookie(&response_options, &token);

        Ok(super::LoginMessages::Successful)
    } else {
        tracing::info!("Invalid password for user: {}", email);
        Ok(super::LoginMessages::Unsuccessful("Invalid credentials".into()))
    }
}

#[cfg(feature = "ssr")]
pub fn get_user_id() -> Option<Uuid> {
    crate::auth::server::get_user_id()
}

#[cfg(not(feature = "ssr"))]
pub fn get_user_id() -> Option<Uuid> {
    crate::auth::client::get_user_id()
}