use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use mongodb::bson::{doc, oid::ObjectId, Document};

use crate::{
    structs,
    utils::{
        check_auth_token::check_auth_token, get_token_data::get_token_data,
        has_permission::has_permission,
    },
    AppState,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetDbSavesInput {
    token: String,
    db_id: String,
}

pub async fn get_db_saves_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<GetDbSavesInput>,
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

    let db_id = ObjectId::parse_str(&body.db_id).unwrap();

    // get from mongodb
    let saves: Vec<structs::DbSave> = get_saves(app_state, db_id).await.unwrap();

    return Json(serde_json::json!({
        "status": "success",
        "saves": saves,
    }));
}

async fn get_saves(
    app_state: Arc<AppState>,
    db_id: ObjectId,
) -> Result<Vec<structs::DbSave>, mongodb::error::Error> {
    let db = &app_state.db;
    let collection: mongodb::Collection<Document> = db.collection("db_saves");

    let mut cursor = collection
        .find(
            doc! {
                "db_id": db_id
            },
            None,
        )
        .await?;

    let mut result: Vec<structs::DbSave> = Vec::new();
    while cursor.advance().await? {
        let doc = cursor.current();
        let _id = doc.get("_id").unwrap().unwrap().as_object_id().unwrap();
        let db_id = doc.get("db_id").unwrap().unwrap().as_object_id().unwrap();
        let timestamp = doc.get("time").unwrap().unwrap().as_i64().unwrap();
        let save = structs::DbSave {
            _id: _id.to_hex(),
            db_id: db_id.to_hex(),
            timestamp: timestamp,
        };
        result.push(save);
    }

    return Ok(result);
}
