use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use mongodb::bson::{doc, Document};
use serde::Deserialize;

use crate::{structs, utils::check_auth_token::check_auth_token, AppState};

#[derive(Deserialize)]
pub struct GetLogsInput {
    token: String,
    target_apps: Option<Vec<String>>,
    target_types: Option<Vec<String>>,
    page_id: u64,
    page_size: u64,
    page_amount: u64,
}

pub async fn get_logs_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<GetLogsInput>,
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
    let app_ids = body.target_apps;
    let types = body.target_types;
    let page_id = body.page_id;
    let page_size = body.page_size;
    let page_amount = body.page_amount;
    let result = get_logs(
        app_state.clone(),
        app_ids.clone(),
        types,
        page_id,
        page_size,
        page_amount,
    )
    .await
    .unwrap();
    let logs: Vec<structs::Log> = result.1;
    let count = result.0;

    let json_response = serde_json::json!({
        "status": "success",
        "logs": logs,
        "next_elements": count,
    });

    return Json(json_response);
}

async fn get_logs(
    app_state: Arc<AppState>,
    app_ids: Option<Vec<String>>,
    types: Option<Vec<String>>,
    page_id: u64,
    page_size: u64,
    page_amount: u64,
) -> Result<(u64, Vec<structs::Log>), mongodb::error::Error> {
    let db = &app_state.db;
    let collection: mongodb::Collection<Document> = db.collection("logs");

    // Do not get logs with deleted field
    let skip: u64 = page_size * page_id;
    let limit = page_size * page_amount;
    let mut filter = doc! {
        // Check if app_id is in target_apps
        "app_id": {
            "$in": app_ids
        },
        "deleted": {
            "$exists": false
        }
    };
    if types.is_some() {
        let types = types.unwrap();
        let types_filter = doc! {
            "$in": types
        };
        filter.insert("type_", types_filter);
    }
    let mut cursor = collection
        .find(
            filter.clone(),
            mongodb::options::FindOptions::builder()
                .sort(doc! { "timestamp": -1 })
                .skip(skip)
                .limit(limit as i64)
                .build(),
        )
        .await?;

    let mut result: Vec<structs::Log> = Vec::new();
    while cursor.advance().await? {
        let doc = cursor.current();
        let message = doc.get("message").unwrap().unwrap().as_str().unwrap();
        let app_id = doc.get("app_id").unwrap().unwrap().as_str().unwrap();
        let timestamp = doc.get("timestamp").unwrap().unwrap().as_i64().unwrap();
        let _id = doc.get("_id").unwrap().unwrap().as_object_id().unwrap();
        let type_ = doc.get("type_").unwrap().unwrap().as_str().unwrap();
        let log = structs::Log {
            _id: Some(_id.to_hex()),
            app_id: Some(app_id.to_string()),
            type_: Some(type_.to_string()),
            message: message.to_string(),
            timestamp: Some(timestamp),
        };
        result.push(log);
    }

    let count = (collection.count_documents(filter, None).await.unwrap()) as i64
        - skip as i64
        - limit as i64;
    let count = if count < 0 { 0 } else { count } as u64;

    return Ok((count, result));
}
