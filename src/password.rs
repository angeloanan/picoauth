use std::{str::FromStr, sync::LazyLock};

use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

static HASHER: LazyLock<Argon2> = std::sync::LazyLock::new(|| {
    let secret = std::env::var("ARGON_SECRET").ok();

    secret.map_or_else(
        || {
            Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                Params::DEFAULT,
            )
        },
        |secret| {
            Argon2::new_with_secret(
                secret.leak().as_bytes(), // oop
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                Params::DEFAULT,
            )
            .expect("Unable to create Argon2 hasher!")
        },
    )
});

pub fn hash(password: &str) -> Box<str> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = HASHER
        .hash_password(password.as_bytes(), salt.as_salt())
        .expect("Unable to hash password!");

    hash.to_string().into_boxed_str()
}

pub fn verify(password: &str, hash: &str) -> bool {
    let pw_hash = PasswordHash::new(hash).expect("Malformed password hash");

    HASHER
        .verify_password(password.as_bytes(), &pw_hash)
        .is_ok()
}
