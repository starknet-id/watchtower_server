use axum::{response::IntoResponse, Json};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection,
};

use mongodb::Database;

pub async fn delete_save(db: &Database, save_id: ObjectId) -> impl IntoResponse {
    let collection: Collection<Document> = db.collection("db_saves");
    // Get save
    let save = collection
        .find_one(doc! {"_id": save_id}, None)
        .await
        .unwrap()
        .unwrap();
    let db_id = save.get_object_id("db_id").unwrap();
    let time = save.get_i64("time").unwrap();
    // Delete file
    let path = format!("db_saves/{}/{}", db_id, time);
    let res = std::fs::remove_dir_all(path);
    if res.is_err() {
        println!("Error deleting db save: {}", res.err().unwrap());
    }
    // Delete save
    collection
        .delete_one(doc! {"_id": save_id}, None)
        .await
        .unwrap();
    return Json(serde_json::json!({
        "status": "success",
    }));
}
