use std::sync::Arc;

use mongodb::bson::{doc, Document};
use mongodb::Collection;

use crate::utils::user::db::delete_save::delete_save;
use crate::AppState;
use chrono::Datelike;

pub async fn clean_db_saves(app_state: Arc<AppState>) -> Result<(), mongodb::error::Error> {
    // Delete old non manual db saves
    // We want to keep every saves from last 7 days
    // 1 save per week for the last month
    // 1 save per month for the last year

    let db = &app_state.db;
    let collection: Collection<Document> = db.collection("db_saves");
    let mut db_saves_cursor = collection
        .clone()
        .find(
            doc! {
                "manual": false
            },
            mongodb::options::FindOptions::builder()
                .sort(doc! {
                    "time": -1
                })
                .build(),
        )
        .await
        .unwrap();

    let now = chrono::Utc::now().timestamp_millis();
    let minute = 1000 * 60;
    let day = minute * 60 * 24;
    let week = day * 7;
    let month = day * 31;
    let year = day * 365;
    while db_saves_cursor.advance().await? {
        let doc = db_saves_cursor.current();
        let _id = doc.get("_id").unwrap().unwrap().as_object_id().unwrap();
        let db_id = doc.get("db_id").unwrap().unwrap().as_object_id().unwrap();
        let timestamp = doc.get("time").unwrap().unwrap().as_i64().unwrap();
        let delta = now - timestamp;
        let date = chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp_millis(timestamp).expect("error"),
            chrono::Utc,
        );
        if delta < month && delta > week {
            if date.day() % 7 != 1 {
                println!(
                    "Deleting save {} from db {} ({} created days ago - {})",
                    _id,
                    db_id,
                    delta / day,
                    timestamp
                );
                delete_save(&app_state.db.clone(), _id).await;
            }
            continue;
        }
        if delta < year && delta > month {
            if date.day() != 1 {
                println!(
                    "Deleting save {} from db {} ({} created months ago - {})",
                    _id,
                    db_id,
                    delta / month,
                    timestamp
                );
                delete_save(&app_state.db.clone(), _id).await;
            }
            continue;
        }
    }
    return Ok(());
}
