use std::sync::Arc;

use axum::Json;
use mongodb::bson::{doc, Document};
use reqwest::StatusCode;
use std::fs::File;

use crate::AppState;

pub async fn add_message(
    app_state: Arc<AppState>,
    _id: Option<String>,
    app_id: Option<String>,
    mut r#type: Option<String>,
    message: String,
    mut timestamp: Option<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let db = &app_state.db;
    let collection: mongodb::Collection<Document> = db.collection("logs");

    if timestamp.is_none() {
        let current_date = chrono::Utc::now();
        let current_timestamp = current_date.timestamp_millis();
        timestamp = Some(current_timestamp);
    }

    if r#type.is_none() {
        r#type = Some("default".to_string());
    }

    let res = collection
        .insert_one(
            doc! {
                "app_id": app_id.clone(),
                "timestamp": timestamp.clone(),
                "type_": r#type.clone(),
                "message": message.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let collection: mongodb::Collection<Document> = db.collection("services");
    let parsed_app_id = mongodb::bson::oid::ObjectId::parse_str(&app_id.clone().unwrap()).unwrap();
    let service = collection
        .find_one(
            doc! {
                "_id": parsed_app_id
            },
            None,
        )
        .await
        .unwrap();

    if service.is_none() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "This service has been deleted (the app_id is valid for the specified token but the service doesn't exist)"));
    }

    let service = service.unwrap();

    let colllection: mongodb::Collection<Document> = db.collection("types");
    let type_ = colllection
        .find_one(
            doc! {
                "name": r#type.clone().unwrap()
            },
            None,
        )
        .await
        .unwrap();
    let conf = app_state.conf.clone();
    if !type_.is_none() {
        let type_ = type_.unwrap();
        let notifications = type_.get("notifications").unwrap().as_array().unwrap();

        if notifications.contains(&"discord".to_string().into()) {
            let config_file = File::open("config.json").unwrap();
            let config_json: serde_json::Value = serde_json::from_reader(config_file).unwrap();
            let discord_webhook = config_json["discord_webhook"].as_str();
            if !discord_webhook.is_none() {
                let discord_webhook = discord_webhook.unwrap();
                let message = format!(
                    "<t:{}> __{}__\n**{}**\n{}\n➡️ [open](https://watch-t.vercel.app/dashboard?page=logs&services={}#log_{})",
                    timestamp.unwrap(),
                    service.get("app_name").unwrap().as_str().unwrap(),
                    r#type.clone().unwrap(),
                    message,
                    app_id.unwrap(),
                    res.inserted_id.as_object_id().unwrap().to_hex()
                );

                let client = reqwest::Client::new();
                client
                    .post(discord_webhook)
                    .form(&serde_json::json!({ "content": message }))
                    .send()
                    .await
                    .unwrap();
            }
        }
        if notifications.contains(&"telegram".to_string().into()) {
            let config_file = File::open("config.json").unwrap();
            let config_json: serde_json::Value = serde_json::from_reader(config_file).unwrap();
            let telegram_chat = config_json["telegram_chat"].as_str();
            if !telegram_chat.is_none() {
                let telegram_chat = telegram_chat.unwrap();
                let message = format!(
                    "<b>{}</b>\n<i>{}</i>\n\n{}",
                    service.get("app_name").unwrap().as_str().unwrap(),
                    r#type.unwrap(),
                    message
                );

                let client = reqwest::Client::new();
                client
                    .post(format!(
                        "https://api.telegram.org/bot{}/sendMessage",
                        conf.connections.telegram_token
                    ))
                    .form(&serde_json::json!({
                        "chat_id": telegram_chat,
                        "text": message,
                        "parse_mode": "HTML",
                    }))
                    .send()
                    .await
                    .unwrap();
            }
        }
    }

    let json_response = serde_json::json!({
        "status": "success",
    });

    return Ok(Json(json_response));
}
