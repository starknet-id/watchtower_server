use std::sync::Arc;

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};

use crate::AppState;

pub fn hash_password(app_state: Arc<AppState>, password: String) -> String {
    let config = app_state.conf.clone();
    let salt = SaltString::encode_b64(&config.security.password_salt.as_bytes()).unwrap();
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    return password_hash;
}
