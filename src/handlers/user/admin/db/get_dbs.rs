use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use mongodb::bson::{doc, Document};

use crate::{
    structs,
    utils::{
        check_auth_token::check_auth_token, get_token_data::get_token_data,
        has_permission::has_permission,
    },
    AppState,
};

pub async fn get_dbs_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<structs::AuthTokenJSON>,
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

    // get from mongodb
    let dbs: Vec<structs::Database> = get_dbs(app_state).await.unwrap();

    return Json(serde_json::json!({
        "status": "success",
        "databases": dbs,
    }));
}

async fn get_dbs(
    app_state: Arc<AppState>,
) -> Result<Vec<structs::Database>, mongodb::error::Error> {
    let db = &app_state.db;
    let collection: mongodb::Collection<Document> = db.collection("databases");

    let mut cursor = collection.find(doc! {}, None).await?;

    let mut result: Vec<structs::Database> = Vec::new();
    while cursor.advance().await? {
        let doc = cursor.current();
        let _id = doc.get("_id").unwrap().unwrap().as_object_id().unwrap();
        let db_name = doc.get("name").unwrap().unwrap().as_str().unwrap();
        let connection_string = doc
            .get("connection_string")
            .unwrap()
            .unwrap()
            .as_str()
            .unwrap();
        let service = structs::Database {
            _id: Some(_id.to_hex()),
            name: db_name.to_string(),
            connection_string: connection_string.to_string(),
        };
        result.push(service);
    }

    return Ok(result);
}
