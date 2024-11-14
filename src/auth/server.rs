use axum::{
    http::{header, Request, StatusCode},
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use leptos::use_context;

const TOKEN_EXPIRATION_SECS: i64 = 3600; // 1 hour in seconds

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,  // Changed from Uuid to String for better compatibility
    pub exp: i64,     // Changed from usize to i64 for consistency
    pub iat: i64,     // Added issued at claim
}

#[tracing::instrument(skip_all)]
pub async fn auth_middleware<B>(req: Request<B>, next: axum::middleware::Next<B>) -> Response {
    let response = match get_user_id_from_headers(req.headers()) {
        Some(user_id) => {
            match crate::models::User::get_by_id(user_id).await {
                Ok(_) => {
                    let path = req.uri().path();
                    if path.starts_with("/login") || path.starts_with("/signup") {
                        add_security_headers(
                            Response::builder()
                                .status(StatusCode::FOUND)
                                .header(header::LOCATION, "/")
                                .body(axum::body::boxed(axum::body::Empty::new()))
                                .unwrap(),
                        )
                    } else {
                        add_security_headers(next.run(req).await)
                    }
                }
                Err(e) => {
                    tracing::error!("Error fetching user: {:?}", e);
                    add_security_headers(handle_unauthenticated(req, next).await)
                }
            }
        }
        None => add_security_headers(handle_unauthenticated(req, next).await),
    };

    response
}

fn add_security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        "max-age=31536000; includeSubDomains; preload".parse().unwrap(),
    );
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    headers.insert(header::X_FRAME_OPTIONS, "DENY".parse().unwrap());
    headers.insert(header::X_XSS_PROTECTION, "1; mode=block".parse().unwrap());
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; connect-src 'self'".parse().unwrap(),
    );
    headers.insert(
        header::REFERRER_POLICY,
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    response
}

#[tracing::instrument(skip_all)]
async fn handle_unauthenticated<B>(req: Request<B>, next: axum::middleware::Next<B>) -> Response {
    let path = req.uri().path();
    if path.starts_with("/settings") || 
       path.starts_with("/editor") || 
       path.starts_with("/api/protected") {
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/login")
            .header(
                header::SET_COOKIE,
                format!(
                    "{}=; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
                    super::AUTH_TOKEN
                ),
            )
            .body(axum::body::boxed(axum::body::Empty::new()))
            .unwrap()
    } else {
        next.run(req).await
    }
}

#[tracing::instrument]
pub fn generate_token(user_id: Uuid) -> String {
    let now = chrono::Utc::now();
    let claims = TokenClaims {
        sub: user_id.to_string(),
        exp: (now + chrono::Duration::seconds(TOKEN_EXPIRATION_SECS)).timestamp(),
        iat: now.timestamp(),
    };

    let secret = std::env!("JWT_SECRET");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Token generation failed")
}

#[tracing::instrument(skip(token))]
pub fn decode_token(
    token: &str,
) -> Result<jsonwebtoken::TokenData<TokenClaims>, jsonwebtoken::errors::Error> {
    let secret = std::env!("JWT_SECRET");
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.validate_nbf = false;
    validation.leeway = 60; // 1 minute of leeway for time comparisons
    
    decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
}

#[tracing::instrument]
pub fn get_user_id_from_headers(headers: &axum::http::HeaderMap) -> Option<Uuid> {
    headers
        .get(header::COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split("; ")
                .find(|cookie| cookie.starts_with(super::AUTH_TOKEN))
                .and_then(|cookie| cookie.split('=').nth(1))
        })
        .and_then(|token| decode_token(token).ok())
        .and_then(|token_data| Uuid::parse_str(&token_data.claims.sub).ok())
}

#[tracing::instrument]
pub fn get_user_id() -> Option<Uuid> {
    use_context::<leptos_axum::RequestParts>()
        .and_then(|req| get_user_id_from_headers(&req.headers))
}