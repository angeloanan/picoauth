use std::sync::LazyLock;

use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

static HASHER: LazyLock<Argon2> = std::sync::LazyLock::new(|| {
    let secret = std::env::var("ARGON_SECRET").ok();

    match secret {
        Some(secret) => Argon2::new_with_secret(
            secret.leak().as_bytes(), // oop
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::DEFAULT,
        )
        .expect("Unable to create Argon2 hasher!"),
        None => Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::DEFAULT,
        ),
    }
});

pub fn hash_password(password: &str) -> Box<str> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = HASHER
        .hash_password(password.as_bytes(), salt.as_salt())
        .expect("Unable to hash password!");
    println!("Password: {hash}");

    hash.to_string().into_boxed_str()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let pw_hash = PasswordHash::new(hash).expect("Malformed password hash");

    HASHER
        .verify_password(password.as_bytes(), &pw_hash)
        .is_ok()
}
