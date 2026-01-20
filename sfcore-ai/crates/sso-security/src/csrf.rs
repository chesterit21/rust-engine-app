//! CSRF protection (placeholder)

use rand::Rng;

pub fn generate_csrf_token() -> String {
    let token: [u8; 32] = rand::thread_rng().gen();
    hex::encode(token)
}

pub fn validate_csrf_token(token: &str, expected: &str) -> bool {
    token == expected
}
