use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use mongodb::bson::doc;
use serde::Deserialize;
use std::fs::File;

use crate::{
    utils::{
        check_auth_token::check_auth_token, get_token_data::get_token_data,
        has_permission::has_permission,
    },
    AppState,
};

#[derive(Deserialize)]
pub struct SetTelegramGroupInput {
    token: String,
    new_group_id: String,
}

pub async fn set_telegram_chat_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<SetTelegramGroupInput>,
) -> impl IntoResponse {
    let token = body.token;
    let valid = check_auth_token(app_state.clone(), token.clone());
    if !valid {
        let json_response = serde_json::json!({
            "status": "error",
            "message": "Invalid token or token expired",
            "error_code": "invalid_token"
        });

        return Json(json_response);
    }

    let token_data = get_token_data(app_state.clone(), token);

    let has_perm = has_permission(
        token_data.user_id,
        "administrator".to_string(),
        app_state.clone(),
    )
    .await;

    if !has_perm {
        let json_response = serde_json::json!({
            "status": "error",
            "message": "You don't have administrator permission",
            "error_code": "permission_denied"
        });

        return Json(json_response);
    }

    let group_id = body.new_group_id;

    // Write in config.json
    let config_file = File::open("config.json").unwrap();
    let mut config: serde_json::Value = serde_json::from_reader(config_file).unwrap();
    config["telegram_chat"] = serde_json::json!(group_id);
    let config_file = File::create("config.json").unwrap();
    serde_json::to_writer_pretty(config_file, &config).unwrap();

    return Json(serde_json::json!({
        "status": "success",
    }));
}
