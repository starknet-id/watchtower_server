use std::sync::Arc;

use crate::{
    utils::logs_service_side::{
        add_message::add_message, check_service_token::check_service_token,
        get_service_token_data::get_service_token_data,
    },
    AppState,
};
use axum::{extract::State, response::IntoResponse, Json};
use mongodb::bson::doc;
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LogInput {
    _id: Option<String>,
    r#type: Option<String>,
    message: String,
    timestamp: Option<i64>,
}

#[derive(Deserialize)]
pub struct AddMessageInput {
    token: String,
    app_id: Option<String>,
    logs: Vec<LogInput>,
}

pub async fn add_messages_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<AddMessageInput>,
) -> impl IntoResponse {
    let token = body.token;

    let valid = check_service_token(app_state.clone(), token.clone()).await;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid token or token expired").into_response());
    }

    let logs = body.logs;
    let token_data = Some(get_service_token_data(app_state.clone(), token.clone()));
    let token_app_id = token_data.unwrap().unwrap().app_id;
    let app_id = body.app_id.clone().unwrap();

    if token_app_id != app_id {
        return Err((
            StatusCode::UNAUTHORIZED,
            format!(
                "You specified a wrong app_id. You specified {} but your token contains {}",
                app_id, token_app_id
            ),
        )
            .into_response());
    }

    let logs_len = logs.len();

    for log in logs {
        let res = add_message(
            app_state.clone(),
            log._id,
            Some(app_id.clone()),
            log.r#type,
            log.message,
            log.timestamp,
        )
        .await;

        if res.is_err() {
            return Ok(res);
        }
    }

    return Err((
        StatusCode::OK,
        format!("Successfully added {} logs to the database", logs_len),
    )
        .into_response());
}
