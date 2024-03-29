use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use mongodb::bson::doc;
use serde::Deserialize;

use crate::{
    utils::{
        check_auth_token::check_auth_token, get_token_data::get_token_data,
        has_permission::has_permission, user::db::update_db_datas::update_db_datas,
    },
    AppState,
};

#[derive(Deserialize)]
pub struct AddDbInput {
    token: String,
    name: String,
    connection_string: String,
}

pub async fn add_db_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<AddDbInput>,
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

    let db_name = body.name;
    let connection_string = body.connection_string;

    // insert into mongodb
    let app = doc! { "name": db_name.clone(), "custom_name": db_name.clone(), "connection_string": connection_string.clone(), "status": "connecting", "collections": []};
    let db = &app_state.db;
    let collection = db.collection("databases");
    let res = collection.insert_one(app, None).await.unwrap();
    let db_id = res.inserted_id.as_object_id().unwrap().to_hex();

    // Do not wait for the update to finish before returning
    tokio::spawn(update_db_datas(
        collection,
        connection_string.to_string(),
        db_name.to_string(),
        db_id.clone(),
    ));

    return Json(serde_json::json!({
        "status": "success",
        "_id": db_id,
    }));
}
