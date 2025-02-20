use std::{string::ToString, sync::LazyLock, time::UNIX_EPOCH};

use jsonwebtoken::{DecodingKey, EncodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};

pub static SECRET_KEY: LazyLock<EncodingKey> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("No JWT_SECRET provided!");

    EncodingKey::from_base64_secret(&secret).expect("Unable to create secret key")
});
pub static DECODING_KEY: LazyLock<DecodingKey> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("No JWT_SECRET provided!");

    DecodingKey::from_base64_secret(&secret).expect("Unable to create decode key")
});

const REFRESH_TOKEN_EXPIRATION: usize = 604_800; // 1 Week
const ACCESS_TOKEN_EXPIRATION: usize = 3_600; // 1 Hour

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub typ: String,      // Must be `Refresh`
    pub auth_time: usize, // Time when authentication occurred,
    pub iat: usize,       // Issued at (as UTC timestamp seconds)
    pub exp: usize,       // Expiration time (as UTC timestamp seconds)
    pub iss: String,      // Issuer
    pub sub: String,      // Subject - User ID
}

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
/// https://www.iana.org/assignments/jwt/jwt.xhtml
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub typ: String,      // Must be `Access`
    pub auth_time: usize, // Time when authentication occurred,
    pub iat: usize,       // Issued at (as UTC timestamp seconds)
    pub exp: usize,       // Expiration time (as UTC timestamp seconds )
    pub iss: String,      // Issuer
    pub sub: String,      // Subject - User ID

    // All custom claims below
    pub preferred_username: String,
    pub nickname: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
}

pub fn issue_refresh_token(user_id: u64, auth_time: Option<usize>) -> Box<str> {
    let utc = UNIX_EPOCH.elapsed().unwrap().as_secs() as usize;

    let claims = RefreshClaims {
        // 1 Week
        typ: "Refresh".to_string(),
        exp: utc + REFRESH_TOKEN_EXPIRATION,
        iat: utc,
        iss: "picoauth".to_string(),
        sub: user_id.to_string(),
        auth_time: auth_time.unwrap_or(utc),
    };

    let jwtstring =
        jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &SECRET_KEY).unwrap();
    jwtstring.into_boxed_str()
}

pub fn issue_access_token(
    user_id: u64,
    username: &str,
    display_name: Option<&str>,
    email: Option<&str>,
    email_verified: Option<bool>,
    auth_time: usize,
) -> Box<str> {
    let utc = UNIX_EPOCH.elapsed().unwrap().as_secs() as usize;

    let claims = Claims {
        typ: "Access".to_string(),
        exp: utc + ACCESS_TOKEN_EXPIRATION,
        iat: utc,
        iss: "picoauth".to_string(),
        sub: user_id.to_string(),
        auth_time,

        preferred_username: username.to_string(),
        nickname: display_name.map_or(username.to_string(), ToString::to_string),
        email: email.map(ToString::to_string),
        email_verified,
    };

    let jwtstring =
        jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &SECRET_KEY).unwrap();
    jwtstring.into_boxed_str()
}

pub fn verify_access_token(token: &str) -> jsonwebtoken::errors::Result<TokenData<Claims>> {
    jsonwebtoken::decode::<Claims>(token, &DECODING_KEY, &Validation::default())
}

pub fn verify_refresh_token(token: &str) -> jsonwebtoken::errors::Result<TokenData<RefreshClaims>> {
    jsonwebtoken::decode::<RefreshClaims>(token, &DECODING_KEY, &Validation::default())
}
