use std::sync::Arc;

use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::structs::JwtUserClaims;

use crate::AppState;

pub fn check_auth_token(app_state: Arc<AppState>, jwt_token: String) -> bool {
    let config = app_state.conf.clone();
    let jwt_secret = &config.jwt.user_secret;

    let token_data = decode::<JwtUserClaims>(
        &jwt_token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    );

    if token_data.is_err() {
        return false;
    }

    let token_data = token_data.unwrap();

    let date = chrono::Utc::now();

    if token_data.claims.exp < date.timestamp() as usize {
        return false;
    }

    return true;
}
