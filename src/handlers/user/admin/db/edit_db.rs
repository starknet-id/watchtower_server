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
        has_permission::has_permission,
    },
    AppState,
};

#[derive(Deserialize)]
pub struct EditDbInput {
    token: String,
    db_id: String,
    name: String,
    custom_name: String,
    connection_string: String,
}

pub async fn edit_db_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<EditDbInput>,
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
    let db_name = body.name;
    let connection_string = body.connection_string;
    let custom_name = body.custom_name;

    let db = &app_state.db;
    let collection: Collection<Document> = db.collection("databases");
    collection
        .update_one(
            doc! {"_id": db_id.clone()},
            doc! {"$set": {
                "name": db_name.clone(),
                "connection_string": connection_string.clone(),
                "custom_name": custom_name.clone(),
            }},
            None,
        )
        .await
        .unwrap();
    return Json(serde_json::json!({
        "status": "success",
    }));
}
