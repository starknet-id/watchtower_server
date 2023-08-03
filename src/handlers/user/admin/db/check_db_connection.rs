use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use serde::Deserialize;

use crate::{
    utils::{
        check_auth_token::check_auth_token, get_token_data::get_token_data,
        has_permission::has_permission, user::db::update_db_datas::update_db_datas,
    },
    AppState,
};

#[derive(Deserialize)]
pub struct CheckDbConnectionInput {
    token: String,
    db_id: String,
}

pub async fn check_db_connection_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<CheckDbConnectionInput>,
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

    let db_id = mongodb::bson::oid::ObjectId::parse_str(&body.db_id).unwrap();

    let db = &app_state.db;
    let collection: Collection<Document> = db.collection("databases");
    let database = collection
        .find_one(doc! {"_id": db_id}, None)
        .await
        .unwrap()
        .unwrap();
    let db_name = database.get_str("name").unwrap();

    let connection_string = database.get_str("connection_string").unwrap();
    let db_id = database.get_object_id("_id").unwrap();
    // Update db status
    collection
        .update_one(
            doc! {"_id": db_id},
            doc! {"$set": {"status": "connecting"}},
            None,
        )
        .await
        .unwrap();
    let res = update_db_datas(
        collection,
        connection_string.to_string(),
        db_name.to_string(),
        db_id.to_hex(),
    )
    .await;

    return Json(serde_json::json!({
        "status": "success",
        "success": res.success,
        "message": res.message,
    }));
}
