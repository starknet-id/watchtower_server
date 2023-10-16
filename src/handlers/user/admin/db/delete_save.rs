use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use mongodb::bson::doc;
use serde::Deserialize;

use crate::{
    utils::{
        check_auth_token::check_auth_token, get_token_data::get_token_data,
        has_permission::has_permission, user::db::delete_save,
    },
    AppState,
};

#[derive(Deserialize)]
pub struct DeleteSaveInput {
    token: String,
    save_id: String,
}

pub async fn delete_save_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<DeleteSaveInput>,
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

    let save_id = mongodb::bson::oid::ObjectId::parse_str(&body.save_id).unwrap();

    delete_save::delete_save(&app_state.db.clone(), save_id).await;

    return Json(serde_json::json!({
        "status": "success",
    }));
}
