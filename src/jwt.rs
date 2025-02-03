use std::{sync::LazyLock, time::UNIX_EPOCH};

use jsonwebtoken::EncodingKey;
use serde::{Deserialize, Serialize};

pub static SECRET_KEY: LazyLock<EncodingKey> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("No JWT_SECRET provided!");

    EncodingKey::from_base64_secret(&secret).expect("Unable to create secret key")
});

const REFRESH_TOKEN_EXPIRATION: usize = 604_800; // 1 Week
const ACCESS_TOKEN_EXPIRATION: usize = 3_600; // 1 Hour

#[derive(Debug, Serialize, Deserialize)]
struct RefreshClaims {
    typ: String, // Must be `Refresh`
    exp: usize,  // Expiration time (as UTC timestamp seconds)
    iat: usize,  // Issued at (as UTC timestamp seconds)
    iss: String, // Issuer
    sub: String, // Subject - User ID
}

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    typ: String, // Must be `Access`
    exp: usize,  // Expiration time (as UTC timestamp seconds )
    iat: usize,  // Issued at (as UTC timestamp seconds)
    iss: String, // Issuer
    sub: String, // Subject - User ID

    // All custom claims below
    preferred_username: String,
    nickname: String,
    email: Option<String>,
    email_verified: Option<bool>,
}

pub fn issue_refresh_token(user_id: u64) -> Box<str> {
    let header = jsonwebtoken::Header::default();

    let utc = UNIX_EPOCH.elapsed().unwrap().as_secs() as usize;

    let claims = RefreshClaims {
        // 1 Week
        typ: "Refresh".to_string(),
        exp: utc + REFRESH_TOKEN_EXPIRATION,
        iat: utc,
        iss: "picoauth".to_string(),
        sub: user_id.to_string(),
    };

    let jwtstring = jsonwebtoken::encode(&header, &claims, &SECRET_KEY).unwrap();
    jwtstring.into_boxed_str()
}

pub fn issue_access_token(
    user_id: u64,
    username: String,
    display_name: String,
    email: Option<String>,
    email_verified: Option<bool>,
) -> Box<str> {
    let header = jsonwebtoken::Header::default();

    let utc = UNIX_EPOCH.elapsed().unwrap().as_secs() as usize;

    let claims = Claims {
        typ: "Access".to_string(),
        exp: utc + ACCESS_TOKEN_EXPIRATION,
        iat: utc,
        iss: "picoauth".to_string(),
        sub: user_id.to_string(),

        preferred_username: username,
        nickname: display_name,
        email,
        email_verified,
    };

    let jwtstring = jsonwebtoken::encode(&header, &claims, &SECRET_KEY).unwrap();
    jwtstring.into_boxed_str()
}
