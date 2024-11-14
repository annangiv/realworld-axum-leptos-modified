use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

static CACHED_USER_ID: Lazy<Mutex<Option<(String, uuid::Uuid)>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    sub: String,
    exp: i64,
}

#[tracing::instrument]
pub fn get_user_id() -> Option<uuid::Uuid> {
    let doc = leptos::document().unchecked_into::<web_sys::HtmlDocument>();

    let cookies = match doc.cookie() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to get cookies: {:?}", e);
            return None;
        }
    };

    let cookie_value = cookies
        .split("; ")
        .find(|x| x.starts_with(super::AUTH_TOKEN))
        .and_then(|x| x.split('=').last())
        .map(ToString::to_string);

    let Some(cookie_value) = cookie_value else {
        tracing::debug!("Auth token not found");
        return None;
    };

    // Check cache first
    if let Ok(cache) = CACHED_USER_ID.lock() {
        if let Some((cached_token, cached_id)) = cache.as_ref() {
            if cached_token == &cookie_value {
                if !is_token_expired(cached_token) {
                    return Some(*cached_id);
                }
                tracing::debug!("Cached token has expired");
            }
        }
    }

    // Parse and cache if needed
    let decoded_jwt = match decode_jwt_payload(&cookie_value) {
        Some(decoded) => decoded,
        None => return None,
    };

    match serde_json::from_str::<JwtClaims>(&decoded_jwt) {
        Ok(claims) => {
            if is_token_expired(&cookie_value) {
                tracing::debug!("Token has expired");
                return None;
            }

            if let Ok(uuid) = uuid::Uuid::parse_str(&claims.sub) {
                if let Ok(mut cache) = CACHED_USER_ID.lock() {
                    *cache = Some((cookie_value, uuid));
                }
                Some(uuid)
            } else {
                tracing::error!("Failed to parse UUID from claims");
                None
            }
        }
        Err(e) => {
            tracing::error!("Failed to parse JWT claims: {:?}", e);
            None
        }
    }
}

fn decode_jwt_payload(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    match base64::decode_config(payload, base64::URL_SAFE_NO_PAD) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(s) => Some(s),
            Err(e) => {
                tracing::error!("Failed to convert decoded JWT to UTF-8: {:?}", e);
                None
            }
        },
        Err(e) => {
            tracing::error!("Failed to decode JWT payload: {:?}", e);
            None
        }
    }
}

fn is_token_expired(token: &str) -> bool {
    if let Some(decoded) = decode_jwt_payload(token) {
        if let Ok(claims) = serde_json::from_str::<JwtClaims>(&decoded) {
            let current_time = (js_sys::Date::now() / 1000.0) as i64;
            return claims.exp < current_time;
        }
    }
    true
}