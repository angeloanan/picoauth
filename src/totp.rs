use rand::{Rng, SeedableRng};

use totp_rs::{Rfc6238, Secret, TOTP};

pub fn generate_secret_bytes() -> [u8; 20] {
    // Initialize new RNG every time function gets called
    // This hopefully ensures that forward secrecy is maintained
    let mut rng = rand::rngs::StdRng::from_os_rng();
    rng.random()
}

pub fn generate_secret() -> Secret {
    Secret::Raw(generate_secret_bytes().to_vec())
}

pub fn check_current(secret_bytes: &[u8], token: &str) -> bool {
    let mut rfc6238 = Rfc6238::with_defaults(secret_bytes.to_vec()).unwrap();
    rfc6238.account_name("Account name here".to_string());
    rfc6238.issuer("Website name".to_string());
    let totp = TOTP::from_rfc6238(rfc6238).unwrap();

    match totp.check_current(token) {
        Ok(_) => true,
        Err(SystemTimeError) => false,
    }
}
